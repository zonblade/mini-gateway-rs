// Protocol server implementation
use log;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;

use super::config::{ProtocolConfig, DEFAULT_BUFFER_SIZE, DEFAULT_ENABLED};
use super::connection::handle_connection;
use super::services::{init as init_services, SharedServiceHandler};

/// # Global Service Handler
///
/// This static variable holds a reference to the global service handler that is shared across
/// all connections. It's initialized during server startup and used to route client requests
/// to the appropriate service implementations.
///
/// Safety: This is unsafe because we're using a static mutable variable. The access is
/// controlled through the `get_service_handler` function which returns a cloned Arc reference,
/// and all mutations happen in a controlled manner during initialization.
static mut SERVICE_HANDLER: Option<SharedServiceHandler> = None;

/// # Initialize the Protocol Server
///
/// This function is the main entry point for starting the protocol server. It:
/// 
/// 1. Checks if the server is enabled via configuration
/// 2. If enabled, retrieves necessary configuration (listen address, buffer size)
/// 3. Initializes the service handler (creates a new instance if not already present)
/// 4. Starts the server by calling the `run_server` function
///
/// ## Configuration Options
///
/// * `ProtocolConfig::Enabled` - Boolean that determines if the server should start
/// * `ProtocolConfig::ListenAddr` - The address and port to listen on (e.g. "0.0.0.0:30099")
/// * `ProtocolConfig::BufferSize` - Size of the buffer for socket I/O operations
///
/// ## Service Handler
///
/// The service handler is initialized once during server startup and stored in a static
/// variable to be shared across all connections. This allows all connections to access
/// the same set of registered services.
///
/// ## Parameters
///
/// * `external_handler` - An optional pre-initialized service handler to use instead of creating a new one
///
/// ## Returns
///
/// * `io::Result<()>` - Ok if the server started successfully (or is disabled by config),
///   or Err if there was an error starting the server
#[allow(static_mut_refs)]
pub async fn init(external_handler: Option<SharedServiceHandler>) -> io::Result<()> {
    // Check if protocol server is enabled
    let enabled = ProtocolConfig::Enabled
        .xget::<bool>()
        .unwrap_or(DEFAULT_ENABLED);

    if !enabled {
        log::debug!("Protocol server disabled by configuration");
        return Ok(());
    }

    // Get configuration values
    let listen_addr = ProtocolConfig::ListenAddr.val();
    let buffer_size = ProtocolConfig::BufferSize
        .xget::<usize>()
        .unwrap_or(DEFAULT_BUFFER_SIZE);

    // Initialize service handler
    unsafe {
        if let Some(handler) = external_handler {
            // Use the provided external handler
            SERVICE_HANDLER = Some(handler);
            log::debug!("Protocol server using externally provided service handler");
        } else {
            // Create a new handler if none was provided
            SERVICE_HANDLER = Some(init_services());
            log::debug!("Protocol server created new service handler");
        }
        
        // Log service count in the handler
        if let Some(handler) = &SERVICE_HANDLER {
            let guard = handler.read().await;
            let service_count = guard.get_services().len();
            let service_names: Vec<String> = guard.get_services().keys().cloned().collect();
            log::debug!("Protocol service handler initialized with {} services: {:?}", service_count, service_names);
        }
    }

    run_server(listen_addr, buffer_size).await
}

/// # Get the Global Service Handler
///
/// This function safely retrieves a reference to the global service handler which contains
/// all the registered services. It returns a cloned Arc reference to ensure thread safety
/// when the service handler is used across multiple connections simultaneously.
///
/// ## Thread Safety
///
/// This function uses unsafe code to access the static SERVICE_HANDLER variable, but
/// the overall pattern is safe because:
/// 
/// 1. We only read the reference, never modify it through this function
/// 2. We clone the Arc pointer, not the data itself
/// 3. The actual service handler is protected by a RwLock for concurrent access
///
/// ## Returns
///
/// * `Option<SharedServiceHandler>` - Some(handler) if the service handler is initialized,
///   or None if it hasn't been initialized yet
#[allow(static_mut_refs)]
fn get_service_handler() -> Option<SharedServiceHandler> {
    unsafe { SERVICE_HANDLER.as_ref().map(|handler| Arc::clone(handler)) }
}

