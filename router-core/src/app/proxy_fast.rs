// Copyright 2025 Cloudflare, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use async_trait::async_trait;
use log::debug;

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;

use pingora::apps::ServerApp;
use pingora::connectors::TransportConnector;
use pingora::protocols::Stream;
use pingora::server::ShutdownWatch;
use pingora::upstreams::peer::BasicPeer;
use regex_automata::meta::Regex;

use crate::config::{self, GatewayNode};

struct RewriteRule {
    pattern: Regex,
    replacement: String,
}

pub struct ProxyApp {
    client_connector: TransportConnector,
    proxy_to: BasicPeer,
    path_rewrites: Vec<RewriteRule>,
}

enum DuplexEvent {
    DownstreamRead(usize),
    UpstreamRead(usize),
}

impl ProxyApp {
    pub fn new(proxy_to: BasicPeer) -> Self {
        let path_rewrites = Self::fetch_config(proxy_to.clone());

        ProxyApp {
            client_connector: TransportConnector::new(None),
            proxy_to,
            path_rewrites,
        }
    }

    fn fetch_config(proxy_to: BasicPeer) -> Vec<RewriteRule> {
        let current_addr = proxy_to._address.to_string();
        let config: Option<Vec<GatewayNode>> =
            config::RoutingData::GatewayRouting.xget::<Vec<GatewayNode>>();
        let mut new_rewrites = Vec::new();
        if let Some(cfg) = config {
            for node in cfg {
                // high speed only
                if node.addr_target == current_addr {
                    // Determine if this is a plain string path, a wildcard path, or a regex pattern
                    let processed_pattern = if Self::is_regex_pattern(&node.path_listen) {
                        // Already a regex pattern (contains regex special chars other than * at the end)
                        log::debug!("Processing as regex pattern: '{}'", node.path_listen);
                        node.path_listen.clone()
                    } else if node.path_listen.ends_with("/*") {
                        // Wildcard pattern (e.g., "/api/*")
                        log::debug!("Processing as wildcard pattern: '{}'", node.path_listen);
                        // Convert "/api/*" to "^/api/.*$"
                        let base_path = &node.path_listen[..node.path_listen.len() - 1];
                        format!("^{}.*$", base_path)
                    } else {
                        // Plain string path (e.g., "/test")
                        log::debug!("Processing as exact match pattern: '{}'", node.path_listen);
                        // Convert "/test" to "^/test$"
                        format!("^{}$", node.path_listen)
                    };

                    // Compile the processed regex pattern
                    let rgx = match Regex::new(&processed_pattern) {
                        Ok(rgx) => rgx,
                        Err(e) => {
                            log::error!(
                                "Failed to compile regex pattern '{}' (from '{}'): {}",
                                processed_pattern,
                                node.path_listen,
                                e
                            );
                            continue;
                        }
                    };

                    new_rewrites.push(RewriteRule {
                        pattern: rgx,
                        replacement: node.path_target,
                    });
                }
            }
        }
        new_rewrites
    }

    /// Helper function to determine if a pattern string contains regex special characters.
    ///
    /// This function checks if a string has regex special metacharacters that would
    /// indicate it should be treated as a regex pattern rather than a literal string.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The pattern string to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the string contains regex special characters, `false` otherwise.
    fn is_regex_pattern(pattern: &str) -> bool {
        // These are common regex metacharacters excluding the wildcard character at the end
        // which we handle specially
        let regex_special_chars = [
            '^', '$', '.', '+', '?', '(', ')', '[', ']', '{', '}', '|', '\\',
        ];

        // Check if the pattern contains any regex special characters
        for &c in &regex_special_chars {
            if pattern.contains(c) {
                return true;
            }
        }

        // If the pattern has a wildcard in the middle (not at the end), treat as regex
        if pattern.contains('*') && !pattern.ends_with("/*") {
            return true;
        }

        // Otherwise it's a plain string or a simple wildcard pattern
        false
    }

    // Process a replacement string, handling $1, $2, etc. references
    fn process_replacement(&self, captures: &[&str], template: &str) -> String {
        let mut result = String::new();
        let mut i = 0;

        while i < template.len() {
            let remainder = &template[i..];

            if remainder.starts_with('$') && remainder.len() > 1 {
                if let Some(digit) = remainder.chars().nth(1).and_then(|c| c.to_digit(10)) {
                    if digit > 0 && (digit as usize) <= captures.len() {
                        // $n references the nth capture
                        result.push_str(captures[digit as usize - 1]);
                        i += 2;
                        continue;
                    }
                }
            }

            // Not a capture reference or invalid index, add current char
            result.push(template.chars().nth(i).unwrap());
            i += 1;
        }

        result
    }

