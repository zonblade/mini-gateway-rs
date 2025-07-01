use std::io::{Read, Write};
use std::net::TcpStream;

/// Very simple HTTP client that only checks response status
/// - Sends path + body via HTTP
/// - Returns Ok(()) for 2xx status codes  
/// - Returns Err(String) for non-2xx status codes
/// - Ignores response body completely
pub struct HttpC {
    host: String,
    port: u16,
}

impl HttpC {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
        }
    }

    /// Send POST request with body - returns success/failure based on status
    pub fn post(&self, path: &str, body: &[u8]) -> Result<(), String> {
        self.send_request("GWRX", path, body)
    }

    /// Generic request sender - only checks status, ignores response body
    fn send_request(&self, method: &str, path: &str, body: &[u8]) -> Result<(), String> {
        // Connect to server
        let mut stream = TcpStream::connect(format!("{}:{}", self.host, self.port))
            .map_err(|e| format!("Connection failed: {}", e))?;

        // Build HTTP request
        let request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\n\r\n",
            method,
            path,
            self.host,
            body.len()
        );

        // Send headers
        stream.write_all(request.as_bytes())
            .map_err(|e| format!("Failed to send request: {}", e))?;
        
        // Send body if present
        if !body.is_empty() {
            stream.write_all(body)
                .map_err(|e| format!("Failed to send body: {}", e))?;
        }
        
        stream.flush()
            .map_err(|e| format!("Failed to flush: {}", e))?;

        // Read only the status line
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer)
            .map_err(|e| format!("Failed to read response: {}", e))?;
        
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);
        
        // Parse status line (first line)
        let status_line = response.lines().next()
            .ok_or("No status line found")?;
        
        // Extract status code
        let parts: Vec<&str> = status_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid status line format".to_string());
        }
        
        let status_code: u16 = parts[1].parse()
            .map_err(|_| "Invalid status code".to_string())?;
        
        // Check if status is success (2xx)
        if status_code >= 200 && status_code < 300 {
            Ok(())
        } else {
            Err(format!("HTTP error: {}", status_code))
        }
    }
}

// Helper functions for common data types
impl HttpC {
    /// Send JSON data - returns success/failure only
    pub fn post_json(&self, path: &str, json: &str) -> Result<(), String> {
        self.post(path, json.as_bytes())
    }

    /// Send text data - returns success/failure only
    pub fn post_text(&self, path: &str, text: &str) -> Result<(), String> {
        self.post(path, text.as_bytes())
    }

    /// Send binary data - returns success/failure only
    pub fn post_bytes(&self, path: &str, data: &[u8]) -> Result<(), String> {
        self.post(path, data)
    }
}