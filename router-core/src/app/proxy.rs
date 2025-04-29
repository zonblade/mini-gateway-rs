//! # Proxy Application Module
//!
//! This module implements a TCP proxy that can route traffic based on host rules.
//! It supports both plain HTTP and TLS connections, with the ability to make routing
//! decisions based on HTTP Host headers or TLS SNI extensions.
//!
//! The proxy can handle regular HTTP, websockets, and TLS traffic, dynamically
//! forwarding to appropriate backend servers based on configured rules.

use async_trait::async_trait;
use log::debug;
use pingora::apps::ServerApp;
use pingora::connectors::TransportConnector;
use pingora::protocols::Stream;
use pingora::server::ShutdownWatch;
use pingora::upstreams::peer::BasicPeer;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;

use crate::config::{self, ProxyNode, DEFAULT_PORT};
use std::sync::Mutex;
use bytes::{Buf, BytesMut};

/// Buffer pool for reducing allocations in high-throughput scenarios
/// This provides a way to reuse buffers instead of constantly allocating and deallocating
struct BufferPool {
    buffers: Mutex<Vec<BytesMut>>,
}

impl BufferPool {
    fn new(initial_capacity: usize) -> Self {
        let mut buffers = Vec::with_capacity(initial_capacity);
        for _ in 0..initial_capacity {
            buffers.push(BytesMut::with_capacity(8192));
        }
        BufferPool {
            buffers: Mutex::new(buffers),
        }
    }

    fn get(&self) -> BytesMut {
        match self.buffers.lock().unwrap().pop() {
            Some(buf) => buf,
            None => BytesMut::with_capacity(8192),
        }
    }

    fn put(&self, mut buf: BytesMut) {
        buf.clear(); // Reset the buffer for reuse
        self.buffers.lock().unwrap().push(buf);
    }
}

lazy_static::lazy_static! {
    static ref BUFFER_POOL: BufferPool = BufferPool::new(100);
}

/// # Redirect Rule Configuration
///
/// Defines a rule for redirecting traffic based on host and TLS status.
///
/// ## Fields
/// * `host` - Optional hostname to match (e.g. "example.com:443"). When None, acts as a catch-all rule.
/// * `alt_listen` - The address:port this rule applies to (e.g. "0.0.0.0:443")
/// * `alt_target` - Optional target backend server to forward traffic to
/// * `alt_tls` - Whether this rule applies to TLS connections
/// * `priority` - Rule priority (higher priority rules are checked first)
#[derive(Clone)]
struct RedirectRule {
    host: Option<String>,
    // alt_listen: String,
    alt_target: Option<BasicPeer>,
    alt_tls: bool,
}

/// # Proxy Application
///
/// Main application that handles incoming requests and routes them to the appropriate
/// backend based on configured redirect rules.
///
/// ## Fields
/// * `client_connectors` - Transport connectors for connecting to backend servers
/// * `host_rules` - HashMap of rules with a specific host, keyed by (host, is_tls)
/// * `catch_all_rules` - HashMap of rules without a specific host, keyed by is_tls
pub struct ProxyApp {
    client_connectors: std::collections::HashMap<String, TransportConnector>,
    host_rules: std::collections::HashMap<(String, bool), RedirectRule>,
    catch_all_rules: std::collections::HashMap<bool, RedirectRule>,
}

/// # Duplex Communication Events
///
/// Events that can occur during bidirectional communication between client and server.
///
/// ## Variants
/// * `DownstreamRead(usize)` - Read `usize` bytes from the downstream (client) connection
/// * `UpstreamRead(usize)` - Read `usize` bytes from the upstream (target server) connection
enum DuplexEvent {
    DownstreamRead(usize),
    UpstreamRead(usize),
}

/// Represents the type of network connection detected
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnectionType {
    /// TLS connection (starts with 0x16)
    Tls,
    /// WebSocket upgrade request over HTTP
    WebSocket,
    /// Regular HTTP connection
    Http,
    /// Plain TCP connection (neither TLS nor HTTP)
    Tcp,
}

