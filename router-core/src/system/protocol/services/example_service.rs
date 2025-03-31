use async_trait::async_trait;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::io::{self, AsyncWriteExt};
use log::{info, debug};

use crate::system::protocol::types::ConnectionParams;
use super::service_protocol::ServiceProtocol;

/// An example service implementation demonstrating how to implement the ServiceProtocol trait
pub struct ExampleService {
    name: String,
}

impl ExampleService {
    /// Create a custom instance with a specific name
    pub fn with_name(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl ServiceProtocol for ExampleService {
    fn new() -> Self {
        Self { 
            name: "example".to_string() 
        }
    }

    async fn upstream_peer(&self, socket: &mut TcpStream, buffer: &[u8], _buffer_size: usize, params: &ConnectionParams) -> io::Result<()> {
        // Example processing - echo the request with a prefix
        let request_str = String::from_utf8_lossy(buffer);
        debug!("Received request: {}", request_str);
        
        // Prepare response
        let response = format!("Service {} processed: {}", self.name, request_str);
        
        // Write response back to client
        socket.write_all(response.as_bytes()).await?;
        socket.flush().await?;
        
        Ok(())
    }

    async fn logging(&self, params: &ConnectionParams, status: Option<&str>, metrics: Option<HashMap<String, String>>) {
        // Log the request details
        let service = &params.service;
        let action = &params.action;
        
        let status_str = status.unwrap_or("unknown");
        
        let metrics_info = match metrics {
            Some(m) => {
                let mut info = String::new();
                for (k, v) in m {
                    info.push_str(&format!("{}={}, ", k, v));
                }
                info
            },
            None => "no metrics".to_string(),
        };
        
        info!(
            "Request [{}]: service={}, action={}, status={}, metrics=[{}]",
            self.name, service, action, status_str, metrics_info
        );
    }
}