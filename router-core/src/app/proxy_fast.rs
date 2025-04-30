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

#[derive(Debug)]
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
            // Increased buffer size from 8192 to 16384 to handle larger path rewrites
            buffers.push(BytesMut::with_capacity(16384));
        }
        BufferPool {
            buffers: Mutex::new(buffers),
        }
    }

    fn get(&self) -> BytesMut {
        match self.buffers.lock().unwrap().pop() {
            Some(buf) => buf,
            None => BytesMut::with_capacity(16384), // Increased from 8192
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
        let pattern5 = b"Connection: Upgrade";  // Additional pattern often in WebSocket handshakes
        
        // Only search a reasonable portion of the header
        let search_len = std::cmp::min(len, 1024);
        
        // Convert to lowercase for case-insensitive search
        let mut has_upgrade = false;
        let mut has_connection = false;
        
        for i in 0..search_len - pattern1.len() {
            let window = &data[i..i + pattern1.len()];
            // Check for websocket upgrade header
            if window.eq_ignore_ascii_case(pattern1) || 
               window.eq_ignore_ascii_case(pattern2) ||
               window.eq_ignore_ascii_case(pattern3) || 
               window.eq_ignore_ascii_case(pattern4) {
                has_upgrade = true;
                if has_connection {
                    return true;
                }
            }
        }
        
        // Look for Connection: Upgrade header
        for i in 0..search_len - pattern5.len() {
            let window = &data[i..i + pattern5.len()];
            if window.eq_ignore_ascii_case(pattern5) {
                has_connection = true;
                if has_upgrade {
                    return true;
                }
            }
        }
        
        // Check for sec-websocket-key which is always present in WS handshakes
        let ws_key_pattern = b"Sec-WebSocket-Key:";
        for i in 0..search_len - ws_key_pattern.len() {
            let window = &data[i..i + ws_key_pattern.len()];
            if window.eq_ignore_ascii_case(ws_key_pattern) {
                log::info!("WebSocket key header found in request");
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
                    log::info!("Found matching gateway node with path_listen: '{}', path_target: '{}'", 
                        node.path_listen, node.path_target);
                    
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
        
        if new_rewrites.is_empty() {
            log::warn!("No path rewrite rules found for address: {}", current_addr);
        } else {
            log::info!("Loaded {} path rewrite rules for {}", new_rewrites.len(), current_addr);
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
        
        log::debug!("Original request line: {}", request_line);
        
        // Extract method, path, and HTTP version from the request line
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() != 3 {
            log::warn!("Invalid HTTP request line format: {}", request_line);
            return length;
        }
        
        let method = parts[0];
        let original_path = parts[1];
        let http_version = parts[2];
        
        log::debug!("Extracted path: {}", original_path);
        
        // Create a new path based on rewrite rules
        let mut new_path = original_path.to_string();
        let mut was_rewritten = false;
        
        // Try each rewrite rule
        for rule in &self.path_rewrites {
            log::debug!("Checking path '{}' against rule pattern", original_path);
            
            // Check if the pattern matches the path part only (not the whole request line)
            if rule.pattern.is_match(original_path.as_bytes()) {
                was_rewritten = true;
                
                // Simple replace - direct path substitution without regex captures
                // This handles straightforward cases like /test.png -> /logo.png
                if !rule.replacement.contains('$') {
                    log::info!("Simple path rewrite: {} -> {}", original_path, rule.replacement);
                    new_path = rule.replacement.clone();
                } else {
                    // For more complex replacements with capture groups
                    // regex_automata doesn't have replace_all, so we need to implement it ourselves
                    // First find all matches
                    let mut matches = Vec::new();
                    for mat in rule.pattern.find_iter(original_path.as_bytes()) {
                        matches.push((mat.start(), mat.end()));
                    }
                    
                    if let Some((start, end)) = matches.first() {
                        // Extract the match and use it with our process_replacement helper
                        let matched_text = &original_path[*start..*end];
                        let captures = vec![matched_text];
                        let replacement = self.process_replacement(&captures, &rule.replacement);
                        
                        // Build the new path with the replacement
                        new_path = format!("{}{}{}", 
                            &original_path[..*start], 
                            replacement,
                            &original_path[*end..]);
                        
                        log::info!("Complex path rewrite: {} -> {}", original_path, new_path);
                    }
                }
                
                break;
            }
        }
        
        if !was_rewritten {
            log::debug!("No rewrite rules matched for path: {}", original_path);
            return length;
        }
        
        // Create the new request line and full request
        let new_request_line = format!("{} {} {}", method, new_path, http_version);
        let new_request = format!("{}{}", new_request_line, rest_of_request);
        
        log::info!("Rewritten request line: {} -> {}", request_line, new_request_line);
        
        // Convert back to bytes and copy to the buffer
        let new_bytes = new_request.as_bytes();
        let new_len = new_bytes.len();
        
        // Make sure we don't overflow the buffer
        if new_len <= buffer.len() {
            // Copy the new request into the buffer
            buffer[..new_len].copy_from_slice(new_bytes);
            return new_len;
        } else {
            // Instead of failing, try to create a truncated but valid HTTP request
            // that at least preserves the rewritten path
            log::warn!("Rewritten request too large for buffer ({} > {}), using truncated request", 
                       new_len, buffer.len());
            
            // Create a minimal HTTP request with just the essential headers
            let minimal_request = format!(
                "{} {} {}\r\nHost: {}\r\n\r\n",
                method, new_path, http_version,
                // Extract host from original request if possible
                request_str.lines()
                    .find(|line| line.to_lowercase().starts_with("host:"))
                    .and_then(|host_line| host_line.split_once(":").map(|(_, v)| v.trim()))
                    .unwrap_or("localhost")
            );
            
            let minimal_bytes = minimal_request.as_bytes();
            let minimal_len = minimal_bytes.len();
            
            if minimal_len <= buffer.len() {
                // Copy the minimal request
                buffer[..minimal_len].copy_from_slice(minimal_bytes);
                log::info!("Using minimal HTTP request with rewritten path: {}", minimal_request);
                return minimal_len;
            } else {
                // If even the minimal request is too large, we have to give up and use the original
                log::error!("Cannot fit even minimal rewritten request in buffer, using original");
                return length;
            }
        }
    }

    async fn duplex(&self, mut server_session: Stream, mut client_session: Stream) {
        // Increased buffer size to handle larger rewritten paths (8KB instead of 4KB)
        let mut upstream_buf = [0; 8192]; 
        let mut downstream_buf = [0; 8192];
        // Set timeout for read operations
        let timeout_duration = std::time::Duration::from_secs(120);

        log::debug!("Starting duplex session between client and server");

        loop {
            log::debug!("Waiting for data from either client or server");
            let downstream_read =
                tokio::time::timeout(timeout_duration, server_session.read(&mut upstream_buf));
            let upstream_read =
                tokio::time::timeout(timeout_duration, client_session.read(&mut downstream_buf));
            let event: DuplexEvent;

            select! {
                result = downstream_read => match result {
                    Ok(Ok(n)) => {
                        log::debug!("Read {} bytes from downstream peer", n);
                        event = DuplexEvent::DownstreamRead(n)
                    },
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
                    Ok(Ok(n)) => {
                        log::debug!("Read {} bytes from upstream peer", n);
                        event = DuplexEvent::UpstreamRead(n)
                    },
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
                    log::debug!("Downstream session closing (EOF)");
                    return;
                }
                DuplexEvent::UpstreamRead(0) => {
                    log::debug!("Upstream session closing (EOF)");
                    return;
                }
                DuplexEvent::DownstreamRead(n) => {
                    // Try to rewrite the request if it's HTTP
                    log::debug!("Processing {} bytes from downstream for possible rewrite", n);
                    let write_len = self.rewrite_http_request(&mut upstream_buf, n);
                    log::debug!("After rewrite: {} bytes to write to upstream", write_len);

                    match client_session.write_all(&upstream_buf[0..write_len]).await {
                        Ok(_) => log::debug!("Successfully wrote {} bytes to upstream", write_len),
                        Err(e) => {
                            log::error!("Failed to write to upstream: {}", e);
                            return;
                        }
                    }

                    match client_session.flush().await {
                        Ok(_) => log::debug!("Successfully flushed data to upstream"),
                        Err(e) => {
                            log::error!("Failed to flush data to upstream: {}", e);
                            return;
                        }
                    }
                }
                DuplexEvent::UpstreamRead(n) => {
                    log::debug!("Forwarding {} bytes from upstream to downstream", n);
                    match server_session.write_all(&downstream_buf[0..n]).await {
                        Ok(_) => log::debug!("Successfully wrote {} bytes to downstream", n),
                        Err(e) => {
                            log::error!("Failed to write to downstream: {}", e);
                            return;
                        }
                    }

                    match server_session.flush().await {
                        Ok(_) => log::debug!("Successfully flushed data to downstream"),
                        Err(e) => {
                            log::error!("Failed to flush data to downstream: {}", e);
                            return;
                        }
                    }
                }
            }
        }
    }

    /// Special duplex mode for WebSocket connections with optimized buffer handling
    /// This mode has larger buffers and special error handling for WebSocket protocol
    async fn duplex_websocket(&self, mut server_session: Stream, mut client_session: Stream) {
        // Larger buffers for WebSocket frames (32KB)
        let mut upstream_buf = [0; 32768]; 
        let mut downstream_buf = [0; 32768];
        // WebSockets can have longer idle periods, so extend timeout
        let timeout_duration = std::time::Duration::from_secs(300);  // 5 minute timeout

        log::info!("Starting WebSocket duplex session");

        // Keep track of bytes transferred
        let mut upstream_bytes = 0usize;
        let mut downstream_bytes = 0usize;

        loop {
            log::debug!("WebSocket duplex: waiting for data");
            let downstream_read =
                tokio::time::timeout(timeout_duration, server_session.read(&mut upstream_buf));
            let upstream_read =
                tokio::time::timeout(timeout_duration, client_session.read(&mut downstream_buf));
            let event: DuplexEvent;

            select! {
                result = downstream_read => match result {
                    Ok(Ok(n)) => {
                        log::debug!("WebSocket: Read {} bytes from client", n);
                        event = DuplexEvent::DownstreamRead(n)
                    },
                    Ok(Err(e)) => {
                        log::error!("WebSocket: Failed to read from client: {}", e);
                        return;
                    },
                    Err(_) => {
                        log::info!("WebSocket: Client read timeout - connection may be idle");
                        return;
                    }
                },
                result = upstream_read => match result {
                    Ok(Ok(n)) => {
                        log::debug!("WebSocket: Read {} bytes from server", n);
                        event = DuplexEvent::UpstreamRead(n)
                    },
                    Ok(Err(e)) => {
                        log::error!("WebSocket: Failed to read from server: {}", e);
                        return;
                    },
                    Err(_) => {
                        log::info!("WebSocket: Server read timeout - connection may be idle");
                        return;
                    }
                },
            }
            match event {
                DuplexEvent::DownstreamRead(0) => {
                    log::info!("WebSocket: Client closed connection (sent {} bytes, received {} bytes)", 
                               upstream_bytes, downstream_bytes);
                    return;
                }
                DuplexEvent::UpstreamRead(0) => {
                    log::info!("WebSocket: Server closed connection (sent {} bytes, received {} bytes)", 
                               upstream_bytes, downstream_bytes);
                    return;
                }
                DuplexEvent::DownstreamRead(n) => {
                    // Direct passthrough for WebSocket - no rewriting
                    match client_session.write_all(&upstream_buf[0..n]).await {
                        Ok(_) => {
                            upstream_bytes += n;
                            log::debug!("WebSocket: Forwarded {} bytes to server (total: {})", n, upstream_bytes);
                        },
                        Err(e) => {
                            log::error!("WebSocket: Failed to write to server: {}", e);
                            return;
                        }
                    };

                    match client_session.flush().await {
                        Ok(_) => {},
                        Err(e) => {
                            log::error!("WebSocket: Failed to flush data to server: {}", e);
                            return;
                        }
                    };
                }
                DuplexEvent::UpstreamRead(n) => {
                    match server_session.write_all(&downstream_buf[0..n]).await {
                        Ok(_) => {
                            downstream_bytes += n;
                            log::debug!("WebSocket: Forwarded {} bytes to client (total: {})", n, downstream_bytes);
                        },
                        Err(e) => {
                            log::error!("WebSocket: Failed to write to client: {}", e);
                            return;
                        }
                    };

                    match server_session.flush().await {
                        Ok(_) => {},
                        Err(e) => {
                            log::error!("WebSocket: Failed to flush data to client: {}", e);
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
    async fn process_new(
        self: &Arc<Self>,
        mut io: Stream,
        _shutdown: &ShutdownWatch,
    ) -> Option<Stream> {

        // Use buffer from the pool instead of allocating a new one
        let mut buffer = BUFFER_POOL.get();
        
        // Read initial data from client with zero-copy buffer
        log::debug!("Reading initial data from client");
        let n = match io.read_buf(&mut buffer).await {
            Ok(n) => {
                log::debug!("Read {} bytes from client", n);
                n
            },
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
        
        // Create a mutable length variable to track any rewrite changes
        let mut current_len = n;
        
        log::debug!("Raw request data (first 100 bytes): {:?}", &buffer[..current_len].iter().take(100).collect::<Vec<_>>());
        
        let conn_type = Self::detect_connection_type(&buffer[..current_len], current_len);
        log::debug!("Detected connection type: {:?}", conn_type);
        
        let host_info = match conn_type {
            ConnectionType::Tls => extract_sni_fast(&buffer[..current_len]),
            ConnectionType::Http | ConnectionType::WebSocket => extract_http_host(&buffer[..current_len], current_len),
            ConnectionType::Tcp => None,
        };

        if let Some(host) = host_info {
            log::info!("Host: {}", host);
            if let Some(sni) = &self.sni {
                if !host.contains(sni) {
                    log::warn!("Sni mismatch: {} != {}", host, sni);
                    BUFFER_POOL.put(buffer);
                    return None;
                }
            }
        }

        // Special handling for WebSocket connections
        let is_websocket = matches!(conn_type, ConnectionType::WebSocket);
        if is_websocket {
            log::info!("Detected WebSocket connection, will use direct passthrough");
        }

        // Check if there are any path routing rules and if any match for HTTP connections
        if (matches!(conn_type, ConnectionType::Http) || is_websocket) && !self.path_rewrites.is_empty() {
            log::debug!("Checking HTTP path rewrites, {} rules available", self.path_rewrites.len());
            
            // Convert buffer to string for checking against regex patterns
            if let Ok(request_str) = std::str::from_utf8(&buffer[..current_len]) {
                log::debug!("HTTP request: {}", request_str);
                
                // Parse the request line for validation
                if let Some(line_end) = request_str.find("\r\n") {
                    let request_line = &request_str[..line_end];
                    
                    // Extract path for checking
                    let parts: Vec<&str> = request_line.split_whitespace().collect();
                    if parts.len() == 3 {
                        let path = parts[1];
                        log::debug!("Extracted request path: {}", path);
                        
                        // Check if any rule matches the path
                        let mut has_match = false;
                        for rule in &self.path_rewrites {
                            if rule.pattern.is_match(path.as_bytes()) {
                                has_match = true;
                                log::info!("Path routing rule matched for: {}", path);
                                break;
                            }
                        }
                        
                        if !has_match {
                            log::warn!("No routing rule matches for path: {}", path);
                            BUFFER_POOL.put(buffer);
                            return None;
                        }
                    }
                }
                
                // Only rewrite the path for HTTP requests, leave WebSocket handshakes as is
                if !is_websocket {
                    // Now perform the actual rewrite on the initial request
                    current_len = self.rewrite_http_request(&mut buffer, current_len);
                    log::debug!("Initial request rewrite resulted in {} bytes", current_len);
                    
                    // Show the rewritten request
                    if let Ok(rewritten) = std::str::from_utf8(&buffer[..current_len]) {
                        log::debug!("Rewritten request: {}", rewritten);
                    }
                } else {
                    log::info!("Skipping path rewrite for WebSocket handshake");
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

        // Forward the initial data to the target server
        match client_session.write_all(&buffer[..current_len]).await {
            Ok(_) => {
                log::debug!("Successfully wrote {} bytes of initial request to upstream", current_len);
            }
            Err(e) => {
                log::error!("Failed to write to upstream peer: {}", e);
                BUFFER_POOL.put(buffer);
                return None;
            }
        };

        match client_session.flush().await {
            Ok(_) => {
                log::debug!("Successfully flushed initial data to upstream peer");
            }
            Err(e) => {
                log::error!("Failed to flush data to upstream peer: {}", e);
                BUFFER_POOL.put(buffer);
                return None;
            }
        };

        // Return buffer to the pool before entering duplex mode
        BUFFER_POOL.put(buffer);

        // Use different duplex mode for WebSocket connections
        if is_websocket {
            log::info!("Starting WebSocket duplex mode");
            self.duplex_websocket(io, client_session).await;
        } else {
            // Establish bidirectional communication
            self.duplex(io, client_session).await;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_path_rewrite() {
        // Create a rule to rewrite /test.png to /logo.png
        let pattern = Regex::new("/test\\.png").unwrap();
        let rewrite_rules = vec![
            RewriteRule {
                pattern,
                replacement: "/logo.png".to_string(),
            }
        ];
        
        // Create a ProxyApp with the rule
        let peer = BasicPeer::new("127.0.0.1:8080");
        let app = ProxyApp {
            client_connector: TransportConnector::new(None),
            proxy_to: peer,
            path_rewrites: rewrite_rules,
            sni: None,
        };
        
        // Test a GET request for /test.png
        let req = "GET /test.png HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let mut buffer = req.as_bytes().to_vec();
        buffer.resize(4096, 0); // Ensure buffer is large enough
        
        // Call the rewrite function
        let new_len = app.rewrite_http_request(&mut buffer, req.len());
        
        // Check the result
        let result = std::str::from_utf8(&buffer[..new_len]).unwrap();
        assert!(result.contains("GET /logo.png HTTP/1.1"), 
                "Expected rewritten path to /logo.png, got: {}", result);
        println!("Rewritten request: {}", result);
    }
}