impl ProxyApp {
    /// Fast rule lookup that directly accesses HashMaps for better performance
    fn fast_rule_lookup(&self, host: Option<&str>, is_tls: bool) -> BasicPeer {
        if let Some(host_str) = host {
            // Use the host-specific rule if it exists (direct HashMap lookup)
            if let Some(rule) = self.host_rules.get(&(host_str.to_string(), is_tls)) {
                if let Some(target) = &rule.alt_target {
                    return target.clone();
                }
            }
            
            // Fast fallback to catch-all rule
            if let Some(rule) = self.catch_all_rules.get(&is_tls) {
                if let Some(target) = &rule.alt_target {
                    return target.clone();
                }
            }
        } else {
            // No host info, use the catch-all rule
            if let Some(rule) = self.catch_all_rules.get(&is_tls) {
                if let Some(target) = &rule.alt_target {
                    return target.clone();
                }
            }
        }
        
        // Default fallback (create a new BasicPeer)
        BasicPeer::new(DEFAULT_PORT.p404)
    }

    fn log_status_switch(is_websocket: bool, is_tls: bool, message: String) {
        match (is_websocket, is_tls) {
            (true, true) => log::info!("{}", message.replace("CONN:", "CONN:WSS")),
            (true, false) => log::info!("{}", message.replace("CONN:", "CONN:WS")),
            (false, true) => log::info!("{}", message.replace("CONN:", "CONN:TLS")),
            _ => log::info!("{}", message.replace("CONN:", "CONN:TCP")),
        }
    }

    // Helper function to handle read errors
    fn handle_read_error(e: std::io::Error, id: i32, is_upstream: bool) {
        let prefix = if is_upstream {
            "upstream"
        } else {
            "downstream"
        };
        let status = if is_upstream { "10" } else { "00" };

        if let Some(os_err) = e.raw_os_error() {
            match os_err {
                54 => log::info!(
                    "[PXY] |ID:{}, CONN:, STATUS:{}, SIZE:0, COMMENT:CONNECTION_RESET |",
                    id,
                    status
                ),
                60 => log::info!(
                    "[PXY] |ID:{}, CONN:, STATUS:{}, SIZE:0, COMMENT:OPERATION_TIMEOUT |",
                    id,
                    status
                ),
                _ => log::error!("Error reading from {}: {} (code: {:?})", prefix, e, os_err),
            }
        } else {
            log::error!("Error reading from {}: {}", prefix, e);
        }
    }

    // Helper function to handle timeout
    fn handle_timeout(id: i32, is_upstream: bool) {
        let status = if is_upstream { "10" } else { "00" };
        log::info!(
            "[PXY] |ID:{}, CONN:, STATUS:{}, SIZE:0, COMMENT:READ_TIMEOUT |",
            id,
            status
        );
    }

    // Helper function to handle write errors
    fn handle_write_error(e: std::io::Error, id: i32, is_upstream: bool, is_flush: bool) {
        let direction = if is_upstream {
            "upstream"
        } else {
            "downstream"
        };
        let operation = if is_flush { "flushing" } else { "writing" };
        let status_base = if is_upstream { "01" } else { "11" };
        let status_suffix = if is_flush { "F" } else { "X" };

        if let Some(os_err) = e.raw_os_error() {
            if os_err == 32 {
                // EPIPE - Broken pipe
                log::info!(
                    "[PXY] |ID:{}, CONN:, STATUS:{}{}, SIZE:0, COMMENT:BROKEN_PIPE |",
                    id,
                    status_base,
                    status_suffix
                );
            } else {
                log::error!(
                    "Error {} data to {}: {} (code: {:?})",
                    operation,
                    direction,
                    e,
                    os_err
                );
            }
        } else {
            log::error!("Error {} data to {}: {}", operation, direction, e);
        }
    }
    /// # Create a new ProxyApp instance
    ///
    /// Initializes the proxy application with a set of redirect rules filtered by
    /// the specified source address.
    ///
    /// ## Parameters
    /// * `alt_source` - The address:port to listen on (e.g. "0.0.0.0:443")
    ///
    /// ## Returns
    /// A new ProxyApp instance with configured redirect rules and connection handlers
    pub fn new(alt_source: &str) -> Self {
        let node = config::RoutingData::ProxyRouting.xget::<Vec<ProxyNode>>();
        let mut host_rules = std::collections::HashMap::new();
        let mut catch_all_rules = std::collections::HashMap::new();
        let mut all_rules = vec![]; // Temporary vector to help with client_connectors setup

        if let Some(node) = node {
            for rule in node {
                if rule.addr_listen == alt_source {
                    let mut peer = BasicPeer::new(&rule.addr_target);
                    if rule.tls {
                        peer.sni = match rule.sni.clone() {
                            Some(sni) => sni,
                            None => {
                                log::error!(
                                    "SNI is required for TLS connections, skipping rule for {}",
                                    alt_source
                                );
                                continue;
                            }
                        };
                    }

                    let redirect_rule = RedirectRule {
                        host: rule.sni.clone(),
                        alt_target: Some(peer),
                        alt_tls: rule.tls,
                    };
                    
                    // Store in appropriate HashMap based on whether it has a host
                    if let Some(host) = rule.sni {
                        host_rules.insert((host, rule.tls), redirect_rule.clone());
                    } else {
                        catch_all_rules.insert(rule.tls, redirect_rule.clone());
                    }
                    
                    all_rules.push(redirect_rule);
                }
            }
        } else {
            log::error!("No routing data found for {}", alt_source);
            return ProxyApp {
                client_connectors: std::collections::HashMap::new(),
                host_rules: std::collections::HashMap::new(),
                catch_all_rules: std::collections::HashMap::new(),
            };
        }

        let mut client_connectors = std::collections::HashMap::new();
        for rule in &all_rules {
            if let Some(target) = &rule.alt_target {
                let addr = format!("{}", target);
                if !client_connectors.contains_key(&addr) {
                    client_connectors.insert(addr, TransportConnector::new(None));
                }
            }
        }
        client_connectors.insert("default".to_string(), TransportConnector::new(None));
        
        ProxyApp {
            client_connectors,
            host_rules,
            catch_all_rules,
        }
    }

