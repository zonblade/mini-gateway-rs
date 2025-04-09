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
/// * `redirects` - List of redirect rules for determining where to send traffic
pub struct ProxyApp {
    client_connectors: std::collections::HashMap<String, TransportConnector>,
    redirects: Vec<RedirectRule>,
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

impl ProxyApp {
    // Helper function to handle read errors
    fn handle_read_error(e: std::io::Error, id: i32, is_upstream: bool) {
        let prefix = if is_upstream { "upstream" } else { "downstream" };
        let status = if is_upstream { "10" } else { "00" };
        
        if let Some(os_err) = e.raw_os_error() {
            match os_err {
                54 => log::info!("|ID:{}, STATUS:{}, SIZE:0, COMMENT:CONNECTION_RESET |", id, status),
                60 => log::info!("|ID:{}, STATUS:{}, SIZE:0, COMMENT:OPERATION_TIMEOUT |", id, status),
                _ => log::error!("Error reading from {}: {} (code: {:?})", prefix, e, os_err),
            }
        } else {
            log::error!("Error reading from {}: {}", prefix, e);
        }
    }
    
    // Helper function to handle timeout
    fn handle_timeout(id: i32, is_upstream: bool) {
        let status = if is_upstream { "10" } else { "00" };
        log::info!("|ID:{}, STATUS:{}, SIZE:0, COMMENT:READ_TIMEOUT |", id, status);
    }
    
