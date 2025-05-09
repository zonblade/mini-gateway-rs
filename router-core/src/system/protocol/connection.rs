// Connection handling for protocol server

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::io;
use std::sync::Arc;
use std::collections::HashMap;
use log;

use super::config::ProtocolConfig;
use super::types::ConnectionParams;
use super::parsing::parse_connection_params;
use super::services::SharedServiceHandler;

/// # Process Gate Connection
///
/// This function handles the main communication loop with the client
/// using the appropriate service based on the connection parameters.
///
/// ## Flow
///
/// 1. Allocates a message buffer of the configured size
/// 2. Enters a read loop to process messages from the client
/// 3. For each message:
///    - If a service handler is provided and a matching service exists:
///      - Delegates processing to the service's `upstream_peer` method
///      - Asynchronously logs the result using the service's `logging` method
///    - Otherwise, falls back to a simple echo response
/// 4. Continues until the connection is closed or an error occurs
///
/// ## Service Handling
///
/// When a service handler is available, this function:
/// - Looks up the requested service by name
/// - Uses the service's `upstream_peer` method for message processing
/// - Collects metrics such as bytes received
/// - Spawns a separate task for asynchronous logging to avoid blocking the response
///
/// ## Fallback
///
/// If no service is found or no handler is available, a simple echo response is sent back
/// that includes the requested service name and action in the format:
/// `Service: {name} | Action: {action} | Echo: {message}`
///
/// ## Parameters
///
/// * `socket` - The TCP socket connected to the client
/// * `buffer_size` - Size of the buffer to use for reading data
/// * `params` - Connection parameters parsed from the handshake
/// * `service_handler` - Optional service handler containing registered services
///
/// ## Returns
///
/// * `io::Result<()>` - Ok if the connection was handled successfully until graceful termination,
///   or Err if an I/O error occurred during communication.
async fn process_gate_connection(
    mut socket: TcpStream, 
    buffer_size: usize,
    params: ConnectionParams,
    service_handler: Option<SharedServiceHandler>
) -> io::Result<()> {
    let mut buffer = vec![0u8; buffer_size];
    
    // Log connection parameters
    if !params.service.is_empty() {
        eprintln!("[-TC-]   Connected to service: {}, action: {}", params.service, params.action);
    }
    
    // Process messages in a loop
    while let Ok(n) = socket.read(&mut buffer).await {
        if n == 0 {
            break;  // Connection closed
        }
        
        // Trim buffer to actual size
        let message_buffer = &buffer[..n];
        
        // Log incoming data before it reaches service
        let message_str = String::from_utf8_lossy(message_buffer);
        eprintln!("[-TC-]   Incoming data for service '{}'", params.service);

        // Try to find and use the appropriate service if service handler exists
        if let Some(handler) = &service_handler {
            // Check if the service exists
            let service_exists = {
                let handler_guard = handler.read().await;
                eprintln!("[-TC-]   Checking for service '{}'", params.service);
                handler_guard.get_service(&params.service).is_some()
            };

            eprintln!("[-TC-]   Service '{}' exists: {}", params.service, service_exists);

            if service_exists {
                // Get the service again for processing
                let result = {
                    let handler_guard = handler.read().await;
                    let service = handler_guard.get_service(&params.service).unwrap();
                    // Clone the necessary data before dropping the guard
                    service.upstream_peer(&mut socket, message_buffer, buffer_size, &params).await
                };
                
                // Log the result
                let status = match &result {
                    Ok(_) => Some("success"),
                    Err(e) => {
                        log::error!("Error processing request: {}", e);
                        Some("error")
                    }
                };
                
                // Collect metrics
                let mut metrics = HashMap::new();
                metrics.insert("bytes_received".to_string(), n.to_string());
                
                // Log asynchronously without blocking the response
                let params_clone = params.clone();
                let service_name_clone = params.service.clone();
                let handler_clone = Arc::clone(handler);
                
                tokio::spawn(async move {
                    let handler_guard = handler_clone.read().await;
                    if let Some(service) = handler_guard.get_service(&service_name_clone) {
                        service.logging(&params_clone, status, Some(metrics)).await;
                    }
                });
                
                if let Err(e) = result {
                    return Err(e);
                }
                
                eprintln!("[-TC-]   Successfully processed message for service '{}'", params.service);
                continue;
            }
        }
        
        // No service found or no handler available - use fallback echo behavior
        let message = String::from_utf8_lossy(message_buffer);
        let response = if !params.service.is_empty() {
            format!("Service: {} | Action: {} | Echo: {}\n", 
                    params.service, params.action, message)
        } else {
            format!("Echo: {}\n", message)
        };
        
        socket.write_all(response.as_bytes()).await?;
    }
    
    Ok(())
}

