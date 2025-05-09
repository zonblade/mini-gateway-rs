//! # Protocol Client Implementation
//! 
//! This module contains the core `Client` implementation for communicating with a custom TCP protocol server.
//! It provides a fluent API for establishing connections, configuring requests, and sending/receiving data.
//! 
//! ## Client Architecture
//! 
//! The client is designed with a builder pattern for configuration, combined with
//! methods for connection management and request execution. It maintains internal state
//! including:
//! 
//! - Connection status and TCP socket
//! - Service name being communicated with
//! - Parameters to include with requests
//! - Buffer sizes for reading and writing
//! 
//! ## Protocol Flow
//! 
//! The protocol implemented by this client follows this sequence:
//! 
//! 1. Establish TCP connection with the server
//! 2. Send a handshake message in the format: `gate://service/action?params`
//! 3. Receive handshake confirmation from server
//! 4. Send serialized payload data
//! 5. Receive and process server response
//! 6. Close connection or reuse for additional requests
//! 
//! ## Usage Patterns
//! 
//! The client supports several usage patterns:
//! 
//! ### Builder-style Configuration
//! 
//! ```rust
//! let client = Client::new()
//!     .with_service("auth")
//!     .param("version", "1.0")
//!     .param("client_id", "app123");
//! ```
//! 
//! ### Connection Management
//! 
//! ```rust
//! let mut client = Client::new();
//! client.connect("127.0.0.1:8080").await?;
//! // Use client...
//! client.close().await?;
//! ```
//! 
//! ### Request Execution
//! 
//! ```rust
//! let response: LoginResponse = client
//!     .with_service("user")
//!     .action("login", &login_payload)
//!     .await?;
//! ```
//! 
//! ## Error Handling
//! 
//! All operations that might fail return a `Result` type, with errors clearly
//! categorized in the `ClientError` enum. This allows for precise error handling
//! and reporting.

use std::collections::HashMap;
use std::fmt::Debug;
use std::net::ToSocketAddrs;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

use crate::client::error::{Result, ClientError};
use crate::client::payload::Payload;

/// Default protocol prefix used by the server in handshake messages.
/// 
/// This constant defines the standard protocol identifier used at the beginning
/// of all handshake messages to identify the protocol to the server.
const DEFAULT_PROTOCOL_PREFIX: &str = "gate://";

/// Default maximum number of retry attempts for operations.
const DEFAULT_MAX_RETRIES: u32 = 3;

/// Default base delay in milliseconds between retry attempts.
const DEFAULT_RETRY_DELAY_MS: u64 = 100;

/// Protocol client for communicating with the custom TCP protocol server.
/// 
/// This struct encapsulates all functionality needed to connect to and
/// communicate with the protocol server, including maintaining connection state,
/// configuring requests, and executing actions.
/// 
/// # Structure
/// 
/// The client maintains several pieces of state:
/// 
/// - The TCP socket for communication
/// - The current service name being targeted
/// - Configuration parameters for requests
/// - Buffer size for network operations
/// - Retry configuration for resilient operations
/// 
/// # Thread Safety
/// 
/// This client is not designed to be shared across threads. For multi-threaded
/// applications, each thread should maintain its own client instance.
pub struct Client {
    /// Optional TCP stream for communication.
    /// 
    /// This is `None` until `connect()` is called successfully, after which
    /// it contains the active socket connection to the server.
    socket: Option<TcpStream>,
    
    /// Optional service name for the client.
    /// 
    /// This is set via the `service()` or `with_service()` methods and is used
    /// in handshake messages to identify which service the action should be routed to.
    service_name: Option<String>,
    
    /// Buffer size for reading and writing data.
    /// 
    /// This determines the maximum size of data that can be read from or written
    /// to the socket in a single operation. The default is 1024 bytes.
    buffer_size: usize,
    
    /// Parameters to include in requests.
    /// 
    /// These key-value pairs are included in the handshake message as query parameters,
    /// allowing for additional context or configuration to be passed to the server.
    params: HashMap<String, String>,
    
    /// Maximum number of retry attempts for operations that fail due to 
    /// transient errors like connection issues.
    max_retries: u32,
    