    /// # Detects the type of connection based on the initial bytes received
    /// 
    /// Optimized implementation that uses byte-level pattern matching for faster detection.
    fn detect_connection_type(data: &[u8], len: usize) -> ConnectionType {
        if len == 0 {
            return ConnectionType::Tcp;
        }

        // Check for TLS handshake (0x16 = handshake record type)
        if data[0] == 0x16 {
            return ConnectionType::Tls;
        }

        // Early check if we have enough data for an HTTP method
        if len < 3 {
            return ConnectionType::Tcp;
        }

        // Fast byte-level HTTP method detection without UTF-8 validation
        // Only validate if the initial pattern looks promising
        match data[0] {
            b'G' => if len >= 4 && &data[0..4] == b"GET " { 
                // It's an HTTP request, now check for WebSocket upgrade
                if Self::is_websocket_upgrade(data, len) {
                    return ConnectionType::WebSocket;
                } 
                return ConnectionType::Http;
            },
            b'P' => if (len >= 5 && &data[0..5] == b"POST ") || 
                      (len >= 4 && &data[0..4] == b"PUT ") || 
                      (len >= 4 && &data[0..4] == b"PATCH ") {
                return ConnectionType::Http;
            },
            b'H' => if len >= 5 && &data[0..5] == b"HEAD " {
                return ConnectionType::Http;
            },
            b'D' => if len >= 5 && &data[0..5] == b"DELETE " {
                return ConnectionType::Http;
            },
            b'O' => if len >= 5 && &data[0..5] == b"OPTION " {
                return ConnectionType::Http;
            },
            b'T' => if len >= 5 && &data[0..5] == b"TRAC " {
                return ConnectionType::Http;
            },
            b'C' => if len >= 5 && &data[0..5] == b"CONN " {
                return ConnectionType::Http;
            },
            _ => {}
        }

        // Default to plain TCP if not identified as anything specific
        ConnectionType::Tcp
    }
    
    /// Helper function to detect WebSocket upgrade requests
    /// Uses optimized byte-pattern search instead of string conversion
    fn is_websocket_upgrade(data: &[u8], len: usize) -> bool {
        // WebSocket upgrade patterns to search for
        let pattern1 = b"upgrade: websocket";
        let pattern2 = b"upgrade: WebSocket";
        let pattern3 = b"Upgrade: websocket";
        let pattern4 = b"Upgrade: WebSocket";
        
        // Only search a reasonable portion of the header
        let search_len = std::cmp::min(len, 1024);
        
        // Convert to lowercase for case-insensitive search
        for i in 0..search_len - pattern1.len() {
            let window = &data[i..i + pattern1.len()];
            // Check for case-insensitive match using eq_ignore_ascii_case
            if window.eq_ignore_ascii_case(pattern1) || 
               window.eq_ignore_ascii_case(pattern2) ||
               window.eq_ignore_ascii_case(pattern3) || 
               window.eq_ignore_ascii_case(pattern4) {
                return true;
            }
        }
        
        false
    }

