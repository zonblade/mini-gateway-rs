use async_trait::async_trait;
use log::debug;
use std::collections::HashMap;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;

use super::service_protocol::ServiceProtocol;
use crate::system::protocol::types::ConnectionParams;

/// # Example Service Implementation
///
/// This struct demonstrates a complete implementation of the `ServiceProtocol` trait,
/// providing a reference that can be used as a template for creating custom services.
///
/// The ExampleService implements a simple echo server that:
/// - Receives data from clients
/// - Formats a response that includes the received data
/// - Sends the response back to the client
/// - Logs information about the request and response
///
/// ## Features Demonstrated
///
/// This example demonstrates:
/// - Basic service implementation
/// - Request processing
/// - Response generation
/// - Asynchronous I/O handling
/// - Logging with metrics
/// - Custom initialization with different names
///
/// ## Usage
///
/// This service is intended to be registered with the service handler:
/// ```rust
/// let example1 = ExampleService::new();
/// let example2 = ExampleService::with_name("custom-echo".to_string());
///
/// let mut handler = service_handler.write().await;
/// handler.add_service("example".to_string(), Box::new(example1));
/// handler.add_service("custom-echo".to_string(), Box::new(example2));
/// ```
pub struct ExampleService {
    /// The name of this service instance
    name: String,
}

impl ExampleService {
    /// Create a custom instance with a specific name
    ///
    /// This factory method allows creating service instances with
    /// custom names, which can be useful for creating multiple
    /// variants of the same service.
    ///
    /// ## Parameters
    ///
    /// * `name` - Custom name for this service instance
    ///
    /// ## Returns
    ///
    /// A new ExampleService with the specified custom name
    pub fn with_name(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl ServiceProtocol for ExampleService {
    /// Creates a new instance with the default name "example"
    ///
    /// This implements the required `new()` method from the ServiceProtocol trait.
    /// It creates a service with a default name that can be registered with the handler.
    fn new() -> Self {
        Self {
            name: "example".to_string(),
        }
    }

    /// Process client requests and send responses
    ///
    /// This method implements the request-response cycle for the example service:
    /// 1. Decode the request from the buffer
    /// 2. Process the request (in this case, just prepare an echo response)
    /// 3. Send the response back to the client
    ///
    /// ## Example Behavior
    ///
    /// If a client sends "Hello world", the service will respond with:
    /// "Service example processed: Hello world"
    async fn upstream_peer(
        &self,
        socket: &mut TcpStream,
        buffer: &[u8],
        _buffer_size: usize,
        params: &ConnectionParams,
    ) -> io::Result<()> {
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

    /// Log information about processed requests
    ///
    /// This method demonstrates how to implement asynchronous logging for a service.
    /// It formats and logs information about the request, including:
    /// - The service name
    /// - The requested action
    /// - Processing status
    /// - Collected metrics
    ///
    /// ## Metrics Handling
    ///
    /// The method shows how to extract and format metrics from the provided HashMap,
    /// which could include timing, byte counts, or other measurements.
    async fn logging(
        &self,
        params: &ConnectionParams,
        status: Option<&str>,
        metrics: Option<HashMap<String, String>>,
    ) {
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
            }
            None => "no metrics".to_string(),
        };

        log::debug!(
            "Request [{}]: service={}, action={}, status={}, metrics=[{}]",
            self.name,
            service,
            action,
            status_str,
            metrics_info
        );
    }
}
