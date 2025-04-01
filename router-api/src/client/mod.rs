//! # Protocol Client Module
//! 
//! This module provides a client implementation for communicating with a custom TCP protocol server.
//! It is designed to handle all aspects of the client-server communication, including:
//! 
//! - Connection establishment and management
//! - Request serialization and transmission
//! - Response handling and deserialization
//! - Error handling and reporting
//! 
//! ## Architecture
//! 
//! The client module is organized into several components:
//! 
//! - `client.rs`: Contains the main `Client` struct and its implementation
//! - `payload.rs`: Defines the `Payload` trait for serializable request/response data
//! - `error.rs`: Defines error types and result alias
//! - `examples.rs`: Contains example usage of the client
//! 
//! ## Usage
//! 
//! To use this client, typically you would:
//! 
//! 1. Create a new client instance
//! 2. Connect to a server
//! 3. Set a service name and optional parameters
//! 4. Execute actions with payloads
//! 5. Process responses
//! 6. Close the connection when done
//! 
//! ### Basic Example
//! 
//! ```rust
//! use router_api::client::{Client, LoginPayload, LoginResponse, Result};
//! 
//! async fn login_example() -> Result<()> {
//!     // Create a new client
//!     let mut client = Client::new();
//!     
//!     // Connect to the server
//!     client.connect("127.0.0.1:30099").await?;
//!     
//!     // Prepare the login payload
//!     let payload = LoginPayload {
//!         username: "user123".to_string(),
//!         password: "password123".to_string(),
//!     };
//!     
//!     // Set the service, add parameters, and execute the action
//!     let response: LoginResponse = client
//!         .with_service("auth")
//!         .param("client_version", "1.0")
//!         .action("login", &payload)
//!         .await?;
//!     
//!     // Process the response
//!     if response.success {
//!         println!("Login successful: {}", response.message);
//!     } else {
//!         println!("Login failed: {}", response.message);
//!     }
//!     
//!     // Close the connection
//!     client.close().await?;
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## Protocol Details
//! 
//! The client communicates using a TCP-based protocol with the following format:
//! 
//! 1. Handshake: `gate://service/action?param1=value1&param2=value2`
//! 2. The server responds with a confirmation message
//! 3. The client sends a serialized payload (typically JSON)
//! 4. The server processes the request and sends a response
//! 
//! ## Error Handling
//! 
//! All client operations return a `Result` type that includes various error
//! conditions that might occur during communication or processing.

// Submodules
mod error;
mod payload;
mod client;
mod examples;

// Re-export important types for easier usage
pub use error::{ClientError, Result};
pub use payload::Payload;
pub use client::Client;

// Re-export examples module for documenting usage
pub use examples::{
    LoginPayload,
    LoginResponse,
    example_usage,
};