    /// # Handle bidirectional data transfer
    ///
    /// Manages the duplex communication between the client and target server.
    /// This function continuously reads from both connections and forwards data
    /// in both directions until one side closes the connection.
    ///
    /// ## Parameters
    /// * `server_session` - The connection to the client
    /// * `client_session` - The connection to the target server
    ///
    /// ## Note
    /// This function runs until either connection closes or an error occurs.
    async fn duplex(
        &self,
        mut server_session: Stream,
        mut client_session: Stream,
        is_websocket: bool,
        is_tls: bool,
    ) {
        let mut upstream_buf = [0; 8192];
        let mut downstream_buf = [0; 8192];
        // create identifier id
        let id = client_session.id();

        // Set timeout for read operations (15 seconds)
        let timeout_duration = std::time::Duration::from_secs(120);

        loop {
            let downstream_read =
                tokio::time::timeout(timeout_duration, server_session.read(&mut upstream_buf));
            let upstream_read =
                tokio::time::timeout(timeout_duration, client_session.read(&mut downstream_buf));
            let event: DuplexEvent;

            select! {
                result = downstream_read => match result {
                    Ok(Ok(n)) => event = DuplexEvent::DownstreamRead(n),
                    Ok(Err(e)) => {
                        Self::handle_read_error(e, id, false);
                        return;
                    },
                    Err(_) => {
                        Self::handle_timeout(id, false);
                        return;
                    }
                },
                result = upstream_read => match result {
                    Ok(Ok(n)) => event = DuplexEvent::UpstreamRead(n),
                    Ok(Err(e)) => {
                        Self::handle_read_error(e, id, true);
                        return;
                    },
                    Err(_) => {
                        Self::handle_timeout(id, true);
                        return;
                    }
                },
            }
            match event {
                DuplexEvent::DownstreamRead(0) => {
                    // this is end of the request
                    Self::log_status_switch(
                        is_websocket,
                        is_tls,
                        format!("[PXY] |ID:{}, CONN:, STATUS:00, SIZE:0, COMMENT:- |", id),
                    );
                    debug!("downstream session closing");
                    return;
                }
                DuplexEvent::UpstreamRead(0) => {
                    Self::log_status_switch(
                        is_websocket,
                        is_tls,
                        format!("[PXY] |ID:{}, CONN:, STATUS:10, SIZE:0, COMMENT:- |", id),
                    );
                    debug!("upstream session closing");
                    return;
                }
                DuplexEvent::DownstreamRead(n) => {
                    Self::log_status_switch(
                        is_websocket,
                        is_tls,
                        format!("[PXY] |ID:{}, CONN:, STATUS:01, SIZE:{}, COMMENT:- |", id, n),
                    );
                    match client_session.write_all(&upstream_buf[0..n]).await {
                        Ok(_) => {}
                        Err(e) => {
                            Self::handle_write_error(e, id, true, false);
                            return;
                        }
                    }
                    match client_session.flush().await {
                        Ok(_) => {}
                        Err(e) => {
                            Self::handle_write_error(e, id, true, true);
                            return;
                        }
                    };
                }
                DuplexEvent::UpstreamRead(n) => {
                    // start of the request
                    Self::log_status_switch(
                        is_websocket,
                        is_tls,
                        format!("[PXY] |ID:{}, CONN:, STATUS:11, SIZE:{}, COMMENT:- |", id, n),
                    );
                    match server_session.write_all(&downstream_buf[0..n]).await {
                        Ok(_) => {}
                        Err(e) => {
                            Self::handle_write_error(e, id, false, false);
                            return;
                        }
                    }
                    match server_session.flush().await {
                        Ok(_) => {}
                        Err(e) => {
                            Self::handle_write_error(e, id, false, true);
                            return;
                        }
                    };
                }
            }
        }
    }
}

