//! # Usage Examples for the Protocol Client
//! 
//! This module provides example code and structures that demonstrate how to use
//! the protocol client in various scenarios. It includes example payload and response
//! types as well as a complete example function showing the client in action.
//! 
//! ## Example Types
//! 
//! The module includes sample data structures that implement the necessary traits
//! for use with the client:
//! 
//! - `LoginPayload`: An example request payload for authentication
//! - `LoginResponse`: An example response structure for authentication
//! 
//! ## Complete Usage Example
//! 
//! The `example_usage()` function demonstrates the full lifecycle of using the client:
//! 
//! 1. Creating and connecting a client
//! 2. Setting service and parameters
//! 3. Preparing a payload
//! 4. Executing an action
//! 5. Processing the response
//! 6. Closing the connection
//! 
//! ## Extending with Your Own Types
//! 
//! You can follow the patterns shown here to create your own payload and response
//! types for your specific application needs.

use serde::{Serialize, Deserialize};

use crate::client::error::Result;
use crate::client::client::Client;

/// Example payload for login requests.
/// 
/// This struct represents a typical authentication request with username
/// and password fields. It automatically implements the `Payload` trait
/// due to the blanket implementation for types that implement `Serialize` and `Debug`.
/// 
/// # Fields
/// 
/// * `username` - The user's login name
/// * `password` - The user's password
/// 
/// # Example Usage
/// 
/// ```rust
/// let payload = LoginPayload {
///     username: "admin".to_string(),
///     password: "secure123".to_string(),
/// };
/// 
/// let response: LoginResponse = client
///     .with_service("auth")
///     .action("login", &payload)
///     .await?;
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    /// Username for login
    pub username: String,
    /// Password for login
    pub password: String,
}

/// Example response for login requests.
/// 
/// This struct represents a typical authentication response containing
/// success status, a possible token, and a message. It's used as the generic
/// type parameter when deserializing responses from authentication actions.
/// 
/// # Fields
/// 
/// * `success` - Boolean indicating if the login was successful
/// * `token` - Optional authentication token returned on successful login
/// * `message` - Descriptive message about the login result
/// 
/// # Example
/// 
/// ```rust
/// if response.success {
///     println!("Login successful! Token: {}", response.token.unwrap_or_default());
/// } else {
///     println!("Login failed: {}", response.message);
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    /// Indicates if the login was successful
    pub success: bool,
    /// Optional token returned on successful login
    pub token: Option<String>,
    /// Message describing the result of the login
    pub message: String,
}

/// Complete example of how to use the client for a login flow.
/// 
/// This function demonstrates the entire lifecycle of client usage,
/// from connection to request execution to connection closure.
/// 
/// # Protocol Flow
/// 
/// 1. Creates a new client
/// 2. Connects to the server at 127.0.0.1:30099
/// 3. Prepares a login payload with username and password
/// 4. Configures the client with service name and parameters
/// 5. Executes the login action and receives a typed response
/// 6. Closes the connection
/// 
/// # Returns
/// 
/// Returns a Result indicating success or failure of the operation
/// 
/// # Errors
/// 
/// Returns various `ClientError` types if any stage of the operation fails
/// 
/// # Example
/// 
/// ```rust
/// // In your application code:
/// if let Err(err) = example_usage().await {
///     eprintln!("Login example failed: {}", err);
/// }
/// ```
pub async fn example_usage() -> Result<()> {
    // Create and connect a client
    let mut client = Client::new();
    client.connect("127.0.0.1:30099").await?;
    
    // Set service and prepare payload
    let payload = LoginPayload {
        username: "admin".to_string(),
        password: "password123".to_string(),
    };
    
    // Execute action and get typed response - note the client is moved through method chains
    let mut client = client.with_service("auth")
        .param("client_version", "1.0");
        
    // Then we can use client.action() since we now have a reference
    let response: LoginResponse = client.action("login", &payload).await?;
    
    // Close connection
    client.close().await?;
    
    Ok(())
}