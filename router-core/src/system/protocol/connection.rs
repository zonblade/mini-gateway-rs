// Connection handling for protocol server

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::io;
use log;

use super::config::ProtocolConfig;
use super::types::ConnectionParams;
use super::parsing::parse_connection_params;

/// # Process Gate Connection
///
/// Handles an established protocol connection after successful handshake.
///
/// This function is responsible for the main communication loop with a client
/// after it has successfully completed the protocol handshake. It reads messages
/// from the client, processes them according to the protocol rules, and sends
/// appropriate responses back.
///
/// ## Connection Lifecycle
///
/// 1. Reads data from socket into buffer
/// 2. Processes the received message using `handle_protocol_message`
/// 3. Sends response back to client
/// 4. Repeats until connection is closed or error occurs
///
/// ## Parameters
///
/// * `socket` - The TCP socket connected to the client
/// * `buffer_size` - Size of the buffer to use for reading data
/// * `params` - Connection parameters parsed from the handshake
///
/// ## Returns
///
/// * `io::Result<()>` - Ok if the connection was handled successfully until graceful termination,
///   or Err if an I/O error occurred during communication.
///
/// ## Error Handling
///
/// This function will return an error if:
/// - Reading from the socket fails
/// - Writing to the socket fails
///
/// ## Logging
///
/// * Info level: Connection details including service and action
/// * Error level: I/O errors during communication
async fn process_gate_connection(
    mut socket: TcpStream, 
    buffer_size: usize,
    params: ConnectionParams
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
        
        // Process the message based on the protocol
        let message = String::from_utf8_lossy(&buffer[..n]);
        
        // For now, we just echo the message back
        // This can be expanded to handle different message types
        let response = handle_protocol_message(&message, &params);
        socket.write_all(response.as_bytes()).await?;
    }
    
    Ok(())
}

/// # Handle Protocol Message
///
/// Processes a message received from a client according to protocol rules.
///
/// This function interprets the received message based on the connection parameters
/// and generates an appropriate response. Currently, it implements a simple echo
/// service that returns the message with some additional context, but this can be
/// expanded to handle different message types and service-specific logic.
///
/// ## Protocol Message Processing
///
/// The current implementation is a simple echo service, but in a production environment,
/// this would be expanded to:
/// - Validate message format
/// - Route messages to appropriate services
/// - Apply protocol-specific transformations
/// - Handle different message types
///
/// ## Parameters
///
/// * `message` - The message received from the client as a string
/// * `params` - Connection parameters containing service and action information
///
/// ## Returns
///
/// A string response to be sent back to the client.
///
/// ## Examples
///
/// ```
/// let params = ConnectionParams {
///     service: "echo".to_string(),
///     action: "uppercase".to_string(),
///     parameters: HashMap::new(),
/// };
///
/// let response = handle_protocol_message("hello world", &params);
/// // Returns: "Service: echo | Action: uppercase | Echo: hello world"
/// ```
fn handle_protocol_message(message: &str, params: &ConnectionParams) -> String {
    // This function can be expanded to handle different message formats and services
    // For now, it just echoes the message
    
    // Check if we have a specific service handler
    if !params.service.is_empty() {
        format!("Service: {} | Action: {} | Echo: {}\n", 
                params.service, params.action, message)
    } else {
        format!("Echo: {}\n", message)
    }
}

/// # Handle Connection
///
/// Main entry point for processing a new client connection.
///
/// This function orchestrates the protocol handshake process and subsequent communication.
/// It first reads the initial handshake message from the client, validates it against the
/// expected protocol prefix, and if valid, proceeds to handle the connection based on the
/// extracted parameters.
///
/// ## Connection Flow
///
/// 1. Read initial handshake data from client
/// 2. Verify the protocol prefix
/// 3. Parse connection parameters from handshake
/// 4. Send handshake confirmation to client
/// 5. Process subsequent messages according to protocol
///
/// ## Parameters
///
/// * `socket` - The TCP socket connected to the client
/// * `buffer_size` - Size of the buffer to use for reading data
///
/// ## Returns
///
/// * `io::Result<()>` - Ok if the connection was handled successfully,
///   or Err if an I/O error occurred during communication.
///
/// ## Protocol Validation
///
/// The function expects the first message from the client to start with the configured
/// protocol prefix (default: "gate://"). If the message doesn't match this format,
/// an error message is sent to the client and the connection is closed.
///
/// ## Logging
///
/// * Info level: Valid handshake received
/// * Warn level: Invalid handshake received
/// * Error level: I/O errors during communication
pub async fn handle_connection(mut socket: TcpStream, buffer_size: usize) -> io::Result<()> {
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
        process_gate_connection(socket, buffer_size, params).await?;
    } else {
        // Invalid protocol
        log::warn!("Invalid protocol handshake received");
        socket.write_all(format!("Invalid protocol. Expected message to start with '{}'\n", protocol_prefix).as_bytes()).await?;
    }
    
    Ok(())
}