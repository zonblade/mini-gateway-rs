//! # Error Types for the Protocol Client
//! 
//! This module defines the error types and result alias used throughout the client module.
//! It provides a comprehensive set of error variants that can occur during client operations,
//! making error handling more specific and informative.
//! 
//! ## Error Categories
//! 
//! The errors are categorized into several types:
//! 
//! - **IO Errors**: Low-level input/output errors during network operations
//! - **Connection Errors**: Problems establishing or maintaining connections
//! - **Serialization Errors**: Issues with converting data to/from wire format
//! - **Protocol Errors**: Violations of the expected protocol sequence or format
//! - **Configuration Errors**: Missing required configuration like service or action names
//! 
//! ## Usage
//! 
//! The module provides a specialized `Result` type alias that can be used for all client
//! operations, allowing for consistent error handling throughout the codebase.
//! 
//! ```rust
//! use router_api::client::{Result, ClientError};
//! 
//! async fn example() -> Result<()> {
//!     // Function implementation that can return any ClientError variant
//!     // ...
//!     Ok(())
//! }
//! ```
//! 
//! ## Error Handling Patterns
//! 
//! Typically, you would handle these errors using pattern matching:
//! 
//! ```rust
//! match client.action("example", &payload).await {
//!     Ok(response) => {
//!         // Process successful response
//!     },
//!     Err(ClientError::ConnectionError(msg)) => {
//!         // Handle connection problems
//!         eprintln!("Connection problem: {}", msg);
//!     },
//!     Err(ClientError::SerializationError(msg)) => {
//!         // Handle serialization issues
//!         eprintln!("Failed to serialize or deserialize data: {}", msg);
//!     },
//!     Err(e) => {
//!         // Handle any other errors
//!         eprintln!("Unexpected error: {}", e);
//!     }
//! }
//! ```

use std::io;
use thiserror::Error;

/// Errors that can occur when using the protocol client.
/// 
/// This enum provides a detailed set of error variants that can occur during
/// various client operations, from connection establishment to data serialization
/// and protocol handling.
#[derive(Error, Debug)]
pub enum ClientError {
    /// IO error occurred during communication.
    /// 
    /// This error variant wraps standard Rust IO errors that may occur during
    /// socket operations such as connection, reading, or writing.
    /// 
    /// # Examples
    /// 
    /// - Connection refused
    /// - Connection reset
    /// - Broken pipe
    /// - Timeout during read/write
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    /// Connection error with a descriptive message.
    /// 
    /// This error occurs when there are problems establishing or maintaining
    /// a connection that aren't directly tied to an IO error, such as unexpected
    /// disconnection or connection state issues.
    /// 
    /// # Examples
    /// 
    /// - "Connection closed unexpectedly"
    /// - "Failed to establish secure connection"
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    /// Serialization error when converting payloads.
    /// 
    /// This error occurs when the client fails to serialize a request payload
    /// or deserialize a response from the server.
    /// 
    /// # Examples
    /// 
    /// - JSON serialization errors
    /// - Missing required fields in response
    /// - Type conversion failures
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Protocol error with a descriptive message.
    /// 
    /// This error indicates that something in the protocol sequence or format
    /// was unexpected or invalid.
    /// 
    /// # Examples
    /// 
    /// - "Invalid handshake response"
    /// - "Unexpected message format"
    /// - "Protocol version mismatch"
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    /// Service name is not set before making a request.
    /// 
    /// This error occurs when attempting to make a request without first
    /// specifying which service to communicate with.
    #[error("Service not set")]
    ServiceNotSet,
    
    /// Action name is not set before making a request.
    /// 
    /// This error occurs when attempting to make a request without specifying
    /// which action to perform on the service.
    #[error("Action not set")]
    ActionNotSet,
    
    /// Retry error occurred after exhausting all retries.
    /// 
    /// This error indicates that an operation failed and the configured number
    /// of retry attempts were exhausted without success.
    /// 
    /// # Examples
    /// 
    /// - "Max retries (3) exceeded"
    /// - "Max retries (5) exceeded during connection"
    #[error("Retry error: {0}")]
    RetryError(String),
}

/// Result type for client operations.
/// 
/// This type alias simplifies return types throughout the client module by
/// providing a standardized Result type that uses ClientError as the error variant.
/// 
/// # Example
/// 
/// ```rust
/// use router_api::client::Result;
/// 
/// async fn example_function() -> Result<String> {
///     // Function implementation
///     // ...
///     Ok("Success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, ClientError>;