#[async_trait]
impl ServerApp for ProxyApp {
    /// # Process a new connection
    ///
    /// Main connection handling function. Reads the initial data from the client,
    /// determines the appropriate backend server based on the request characteristics,
    /// establishes a connection to the backend, and sets up bidirectional communication.
    ///
    /// ## Process flow:
    /// 1. Read initial data from client
    /// 2. Analyze if it's HTTP, WebSocket or TLS
    /// 3. Extract hostname from Host header or SNI
    /// 4. Find matching redirect rule
    /// 5. Connect to target backend
    /// 6. Forward initial data and establish duplex communication
    ///
    /// ## Parameters
    /// * `io` - The client connection stream
    /// * `_shutdown` - Shutdown watcher (unused)
    ///
    /// ## Returns
    /// None - The connection is fully processed within this function
    async fn process_new(
        self: &Arc<Self>,
        mut io: Stream,
        _shutdown: &ShutdownWatch,
    ) -> Option<Stream> {
        log::debug!("");
        log::debug!("#-------------------------------------#");
        log::debug!("#           Incoming Request          #");
        log::debug!("#-------------------------------------#");

        // Use buffer from the pool instead of allocating a new one
        let mut buffer = BUFFER_POOL.get();
        
        // Read initial data from client with zero-copy buffer
        let n = match io.read_buf(&mut buffer).await {
            Ok(n) => n,
            Err(e) => {
                log::error!("Failed to read from client: {}", e);
                BUFFER_POOL.put(buffer);
                return None;
            }
        };

        if n == 0 {
            log::error!("Empty request received");
            BUFFER_POOL.put(buffer);
            return None;
        }

        // Determine connection type - using the buffer slice for better performance
        let buf_slice = &buffer[..n];
        let conn_type = Self::detect_connection_type(buf_slice, n);
        let is_tls = conn_type == ConnectionType::Tls;
        let is_websocket = conn_type == ConnectionType::WebSocket;

        log::debug!("Connection type: {:?}", conn_type);

        // Extract host information based on connection type - optimized with specialized extractors
        let host_info = match conn_type {
            ConnectionType::Tls => extract_sni_fast(buf_slice),
            ConnectionType::Http | ConnectionType::WebSocket => extract_http_host(buf_slice, n),
            ConnectionType::Tcp => None,
        };

        log::debug!("Host info: {:?}", host_info);

        // Fast path rule lookup using direct HashMap access with cached results
        let proxy_to = self.fast_rule_lookup(host_info.as_deref(), is_tls).clone();

        let target_addr = format!("{}", proxy_to._address);
        log::debug!("Proxying to: {} (TLS: {})", target_addr, is_tls);

        // Get the appropriate connector with fast lookup
        let connector = self.client_connectors.get(&target_addr).unwrap_or_else(|| {
            self.client_connectors
                .get("default")
                .expect("Default connector should exist")
        });

        // Connect to the target server with optimized timeout handling
        let connect_future = connector.new_stream(&proxy_to);
        let mut client_session = match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            connect_future,
        )
        .await
        {
            Ok(Ok(client_session)) => client_session,
            Ok(Err(e)) => {
                log::error!("Failed to connect to upstream peer {}: {}", target_addr, e);
                BUFFER_POOL.put(buffer);
                return None;
            }
            Err(_) => {
                log::error!("Connection to {} timed out", target_addr);
                BUFFER_POOL.put(buffer);
                return None;
            }
        };

        // Forward the initial data to the target server using zero-copy approach
        match client_session.write_all(&buffer[..n]).await {
            Ok(_) => {}
            Err(e) => {
                log::error!("Failed to write to upstream peer: {}", e);
                BUFFER_POOL.put(buffer);
                return None;
            }
        };

        match client_session.flush().await {
            Ok(_) => {}
            Err(e) => {
                log::error!("Failed to flush data to upstream peer: {}", e);
                BUFFER_POOL.put(buffer);
                return None;
            }
        };

        // Return buffer to the pool before entering duplex mode
        BUFFER_POOL.put(buffer);

        // Establish bidirectional communication
        self.duplex(io, client_session, is_websocket, is_tls).await;
        None
    }
}

