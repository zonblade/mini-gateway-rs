//! # Default HTTP Error Page Server Module
//!
//! This module provides functionality for creating simple HTTP servers that serve
//! static error pages. It's particularly useful for displaying error messages when the
//! main application server is unavailable or when specific error conditions are encountered.
//!
//! The server implemented here is intentionally minimal and designed for reliability
//! in error conditions.

use std::io::Write;
use std::net::TcpListener;

/// Runs a simple HTTP server that serves a generic error page with the specified status code.
///
/// This function creates a TCP listener on the provided address and continuously accepts
/// incoming connections. For each connection, it serves a basic HTML page with the provided
/// error information. Each connection is handled in a separate thread to allow for concurrent
/// processing.
///
/// # Arguments
/// * `bind_addr` - Socket address to bind the server to (e.g., "127.0.0.1:8080" or "0.0.0.0:3000")
/// * `status_code` - HTTP status code to return in the response (e.g., 404, 500, 503)
/// * `status_text` - HTTP status text corresponding to the code (e.g., "Not Found", "Internal Server Error")
/// * `server_type` - Description of the server for logging purposes (e.g., "Error", "Maintenance")
///
/// # Behavior
///
/// This function starts an HTTP server that:
/// 1. Binds to the specified address
/// 2. Logs the successful binding using the `log` crate
/// 3. Serves a simple HTML page for all requests regardless of path
/// 4. Responds with the specified HTTP status code and text
/// 5. Handles each connection in a separate thread
///
/// # Examples
///
/// ```no_run
/// use router_core::system::default_page::p_base;
///
/// // Start a "Not Found" error page server on port 8404
/// p_base::run_error_page_server(
///     "127.0.0.1:8404",
///     404,
///     "Not Found",
///     "Not Found Error Page"
/// );
/// ```
///
/// # Panics
///
/// This function will panic if:
/// * The TCP listener cannot bind to the specified address
/// * The server lacks permissions to bind to the port
/// * The address format is invalid
///
/// # Logging
///
/// This function logs:
/// * An info-level message when the server starts successfully
/// * Error-level messages when connection handling fails
/// * Error-level messages when writing to the connection stream fails
///
/// # Thread Safety
///
/// Each incoming connection is handled in a separate thread, allowing for concurrent
/// processing of multiple requests. The response data is cloned for each thread to
/// ensure thread safety.
///
/// # Note
///
/// This server is intended for simple error pages only and doesn't include features like:
/// * Request parsing
/// * Dynamic content generation
/// * Connection keep-alive
/// * HTTP headers processing
/// * Security features
pub fn run_error_page_server(
    bind_addr: &str,
    status_code: u16,
    status_text: &str,
    server_type: &str,
) {
    let listener = match TcpListener::bind(bind_addr) {
        Ok(listener) => listener,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                log::warn!("Failed to bind {} server, address already in use", server_type);
                return;
            }
            panic!("Failed to bind {} server: {}", server_type, e);
        }
    };

    log::debug!("{} server listening on {}", server_type, bind_addr);

    // HTML content for the error page
    let html_content = "<!DOCTYPE html>\
                         <html>\
                         <head><title>Mini Router</title></head>\
                         <body>\
                         <center><h1>Gateway.rs</h1></center>\
                         <hr>\
                         <center>mini-gateway</center>\
                         </body>\
                         </html>";

    // Calculate content length dynamically
    let content_length = html_content.len();

    // Build the full HTTP response
    let error_response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: text/html\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        status_code, status_text, content_length, html_content
    );

    // Accept connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // Create a new owned response for each thread
                let response = error_response.clone();

                // Handle each connection in a new thread
                std::thread::spawn(move || {
                    // Write the response
                    if let Err(e) = stream.write_all(response.as_bytes()) {
                        log::error!("Failed to write to stream: {}", e);
                    }
                });
            }
            Err(e) => {
                log::error!("Connection failed: {}", e);
            }
        }
    }
}