    // Regex-based HTTP request line parser and rewriter
    fn rewrite_http_request(&self, buffer: &mut [u8], length: usize) -> usize {
        // First convert the buffer to a string for processing
        let request_str = match std::str::from_utf8(&buffer[..length]) {
            Ok(s) => s,
            Err(_) => return length, // Not valid UTF-8, return unchanged
        };

        // Check if this looks like an HTTP request
        if !request_str.starts_with("GET ")
            && !request_str.starts_with("POST ")
            && !request_str.starts_with("PUT ")
            && !request_str.starts_with("DELETE ")
            && !request_str.starts_with("CONNECT ")
            && !request_str.starts_with("OPTIONS ")
        {
            return length;
        }
        
        // Flag to track if this is a WebSocket upgrade request
        let is_websocket = request_str.contains("Upgrade: websocket") || 
                           request_str.contains("Upgrade: WebSocket");

        // Find the first line of the request (the request line)
        let line_end = match request_str.find("\r\n") {
            Some(pos) => pos,
            None => return length, // Not a complete HTTP request line
        };

        let request_line = &request_str[..line_end];
        let rest_of_request = &request_str[line_end..];

        // Try each rewrite rule
        for rule in &self.path_rewrites {
            // Find all matches in the request line
            let mut matches = Vec::new();
            let mut captures = Vec::new();

            // Use regex-automata to find matches
            for mat in rule.pattern.find_iter(request_line.as_bytes()) {
                matches.push((mat.start(), mat.end()));

                // Extract capture groups
                // This is simplified since regex-automata's Match doesn't directly provide captures
                // In a real implementation, you'd need to extract captures based on the match bounds
                let matched_text = &request_line[mat.start()..mat.end()];
                captures.push(matched_text);
            }

            // If we have a match, perform the rewrite
            if let Some((start, end)) = matches.first() {
                debug!("Matched regex pattern for rewrite");

                // Get parts before and after the match
                let before = &request_line[..*start];
                let after = &request_line[*end..];

                // Process replacement template with capture references
                let replacement = self.process_replacement(
                    &captures.iter().map(|s| *s).collect::<Vec<&str>>(),
                    &rule.replacement,
                );

                // Create the new request line and full request
                let new_request_line = format!("{}{}{}", before, replacement, after);
                let new_request = format!("{}{}", new_request_line, rest_of_request);

                // Log rewrite information with special note for WebSocket upgrades
                if is_websocket {
                    debug!("Rewrote WebSocket upgrade request: {} -> {}", request_line, new_request_line);
                } else {
                    debug!("Rewrote request: {} -> {}", request_line, new_request_line);
                }

                // Convert back to bytes and copy to the buffer
                let new_bytes = new_request.as_bytes();
                let new_len = new_bytes.len();

                // Make sure we don't overflow the buffer
                if new_len <= buffer.len() {
                    // Copy the new request into the buffer
                    buffer[..new_len].copy_from_slice(new_bytes);
                    return new_len;
                } else {
                    debug!("Rewritten request too large for buffer");
                    return length; // Return original length if new request is too large
                }
            }
        }

        // No rewrite performed, return original length
        length
    }

    async fn duplex(&self, mut server_session: Stream, mut client_session: Stream) {
        let mut upstream_buf = [0; 4096]; // Increased buffer size for HTTP headers
        let mut downstream_buf = [0; 4096];
        let timeout_duration = std::time::Duration::from_secs(60);

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
                        log::error!("Error reading from downstream: {}", e);
                        return;
                    },
                    Err(_) => {
                        log::error!("Timeout reading from downstream");
                        return;
                    }
                },
                result = upstream_read => match result {
                    Ok(Ok(n)) => event = DuplexEvent::UpstreamRead(n),
                    Ok(Err(e)) => {
                        log::error!("Error reading from upstream: {}", e);
                        return;
                    },
                    Err(_) => {
                        log::error!("Timeout reading from upstream");
                        return;
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
                    // Try to rewrite the request if it's HTTP
                    let write_len = self.rewrite_http_request(&mut upstream_buf, n);

                    client_session
                        .write_all(&upstream_buf[0..write_len])
                        .await
                        .unwrap();
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
    async fn process_new(
        self: &Arc<Self>,
        io: Stream,
        _shutdown: &ShutdownWatch,
    ) -> Option<Stream> {
        let client_session = self.client_connector.new_stream(&self.proxy_to).await;

        match client_session {
            Ok(client_session) => {
                self.duplex(io, client_session).await;
                None
            }
            Err(e) => {
                debug!("Failed to create client session: {}", e);
                None
            }
        }
    }
}
