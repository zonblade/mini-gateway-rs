//! # 404 Not Found Error Page Handler
//! 
//! This module provides a specialized handler for serving HTTP 404 Not Found error responses.
//! It initializes a TCP listener on a configured port that responds to all incoming requests
//! with a standard 404 error page.
//! 
//! ## Purpose
//! 
//! The 404 handler serves as a fallback destination for requests that don't match any
//! configured routing rules in the gateway. Instead of dropping connections or sending
//! empty responses, the system routes these requests to this handler to provide a proper
//! HTTP error response with appropriate status code and message.
//! 
//! ## Configuration
//! 
//! The handler uses the address and port defined in `DEFAULT_PORT.p404`, which is typically
//! set to "127.0.0.1:60404" unless otherwise configured.

use crate::config::DEFAULT_PORT;
use super::p_base::run_error_page_server;

/// Initialize the 404 Not Found error page handler.
///
/// This function starts a dedicated TCP server that listens for connections on the
/// configured 404 handler port. All incoming requests to this port will receive a
/// standard HTTP 404 Not Found response.
///
/// ## Implementation Details
///
/// The function uses the shared `run_error_page_server` implementation from the
/// `p_base` module, which:
/// 
/// 1. Creates a TCP listener on the specified address/port
/// 2. Accepts incoming connections in a loop
/// 3. For each connection, sends a standard HTTP error response with:
///    - The specified status code (404)
///    - A brief reason phrase ("Not Found")
///    - A simple HTML error page with the specified message
///
/// ## Behavior
///
/// The server runs indefinitely in its own thread until the application terminates.
/// It handles each connection sequentially, serving the same error page to all clients.
///
/// ## Logging
///
/// Connection attempts to this handler are logged with source IP information, which
/// can be useful for monitoring failed access attempts or misconfigured clients.
pub fn init() {
    run_error_page_server(
        DEFAULT_PORT.p404,
        404,
        "Not Found",
        "Default 404 page"
    );
}