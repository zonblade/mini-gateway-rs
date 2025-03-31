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

// Re-export public items
pub use config::{ProtocolConfig, init_config};
pub use server::{init, run_server, shutdown};
pub use connection::handle_connection;
pub use parsing::parse_connection_params;
pub use types::ConnectionParams;