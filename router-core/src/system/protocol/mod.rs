/// # Protocol Module
///
/// The protocol module implements the MQLESS (Message Queue-Less) protocol server for
/// the router-core system. This custom protocol facilitates direct communication between
/// clients and the router gateway without requiring an intermediate message queue.
///
/// ## Overview
///
/// This module provides a TCP socket-based protocol implementation that:
/// - Listens for incoming connections on a configurable address
/// - Performs protocol handshakes with connecting clients
/// - Parses connection parameters to determine routing
/// - Handles message exchange according to the protocol specification
///
/// ## Protocol Specification
///
/// The protocol uses a URI-like format for handshakes:
/// ```
/// gate://<service_name>/<action>?<param1>=<value1>&<param2>=<value2>...
/// ```
///
/// After handshake, messages are exchanged as plain text with service-specific formatting.
///
/// ## Module Structure
///
/// - `config`: Protocol configuration settings and defaults
/// - `types`: Data structures used throughout the protocol implementation
/// - `server`: Server initialization and TCP connection acceptance
/// - `connection`: Connection handling and protocol message processing
/// - `parsing`: Utilities for parsing protocol messages and parameters
/// - `services`: Trait-based service system for dynamic service handling
///
/// ## Usage
///
/// To start the protocol server:
///
/// ```rust
/// use router_core::system::protocol;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     // Initialize protocol configuration
///     protocol::init_config();
///     
///     // Start the protocol server
///     protocol::init().await?;
///     
///     // ... rest of application logic
///     
///     Ok(())
/// }
/// ```
///
/// ## Configuration
///
/// The protocol behavior can be configured through the `ProtocolConfig` enum:
///
/// ```rust
/// use router_core::system::protocol::ProtocolConfig;
///
/// // Change the listen address
/// ProtocolConfig::ListenAddr.set("0.0.0.0:30099");
///
/// // Change buffer size
/// ProtocolConfig::BufferSize.set("2048");
///
/// // Disable the protocol server
/// ProtocolConfig::Enabled.xset::<bool>(false);
/// ```

mod config;
mod types;
mod server;
mod connection;
mod parsing;
pub mod services;

use std::thread;

// Re-export public items
use config::{ProtocolConfig, init_config};
use server::{init, shutdown};
use connection::handle_connection;
use parsing::parse_connection_params;
use services::ServiceProtocol;
use types::ConnectionParams;

pub fn start_interface(){
    init_config();
    thread::spawn(|| {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        runtime.block_on(async {
            // Initialize the service handler
            let service_handler = services::init();
            
            // Register example services
            let mut handler = service_handler.write().await;
            
            // Example of registering services
            let example_service = services::ExampleService::new();
            let example_service_2 = services::ExampleService::with_name("echo".to_string());
            
            // Create the services list
            let services = vec![
                services::register_service("example", example_service),
                services::register_service("echo", example_service_2),
            ];
            
            // Add services to the handler
            handler.add_services(services);
            
            // Run the service handler
            handler.join();
            
            // Release the write lock before starting the server
            drop(handler);
            
            // Start the protocol server
            if let Err(e) = init().await {
                log::error!("Protocol server failed to start: {}", e);
            }
        });
    });
}