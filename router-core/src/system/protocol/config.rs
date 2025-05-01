// Protocol configuration

use mini_config::Configure;

/// # Protocol Configuration Enum
///
/// Configuration parameters for the MQLESS protocol server.
///
/// This enum defines the available configuration options for the protocol
/// server using the `mini_config` crate's `Configure` trait. It provides
/// a type-safe way to access different configuration values related to
/// the protocol implementation.
///
/// ## Configuration Options
///
/// * `ListenAddr` - The network address and port the protocol server will listen on.
///   Default is `127.0.0.1:30099`.
///
/// * `BufferSize` - Size of the buffer (in bytes) for reading from and writing to socket connections.
///   Default is 16384 bytes.
///
/// * `ProtocolPrefix` - Prefix string that identifies messages using this protocol.
///   Default is `gate://`.
///
/// * `Enabled` - Whether the protocol server is enabled or disabled.
///   Default is `true`.
///
/// ## Usage
///
/// ```rust
/// // Get the configured listen address
/// let addr = ProtocolConfig::ListenAddr.val();
///
/// // Check if protocol server is enabled
/// let enabled = ProtocolConfig::Enabled.xget::<bool>().unwrap_or(DEFAULT_ENABLED);
///
/// // Change buffer size
/// ProtocolConfig::BufferSize.set("2048");
/// ```
///
/// ## Integration
///
/// This configuration integrates with the `mini_config` system, providing
/// a consistent configuration interface across the application.
#[derive(Debug, Clone, Configure)]
pub enum ProtocolConfig {
    /// Network address the protocol server will listen on (e.g., "127.0.0.1:30099")
    ListenAddr,
    
    /// Size of the buffer for socket I/O operations
    BufferSize,
    
    /// Prefix string that identifies this protocol (e.g., "gate://")
    ProtocolPrefix,
    
    /// Whether the protocol server is enabled
    Enabled,
}

/// Default address for the protocol server to listen on
///
/// This is the IP address and port where the protocol server will bind by default
/// if no specific value is set in the configuration.
pub const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:30099";

/// Default buffer size for socket operations in bytes
///
/// This determines how much data can be read from or written to a socket in a single operation.
/// The value represents a balance between memory usage and efficient I/O operations.
pub const DEFAULT_BUFFER_SIZE: usize = 16384;

/// Default prefix that identifies protocol messages
///
/// This string prefix is used to identify whether an incoming message is using
/// this protocol. Client connection attempts must start with this prefix.
pub const DEFAULT_PROTOCOL_PREFIX: &str = "gate://";

/// Default enabled state for the protocol server
///
/// When `true`, the protocol server will start automatically when the application runs.
/// When `false`, the protocol server component will not be started.
pub const DEFAULT_ENABLED: bool = true;

/// Initialize protocol configuration with default values
///
/// This function sets up the initial configuration for the protocol server
/// using the default constants defined in this module. It should be called
/// during application startup before any protocol operations are performed.
///
/// # Example
///
/// ```rust
/// // Initialize protocol configuration at application startup
/// fn main() {
///     protocol::init_config();
///     // ... rest of application initialization
/// }
/// ```
pub fn init_config() {
    ProtocolConfig::ListenAddr.set(DEFAULT_LISTEN_ADDR);
    ProtocolConfig::BufferSize.set(&DEFAULT_BUFFER_SIZE.to_string());
    ProtocolConfig::ProtocolPrefix.set(DEFAULT_PROTOCOL_PREFIX);
    ProtocolConfig::Enabled.xset::<bool>(DEFAULT_ENABLED);
}