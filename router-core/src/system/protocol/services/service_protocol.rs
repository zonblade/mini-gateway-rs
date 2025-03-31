use async_trait::async_trait;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::io;

use crate::system::protocol::types::ConnectionParams;

/// ServiceProtocol trait that defines the interface for gateway services
/// Similar to Pingora's service interface
#[async_trait]
pub trait ServiceProtocol: Send + Sync {
    /// Creates a new instance of this service
    fn new() -> Self where Self: Sized;
    
    /// Process the upstream connection peer
    /// Handles the request and response after service filtering
    ///
    /// # Arguments
    ///
    /// * `socket` - TCP socket for communication with the client
    /// * `buffer` - The buffer containing the message data
    /// * `buffer_size` - Size of the buffer to use for reading data
    /// * `params` - Connection parameters from the handshake
    ///
    /// # Returns
    ///
    /// A Result containing either Ok(()) if processing was successful or an io::Error
    async fn upstream_peer(&self, socket: &mut TcpStream, buffer: &[u8], buffer_size: usize, params: &ConnectionParams) -> io::Result<()>;

    /// Log information about the request/response asynchronously
    ///
    /// # Arguments
    ///
    /// * `params` - Connection parameters from the handshake
    /// * `status` - Optional status information about the processing
    /// * `metrics` - Optional metrics or measurements to be logged
    async fn logging(&self, params: &ConnectionParams, status: Option<&str>, metrics: Option<HashMap<String, String>>);
}