/// # Handle Connection
///
/// Main entry point for processing a new client connection to the protocol server.
///
/// This function orchestrates the protocol handshake and subsequent message processing.
/// It acts as the primary interface between the TCP socket accepting code and 
/// the protocol-specific message handling logic.
///
/// ## Connection Lifecycle
///
/// 1. Accept initial handshake data from the client
/// 2. Validate against the expected protocol prefix
/// 3. If valid:
///    - Parse connection parameters from the handshake
///    - Send a confirmation response
///    - Pass the connection to `process_gate_connection` for message handling
/// 4. If invalid:
///    - Send an error message with the expected prefix
///    - Close the connection
///
/// ## Protocol Validation
///
/// The function expects the handshake message to start with the configured protocol prefix
/// (default: "gate://"). This ensures that only clients speaking the correct protocol
/// are allowed to establish a connection.
///
/// ## Service Selection
///
/// The handshake message contains information about which service the client wants to use.
/// This information is extracted into connection parameters and used for routing messages
/// to the appropriate service handler.
///
/// ## Thread Safety
///
/// This function can be safely called from multiple threads, as it operates on a
/// unique socket per connection. The service handler is wrapped in an Arc<RwLock<>>
/// to allow safe concurrent access across multiple connections.
///
/// ## Parameters
///
/// * `socket` - The TCP socket connected to the client
/// * `buffer_size` - Size of the buffer to use for reading data
/// * `service_handler` - Optional service handler containing registered services
///
/// ## Returns
///
/// * `io::Result<()>` - Ok if the connection was handled successfully,
///   or Err if an I/O error occurred during communication.
pub async fn handle_connection(
    mut socket: TcpStream, 
    buffer_size: usize,
    service_handler: Option<SharedServiceHandler>
) -> io::Result<()> {
    let mut buffer = vec![0u8; buffer_size];
    let protocol_prefix = ProtocolConfig::ProtocolPrefix.val();

    // Read initial handshake data
    let n = socket.read(&mut buffer).await?;
    if n == 0 {
        return Ok(());  // Connection closed
    }

    // Process handshake
    let handshake = String::from_utf8_lossy(&buffer[..n]);
    
    if handshake.starts_with(&protocol_prefix) {
        eprintln!("[-TC-]   Received valid protocol handshake: {}", handshake);
        
        // Extract connection parameters from handshake
        let params = parse_connection_params(&handshake, &protocol_prefix);
        
        eprintln!("[-TC-]   Parsed connection parameters: {:?}", params);
        // Send confirmation
        socket.write_all(b"Gate protocol handshake successful!\n").await?;
        
        eprintln!("[-TC-]   Sending confirmation to client");
        // Process subsequent messages based on connection type
        process_gate_connection(socket, buffer_size, params, service_handler).await?;
    } else {
        // Invalid protocol
        eprintln!("[-TC-]   Invalid protocol handshake received");
        socket.write_all(format!("Invalid protocol. Expected message to start with '{}'\n", protocol_prefix).as_bytes()).await?;
    }
    
    Ok(())
}