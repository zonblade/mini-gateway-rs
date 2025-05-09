// UDP Sender implementation for mini-gateway
// This module provides functionality to send plain text messages via UDP
// Thread-safe implementation that allows concurrent sending from multiple threads
// Includes a global instance for easy access from anywhere in the application

use std::io::{self, Result};
use std::net::{SocketAddr, UdpSocket};
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
        log::debug!("Creating new UdpSender with bind address: {}", bind_addr);
        let socket = UdpSocket::bind(bind_addr)?;
        
        // Make socket non-blocking for concurrent access
        socket.set_nonblocking(true)?;
        
        // Get and log the actual local address after binding
        if let Ok(local_addr) = socket.local_addr() {
            log::info!("UdpSender bound to local address: {}", local_addr);
        }
        
        Ok(Self {
            socket: Arc::new(socket),
        })
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
        let addr: SocketAddr = match target_addr.parse() {
            Ok(addr) => addr,
            Err(e) => {
                log::error!("Invalid target address '{}': {}", target_addr, e);
                return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid target address '{}': {}", target_addr, e)));
            }
        };
        
        log::debug!("Sending UDP message to {}", addr);
        match self.socket.send_to(message.as_bytes(), addr) {
            Ok(bytes) => {
                log::trace!("Successfully sent {} bytes to {}", bytes, addr);
                Ok(bytes)
            },
            Err(e) => {
                // Handle WouldBlock specifically for non-blocking sockets
                if e.kind() == io::ErrorKind::WouldBlock {
                    // Just return 0 bytes sent instead of an error for WouldBlock
                    log::debug!("UDP socket would block, message queued");
                    Ok(0)
                } else {
                    log::error!("Failed to send UDP message to {}: {}", addr, e);
                    Err(e)
                }
            }
        }
    }

    /// Creates a clone of this sender that can be moved to another thread.
    ///
    /// This is a lightweight operation as it only clones the Arc reference.
    ///
    /// # Returns
    /// * `UdpSender` - A new UdpSender that shares the same underlying socket
    pub fn clone(&self) -> Self {
        Self {
            socket: Arc::clone(&self.socket),
        }
    }
}

/// Global instance of UdpSender for application-wide use
use std::sync::OnceLock;
static GLOBAL_UDP_SENDER: OnceLock<UdpSender> = OnceLock::new();
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
pub fn global_sender() -> Result<UdpSender> {
    INIT.call_once(|| {
        // This will only run once across all threads
        match UdpSender::new("0.0.0.0:0") { // Use any available port instead of a fixed one
            Ok(sender) => {
                log::info!("Initializing global UDP sender");
                // We only set this once, so the Result can be safely ignored
                let _ = GLOBAL_UDP_SENDER.set(sender);
            }
            Err(e) => {
                log::error!("Failed to initialize global UDP sender: {}", e);
                // We don't set GLOBAL_UDP_SENDER, so it remains uninitialized
            }
        }
    });

    // Return a clone of the global sender, or an error if initialization failed
    match GLOBAL_UDP_SENDER.get() {
        Some(sender) => Ok(sender.clone()),
        None => Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to initialize global UDP sender",
        )),
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
pub fn init_global_sender() -> Result<()> {
    // Use a simple flag to track if we've already initialized
    let mut initialized = false;
    // Bind to 0.0.0.0 to allow sending from any interface
    let bind_addr = "127.0.0.1:0";

    INIT.call_once(|| {
        match UdpSender::new(bind_addr) {
            Ok(sender) => {
                log::info!("Initializing global UDP sender with address: {}", bind_addr);
                // OnceLock::set returns the error if there's already a value
                if GLOBAL_UDP_SENDER.set(sender).is_ok() {
                    initialized = true;
                } else {
                    // This should never happen as we're in the Once::call_once closure
                    log::error!("Failed to set global UDP sender - this should not happen!");
                }
            }
            Err(e) => {
                log::error!("Failed to initialize global UDP sender: {}", e);
                // Leave GLOBAL_UDP_SENDER uninitialized
            }
        }
    });

    if initialized {
        Ok(())
    } else {
        // If we've already called INIT.call_once(), then the global sender is either
        // already initialized or failed to initialize
        match GLOBAL_UDP_SENDER.get() {
            Some(_) => Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Global UDP sender is already initialized with a different address",
            )),
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to initialize global UDP sender",
            )),
        }
    }
}

pub fn log_to_proxy(message: &str) {
    if let Ok(sender) = global_sender() {
        // Include port explicitly in target_addr and add more debugging
        let target_addr = "127.0.0.1:24401";
        let result = sender.send_text(message, target_addr);
        if let Err(e) = result {
            log::error!("Failed to send message to proxy at {}: {}", target_addr, e);
        } else if let Ok(bytes) = result {
            log::debug!("Successfully sent {} bytes to proxy at {}", bytes, target_addr);
        }
    }
}

pub fn log_to_gateway(message: &str){
    if let Ok(sender) = global_sender() {
        // Include port explicitly in target_addr and add more debugging
        let target_addr = "127.0.0.1:24402";
        let result = sender.send_text(message, target_addr);
        if let Err(e) = result {
            log::error!("Failed to send message to gateway at {}: {}", target_addr, e);
        } else if let Ok(bytes) = result {
            log::debug!("Successfully sent {} bytes to gateway at {}", bytes, target_addr);
        }
    }
}

pub fn log_to_normal(message: &str){
    if let Ok(sender) = global_sender() {
        // Include port explicitly in target_addr and add more debugging
        let target_addr = "127.0.0.1:24403";
        let result = sender.send_text(message, target_addr);
        if let Err(e) = result {
            log::error!("Failed to send message to normal at {}: {}", target_addr, e);
        } else if let Ok(bytes) = result {
            log::debug!("Successfully sent {} bytes to normal at {}", bytes, target_addr);
        }
    }
}

pub fn switch_log(marker:&str, message: &str) {
    match marker {
        _ => log_to_normal(message),
    }
}
