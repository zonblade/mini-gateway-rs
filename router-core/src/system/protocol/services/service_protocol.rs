use async_trait::async_trait;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::io;

use crate::system::protocol::types::ConnectionParams;

/// # ServiceProtocol Trait
///
/// This trait defines the contract that all protocol services must implement.
/// It provides a standardized interface for services to process client requests
/// and log the processing results.
///
/// The design is inspired by Pingora's service architecture and allows for:
/// - Dynamic service creation via a standardized `new()` method
/// - Request processing via the `upstream_peer()` method
/// - Asynchronous logging via the `logging()` method
///
/// ## Implementation Requirements
///
/// Services implementing this trait must be:
/// - `Send` and `Sync` to allow safe usage across threads
/// - Capable of processing requests asynchronously
/// - Self-contained with all necessary state for request handling
///
/// ## Lifecycle
///
/// 1. Services are created using the `new()` method
/// 2. Services are registered with the service handler
/// 3. For each matching request, `upstream_peer()` is called
/// 4. After processing, `logging()` is called asynchronously
///
/// ## Thread Safety
///
/// Services must be safe to use from multiple threads simultaneously.
/// The trait bounds `Send + Sync` ensure this requirement is enforced
/// at compile time for all implementing types.
#[async_trait]
pub trait ServiceProtocol: Send + Sync {
    /// Creates a new instance of this service
    ///
    /// This factory method allows services to be created dynamically
    /// and registered with the service handler. It enables a consistent 
    /// way to instantiate services without knowing their concrete types.
    ///
    /// ## Returns
    ///
    /// A new instance of the service.
    fn new() -> Self where Self: Sized;
    
    /// Process the upstream connection peer
    ///
    /// This is the main request processing method that handles client
    /// communication after a connection has been established and routed
    /// to this service. It is responsible for:
    ///
    /// 1. Reading and interpreting the request data
    /// 2. Performing the requested operation
    /// 3. Sending a response back to the client
    ///
    /// ## Parameters
    ///
    /// * `socket` - TCP socket for bi-directional communication with the client
    /// * `buffer` - The buffer containing the initial message data
    /// * `buffer_size` - Size of the buffer to use for additional reading if needed
    /// * `params` - Connection parameters extracted from the handshake
    ///
    /// ## Returns
    ///
    /// A Result containing either:
    /// - `Ok(())` if processing was successful
    /// - `Err(io::Error)` if an I/O error occurred during processing
    ///
    /// ## Concurrency
    ///
    /// This method may be called concurrently for different connections.
    /// Implementations must ensure they handle concurrent access safely.
    async fn upstream_peer(&self, socket: &mut TcpStream, buffer: &[u8], buffer_size: usize, params: &ConnectionParams) -> io::Result<()>;

    /// Log information about the request/response asynchronously
    ///
    /// This method allows services to log information about processed requests
    /// without blocking the response to the client. It is typically called after
    /// the `upstream_peer` method has completed processing.
    ///
    /// ## Logging Strategy
    ///
    /// - Implementations should be non-blocking
    /// - Logging should be asynchronous to avoid impacting response times
    /// - Services can log to multiple destinations (files, databases, etc.)
    ///
    /// ## Parameters
    ///
    /// * `params` - Connection parameters extracted from the handshake
    /// * `status` - Optional status code or message about the processing outcome
    /// * `metrics` - Optional collection of metrics to record (e.g., timing, sizes)
    ///
    /// ## Metrics
    ///
    /// The `metrics` parameter allows recording various data points about the request:
    /// - Response time
    /// - Bytes received/sent
    /// - Cache hits/misses
    /// - Any service-specific metrics
    async fn logging(&self, params: &ConnectionParams, status: Option<&str>, metrics: Option<HashMap<String, String>>);
}