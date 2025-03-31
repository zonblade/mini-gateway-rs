// Protocol server implementation
use tokio::net::{TcpListener};
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use log;

use super::config::{ProtocolConfig, DEFAULT_ENABLED, DEFAULT_BUFFER_SIZE};
use super::connection::handle_connection;
use super::services::{init as init_services, SharedServiceHandler};

// Global shared service handler
static mut SERVICE_HANDLER: Option<SharedServiceHandler> = None;

/// Initialize the protocol server
///
/// Starts the protocol server if enabled in configuration and
/// initializes the service handler for processing requests.
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
    
    // Initialize service handler
    unsafe {
        SERVICE_HANDLER = Some(init_services());
        log::info!("Protocol service handler initialized");
    }
    
    run_server(listen_addr, buffer_size).await
}

/// Get the global service handler
/// 
/// Returns a reference to the global service handler for use by connection handlers.
fn get_service_handler() -> Option<SharedServiceHandler> {
    unsafe {
        SERVICE_HANDLER.as_ref().map(|handler| Arc::clone(handler))
    }
}

/// Run the protocol server
///
/// Listens for connections and handles them using the service handler.
async fn run_server(listen_addr: String, buffer_size: usize) -> io::Result<()> {
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
        // Accept new connections
        match listener.accept().await {
            Ok((socket, addr)) => {
                log::debug!("Accepted protocol connection from {}", addr);
                let conn_buffer_size = buffer_size;
                
                // Get service handler reference for this connection
                let service_handler = get_service_handler();
                
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, conn_buffer_size, service_handler).await {
                        log::error!("Protocol connection error: {}", e);
                    }
                });
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
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

/// Trigger a graceful shutdown of the server
pub fn shutdown(shutdown: &Arc<AtomicBool>) {
    shutdown.store(true, Ordering::Relaxed);
}