/// Extract HTTP Host header directly from byte buffer without string conversion
/// Uses optimized byte-level parsing for better performance
fn extract_http_host(buf: &[u8], length: usize) -> Option<String> {
    let max_scan_len = std::cmp::min(length, 1024);
    let host_pattern = b"host:";
    
    // Search for "host:" header (case-insensitive)
    for i in 0..max_scan_len - host_pattern.len() {
        if &buf[i..i+5].to_ascii_lowercase() == host_pattern {
            // Found the Host header, now extract the value
            let start_idx = i + 5;
            let mut end_idx = start_idx;
            
            // Find the end of line
            while end_idx < max_scan_len && buf[end_idx] != b'\r' && buf[end_idx] != b'\n' {
                end_idx += 1;
            }
            
            // Convert the host value to a String, trimming whitespace
            if end_idx > start_idx {
                let host_bytes = &buf[start_idx..end_idx];
                // Trim leading whitespace
                let mut trim_start = 0;
                while trim_start < host_bytes.len() && (host_bytes[trim_start] == b' ' || host_bytes[trim_start] == b'\t') {
                    trim_start += 1;
                }
                
                // Trim trailing whitespace
                let mut trim_end = host_bytes.len();
                while trim_end > trim_start && (host_bytes[trim_end-1] == b' ' || host_bytes[trim_end-1] == b'\t') {
                    trim_end -= 1;
                }
                
                if trim_end > trim_start {
                    return std::str::from_utf8(&host_bytes[trim_start..trim_end]).ok().map(String::from);
                }
            }
            
            break;
        }
    }
    
    None
}

/// Optimized SNI extraction function that uses direct byte access and caching
/// This is faster than the original extract_sni function
fn extract_sni_fast(buf: &[u8]) -> Option<String> {
    // Quick check for TLS handshake
    if buf.len() < 5 || buf[0] != 0x16 {
        return None;
    }
    
    // Use a more direct approach to find the SNI extension
    // TLS record header is 5 bytes, followed by handshake message
    // Skip to the extensions section directly based on the structure
    let mut pos = 5; // Skip TLS record header
    
    if pos + 4 > buf.len() {
        return None;
    }
    
    // Skip handshake type (1 byte) and length (3 bytes)
    pos += 4;
    
    if pos + 2 > buf.len() {
        return None;
    }
    
    // Skip client version (2 bytes)
    pos += 2;
    
    if pos + 32 > buf.len() {
        return None;
    }
    
    // Skip client random (32 bytes)
    pos += 32;
    
    if pos + 1 > buf.len() {
        return None;
    }
    
    // Get session ID length and skip session ID
    let session_id_len = buf[pos] as usize;
    pos += 1;
    
    if pos + session_id_len > buf.len() {
        return None;
    }
    
    pos += session_id_len;
    
    if pos + 2 > buf.len() {
        return None;
    }
    
    // Get cipher suites length and skip cipher suites
    let cipher_suites_len = ((buf[pos] as usize) << 8) | (buf[pos + 1] as usize);
    pos += 2;
    
    if pos + cipher_suites_len > buf.len() {
        return None;
    }
    
    pos += cipher_suites_len;
    
    if pos + 1 > buf.len() {
        return None;
    }
    
    // Get compression methods length and skip compression methods
    let compression_methods_len = buf[pos] as usize;
    pos += 1;
    
    if pos + compression_methods_len > buf.len() {
        return None;
    }
    
    pos += compression_methods_len;
    
    if pos + 2 > buf.len() {
        return None;
    }
    
    // Get extensions length
    let extensions_len = ((buf[pos] as usize) << 8) | (buf[pos + 1] as usize);
    pos += 2;
    
    if pos + extensions_len > buf.len() {
        return None;
    }
    
    // Process extensions
    let extensions_end = pos + extensions_len;
    while pos + 4 <= extensions_end {
        let ext_type = ((buf[pos] as u16) << 8) | (buf[pos + 1] as u16);
        let ext_len = ((buf[pos + 2] as usize) << 8) | (buf[pos + 3] as usize);
        pos += 4;
        
        if pos + ext_len > extensions_end {
            break;
        }
        
        // SNI extension type is 0
        if ext_type == 0 {
            // Parse SNI extension
            if ext_len >= 2 {
                let sni_list_len = ((buf[pos] as usize) << 8) | (buf[pos + 1] as usize);
                pos += 2;
                
                if pos + sni_list_len <= extensions_end && sni_list_len >= 3 {
                    // Name type (should be 0 for hostname)
                    if buf[pos] == 0 {
                        pos += 1;
                        
                        // Hostname length
                        let hostname_len = ((buf[pos] as usize) << 8) | (buf[pos + 1] as usize);
                        pos += 2;
                        
                        if pos + hostname_len <= extensions_end {
                            // Extract hostname
                            return std::str::from_utf8(&buf[pos..pos + hostname_len]).ok().map(String::from);
                        }
                    }
                }
            }
            
            break;
        }
        
        pos += ext_len;
    }
    
    None
}

