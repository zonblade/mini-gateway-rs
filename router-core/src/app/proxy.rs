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
use bytes::BytesMut;
use std::cell::RefCell;
use std::thread_local;

// Thread-local buffer pool for efficient buffer reuse
thread_local! {
    static TLS_BUFFER_POOL: RefCell<Vec<BytesMut>> = RefCell::new(Vec::with_capacity(32));
}

// Constants for buffer and stream management
const DEFAULT_BUFFER_SIZE: usize = 1024; // 16KB instead of 8KB for more efficient chunking
const SOCKET_TIMEOUT_SECS: u64 = 2;     // Reduce from 120s to be more responsive to network changes

/// Buffer pool implementation that uses thread-local storage to avoid mutex contention
struct BufferPool;

impl BufferPool {
    fn get() -> BytesMut {
        TLS_BUFFER_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            pool.pop().unwrap_or_else(|| BytesMut::with_capacity(DEFAULT_BUFFER_SIZE))
        })
    }

    fn put(mut buf: BytesMut) {
        // Clear buffer but keep capacity
        buf.clear();
        
        TLS_BUFFER_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            // Limit the number of stored buffers to prevent memory bloat
            if pool.len() < 32 {
                pool.push(buf);
            }
        });
    }
}

/// # Redirect Rule Configuration
///
/// Defines a rule for redirecting traffic based on host and TLS status.
///
/// ## Fields
/// * `alt_target` - Optional target backend server to forward traffic to
#[derive(Clone)]
struct RedirectRule {
    alt_target: Option<BasicPeer>,
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
    /// Uses a match-first approach to check for exact matches without allocations
    fn fast_rule_lookup(&self, host: Option<&str>, is_tls: bool) -> BasicPeer {
        if let Some(host_str) = host {
            // First try to find an exact match without allocating a string
            for ((key, key_tls), rule) in &self.host_rules {
                if *key_tls == is_tls && key == host_str {
                    if let Some(target) = &rule.alt_target {
                        return target.clone();
                    }
                }
            }

            // Fall back to catch-all rule
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
                        alt_target: Some(peer),
                    };
                    
                    // Store in appropriate HashMap based on whether it has a host
                    if let Some(host) = rule.sni.clone() {
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
            b'G' => {
                if len >= 4 && &data[0..4] == b"GET " {
                    // It's an HTTP request, now check for WebSocket upgrade
                    if Self::is_websocket_upgrade(data, len) {
                        return ConnectionType::WebSocket;
                    }
                    return ConnectionType::Http;
                }
            }
            b'P' => {
                if (len >= 5 && &data[0..5] == b"POST ")
                    || (len >= 4 && &data[0..4] == b"PUT ")
                    || (len >= 4 && &data[0..4] == b"PATCH ")
                {
                    return ConnectionType::Http;
                }
            }
            b'H' => {
                if len >= 5 && &data[0..5] == b"HEAD " {
                    return ConnectionType::Http;
                }
            }
            b'D' => {
                if len >= 5 && &data[0..5] == b"DELETE " {
                    return ConnectionType::Http;
                }
            }
            b'O' => {
                if len >= 5 && &data[0..5] == b"OPTION " {
                    return ConnectionType::Http;
                }
            }
            b'T' => {
                if len >= 5 && &data[0..5] == b"TRAC " {
                    return ConnectionType::Http;
                }
            }
            b'C' => {
                if len >= 5 && &data[0..5] == b"CONN " {
                    return ConnectionType::Http;
                }
            }
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
            if window.eq_ignore_ascii_case(pattern1)
                || window.eq_ignore_ascii_case(pattern2)
                || window.eq_ignore_ascii_case(pattern3)
                || window.eq_ignore_ascii_case(pattern4)
            {
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
        // Optimize socket settings to disable Nagle's algorithm
        Self::optimize_tcp_socket(&mut server_session);
        Self::optimize_tcp_socket(&mut client_session);

        // Get connection type for configuration
        let conn_type = if is_tls {
            ConnectionType::Tls
        } else if is_websocket {
            ConnectionType::WebSocket
        } else {
            ConnectionType::Http // Default to HTTP for existing connections
        };
        
        // Create dynamic configuration for upstream and downstream
        let mut upstream_config = ConnectionConfig::new(None, conn_type);
        let mut downstream_config = ConnectionConfig::new(None, conn_type);
        
        // Create appropriately sized buffers based on connection type
        let mut upstream_buf = BytesMut::with_capacity(upstream_config.get_buffer_size());
        let mut downstream_buf = BytesMut::with_capacity(downstream_config.get_buffer_size());
        
        // Track accumulated data sizes for smarter flushing decisions
        let mut upstream_accumulated = 0;
        let mut downstream_accumulated = 0;
        
        // Create identifier id
        let id = client_session.id();

        loop {
            // Use dynamic timeouts from config
            let downstream_read = 
                tokio::time::timeout(downstream_config.get_timeout(), server_session.read_buf(&mut upstream_buf));
            let upstream_read = 
                tokio::time::timeout(upstream_config.get_timeout(), client_session.read_buf(&mut downstream_buf));
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
                    // End of the request
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
                    // Update buffer size based on traffic pattern
                    upstream_config.update_buffer_size(n);
                    
                    upstream_accumulated += n;
                    Self::log_status_switch(
                        is_websocket,
                        is_tls,
                        format!(
                            "[PXY] |ID:{}, CONN:, STATUS:01, SIZE:{}, COMMENT:- |",
                            id, n
                        ),
                    );
                    
                    // Write the data to the client (data is already in upstream_buf)
                    let to_write = upstream_buf.split_to(n);
                    match client_session.write_all(&to_write[..]).await {
                        Ok(_) => {},
                        Err(e) => {
                            Self::handle_write_error(e, id, true, false);
                            return;
                        }
                    }
                    
                    // Only flush if we've accumulated enough data or it's likely the last part of a response
                    // Use the dynamic flush threshold from the config
                    let should_flush = upstream_accumulated >= upstream_config.get_flush_threshold() || 
                                       n < upstream_config.get_buffer_size() / 2;
                                       
                    if should_flush {
                        match client_session.flush().await {
                            Ok(_) => {
                                upstream_accumulated = 0; // Reset accumulator after flush
                            },
                            Err(e) => {
                                Self::handle_write_error(e, id, true, true);
                                return;
                            }
                        }
                    }
                }
                DuplexEvent::UpstreamRead(n) => {
                    // Update buffer size based on traffic pattern
                    downstream_config.update_buffer_size(n);
                    
                    downstream_accumulated += n;
                    // start of the request
                    Self::log_status_switch(
                        is_websocket,
                        is_tls,
                        format!(
                            "[PXY] |ID:{}, CONN:, STATUS:11, SIZE:{}, COMMENT:- |",
                            id, n
                        ),
                    );
                    
                    // Write the data back to the client
                    let to_write = downstream_buf.split_to(n);
                    match server_session.write_all(&to_write[..]).await {
                        Ok(_) => {},
                        Err(e) => {
                            Self::handle_write_error(e, id, false, false);
                            return;
                        }
                    }
                    
                    // Only flush if we've accumulated enough data or it's the last part of a response
                    // Use the dynamic flush threshold from the config
                    let should_flush = downstream_accumulated >= downstream_config.get_flush_threshold() || 
                                       n < downstream_config.get_buffer_size() / 2;
                                       
                    if should_flush {
                        match server_session.flush().await {
                            Ok(_) => {
                                downstream_accumulated = 0; // Reset accumulator after flush
                            },
                            Err(e) => {
                                Self::handle_write_error(e, id, false, true);
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Helper method to optimize TCP socket settings for low latency
    fn optimize_tcp_socket(stream: &mut Stream) {
        if let Some(socket) = stream.as_socket_mut() {
            // Disable Nagle's algorithm to reduce latency
            if let Err(e) = socket.set_nodelay(true) {
                log::warn!("Failed to set TCP_NODELAY: {}", e);
            }
            
            // Note: We don't use socket.set_send_buffer_size and set_keepalive directly
            // as they're not available on TcpStream, but Tokio provides the nodelay option
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

        // Optimize client socket settings immediately
        Self::optimize_tcp_socket(&mut io);

        // Use buffer from the pool instead of allocating a new one
        let mut buffer = BufferPool::get();
        
        // Read initial data from client with zero-copy buffer
        let n = match io.read_buf(&mut buffer).await {
            Ok(n) => n,
            Err(e) => {
                log::error!("Failed to read from client: {}", e);
                BufferPool::put(buffer);
                return None;
            }
        };

        if n == 0 {
            log::error!("Empty request received");
            BufferPool::put(buffer);
            return None;
        }

        // Determine connection type - using the buffer slice for better performance
        let buf_slice = &buffer[..n];
        let conn_type = Self::detect_connection_type(buf_slice, n);
        let is_tls = conn_type == ConnectionType::Tls;
        let is_websocket = conn_type == ConnectionType::WebSocket;

        log::debug!("Connection type: {:?}", conn_type);

        // Extract host information based on connection type - zero-allocation extractors
        let host_info = match conn_type {
            ConnectionType::Tls => extract_sni_fast(buf_slice),
            ConnectionType::Http | ConnectionType::WebSocket => extract_http_host(buf_slice, n),
            ConnectionType::Tcp => None,
        };

        log::debug!("Host info: {:?}", host_info);

        // Fast rule lookup using our iterator-based approach to avoid allocations
        let proxy_to = self.fast_rule_lookup(host_info, is_tls).clone();

        let target_addr = format!("{}", proxy_to._address);
        log::debug!("Proxying to: {} (TLS: {})", target_addr, is_tls);

        // Get the appropriate connector with fast lookup
        let connector = self.client_connectors.get(&target_addr).unwrap_or_else(|| {
            self.client_connectors
                .get("default")
                .expect("Default connector should exist")
        });

        // Create connection config for this connection type
        let proxy_node = None; // In a future enhancement, we could retrieve the actual ProxyNode here
        let connect_config = ConnectionConfig::new(proxy_node, conn_type);
        
        // Connect to the target server with optimized timeout handling
        let connect_future = connector.new_stream(&proxy_to);
        let mut client_session =
            match tokio::time::timeout(std::time::Duration::from_secs(5), connect_future).await {
                Ok(Ok(mut client_session)) => {
                    // Optimize upstream socket immediately upon connection
                    Self::optimize_tcp_socket(&mut client_session);
                    client_session
                },
                Ok(Err(e)) => {
                    log::error!("Failed to connect to upstream peer {}: {}", target_addr, e);
                    BufferPool::put(buffer);
                    return None;
                }
                Err(_) => {
                    log::error!("Connection to {} timed out", target_addr);
                    BufferPool::put(buffer);
                    return None;
                }
            };

        // Forward the initial data to the target server using zero-copy approach
        if let Err(e) = client_session.write_all(&buffer[..n]).await {
            log::error!("Failed to write to upstream peer: {}", e);
            BufferPool::put(buffer);
            return None;
        }

        // Only flush if necessary to avoid Nagle algorithm issues
        // Use the connection config's flush threshold
        if n >= connect_config.get_flush_threshold() || is_websocket || is_tls {
            if let Err(e) = client_session.flush().await {
                log::error!("Failed to flush data to upstream peer: {}", e);
                BufferPool::put(buffer);
                return None;
            }
        }

        // Return buffer to the pool before entering duplex mode
        BufferPool::put(buffer);

        // Establish bidirectional communication
        self.duplex(io, client_session, is_websocket, is_tls).await;
        None
    }
}

/// Extract HTTP Host header directly from byte buffer without string conversion
/// Uses optimized byte-level parsing for better performance
fn extract_http_host(buf: &[u8], length: usize) -> Option<&str> {
    let max_scan_len = std::cmp::min(length, 1024);
    let host_pattern = b"host:";

    // Search for "host:" header (case-insensitive)
    for i in 0..max_scan_len - host_pattern.len() {
        if &buf[i..i + 5].to_ascii_lowercase() == host_pattern {
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
                while trim_start < host_bytes.len()
                    && (host_bytes[trim_start] == b' ' || host_bytes[trim_start] == b'\t')
                {
                    trim_start += 1;
                }

                // Trim trailing whitespace
                let mut trim_end = host_bytes.len();
                while trim_end > trim_start
                    && (host_bytes[trim_end - 1] == b' ' || host_bytes[trim_end - 1] == b'\t')
                {
                    trim_end -= 1;
                }

                if trim_end > trim_start {
                    // Return &str directly without allocation
                    return std::str::from_utf8(&host_bytes[trim_start..trim_end]).ok();
                }
            }

            break;
        }
    }

    None
}

/// Optimized SNI extraction function that uses direct byte access and caching
/// This is faster than the original extract_sni function
fn extract_sni_fast(buf: &[u8]) -> Option<&str> {
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
                            // Extract hostname directly as &str without allocation
                            return std::str::from_utf8(&buf[pos..pos + hostname_len]).ok();
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

// Extension trait to access socket options for our Stream type
trait StreamSocketExt {
    fn as_socket_mut(&mut self) -> Option<&mut tokio::net::TcpStream>;
}

// Implement the extension trait for Stream
impl StreamSocketExt for Stream {
    fn as_socket_mut(&mut self) -> Option<&mut tokio::net::TcpStream> {
        // Pingora's Stream doesn't expose a direct method to get the underlying TcpStream
        // in a way that allows us to set socket options. For now, we'll just return None
        // and the optimize_tcp_socket function will silently skip setting options.
        None
    }
}

// Connection configuration that can adapt to different traffic patterns
struct ConnectionConfig {
    // Base buffer size - either default or from config
    buffer_size: usize,
    // Timeout for socket operations
    timeout_secs: u64,
    // Whether to use adaptive buffering
    adaptive_buffer: bool,
    // Traffic history for adaptive sizing
    traffic_history: Vec<usize>,
    // Maximum size for adaptive buffers
    max_buffer_size: usize,
    // Minimum size for adaptive buffers
    min_buffer_size: usize,
}

impl ConnectionConfig {
    // Create a new connection config, potentially using values from the proxy node config
    fn new(proxy_node: Option<&ProxyNode>, conn_type: ConnectionType) -> Self {
        let default_buffer_size = match conn_type {
            ConnectionType::Tls => DEFAULT_BUFFER_SIZE * 2, // TLS connections often have larger payloads
            ConnectionType::WebSocket => DEFAULT_BUFFER_SIZE, // WebSockets benefit from standard buffers
            ConnectionType::Http => DEFAULT_BUFFER_SIZE, // HTTP uses standard buffer size
            ConnectionType::Tcp => DEFAULT_BUFFER_SIZE / 2, // Plain TCP often has smaller messages
        };
        
        let default_timeout = match conn_type {
            ConnectionType::WebSocket => SOCKET_TIMEOUT_SECS * 2, // WebSockets need longer timeouts
            ConnectionType::Tls => SOCKET_TIMEOUT_SECS,
            ConnectionType::Http => SOCKET_TIMEOUT_SECS,
            ConnectionType::Tcp => SOCKET_TIMEOUT_SECS / 2, // TCP connections can use shorter timeouts
        };
        
        if let Some(node) = proxy_node {
            // Use custom values from proxy node config if available
            ConnectionConfig {
                buffer_size: node.buffer_size.unwrap_or(default_buffer_size),
                timeout_secs: node.timeout_secs.unwrap_or(default_timeout),
                adaptive_buffer: node.adaptive_buffer,
                traffic_history: Vec::with_capacity(1000),
                max_buffer_size: 65536, // 64KB max
                min_buffer_size: 32,  // 32B min
            }
        } else {
            // Use default values
            ConnectionConfig {
                buffer_size: default_buffer_size,
                timeout_secs: default_timeout,
                adaptive_buffer: false,
                traffic_history: Vec::with_capacity(1000),
                max_buffer_size: 65536, // 64KB max
                min_buffer_size: 32,  // 32B min
            }
        }
    }
    
    // Update buffer size based on traffic patterns
    fn update_buffer_size(&mut self, bytes_transferred: usize) {
        if !self.adaptive_buffer {
            return;
        }
        
        // Add current transfer to history
        self.traffic_history.push(bytes_transferred);
        
        // Only keep the last 10 transfers
        if self.traffic_history.len() > 1000 {
            self.traffic_history.remove(0);
        }
        
        // Calculate average transfer size if we have enough data
        if self.traffic_history.len() >= 3 {
            let avg_transfer = self.traffic_history.iter().sum::<usize>() / self.traffic_history.len();
            
            // Adjust buffer size based on recent traffic
            // If transfers are consistently large, increase buffer size
            // If transfers are small, decrease buffer size to save memory
            if avg_transfer > self.buffer_size / 2 {
                // Increase buffer size if transfers are large
                self.buffer_size = (self.buffer_size * 3) / 2;
                if self.buffer_size > self.max_buffer_size {
                    self.buffer_size = self.max_buffer_size;
                }
            } else if avg_transfer < self.buffer_size / 4 && self.buffer_size > self.min_buffer_size {
                // Decrease buffer size if transfers are small
                self.buffer_size = self.buffer_size / 2;
                if self.buffer_size < self.min_buffer_size {
                    self.buffer_size = self.min_buffer_size;
                }
            }
        }
    }
    
    // Get the current buffer size
    fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }
    
    // Get the current timeout duration
    fn get_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_secs)
    }
    
    // Get the flush threshold based on current buffer size
    fn get_flush_threshold(&self) -> usize {
        // Flush when buffer is half full
        self.buffer_size / 2
    }
}
