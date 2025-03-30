use async_trait::async_trait;
use log::debug;
use regex::Regex;

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;

use pingora::apps::ServerApp;
use pingora::connectors::TransportConnector;
use pingora::protocols::Stream;
use pingora::server::ShutdownWatch;
use pingora::upstreams::peer::BasicPeer;

/// Rule defining how to match and transform HTTP requests.
///
/// `RedirectRule` determines how incoming requests should be routed based on their path.
/// Each rule includes a regex pattern to match the path, a target path template for rewriting,
/// and information about which backend server should handle the request.
pub struct RedirectRule {
    /// Regular expression pattern to match against request paths
    pattern: Regex,

    /// Target path template (may include capture group references like $1, $2)
    target: String,

    /// The listening address this rule applies to
    alt_listen: String,

    /// Optional alternate backend server to route matching requests to
    alt_target: Option<BasicPeer>,

    /// Priority of this rule (higher values = higher priority)
    /// Rules are sorted by priority when matching, so higher priority rules are checked first.
    priority: usize,
}

/// Proxy application that routes HTTP requests to backend servers.
///
/// `ProxyApp` handles incoming HTTP connections, applies routing rules to determine
/// the appropriate backend server, and manages the bidirectional data transfer between
/// the client and the backend server.
pub struct ProxyApp {
    /// Map of target server addresses to their respective connection pools
    client_connectors: std::collections::HashMap<String, TransportConnector>,

    /// List of routing rules, sorted by priority
    redirects: Vec<RedirectRule>,
}

/// Events representing data read from either the downstream or upstream streams.
///
/// Used in the duplex proxying process to handle bidirectional data transfer.
enum DuplexEvent {
    /// Data read from the client (downstream) connection
    DownstreamRead(usize),

    /// Data read from the server (upstream) connection
    UpstreamRead(usize),
}

impl ProxyApp {
    /// Creates a new proxy application that handles requests on the specified listening address.
    ///
    /// # Arguments
    ///
    /// * `alt_source` - The listening address to match rules against (e.g., "127.0.0.1:9010")
    ///
    /// # Returns
    ///
    /// A configured `ProxyApp` instance with rules matching the specified listening address.
    ///
    /// # Example
    ///
    /// ```
    /// let proxy = ProxyApp::new("127.0.0.1:9010");
    /// ```
    pub fn new(alt_source: &str) -> Self {
        let mut redirects = vec![
            // RedirectRule {
            //     pattern: Regex::new("^/sometimes\\.ico$").unwrap(),
            //     target: "/favicon.ico".to_string(),
            //     alt_target: None,
            //     priority: 4,
            // },
            RedirectRule {
                pattern: Regex::new("^/favicon\\.ico$").unwrap(),
                target: "/favicon.ico".to_string(),
                alt_target: Some(BasicPeer::new("127.0.0.1:3000")),
                alt_listen: "127.0.0.1:9010".to_string(),
                priority: 0,
            },
            // RedirectRule {
            //     pattern: Regex::new("^/videos/([^/]+)/play$").unwrap(),
            //     target: "/watch/$1".to_string(),
            //     alt_target: None,
            //     priority: 2,
            // },
            RedirectRule {
                pattern: Regex::new("^/api/(.*)$").unwrap(),
                target: "/v2/api/$1".to_string(),
                alt_target: Some(BasicPeer::new("127.0.0.1:8080")),
                alt_listen: "127.0.0.1:9010".to_string(),
                priority: 1,
            },
            RedirectRule {
                pattern: Regex::new(r"^/(.*)$").unwrap(),
                target: "/$1".to_string(),
                alt_target: Some(BasicPeer::new("127.0.0.1:3002")),
                alt_listen: "127.0.0.1:9010".to_string(),
                priority: 0,
            },
            RedirectRule {
                pattern: Regex::new(r"^/(.*)$").unwrap(),
                target: "/$1".to_string(),
                alt_target: Some(BasicPeer::new("127.0.0.1:8080")),
                alt_listen: "127.0.0.1:9011".to_string(),
                priority: 0,
            },
        ];
        redirects.retain(|rule| rule.alt_listen == alt_source);
        redirects.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Create a map of connectors
        let mut client_connectors = std::collections::HashMap::new();

        // Add connectors for each unique target
        for rule in &redirects {
            if let Some(target) = &rule.alt_target {
                let addr = format!("{}", target);
                if !client_connectors.contains_key(&addr) {
                    client_connectors.insert(addr, TransportConnector::new(None));
                }
            }
        }

        // Add default connector
        client_connectors.insert("default".to_string(), TransportConnector::new(None));

        ProxyApp {
            client_connectors,
            redirects,
        }
    }

