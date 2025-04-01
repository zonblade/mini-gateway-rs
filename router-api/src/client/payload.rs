//! # Payload Trait for Request and Response Serialization
//! 
//! This module defines the `Payload` trait which provides a standardized way to convert
//! data structures to and from a wire format (bytes) for transmission between client and server.
//! 
//! ## Purpose
//! 
//! The Payload trait is a key part of the client's architecture as it:
//! 
//! - Provides a uniform interface for serializing different types of data
//! - Abstracts away the serialization details from the rest of the client code
//! - Allows for potential future serialization format changes with minimal impact
//! 
//! ## Default Implementation
//! 
//! By default, the trait uses JSON serialization via serde_json, but the trait
//! could be implemented differently for specific types if needed (e.g., for binary protocols
//! or other serialization formats).
//! 
//! ## Usage
//! 
//! This trait is automatically implemented for any type that implements both `Serialize` and `Debug`.
//! This means you can use your own custom types as payloads without any additional work:
//! 
//! ```rust
//! use serde::{Serialize, Deserialize};
//! use router_api::client::Payload;
//! 
//! #[derive(Debug, Serialize, Deserialize)]
//! struct MyCustomPayload {
//!     name: String,
//!     value: i32,
//!     data: Vec<String>,
//! }
//! 
//! // MyCustomPayload automatically implements Payload
//! // and can be used with client.action()
//! ```
//! 
//! ## Error Handling
//! 
//! Serialization errors are captured and returned as `ClientError::SerializationError`
//! with descriptive messages to aid in debugging.

use std::fmt::Debug;
use serde::Serialize;
use async_trait::async_trait;

use crate::client::error::{Result, ClientError};

/// Trait for request payloads that can be serialized for transmission.
/// 
/// This trait is implemented automatically for all types that implement both
/// `Serialize` and `Debug`. It provides a method to convert the payload to bytes
/// which is used internally by the client when sending requests.
/// 
/// # Type Parameters
/// 
/// The trait is designed to work with any type that can be serialized and
/// debugged, which enables a wide variety of payload structures.
/// 
/// # Examples
/// 
/// ```rust
/// use serde::{Serialize, Deserialize};
/// 
/// #[derive(Debug, Serialize, Deserialize)]
/// struct UserInfo {
///     username: String,
///     email: String,
///     role: String,
/// }
/// 
/// // UserInfo automatically implements Payload and can be used in client requests
/// ```
#[async_trait]
pub trait Payload: Serialize + Debug {
    /// Convert the payload to bytes for transmission.
    /// 
    /// This method serializes the payload object to a byte vector
    /// using JSON serialization by default. The resulting bytes
    /// are ready to be sent over the network.
    /// 
    /// # Returns
    /// 
    /// - `Ok(Vec<u8>)` - The serialized payload as bytes
    /// - `Err(ClientError)` - If serialization fails
    /// 
    /// # Errors
    /// 
    /// Returns a `ClientError::SerializationError` if the payload
    /// cannot be serialized, which might happen due to:
    /// 
    /// - Invalid data structures that cannot be represented in JSON
    /// - Circular references
    /// - Other serialization-specific issues
    async fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| ClientError::SerializationError(e.to_string()))
    }
}

/// Blanket implementation of Payload for all types that implement Serialize and Debug.
/// 
/// This implementation makes the Payload trait automatically available for any
/// type that can be serialized and has debugging capabilities, which simplifies
/// the creation of custom payload types.
/// 
/// # Implementation Details
/// 
/// The implementation uses the default `to_bytes` method from the trait,
/// which means it will serialize the payload using JSON.
impl<T: Serialize + Debug> Payload for T {}