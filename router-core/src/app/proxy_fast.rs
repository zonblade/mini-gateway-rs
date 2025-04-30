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
use bytes::BytesMut;
use log::debug;

use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;

use pingora::apps::ServerApp;
use pingora::connectors::TransportConnector;
use pingora::protocols::Stream;
use pingora::server::ShutdownWatch;
use pingora::upstreams::peer::BasicPeer;
use regex_automata::meta::Regex;

use crate::app::proxy_host::extract_http_host;
use crate::app::proxy_sni::extract_sni_fast;
use crate::config::{self, GatewayNode};

struct RewriteRule {
    pattern: Regex,
    replacement: String,
}

pub struct ProxyApp {
    client_connector: TransportConnector,
    proxy_to: BasicPeer,
    path_rewrites: Vec<RewriteRule>,
    sni: Option<String>,
}

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

enum DuplexEvent {
    DownstreamRead(usize),
    UpstreamRead(usize),
}


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


impl ProxyApp {
    pub fn new(proxy_to: BasicPeer, sni: Option<String>) -> Self {
        let path_rewrites = Self::fetch_config(proxy_to.clone());
        
        ProxyApp {
            client_connector: TransportConnector::new(None),
            proxy_to,
            path_rewrites,
            sni,
        }
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

    fn fetch_config(proxy_to: BasicPeer) -> Vec<RewriteRule> {
        let current_addr = proxy_to._address.to_string();
        let config: Option<Vec<GatewayNode>> =
            config::RoutingData::GatewayRouting.xget::<Vec<GatewayNode>>();
        let mut new_rewrites = Vec::new();
        if let Some(cfg) = config {
            for node in cfg {
                if node.addr_target == current_addr {
                    let rgx = match Regex::new(&node.path_listen) {
                        Ok(rgx) => rgx,
                        Err(e) => {
                            log::error!("Failed to compile regex for path_listen: {}", e);
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
            && !request_str.starts_with("DELETE ") {
            return length;
        }

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
                let replacement = self.process_replacement(&captures.iter().map(|s| *s).collect::<Vec<&str>>(), &rule.replacement);
                
                // Create the new request line and full request
                let new_request_line = format!("{}{}{}", before, replacement, after);
                let new_request = format!("{}{}", new_request_line, rest_of_request);
                
                debug!("Rewrote request: {} -> {}", request_line, new_request_line);
                
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
                        log::error!("Failed to read from downstream peer: {}", e);
                        return;
                    },
                    Err(_) => {
                        log::error!("Downstream peer read timeout");
                        return;
                    }
                },
                result = upstream_read => match result {
                    Ok(Ok(n)) => event = DuplexEvent::UpstreamRead(n),
                    Ok(Err(e)) => {
                        log::error!("Failed to read from upstream peer: {}", e);
                        return;
                    },
                    Err(_) => {
                        log::error!("Upstream peer read timeout");
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
        mut io: Stream,
        _shutdown: &ShutdownWatch,
    ) -> Option<Stream> {

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
        let buf_slice = &buffer[..n];
        let conn_type = Self::detect_connection_type(buf_slice, n);
        let host_info = match conn_type {
            ConnectionType::Tls => extract_sni_fast(buf_slice),
            ConnectionType::Http | ConnectionType::WebSocket => extract_http_host(buf_slice, n),
            ConnectionType::Tcp => None,
        };

        if let Some(host) = host_info {
            log::info!("Host: {}", host);
            if let Some(sni) = &self.sni {
                if !host.contains(sni) {
                    BUFFER_POOL.put(buffer);
                    return None;
                }
            }
        }

        log::info!("Processing new connection");
        
        let connect_future = self.client_connector.new_stream(&self.proxy_to);
        let mut client_session = match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            connect_future,
        )
        .await
        {
            Ok(Ok(client_session)) => client_session,
            Ok(Err(e)) => {
                log::error!("Failed to connect to upstream peer {}: {}", &self.proxy_to._address.to_string(), e);
                BUFFER_POOL.put(buffer);
                return None;
            }
            Err(_) => {
                log::error!("Connection to {} timed out", &self.proxy_to._address.to_string());
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
        self.duplex(io, client_session).await;
        None
    }
}
