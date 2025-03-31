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

use crate::config::DEFAULT_PORT;

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
pub struct RedirectRule {
    host: Option<String>,
    alt_listen: String,
    alt_target: Option<BasicPeer>,
    alt_tls: bool,
    priority: usize,
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
        let mut redirects = vec![
            RedirectRule {
                host: Some("localhost:2000".to_string()),
                alt_target: Some(BasicPeer::new("127.0.0.1:30001")),
                alt_listen: "0.0.0.0:2000".to_string(),
                alt_tls: false,
                priority: 0,
            },
            RedirectRule {
                host: None,
                alt_target: Some(BasicPeer::new("127.0.0.1:30003")),
                alt_listen: "0.0.0.0:2000".to_string(),
                alt_tls: false,
                priority: 1,
            },
            // RedirectRule {
            //     pattern: Regex::new(r"^/(.*)$").unwrap(),
            //     target: "/$1".to_string(),
            //     alt_target: Some(BasicPeer::new("127.0.0.1:3002")),
            //     alt_listen: "127.0.0.1:9010".to_string(),
            //     priority: 0,
            // },
        ];
        redirects.retain(|rule| rule.alt_listen == alt_source);
        redirects.sort_by(|a, b| b.priority.cmp(&a.priority));
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
        loop {
            let downstream_read = server_session.read(&mut upstream_buf);
            let upstream_read = client_session.read(&mut downstream_buf);
            let event: DuplexEvent;
            select! {
                n = downstream_read => {
                    event = match n {
                        Ok(n) => DuplexEvent::DownstreamRead(n),
                        Err(e) => {
                            log::error!("Error reading from downstream: {}", e);
                            return;
                        }
                    }
                },
                n = upstream_read => {
                    event = match n {
                        Ok(n) => DuplexEvent::UpstreamRead(n),
                        Err(e) => {
                            log::error!("Error reading from upstream: {}", e);
                            return;
                        }
                    }
                },
            }
            match event {
                DuplexEvent::DownstreamRead(0) => {
                    debug!("downstream session closing");
                    return;
                }
                DuplexEvent::UpstreamRead(0) => {
                    debug!("upstream session closing");
                    return;
                }
                DuplexEvent::DownstreamRead(n) => {
                    client_session.write_all(&upstream_buf[0..n]).await.unwrap();
                    client_session.flush().await.unwrap();
                }
                DuplexEvent::UpstreamRead(n) => {
                    server_session
                        .write_all(&downstream_buf[0..n])
                        .await
                        .unwrap();
                    server_session.flush().await.unwrap();
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
        log::info!("");
        log::info!("#-------------------------------------#");
        log::info!("#           Incoming Request          #");
        log::info!("#-------------------------------------#");
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
        log::info!("Request preview : {}", first_line);

        // In your process_new implementation, modify the host extraction:
        // Determine if this is a TLS connection based on the first byte
        let is_tls = n > 0 && buf[0] == 0x16;
        log::info!(
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

        log::info!("Host header     : {:?}", host_header);

        // Check for WebSocket upgrade
        let is_websocket = !is_tls
            && preview
                .lines()
                .any(|line| line.to_lowercase().contains("upgrade: websocket"));
        if is_websocket {
            log::info!("Upgrade request : WebSocket");
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

        log::info!("Host info       : {:?}", host_info);

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
        log::info!("Proxying to     : {} (TLS: {})", target_addr, is_tls);

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
/// Attempts to extract the SNI hostname from a TLS Client message.
/// This is a simplified implementation that may not work for all TLS variants.
///
/// ## Parameters
/// * `buf` - The raw bytes from the TLS handshake
///
/// ## Returns
/// * `Some(String)` - The extracted hostname if found
/// * `None` - If this is not a valid TLS Client or SNI could not be extracted
///
/// ## Note
/// This is a simplified implementation. For production use, consider using a
/// dedicated TLS parser library.
fn extract_sni(buf: &[u8]) -> Option<String> {
    // This is a very simplified SNI extractor
    // In a real implementation, you would parse the Client properly
    // TLS handshake format: 0x16 (handshake) + 0x03 0x01 (TLS version) + 2-byte length
    if buf.len() < 5 || buf[0] != 0x16 {
        return None;
    }

    // Try to find SNI extension
    // This is a simplified implementation and might not work for all cases
    // For production, use a proper TLS parser library
    if let Some(pos) = find_sni_extension(buf) {
        // Extract hostname from SNI extension
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

/// # Find SNI extension in TLS Client
///
/// Helper function that searches for the SNI extension within a TLS Client message.
/// This is a simplified implementation that looks for certain byte patterns.
///
/// ## Parameters
/// * `buf` - The raw bytes from the TLS handshake
///
/// ## Returns
/// * `Some(usize)` - Position where the SNI hostname data begins
/// * `None` - If SNI extension could not be found
///
/// ## Note
/// This is a very simplified implementation and may produce false positives.
/// For production use, consider a proper TLS parser.
fn find_sni_extension(buf: &[u8]) -> Option<usize> {
    // This is a very simplified implementation
    // In a real implementation, you would parse the TLS Client properly
    // Search for SNI extension (0x00 0x00) - SIMPLIFIED, NOT ACCURATE!
    for i in 0..buf.len() - 4 {
        if buf[i] == 0x00 && buf[i + 1] == 0x00 && buf[i + 2] == 0x00 {
            return Some(i + 3);
        }
    }
    None
}