    /// Handles bidirectional data transfer between client and server connections.
    ///
    /// This is the core proxying functionality that:
    /// 1. Concurrently reads from both client and server connections
    /// 2. Forwards data in both directions
    /// 3. Handles termination when either connection closes
    ///
    /// # Arguments
    ///
    /// * `server_session` - The client-facing connection
    /// * `client_session` - The backend server connection
    ///
    /// # Note
    ///
    /// This method will run until either connection is closed.
    async fn duplex(&self, mut server_session: Stream, mut client_session: Stream) {
        // Buffers for reading data from the streams.
        let mut upstream_buf = [0; 1024];
        let mut downstream_buf = [0; 1024];
        loop {
            // Concurrently read from both streams.
            let downstream_read = server_session.read(&mut upstream_buf);
            let upstream_read = client_session.read(&mut downstream_buf);
            let event: DuplexEvent;
            select! {
                n = downstream_read => event
                    = DuplexEvent::DownstreamRead(n.unwrap()),
                n = upstream_read => event
                    = DuplexEvent::UpstreamRead(n.unwrap()),
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
                    // Write data from downstream to upstream.
                    client_session.write_all(&upstream_buf[0..n]).await.unwrap();
                    client_session.flush().await.unwrap();
                }
                DuplexEvent::UpstreamRead(n) => {
                    // Write data from upstream to downstream.
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
    /// Processes a new incoming HTTP connection.
    ///
    /// This method:
    /// 1. Reads and parses the initial HTTP request
    /// 2. Extracts the request path
    /// 3. Applies matching redirect rules based on the path
    /// 4. Rewrites the request path according to the matching rule
    /// 5. Establishes a connection to the appropriate backend server
    /// 6. Forwards the (possibly rewritten) request
    /// 7. Sets up bidirectional proxying between client and server
    ///
    /// # Arguments
    ///
    /// * `io` - The client connection stream
    /// * `_shutdown` - A shutdown watcher for graceful server termination
    ///
    /// # Returns
    ///
    /// Always returns `None` as the connection is fully consumed by the proxy.
    async fn process_new(
        self: &Arc<Self>,
        mut io: Stream,
        _shutdown: &ShutdownWatch,
    ) -> Option<Stream> {
        log::info!("\n\n\nIncoming Request");

        // Read the initial data
        let mut buf = [0; 4098];
        let mut n = match io.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                log::error!("Failed to read from client: {}", e);
                return None;
            }
        };

        // if n == 0 {
        //     log::info!("Empty request received (0 bytes)");
        //     return None;
        // }

        // Attempt to determine the connection type
        let preview = String::from_utf8_lossy(&buf[..std::cmp::min(n, 200)]);
        let first_line = preview.lines().next().unwrap_or("Empty request");

        // Log the first line of the request
        log::info!("Request preview: {}", first_line);

        // Detect connection type
        let connection_type = if first_line.starts_with("GET ")
            || first_line.starts_with("POST ")
            || first_line.starts_with("PUT ")
            || first_line.starts_with("DELETE ")
            || first_line.starts_with("HEAD ")
            || first_line.starts_with("OPTIONS ")
        {
            // Check for WebSocket upgrade
            if preview.contains("Upgrade: websocket") && preview.contains("Connection: Upgrade") {
                "WebSocket"
            } else {
                "HTTP"
            }
        } else if preview.contains("\0") {
            // Simple binary data check
            "Binary/TCP"
        } else {
            "Unknown"
        };

        log::info!("Connection type detected: {}", connection_type);

        // Process according to connection type
        match connection_type {
            "WebSocket" => {
                // For WebSockets, you might want to handle differently or just pass through
                log::info!("WebSocket connection detected");
                // Continue with your normal processing as WebSockets start as HTTP
            }
            "HTTP" => {
                // This is your standard HTTP flow
                log::info!("Standard HTTP connection detected");
                // Continue with standard HTTP processing
            }
            "Binary/TCP" => {
                log::info!("Binary/TCP data detected, not HTTP");
                // You could either reject these connections or implement special handling
                // For now, continue trying to process as HTTP, which will likely fail gracefully
            }
            _ => {
                log::info!("Unknown protocol data");
                // Similar to binary/TCP case
            }
        }

        // Continue with your existing HTTP parsing logic
        let request = String::from_utf8_lossy(&buf[..n]);
        // Parse the request to extract the path
        let first_line = match request.lines().next() {
            Some(line) => line,
            None => {
                log::error!("No lines in request");
                return None;
            }
        };

        let (_, rest) = match first_line.split_once(' ') {
            Some(parts) => parts,
            None => return None, // Early return if the first line does not contain a space
        };

        let (path, _) = match rest.split_once(' ') {
            Some(parts) => parts,
            None => return None, // Early return if the rest does not contain a space
        };

        log::info!("Request path: {}", path);

        // Determine the proxy target based on the path
        let mut proxy_to = BasicPeer::new("127.0.0.1:8080"); // Default fallback
        for rule in &self.redirects {
            if let Some(captures) = rule.pattern.captures(path) {
                // Generate the target path by replacing capture groups
                let mut target_path = rule.target.clone();
                for (i, capture) in captures.iter().enumerate().skip(1) {
                    if let Some(capture) = capture {
                        target_path = target_path.replace(&format!("${}", i), capture.as_str());
                    }
                }

                // log::info!("Matched rule: {:?} -> {}", rule.pattern, target_path);

                // Update proxy target if alternate is provided
                if let Some(alt_target) = &rule.alt_target {
                    proxy_to = alt_target.clone();
                    // log::info!("Redirecting to alternate target: {:?}", proxy_to);
                }

                // Rewrite the first line with the new path
                let new_first_line = first_line.replacen(path, &target_path, 1);
                // log::info!("Rewriting path: {} -> {}", path, target_path);
                // log::info!("Rewritten first line: {}", new_first_line);

                // Rebuild the HTTP request with the modified path
                let new_request = request.replacen(first_line, &new_first_line, 1);
                let new_buf = new_request.as_bytes();

                if new_buf.len() <= buf.len() {
                    buf[..new_buf.len()].copy_from_slice(new_buf);
                    n = new_buf.len();
                } else {
                    log::warn!("Modified request is larger than buffer, keeping original");
                }

                break; // Exit the loop after processing the first matching rule
            }
        }

        // Get the appropriate connector
        let target_addr = format!("{}", proxy_to);
        let connector = self.client_connectors.get(&target_addr).unwrap_or_else(|| {
            // Fallback to default connector if no specific one exists
            self.client_connectors
                .get("default")
                .expect("Default connector should exist")
        });

        // Use a timeout for connection establishment
        let mut client_session = match tokio::time::timeout(
            std::time::Duration::from_millis(120),
            connector.new_stream(&proxy_to),
        )
        .await
        {
            Ok(Ok(client_session)) => {
                // log::info!("Connected to upstream peer {}", target_addr);
                client_session
            }
            Ok(Err(e)) => {
                log::error!("Failed to connect to upstream peer {}: {}", target_addr, e);
                return None;
            }
            Err(_) => {
                log::error!("Connection to {} timed out", target_addr);
                return None;
            }
        };

        // Forward the initial data we captured
        match client_session.write_all(&buf[0..n]).await {
            Ok(_) => {
                // log::info!("Forwarded {} bytes to upstream peer", n);
            }
            Err(e) => {
                log::error!("Failed to write to upstream peer: {}", e);
                return None;
            }
        };
        match client_session.flush().await {
            Ok(_) => {
                // log::info!("Flushed data to upstream peer");
            }
            Err(e) => {
                log::error!("Failed to flush data to upstream peer: {}", e);
                return None;
            }
        };

        // Begin regular duplex proxying
        self.duplex(io, client_session).await;
        None
    }
}
