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

/// Process a gate connection after a successful handshake
///
/// This function handles the main communication loop with the client
/// using the appropriate service based on the connection parameters.
///
/// If a service handler is provided and contains a service matching the
/// requested service name, that service will process the connection.
/// Otherwise, a simple echo response is used as a fallback.
async fn process_gate_connection(
    mut socket: TcpStream, 
    buffer_size: usize,
    params: ConnectionParams,
    service_handler: Option<SharedServiceHandler>
) -> io::Result<()> {
    let mut buffer = vec![0u8; buffer_size];
    
    // Log connection parameters
    if !params.service.is_empty() {
        log::info!("Connected to service: {}, action: {}", params.service, params.action);
    }
    
    // Process messages in a loop
    while let Ok(n) = socket.read(&mut buffer).await {
        if n == 0 {
            break;  // Connection closed
        }
        
        // Trim buffer to actual size
        let message_buffer = &buffer[..n];
        
        // Try to find and use the appropriate service if service handler exists
        if let Some(handler) = &service_handler {
            // Check if the service exists
            let service_exists = {
                let handler_guard = handler.read().await;
                handler_guard.get_service(&params.service).is_some()
            };
            
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

/// Main entry point for processing a new client connection
///
/// Handles the protocol handshake and passes the connection to the
/// appropriate handler based on the connection parameters.
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
        log::info!("Received valid protocol handshake: {}", handshake);
        
        // Extract connection parameters from handshake
        let params = parse_connection_params(&handshake, &protocol_prefix);
        
        // Send confirmation
        socket.write_all(b"Gate protocol handshake successful!\n").await?;
        
        // Process subsequent messages based on connection type
        process_gate_connection(socket, buffer_size, params, service_handler).await?;
    } else {
        // Invalid protocol
        log::warn!("Invalid protocol handshake received");
        socket.write_all(format!("Invalid protocol. Expected message to start with '{}'\n", protocol_prefix).as_bytes()).await?;
    }
    
    Ok(())
}