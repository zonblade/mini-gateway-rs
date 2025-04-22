// UDP Sender implementation for mini-router
// This module provides functionality to send plain text messages via UDP
// Thread-safe implementation that allows concurrent sending from multiple threads
// Includes a global instance for easy access from anywhere in the application

use std::net::{UdpSocket, SocketAddr};
use std::io::{self, Result};
use std::sync::{Arc, Once};

/// A thread-safe UDP sender that can send plain text messages to a target address
/// 
/// This implementation is safe to share across multiple threads without additional locking.
/// It internally uses `Arc` to allow concurrent access to the socket.
pub struct UdpSender {
    socket: Arc<UdpSocket>,
}

impl UdpSender {
    /// Creates a new UdpSender bound to the given local address
    /// 
    /// # Arguments
    /// * `bind_addr` - The local address to bind to (e.g. "0.0.0.0:0" for any interface and random port)
    /// 
    /// # Returns
    /// * `Result<UdpSender>` - A new UdpSender instance or an io::Error
    pub fn new(bind_addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        // Make socket non-blocking for concurrent access
        socket.set_nonblocking(true)?;
        Ok(Self { socket: Arc::new(socket) })
    }

    /// Creates a new UdpSender that binds to any available interface and lets the OS choose a port
    /// 
    /// # Returns
    /// * `Result<UdpSender>` - A new UdpSender instance or an io::Error
    pub fn new_any_port() -> Result<Self> {
        Self::new("0.0.0.0:0")
    }

    /// Sends a plain text message to the target address
    /// 
    /// This method is thread-safe and can be called concurrently from multiple threads.
    /// 
    /// # Arguments
    /// * `message` - The message to send as a string slice
    /// * `target_addr` - The target socket address (IP and port)
    /// 
    /// # Returns
    /// * `Result<usize>` - Number of bytes sent or an io::Error
    pub fn send_text(&self, message: &str, target_addr: &str) -> Result<usize> {
        let addr: SocketAddr = target_addr.parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        self.socket.send_to(message.as_bytes(), addr)
    }

    /// Sends a raw byte message to the target address
    /// 
    /// This method is thread-safe and can be called concurrently from multiple threads.
    /// 
    /// # Arguments
    /// * `data` - The raw bytes to send
    /// * `target_addr` - The target socket address (IP and port)
    /// 
    /// # Returns
    /// * `Result<usize>` - Number of bytes sent or an io::Error
    pub fn send_bytes(&self, data: &[u8], target_addr: &str) -> Result<usize> {
        let addr: SocketAddr = target_addr.parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        self.socket.send_to(data, addr)
    }

    /// Gets the local address this sender is bound to
    /// 
    /// # Returns
    /// * `Result<SocketAddr>` - The local socket address or an io::Error
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
    
    /// Creates a clone of this sender that can be moved to another thread.
    /// 
    /// This is a lightweight operation as it only clones the Arc reference.
    /// 
    /// # Returns
    /// * `UdpSender` - A new UdpSender that shares the same underlying socket
    pub fn clone(&self) -> Self {
        Self {
            socket: Arc::clone(&self.socket)
        }
    }
}

/// Global instance of UdpSender for application-wide use
static mut GLOBAL_UDP_SENDER: Option<UdpSender> = None;
static INIT: Once = Once::new();

/// Gets the global UDP sender instance, initializing it if necessary
///
/// # Example
/// ```
/// use router_core::system::udp_sender::global_sender;
///
/// fn example() -> std::io::Result<()> {
///     // Get the global UDP sender
///     let sender = global_sender()?;
///     
///     // Send a message using the global sender
///     sender.send_text("Hello from global sender", "127.0.0.1:8080")?;
///     
///     Ok(())
/// }
/// ```
///
/// # Safety
/// This function uses unsafe code to access a static mutable variable,
/// but it's safe because we use `std::sync::Once` to ensure initialization
/// happens exactly once across all threads.
///
/// # Returns
/// A reference to the global UDP sender instance
pub fn global_sender() -> Result<&'static UdpSender> {
    unsafe {
        INIT.call_once(|| {
            // This will only run once across all threads
            match UdpSender::new_any_port() {
                Ok(sender) => {
                    log::info!("Initializing global UDP sender");
                    GLOBAL_UDP_SENDER = Some(sender);
                },
                Err(e) => {
                    log::error!("Failed to initialize global UDP sender: {}", e);
                    // We don't set GLOBAL_UDP_SENDER, so it remains None
                }
            }
        });
        
        // Return a reference to the global sender, or an error if initialization failed
        match &GLOBAL_UDP_SENDER {
            Some(sender) => Ok(sender),
            None => Err(io::Error::new(
                io::ErrorKind::Other, 
                "Failed to initialize global UDP sender"
            ))
        }
    }
}

/// Initialize the global UDP sender with a specific binding address
///
/// This should be called early in your application startup if you need
/// to bind to a specific address. Otherwise, the default configuration
/// will be used the first time global_sender() is called.
///
/// # Arguments
/// * `bind_addr` - The address to bind the global UDP sender to
///
/// # Returns
/// `Ok(())` if initialization was successful, or an error if it failed
/// or if the global sender was already initialized
pub fn init_global_sender(bind_addr: &str) -> Result<()> {
    // Use a simple flag to track if we've already initialized
    let mut initialized = false;
    
    unsafe {
        INIT.call_once(|| {
            match UdpSender::new(bind_addr) {
                Ok(sender) => {
                    log::info!("Initializing global UDP sender with address: {}", bind_addr);
                    GLOBAL_UDP_SENDER = Some(sender);
                    initialized = true;
                },
                Err(e) => {
                    log::error!("Failed to initialize global UDP sender: {}", e);
                    // Leave GLOBAL_UDP_SENDER as None
                }
            }
        });
    }
    
    if initialized {
        Ok(())
    } else {
        // If we've already called INIT.call_once(), then the global sender is either
        // already initialized or failed to initialize
        unsafe {
            match &GLOBAL_UDP_SENDER {
                Some(_) => Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Global UDP sender is already initialized with a different address"
                )),
                None => Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to initialize global UDP sender"
                ))
            }
        }
    }
}

/// Example usage with direct instance creation:
/// ```
/// use router_core::system::udp_sender::UdpSender;
/// 
/// fn example() -> std::io::Result<()> {
///     // Create a new UDP sender bound to any available port
///     let sender = UdpSender::new_any_port()?;
///     
///     // Send a plain text message to a target (e.g., 127.0.0.1:8080)
///     let bytes_sent = sender.send_text("Hello, UDP receiver!", "127.0.0.1:8080")?;
///     println!("Sent {} bytes successfully", bytes_sent);
///     
///     Ok(())
/// }
/// ```
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_sender() {
        let sender = UdpSender::new_any_port();
        assert!(sender.is_ok());
    }
    
    // Note: Additional tests would require a proper test environment with UDP receivers
}