    /// Base delay in milliseconds between retry attempts.
    /// This value doubles after each retry attempt (exponential backoff).
    retry_delay_ms: u64,
    
    /// Flag to enable or disable the retry mechanism.
    /// When set to false, operations will not be retried regardless of max_retries.
    retry_enabled: bool,
}

impl Client {
    /// Create a new client with default buffer size (1024 bytes).
    /// 
    /// This is the most common way to instantiate a client, and is suitable
    /// for most use cases with moderate data sizes.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let client = Client::new();
    /// ```
    pub fn new() -> Self {
        Self {
            socket: None,
            service_name: None,
            buffer_size: 1024,  // Default buffer size
            params: HashMap::new(),
            max_retries: DEFAULT_MAX_RETRIES,
            retry_delay_ms: DEFAULT_RETRY_DELAY_MS,
            retry_enabled: true,
        }
    }
    
    /// Create a new client with custom buffer size.
    /// 
    /// This constructor is useful when working with larger payloads or when
    /// optimizing for specific network conditions.
    /// 
    /// # Arguments
    /// 
    /// * `buffer_size` - The size of the buffer to use for reading and writing data.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// // Create a client with a 4KB buffer for larger payloads
    /// let client = Client::with_buffer_size(4096);
    /// ```
    #[allow(dead_code)]
    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self {
            socket: None,
            service_name: None,
            buffer_size,
            params: HashMap::new(),
            max_retries: DEFAULT_MAX_RETRIES,
            retry_delay_ms: DEFAULT_RETRY_DELAY_MS,
            retry_enabled: true,
        }
    }
    
    /// Connect to the protocol server at the specified address.
    /// 
    /// This method establishes a TCP connection to the server at the given address.
    /// If successful, it stores the socket for future communications.
    /// 
    /// # Arguments
    /// 
    /// * `addr` - The address of the server to connect to. This can be any type
    ///   that implements the `ToSocketAddrs` trait, such as an IP address and port tuple.
    /// 
    /// # Returns
    /// 
    /// Returns a mutable reference to the client to allow for method chaining
    /// after connection.
    /// 
    /// # Errors
    /// 
    /// Returns a `ClientError::IoError` if the connection fails for any reason,
    /// such as server not running, network issues, or permissions problems.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let mut client = Client::new();
    /// client.connect("127.0.0.1:8080").await?;
    /// ```
    pub async fn connect<A: ToSocketAddrs + Debug + tokio::net::ToSocketAddrs>(&mut self, addr: A) -> Result<&mut Self> {
        match TcpStream::connect(addr).await {
            Ok(socket) => {
                self.socket = Some(socket);
                Ok(self)
            },
            Err(e) => Err(ClientError::IoError(e)),
        }
    }
    
    /// Set the service to communicate with.
    /// 
    /// This method configures which service on the server the client will interact with.
    /// It's required to set a service before making any requests.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the service to target. This is typically a string
    ///   identifier known to the server.
    /// 
    /// # Returns
    /// 
    /// Returns the client to allow for method chaining.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let client = Client::new().service("authentication");
    /// ```
    pub fn service<S: Into<String>>(mut self, name: S) -> Self {
        self.service_name = Some(name.into());
        self
    }
    
    /// Add a parameter to the request.
    /// 
    /// Parameters are included in the handshake message as query parameters 
    /// (e.g., `gate://service/action?param1=value1`). They can be used to pass
    /// additional context or configuration to the server.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key/name of the parameter.
    /// * `value` - The value of the parameter.
    /// 
    /// # Returns
    /// 
    /// Returns the client to allow for method chaining.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let client = Client::new()
    ///     .service("user")
    ///     .param("version", "1.0")
    ///     .param("features", "extended");
    /// ```
    pub fn param<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
    
    /// Add multiple parameters to the request.
    /// 
    /// This is a convenience method for adding multiple parameters at once,
    /// which is useful when working with a pre-existing collection of parameters.
    /// 
    /// # Arguments
    /// 
    /// * `params` - A hashmap of parameters to add, where keys and values are strings.
    /// 
    /// # Returns
    /// 
    /// Returns the client to allow for method chaining.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let mut params = HashMap::new();
    /// params.insert("version".to_string(), "1.0".to_string());
    /// params.insert("client_id".to_string(), "app123".to_string());
    /// 
    /// let client = Client::new().service("auth").params(params);
    /// ```
    #[allow(dead_code)]
    pub fn params(mut self, params: HashMap<String, String>) -> Self {
        self.params.extend(params);
        self
    }
    
    /// Build a handshake message for the current service and action.
    /// 
    /// This internal method constructs the protocol handshake message based on
    /// the currently configured service, the specified action, and any parameters.
    /// The format follows: `gate://service/action?param1=value1&param2=value2`
    /// 
    /// # Arguments
    /// 
    /// * `action` - The action to perform on the service.
    /// 
    /// # Returns
    /// 
    /// Returns the formatted handshake message as a string.
    /// 
    /// # Errors
    /// 
    /// Returns a `ClientError::ServiceNotSet` if the service name has not been configured.
    fn build_handshake(&self, action: &str) -> Result<String> {
        let service = self.service_name.as_ref()
            .ok_or(ClientError::ServiceNotSet)?;
        
        // Start with the basic handshake format
        let mut handshake = format!("{}{}/{}", DEFAULT_PROTOCOL_PREFIX, service, action);
        
        // Add parameters if any exist
        if !self.params.is_empty() {
            handshake.push('?');
            let param_strings: Vec<String> = self.params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            handshake.push_str(&param_strings.join("&"));
        }
        
        Ok(handshake)
    }
    
    /// Execute an action on the current service with the given payload.
    /// 
    /// This is the main method for sending requests to the server. It performs
    /// the full protocol sequence: handshake, payload transmission, and response handling.
    /// The response is deserialized into the specified return type.
    /// 
    /// # Type Parameters
    /// 
    /// * `P` - The payload type, which must implement the `Payload` trait.
    /// * `R` - The response type, which must be deserializable.
    /// 
    /// # Arguments
    /// 
    /// * `action_name` - The name of the action to perform on the service.
    /// * `payload` - The payload to send with the request.
    /// 
    /// # Returns
    /// 
    /// Returns the deserialized response from the server.
    /// 
    /// # Errors
    /// 
    /// Returns various `ClientError` types if the operation fails at any stage.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let login_payload = LoginPayload {
    ///     username: "user123".to_string(),
    ///     password: "password".to_string(),
    /// };
    /// 
    /// let response: LoginResponse = client
    ///     .with_service("auth")
    ///     .action("login", &login_payload)
    ///     .await?;
    /// ```
    pub async fn action<P: Payload + std::marker::Sync, R: DeserializeOwned>(
        &mut self, 
        action_name: impl Into<String>, 
        payload: &P
    ) -> Result<R> {
        // Build handshake first to avoid borrowing issues
        let action = action_name.into();
        let handshake = self.build_handshake(&action)?;
        
        let socket = match &mut self.socket {
            Some(s) => s,
            None => return Err(ClientError::ConnectionError("Not connected".into())),
        };
        
        // Send handshake
        socket.write_all(handshake.as_bytes()).await
            .map_err(ClientError::IoError)?;
        
        // Read handshake confirmation
        let mut buffer = vec![0u8; self.buffer_size];
        let n = socket.read(&mut buffer).await
            .map_err(ClientError::IoError)?;
        
        if n == 0 {
            return Err(ClientError::ConnectionError("Connection closed during handshake".into()));
        }
        
        let response = String::from_utf8_lossy(&buffer[..n]);
        if !response.contains("successful") {
            return Err(ClientError::ProtocolError(format!("Handshake failed: {}", response)));
        }
        
        // Send the payload
        let payload_bytes = payload.to_bytes().await?;
        socket.write_all(&payload_bytes).await
            .map_err(ClientError::IoError)?;
        
        // Read the response
        let mut buffer = vec![0u8; self.buffer_size];
        let n = socket.read(&mut buffer).await
            .map_err(ClientError::IoError)?;
        
        if n == 0 {
            return Err(ClientError::ConnectionError("Connection closed before receiving response".into()));
        }
        
        // Parse the response
        serde_json::from_slice(&buffer[..n])
            .map_err(|e| ClientError::SerializationError(format!("Failed to deserialize response: {}", e)))
    }
    
    /// Execute an action with retry capability.
    /// 
    /// This method wraps the `action` method with retry logic, automatically
    /// retrying failed operations up to the configured maximum retry count.
    /// It uses exponential backoff for retry delay.
    /// 
    /// # Type Parameters
    /// 
    /// * `P` - The payload type, which must implement the `Payload` trait.
    /// * `R` - The response type, which must be deserializable.
    /// 
    /// # Arguments
    /// 
    /// * `action_name` - The name of the action to perform on the service.
    /// * `payload` - The payload to send with the request.
    /// 
    /// # Returns
    /// 
    /// Returns the deserialized response from the server.
    /// 
    /// # Errors
    /// 
    /// Returns the last error encountered after all retry attempts have been exhausted.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let login_payload = LoginPayload {
    ///     username: "user123".to_string(),
    ///     password: "password".to_string(),
    /// };
    /// 
    /// let response: LoginResponse = client
    ///     .with_service("auth")
    ///     .with_max_retries(5)
    ///     .action_with_retry("login", &login_payload)
    ///     .await?;
    /// ```
    #[allow(dead_code)]
    pub async fn action_with_retry<P: Payload + std::marker::Sync, R: DeserializeOwned>(
        &mut self, 
        action_name: impl Into<String> + Clone, 
        payload: &P
    ) -> Result<R> {
        if !self.retry_enabled || self.max_retries == 0 {
            return self.action(action_name, payload).await;
        }
        
        let mut last_error = None;
        let mut retry_count = 0;
        let mut delay = self.retry_delay_ms;
        
        while retry_count <= self.max_retries {
            match self.action(action_name.clone(), payload).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // Only retry on certain error types that might be transient
                    match &e {
                        ClientError::IoError(_) | 
                        ClientError::ConnectionError(_) => {
                            last_error = Some(e);
                            
                            if retry_count < self.max_retries {
                                // Wait before retrying with exponential backoff
                                sleep(Duration::from_millis(delay)).await;
                                delay *= 2; // Exponential backoff
                                retry_count += 1;
                            } else {
                                break;
                            }
                        },
                        // Don't retry on other error types
                        _ => return Err(e),
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or(ClientError::RetryError(
            format!("Max retries ({}) exceeded", self.max_retries)
        )))
    }
    
    /// Execute an action on the current service with the given payload, returning the raw response.
    /// 
    /// This method is similar to `action()` but returns the raw bytes of the response
    /// instead of deserializing them. This is useful when working with non-JSON responses
    /// or when custom deserialization is needed.
    /// 
    /// # Type Parameters
    /// 
    /// * `P` - The payload type, which must implement the `Payload` trait.
    /// 
    /// # Arguments
    /// 
    /// * `action_name` - The name of the action to perform on the service.
    /// * `payload` - The payload to send with the request.
    /// 
    /// # Returns
    /// 
    /// Returns the raw bytes of the server response.
    /// 
    /// # Errors
    /// 
    /// Returns various `ClientError` types if the operation fails at any stage.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let raw_response = client
    ///     .with_service("data")
    ///     .action_raw("get_binary", &request)
    ///     .await?;
    /// 
    /// // Process the raw bytes as needed
    /// ```
    #[allow(dead_code)]
    pub async fn action_raw<P: Payload + std::marker::Sync>(
        &mut self, 
        action_name: impl Into<String>, 
        payload: &P
    ) -> Result<Vec<u8>> {
        // Build handshake first to avoid borrowing issues
        let action = action_name.into();
        let handshake = self.build_handshake(&action)?;
        
        let socket = match &mut self.socket {
            Some(s) => s,
            None => return Err(ClientError::ConnectionError("Not connected".into())),
        };
        
        // Send handshake
        socket.write_all(handshake.as_bytes()).await
            .map_err(ClientError::IoError)?;
        
        // Read handshake confirmation
        let mut buffer = vec![0u8; self.buffer_size];
        let n = socket.read(&mut buffer).await
            .map_err(ClientError::IoError)?;
        
        if n == 0 {
            return Err(ClientError::ConnectionError("Connection closed during handshake".into()));
        }
        
        let response = String::from_utf8_lossy(&buffer[..n]);
        if !response.contains("successful") {
            return Err(ClientError::ProtocolError(format!("Handshake failed: {}", response)));
        }
        
        // Send the payload
        let payload_bytes = payload.to_bytes().await?;
        socket.write_all(&payload_bytes).await
            .map_err(ClientError::IoError)?;
        
        // Read the response
        let mut buffer = vec![0u8; self.buffer_size];
        let n = socket.read(&mut buffer).await
            .map_err(ClientError::IoError)?;
        
        if n == 0 {
            return Err(ClientError::ConnectionError("Connection closed before receiving response".into()));
        }
        
        Ok(buffer[..n].to_vec())
    }
    
    /// Execute an action with raw response and retry capability.
    /// 
    /// This method wraps the `action_raw` method with retry logic, automatically
    /// retrying failed operations up to the configured maximum retry count.
    /// It uses exponential backoff for retry delay.
    /// 
    /// # Type Parameters
    /// 
    /// * `P` - The payload type, which must implement the `Payload` trait.
    /// 
    /// # Arguments
    /// 
    /// * `action_name` - The name of the action to perform on the service.
    /// * `payload` - The payload to send with the request.
    /// 
    /// # Returns
    /// 
    /// Returns the raw bytes of the server response.
    /// 
    /// # Errors
    /// 
    /// Returns the last error encountered after all retry attempts have been exhausted.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let raw_response = client
    ///     .with_service("data")
    ///     .with_max_retries(3)
    ///     .action_raw_with_retry("get_binary", &request)
    ///     .await?;
    /// ```
    #[allow(dead_code)]
    pub async fn action_raw_with_retry<P: Payload + std::marker::Sync>(
        &mut self, 
        action_name: impl Into<String> + Clone, 
        payload: &P
    ) -> Result<Vec<u8>> {
        if !self.retry_enabled || self.max_retries == 0 {
            return self.action_raw(action_name, payload).await;
        }
        
        let mut last_error = None;
        let mut retry_count = 0;
        let mut delay = self.retry_delay_ms;
        
        while retry_count <= self.max_retries {
            match self.action_raw(action_name.clone(), payload).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // Only retry on certain error types that might be transient
                    match &e {
                        ClientError::IoError(_) | 
                        ClientError::ConnectionError(_) => {
                            last_error = Some(e);
                            
                            if retry_count < self.max_retries {
                                // Wait before retrying with exponential backoff
                                sleep(Duration::from_millis(delay)).await;
                                delay *= 2; // Exponential backoff
                                retry_count += 1;
                            } else {
                                break;
                            }
                        },
                        // Don't retry on other error types
                        _ => return Err(e),
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or(ClientError::RetryError(
            format!("Max retries ({}) exceeded", self.max_retries)
        )))
    }
    
    /// Simple ping to check if the connection is alive.
    /// 
    /// This method sends a simple "ping" message to the server and returns
    /// the server's response. It's useful for checking connection status
    /// or keeping connections alive in long-running applications.
    /// 
    /// # Returns
    /// 
    /// Returns the server's response as a string.
    /// 
    /// # Errors
    /// 
    /// Returns a `ClientError::ConnectionError` if the connection is not alive.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// match client.ping().await {
    ///     Ok(response) => println!("Server is alive: {}", response),
    ///     Err(_) => println!("Connection lost"),
    /// }
    /// ```
    #[allow(dead_code)]
    pub async fn ping(&mut self) -> Result<String> {
        let socket = self.socket.as_mut()
            .ok_or_else(|| ClientError::ConnectionError("Not connected".into()))?;
        
        // Send a simple ping message
        socket.write_all(b"ping").await
            .map_err(ClientError::IoError)?;
        
        // Read the response
        let mut buffer = vec![0u8; self.buffer_size];
        let n = socket.read(&mut buffer).await
            .map_err(ClientError::IoError)?;
        
        if n == 0 {
            return Err(ClientError::ConnectionError("Connection closed during ping".into()));
        }
        
        Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
    }
    
    /// Close the connection.
    /// 
    /// This method gracefully closes the TCP connection to the server.
    /// After calling this method, the client will need to connect again
    /// before making any more requests.
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if the connection was closed successfully.
    /// 
    /// # Errors
    /// 
    /// Returns a `ClientError` if closing the connection fails.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// // Close the connection when done
    /// client.close().await?;
    /// ```
    pub async fn close(&mut self) -> Result<()> {
        if let Some(socket) = self.socket.take() {
            drop(socket);
        }
        Ok(())
    }

    /// Utility method for direct service/action call.
    /// 
    /// This is a convenience method that sets the service name and returns
    /// the client, making it easier to chain method calls.
    /// 
    /// # Arguments
    /// 
    /// * `service` - The name of the service to set.
    /// 
    /// # Returns
    /// 
    /// Returns the client to allow for method chaining.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let response = client
    ///     .with_service("auth")
    ///     .action("login", &payload)
    ///     .await?;
    /// ```
    pub fn with_service<S: Into<String>>(mut self, service: S) -> Self {
        self.service_name = Some(service.into());
        self
    }

    /// Connect to the protocol server with retry capability.
    ///
    /// This method wraps the `connect` method with retry logic, automatically
    /// retrying failed connection attempts up to the configured maximum retry count.
    /// It uses exponential backoff for retry delay.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address of the server to connect to.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the client to allow for method chaining.
    ///
    /// # Errors
    ///
    /// Returns the last error encountered after all retry attempts have been exhausted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut client = Client::new().with_max_retries(5);
    /// client.connect_with_retry("127.0.0.1:8080").await?;
    /// ```
    #[allow(dead_code)]
    pub async fn connect_with_retry<A: ToSocketAddrs + Debug + tokio::net::ToSocketAddrs + Clone>(
        &mut self, 
        addr: A
    ) -> Result<&mut Self> {
        if !self.retry_enabled || self.max_retries == 0 {
            return self.connect(addr).await;
        }
        
        let mut last_error = None;
        let mut retry_count = 0;
        let mut delay = self.retry_delay_ms;
        
        while retry_count <= self.max_retries {
            match self.connect(addr.clone()).await {
                Ok(_) => return Ok(self),
                Err(e) => {
                    last_error = Some(e);
                    
                    if retry_count < self.max_retries {
                        // Wait before retrying with exponential backoff
                        sleep(Duration::from_millis(delay)).await;
                        delay *= 2; // Exponential backoff
                        retry_count += 1;
                    } else {
                        break;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or(ClientError::RetryError(
            format!("Max retries ({}) exceeded during connection", self.max_retries)
        )))
    }
    
    /// Set the maximum number of retry attempts.
    ///
    /// This method configures how many times operations will be retried
    /// before giving up.
    ///
    /// # Arguments
    ///
    /// * `retries` - The maximum number of retry attempts.
    ///
    /// # Returns
    ///
    /// Returns the client to allow for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let client = Client::new().with_max_retries(5);
    /// ```
    #[allow(dead_code)]
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
    
    /// Set the base delay between retry attempts.
    ///
    /// This method configures the initial delay between retry attempts.
    /// The actual delay will increase exponentially with each retry.
    ///
    /// # Arguments
    ///
    /// * `delay_ms` - The base delay in milliseconds.
    ///
    /// # Returns
    ///
    /// Returns the client to allow for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let client = Client::new().with_retry_delay(200);
    /// ```
    #[allow(dead_code)]
    pub fn with_retry_delay(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = delay_ms;
        self
    }
    
    /// Enable or disable retry functionality.
    ///
    /// This method allows enabling or disabling the retry mechanism entirely,
    /// regardless of the maximum retry count.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable retry functionality.
    ///
    /// # Returns
    ///
    /// Returns the client to allow for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let client = Client::new().with_retry_enabled(false); // Disable retries
    /// ```
    #[allow(dead_code)]
    pub fn with_retry_enabled(mut self, enabled: bool) -> Self {
        self.retry_enabled = enabled;
        self
    }
}

impl Default for Client {
    /// Create a default client instance.
    /// 
    /// This implementation simply calls `Client::new()` to create a client
    /// with default settings.
    fn default() -> Self {
        Self::new()
    }
}