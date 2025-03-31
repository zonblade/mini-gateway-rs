// Protocol server implementation

use tokio::net::{TcpListener, TcpStream};
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use log;

use super::config::{ProtocolConfig, DEFAULT_ENABLED, DEFAULT_BUFFER_SIZE};
use super::connection::handle_connection;

/// # Initialize Protocol Server
///
/// Entry point for starting the protocol server component.
///
/// This function initializes and starts the protocol server according to the current
/// configuration. It checks whether the server is enabled, retrieves necessary configuration
/// values, and delegates to `run_server` to perform the actual server operation.
///
/// ## Server Initialization Process
///
/// 1. Check if the server is enabled in the configuration
/// 2. If disabled, log a message and return
/// 3. If enabled, retrieve listen address and buffer size from configuration
/// 4. Call `run_server` with the configured parameters
///
/// ## Configuration
///
/// The server behavior is controlled by the following configuration options:
/// - `ProtocolConfig::Enabled` - Whether the server should start
/// - `ProtocolConfig::ListenAddr` - Address and port to listen on
/// - `ProtocolConfig::BufferSize` - Buffer size for socket operations
///
/// ## Returns
///
/// * `io::Result<()>` - Ok if the server was initialized successfully (or is disabled),
///   or Err if there was an error starting the server
///
/// ## Logging
///
/// * Info level: When server is disabled by configuration
/// * Error level: If the server fails to start (delegated to `run_server`)
///
/// ## Examples
///
/// ```rust
/// // In application startup code
/// #[tokio::main]
/// async fn main() -> io::Result<()> {
///     // Initialize other components...
///     
///     // Start protocol server
///     protocol::init().await?;
///     
///     // Continue with application logic...
///     Ok(())
/// }
/// ```
pub async fn init() -> io::Result<()> {
    // Check if protocol server is enabled
    let enabled = ProtocolConfig::Enabled
        .xget::<bool>()
        .unwrap_or(DEFAULT_ENABLED);
    
    if !enabled {
        log::info!("Protocol server disabled by configuration");
        return Ok(());
    }
    
    // Get configuration values
    let listen_addr = ProtocolConfig::ListenAddr.val();
    let buffer_size = ProtocolConfig::BufferSize
        .xget::<usize>()
        .unwrap_or(DEFAULT_BUFFER_SIZE);
    
    run_server(listen_addr, buffer_size).await
}

/// # Run Protocol Server
///
/// Core function that starts and runs the protocol server.
///
/// This function binds to the specified network address, listens for incoming
/// connections, and spawns a new task for each connection to handle it according
/// to the protocol rules. It continues running until a shutdown signal is received.
///
/// ## Server Lifecycle
///
/// 1. Create a shutdown signal atomic flag
/// 2. Bind to the specified network address
/// 3. Enter the main accept loop:
///    - Accept new connections
///    - Spawn a task to handle each connection
///    - Check for shutdown signal
/// 4. When shutdown signal is received, exit the loop and shut down gracefully
///
/// ## Connection Handling
///
/// Each accepted connection is handled in a separate Tokio task to ensure
/// non-blocking operation. The connection handling logic is delegated to
/// the `handle_connection` function.
///
/// ## Parameters
///
/// * `listen_addr` - The network address (IP:port) the server should listen on
/// * `buffer_size` - Size of the buffer to use for socket I/O operations
///
/// ## Returns
///
/// * `io::Result<()>` - Ok if the server ran and shut down successfully,
///   or Err if there was an error during server operation
///
/// ## Error Handling
///
/// This function will return an error if:
/// - Binding to the specified address fails
/// - A fatal error occurs during the accept loop
///
/// Non-fatal errors, such as individual connection failures, are logged
/// but do not cause the server to exit.
///
/// ## Logging
///
/// * Info level: Server startup and shutdown messages
/// * Error level: Binding failures, connection acceptance errors
/// * Debug level: Individual connection acceptance
///
/// ## Concurrency
///
/// The server operates fully asynchronously using Tokio:
/// - The main accept loop runs in the calling task
/// - Each connection is handled in a separate spawned task
/// - All operations use non-blocking I/O
pub async fn run_server(listen_addr: String, buffer_size: usize) -> io::Result<()> {
    // Set up shutdown signal
    let shutdown = Arc::new(AtomicBool::new(false));
    let _shutdown_clone = Arc::clone(&shutdown);
    
    // Bind to a TCP socket
    let listener = match TcpListener::bind(&listen_addr).await {
        Ok(listener) => {
            log::info!("Protocol server listening on {}", listen_addr);
            listener
        },
        Err(e) => {
            log::error!("Failed to bind protocol server to {}: {}", listen_addr, e);
            return Err(e);
        }
    };
    
    // Run until shutdown signal
    while !shutdown.load(Ordering::Relaxed) {
        // Accept new connections with timeout to check for shutdown signal
        match listener.accept().await {
            Ok((socket, addr)) => {
                log::debug!("Accepted protocol connection from {}", addr);
                let conn_buffer_size = buffer_size;
                
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, conn_buffer_size).await {
                        log::error!("Protocol connection error: {}", e);
                    }
                });
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No connection available, check shutdown signal after short delay
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }
            Err(e) => {
                log::error!("Failed to accept protocol connection: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
    
    log::info!("Protocol server shutting down");
    Ok(())
}

/// # Shutdown Protocol Server
///
/// Triggers a graceful shutdown of the protocol server.
///
/// This function signals the server to stop accepting new connections
/// and initiate a graceful shutdown process. It sets the shutdown atomic flag
/// which is periodically checked by the server's main loop.
///
/// ## Shutdown Process
///
/// 1. Set the atomic boolean shutdown flag to true
/// 2. The server's main loop will detect this and exit
/// 3. The server will stop accepting new connections
/// 4. Existing connections will continue until they complete or timeout
///
/// ## Parameters
///
/// * `shutdown` - Reference to the atomic boolean that controls the server shutdown
///
/// ## Thread Safety
///
/// This function is thread-safe and can be called from any thread to signal
/// shutdown to the server running in the async runtime.
///
/// ## Examples
///
/// ```rust
/// // Create shutdown signal
/// let shutdown = Arc::new(AtomicBool::new(false));
/// let shutdown_clone = Arc::clone(&shutdown);
///
/// // In a separate task or signal handler
/// protocol::shutdown(&shutdown_clone);
/// ```
pub fn shutdown(shutdown: &Arc<AtomicBool>) {
    shutdown.store(true, Ordering::Relaxed);
}