/// # Run the Protocol Server
///
/// This function performs the actual work of running the protocol server. It:
///
/// 1. Sets up a shutdown signal mechanism
/// 2. Binds to the specified network address
/// 3. Enters a loop to accept and handle connections
/// 4. Spawns each connection in its own task for concurrent processing
/// 5. Continues until a shutdown signal is received
///
/// ## Connection Handling
///
/// Each connection is handled asynchronously in its own Tokio task. The function:
/// 
/// 1. Accepts a connection from the listener
/// 2. Retrieves a reference to the service handler
/// 3. Spawns a task to handle the connection
/// 4. The task calls `handle_connection` to process the connection
///
/// ## Error Handling
///
/// The function uses a robust error handling strategy:
///
/// * Binding failures are logged and immediately return an error
/// * Connection acceptance errors are logged but the server continues running
/// * Connection handling errors are logged within the spawned tasks
/// * A delay is added after accept errors to prevent CPU spinning
///
/// ## Parameters
///
/// * `listen_addr` - The address to bind to, in the format "ip:port"
/// * `buffer_size` - The size of buffers to use for socket operations
///
/// ## Returns
///
/// * `io::Result<()>` - Ok if the server ran and shut down gracefully,
///   or Err if there was an error binding to the address
async fn run_server(listen_addr: String, buffer_size: usize) -> io::Result<()> {
    eprintln!("[----]   $ Starting protocol server on {}", listen_addr);
    // Set up shutdown signal
    let shutdown = Arc::new(AtomicBool::new(false));
    let _shutdown_clone = Arc::clone(&shutdown);

    // Bind to a TCP socket
    let listener = match TcpListener::bind(&listen_addr).await {
        Ok(listener) => {
            eprintln!("[----]   $ Protocol server listening on {}", listen_addr);
            // println!("Success to bind {}", listen_addr);
            listener
        }
        Err(e) => {
            eprintln!("[ERR-]   $ Failed to bind protocol server to {}: {}", listen_addr, e);
            // println!("Failed to bind protocol server to {}: {}", listen_addr, e);
            return Err(e);
        }
    };

    // Run until shutdown signal
    while !shutdown.load(Ordering::Relaxed) {
        // Accept new connections
        match listener.accept().await {
            Ok((socket, addr)) => {
                let timestamp = chrono::Local::now();
                let rfc_3339 = timestamp.to_rfc3339();
                eprintln!("[----]   $ Accepted protocol connection from {}  <[{}]>", addr, rfc_3339);
                let conn_buffer_size = buffer_size;

                // Get service handler reference for this connection
                let service_handler = get_service_handler();

                tokio::spawn(async move {
                    if let Err(e) =
                        handle_connection(socket, conn_buffer_size, service_handler).await
                    {
                        eprintln!("[ERR-]   $ Protocol connection error: {}", e);
                    }
                });
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }
            Err(e) => {
                eprintln!("[ERR-]   $ Failed to accept protocol connection: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }

    eprintln!("[----]   $ Protocol server shutting down");
    Ok(())
}

/// # Trigger a Graceful Shutdown of the Server
///
/// This function signals the server to stop accepting new connections and initiate
/// a graceful shutdown. It sets the atomic boolean flag that is checked in the
/// server's main loop.
///
/// ## Graceful Shutdown Process
///
/// When this function is called, the server will:
///
/// 1. Stop accepting new connections once the current accept operation completes
/// 2. Allow existing connections to complete naturally
/// 3. Exit the main server loop
/// 4. The server task will complete, allowing the runtime to clean up resources
///
/// ## Thread Safety
///
/// This function is thread-safe and can be called from any thread or task to trigger
/// the shutdown process. The atomic flag ensures visibility across threads.
///
/// ## Parameters
///
/// * `shutdown` - Reference to the atomic boolean that controls server shutdown
#[allow(dead_code)]
pub fn shutdown(shutdown: &Arc<AtomicBool>) {
    shutdown.store(true, Ordering::Relaxed);
}
