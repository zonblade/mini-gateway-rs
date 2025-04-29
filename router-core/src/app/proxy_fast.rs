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

pub struct ProxyApp {
    client_connector: TransportConnector,
    proxy_to: BasicPeer,
    // Map of path rewrites (from -> to)
    path_rewrites: Vec<(String, String)>,
}

enum DuplexEvent {
    DownstreamRead(usize),
    UpstreamRead(usize),
}

impl ProxyApp {
    pub fn new(proxy_to: BasicPeer) -> Self {
        let mut path_rewrites = Vec::new();
        // Example: rewrite /logo.png to /image2.png
        path_rewrites.push(("/test.png".to_string(), "/logo.png".to_string()));
        
        ProxyApp {
            client_connector: TransportConnector::new(None),
            proxy_to,
            path_rewrites,
        }
    }
    
    // Add a path rewrite rule
    pub fn add_rewrite(&mut self, from: &str, to: &str) {
        self.path_rewrites.push((from.to_string(), to.to_string()));
    }

    // Simple HTTP request line parser and rewriter
    fn rewrite_http_request(&self, buffer: &mut [u8], length: usize) -> usize {
        // Convert to string for easier processing
        if let Ok(request_str) = std::str::from_utf8(&buffer[..length]) {
            // Check if this looks like an HTTP request
            if request_str.starts_with("GET ") || request_str.starts_with("POST ") 
               || request_str.starts_with("PUT ") || request_str.starts_with("DELETE ") {
                
                // For each rewrite rule
                for (from_path, to_path) in &self.path_rewrites {
                    // Look for the path in the request
                    let path_pattern = format!(" {} ", from_path);
                    let http_version_pattern = " HTTP/1.";
                    
                    if let Some(path_start) = request_str.find(&path_pattern) {
                        if let Some(http_pos) = request_str[path_start+1..].find(http_version_pattern) {
                            let path_end = http_pos + path_start + 1;
                            
                            // Found the path to rewrite
                            debug!("Rewriting path from {} to {}", from_path, to_path);
                            
                            // Create modified request
                            let before_path = &request_str[..path_start+1]; // +1 to include the space
                            let after_path = &request_str[path_end..];
                            let new_request = format!("{}{}{}", before_path, to_path, after_path);
                            
                            // Copy back to buffer
                            let new_bytes = new_request.as_bytes();
                            let new_len = new_bytes.len();
                            buffer[..new_len].copy_from_slice(new_bytes);
                            
                            return new_len;
                        }
                    }
                }
            }
        }
        
        // No rewrite performed, return original length
        length
    }

    async fn duplex(&self, mut server_session: Stream, mut client_session: Stream) {
        let mut upstream_buf = [0; 4096]; // Increased buffer size for HTTP headers
        let mut downstream_buf = [0; 4096];
        loop {
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
                    // Try to rewrite the request if it's HTTP
                    let write_len = self.rewrite_http_request(&mut upstream_buf, n);
                    
                    client_session.write_all(&upstream_buf[0..write_len]).await.unwrap();
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