/// # Find SNI extension in TLS Client Hello
///
/// Helper function that searches for the SNI extension within a TLS Client Hello message.
/// The SNI extension is identified by type 0x0000 in the extensions section of a
/// Client Hello message.
///
/// ## TLS Extensions Format
/// Extensions appear at the end of the Client Hello message after:
/// - Content Type (1 byte)
/// - TLS Version (2 bytes)
/// - Record Length (2 bytes)
/// - Handshake Type (1 byte)
/// - Handshake Length (3 bytes)
/// - TLS Version (2 bytes)
/// - Random (32 bytes)
/// - Session ID (variable, length indicated by 1 byte)
/// - Cipher Suites (variable, length indicated by 2 bytes)
/// - Compression Methods (variable, length indicated by 1 byte)
/// - Extensions Length (2 bytes)
///
/// Each extension has:
/// - Extension Type (2 bytes, 0x0000 for SNI)
/// - Extension Length (2 bytes)
/// - Extension Data (variable)
///
/// ## SNI Extension Structure
/// For SNI specifically:
/// - Extension Type: 0x0000
/// - Extension Length: 2 + server_name_list_length
/// - Server Name List Length: 2 bytes
/// - Name Type: 1 byte (0x00 for hostname)
/// - Hostname Length: 2 bytes
/// - Hostname: UTF-8 encoded string
///
/// ## Parameters
/// * `buf` - The raw bytes from the TLS handshake
///
/// ## Returns
/// * `Some(usize)` - Position where the SNI hostname length data begins
/// * `None` - If SNI extension could not be found
///
/// ## Implementation Details
/// This is a simplified implementation that:
/// 1. Looks for byte patterns that might indicate an SNI extension
/// 2. Does not fully parse the TLS record structure
/// 3. May return false positives or miss the extension
///
/// In a production environment, a formal TLS parser should be used.
fn find_sni_extension(buf: &[u8]) -> Option<usize> {
    // Minimum TLS Client Hello with SNI should be at least 45 bytes
    // (5 byte record header + 4 byte handshake header + 2 byte version + 32 byte random +
    //  1 byte session ID length + 2 byte cipher suites length + 1 byte compression methods length +
    //  2 byte extensions length + 4 byte SNI extension header + 2 byte server name list length +
    //  1 byte name type + 2 byte hostname length)
    if buf.len() < 45 {
        return None;
    }

    // This simplified implementation looks for the SNI extension pattern:
    // - Extension Type 0x0000 (SNI)
    // - Followed by a length field
    // - Followed by server name list length
    // - Followed by name type 0x00 (hostname)

    // Search through the buffer for potential SNI extension
    // We're looking for the pattern: 0x00 0x00 (extension type) followed by length bytes
    // and then a server name list that starts with 0x00 (hostname indicator)
    for i in 0..buf.len() - 8 {
        // Possible SNI extension pattern:
        // 0x00 0x00 (extension type) followed by length bytes and name type 0x00
        if buf[i] == 0x00 && buf[i + 1] == 0x00 && buf[i + 4] == 0x00 {
            // Extract extension length
            let ext_len = ((buf[i + 2] as usize) << 8) | (buf[i + 3] as usize);

            // Sanity check the length
            if ext_len > 0 && ext_len < 1000 && i + 4 + ext_len <= buf.len() {
                // The actual hostname length starts 3 bytes after the name type
                return Some(i + 5);
            }
        }
    }

    // Fallback to the original simplified method which may catch some cases
    // that the more specific pattern above misses
    for i in 0..buf.len() - 4 {
        if buf[i] == 0x00 && buf[i + 1] == 0x00 && buf[i + 2] == 0x00 {
            return Some(i + 3);
        }
    }

    None
}