    // Helper function to handle write errors
    fn handle_write_error(e: std::io::Error, id: i32, is_upstream: bool, is_flush: bool) {
        let direction = if is_upstream { "upstream" } else { "downstream" };
        let operation = if is_flush { "flushing" } else { "writing" };
        let status_base = if is_upstream { "01" } else { "11" };
        let status_suffix = if is_flush { "F" } else { "X" };
        
        if let Some(os_err) = e.raw_os_error() {
            if os_err == 32 { // EPIPE - Broken pipe
                log::info!("|ID:{}, STATUS:{}{}, SIZE:0, COMMENT:BROKEN_PIPE |", id, status_base, status_suffix);
            } else {
                log::error!("Error {} data to {}: {} (code: {:?})", operation, direction, e, os_err);
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
        let mut redirects = vec![];

        if let Some(node) = node {
            for rule in node {
                if rule.addr_listen == alt_source {
                    let mut peer = BasicPeer::new(&rule.addr_target);
                    if rule.tls {
                        peer.sni = match rule.sni.clone() {
                            Some(sni) => sni,
                            None => {
                                log::error!("SNI is required for TLS connections, skipping rule for {}", alt_source);
                                continue;
                            }
                        };
                    }

                    let redirect_rule = RedirectRule {
                        host: rule.sni,
                        alt_target: Some(peer),
                        alt_tls: rule.tls,
                    };
                    redirects.push(redirect_rule);
                }
            }
        } else {
            log::error!("No routing data found for {}", alt_source);
            return ProxyApp {
                client_connectors: std::collections::HashMap::new(),
                redirects: vec![],
            };
        }

        
        let mut client_connectors = std::collections::HashMap::new();
        for rule in &redirects {
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
            redirects,
        }
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
    async fn duplex(&self, mut server_session: Stream, mut client_session: Stream) {
        let mut upstream_buf = [0; 8192];
        let mut downstream_buf = [0; 8192];
        // create identifier id
        let id = client_session.id();
        
        // Set timeout for read operations (15 seconds)
        let timeout_duration = std::time::Duration::from_secs(120);

        loop {
            let downstream_read = tokio::time::timeout(
                timeout_duration, 
                server_session.read(&mut upstream_buf)
            );
            let upstream_read = tokio::time::timeout(
                timeout_duration,
                client_session.read(&mut downstream_buf)
            );
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
                    log::info!("|ID:{}, STATUS:00, SIZE:0, COMMENT:- |", id);
                    debug!("downstream session closing");
                    return;
                }
                DuplexEvent::UpstreamRead(0) => {
                    log::info!("|ID:{}, STATUS:10, SIZE:0, COMMENT:- |", id);
                    debug!("upstream session closing");
                    return;
                }
                DuplexEvent::DownstreamRead(n) => {
                    log::info!("|ID:{}, STATUS:01, SIZE:{}, COMMENT:- |", id, n);
                    match client_session.write_all(&upstream_buf[0..n]).await {
                        Ok(_) => {},
                        Err(e) => {
                            Self::handle_write_error(e, id, true, false);
                            return;
                        }
                    }
                    match client_session.flush().await {
                        Ok(_) => {},
                        Err(e) => {
                            Self::handle_write_error(e, id, true, true);
                            return;
                        }
                    };
                }
                DuplexEvent::UpstreamRead(n) => {
                    log::info!("|ID:{}, STATUS:11, SIZE:{}, COMMENT:- |", id, n);
                    match server_session.write_all(&downstream_buf[0..n]).await {
                        Ok(_) => {},
                        Err(e) => {
                            Self::handle_write_error(e, id, false, false);
                            return;
                        }
                    }
                    match server_session.flush().await {
                        Ok(_) => {},
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
        let mut buf = [0; 8192]; // Increased buffer size for larger headers
        let n = match io.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                log::error!("Failed to read from client: {}", e);
                return None;
            }
        };

        if n == 0 {
            log::error!("Empty request received");
            return None;
        }

        let preview = String::from_utf8_lossy(&buf[..std::cmp::min(n, 200)]);
        let first_line = preview.lines().next().unwrap_or("Empty request");
        log::debug!("Request preview : {}", first_line);

        // In your process_new implementation, modify the host extraction:
        // Determine if this is a TLS connection based on the first byte
        let is_tls = n > 0 && buf[0] == 0x16;
        log::debug!(
            "Connection type : {}",
            if is_tls { "TLS" } else { "Plain HTTP" }
        );

        // Extract the host header (only for non-TLS connections)
        let host_header = if !is_tls {
            preview.lines().find_map(|line| {
                if line.to_lowercase().starts_with("host:") {
                    Some(line[5..].trim().to_string())
                } else {
                    None
                }
            })
        } else {
            None
        };

        log::debug!("Host header     : {:?}", host_header);

        // Check for WebSocket upgrade
        let is_websocket = !is_tls
            && preview
                .lines()
                .any(|line| line.to_lowercase().contains("upgrade: websocket"));
        if is_websocket {
            log::debug!("Upgrade request : WebSocket");
        }

        // Extract host information
        let host_info = if is_tls {
            // For TLS, try to extract SNI from the Client
            extract_sni(&buf[0..n])
        } else {
            // For plain HTTP, extract Host header
            preview.lines().find_map(|line| {
                if line.to_lowercase().starts_with("host:") {
                    Some(line[5..].trim().to_string())
                } else {
                    None
                }
            })
        };

        log::debug!("Host info       : {:?}", host_info);

        // Find matching redirect rule based on host info and TLS status
        let proxy_to = if let Some(host) = host_info {
            // First try to find a rule with exact host match
            let host_match = self.redirects
                .iter()
                .find(|rule| {
                    rule.host.as_ref().map_or(false, |h| h == &host) && rule.alt_tls == is_tls
                });
            
            if host_match.is_some() {
                // We have a specific host match
                host_match
            } else {
                // Try to find a catch-all rule (host: None) with matching TLS status
                self.redirects
                    .iter()
                    .find(|rule| rule.host.is_none() && rule.alt_tls == is_tls)
            }
        } else {
            // No host info, just match on TLS status
            self.redirects
                .iter()
                .find(|rule| rule.host.is_none() && rule.alt_tls == is_tls)
        }
        .map(|rule| {
            rule.alt_target
                .as_ref()
                .unwrap_or(&BasicPeer::new(DEFAULT_PORT.p404))
                .clone()
        })
        .unwrap_or_else(|| BasicPeer::new(DEFAULT_PORT.p500));

        let target_addr = format!("{}", proxy_to._address);
        log::debug!("Proxying to     : {} (TLS: {})", target_addr, is_tls);

        // Get the appropriate connector
        let connector = self.client_connectors.get(&target_addr).unwrap_or_else(|| {
            self.client_connectors
                .get("default")
                .expect("Default connector should exist")
        });

        // Increase timeout to at least 5 seconds
        let mut client_session = match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            connector.new_stream(&proxy_to),
        )
        .await
        {
            Ok(Ok(client_session)) => client_session,
            Ok(Err(e)) => {
                log::error!("Failed to connect to upstream peer {}: {}", target_addr, e);
                return None;
            }
            Err(_) => {
                log::error!("Connection to {} timed out", target_addr);
                return None;
            }
        };

        match client_session.write_all(&buf[0..n]).await {
            Ok(_) => {}
            Err(e) => {
                log::error!("Failed to write to upstream peer: {}", e);
                return None;
            }
        };

        match client_session.flush().await {
            Ok(_) => {}
            Err(e) => {
                log::error!("Failed to flush data to upstream peer: {}", e);
                return None;
            }
        };

        self.duplex(io, client_session).await;
        None
    }
}

/// # Extract Server Name Indication from TLS Client
///
/// Attempts to extract the SNI hostname from a TLS Client Hello message.
/// SNI (Server Name Indication) is a TLS extension defined in RFC 6066 that allows
/// a client to specify the hostname it is attempting to connect to at the start
/// of the handshake process.
///
/// ## TLS Handshake Format
/// The TLS handshake begins with a Client Hello message that has the following structure:
/// - 1 byte: Content Type (0x16 for Handshake)
/// - 2 bytes: TLS Version (e.g., 0x0301 for TLS 1.0, 0x0303 for TLS 1.2)
/// - 2 bytes: Length of the handshake message
/// - 1 byte: Handshake Type (0x01 for Client Hello)
/// - 3 bytes: Length of Client Hello
/// - 2 bytes: TLS Version (client supported)
/// - 32 bytes: Client Random
/// - 1 byte: Session ID Length
/// - Variable: Session ID
/// - 2 bytes: Cipher Suites Length
/// - Variable: Cipher Suites
/// - 1 byte: Compression Methods Length
/// - Variable: Compression Methods
/// - 2 bytes: Extensions Length
/// - Variable: Extensions (including SNI)
///
/// ## SNI Extension Format
/// The SNI extension has the following structure:
/// - 2 bytes: Extension Type (0x0000 for SNI)
/// - 2 bytes: Extension Length
/// - 2 bytes: Server Names List Length
/// - 1 byte: Name Type (0x00 for hostname)
/// - 2 bytes: Hostname Length
/// - Variable: Hostname (UTF-8 encoded)
///
/// ## Parameters
/// * `buf` - The raw bytes from the TLS handshake (Client Hello message)
///
/// ## Returns
/// * `Some(String)` - The extracted hostname if found
/// * `None` - If this is not a valid TLS Client Hello or SNI could not be extracted
///
/// ## Limitations
/// This implementation is simplified and may not work for all TLS variants or implementations:
/// - It doesn't properly validate the entire TLS handshake structure
/// - It uses a simplified pattern matching approach to locate the SNI extension
/// - It may not handle malformed TLS packets correctly
/// - It doesn't support TLS 1.3 session resumption or other complex scenarios
///
/// For production use, consider using a dedicated TLS parser library like 'rustls',
/// 'webpki', or bindings to OpenSSL.
fn extract_sni(buf: &[u8]) -> Option<String> {
    // Verify this is a TLS handshake (content type 0x16) with sufficient length
    if buf.len() < 5 || buf[0] != 0x16 {
        return None;
    }

    // Try to find SNI extension
    if let Some(pos) = find_sni_extension(buf) {
        // Extract hostname from SNI extension
        // The first 2 bytes are the hostname length
        if pos + 5 < buf.len() {
            let hostname_len = ((buf[pos] as usize) << 8) | (buf[pos + 1] as usize);
            if pos + 2 + hostname_len <= buf.len() {
                if let Ok(hostname) = std::str::from_utf8(&buf[pos + 2..pos + 2 + hostname_len]) {
                    return Some(hostname.to_string());
                }
            }
        }
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
