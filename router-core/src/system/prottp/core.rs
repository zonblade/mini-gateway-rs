use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
// use serde_json::Value;
use std::io::Read;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    // pub headers: std::collections::HashMap<String, String>,
    pub body: Vec<u8>,
    // pub json: Option<Value>,
    pub stream: TcpStream,
}

pub struct HttpServer {
    address: String,
}

impl HttpServer {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
        }
    }

    pub fn start<F>(&self, handler: F) -> std::io::Result<()>
    where
        F: Fn(HttpRequest) + Send + Sync + 'static,
    {
        let listener = TcpListener::bind(&self.address)?;
        println!("[-PT-] HTTP Server listening on {}", self.address);

        let handler = std::sync::Arc::new(handler);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = handler.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream, handler) {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("Failed to accept connection: {}", e),
            }
        }

        Ok(())
    }
}

fn handle_connection<F>(stream: TcpStream, handler: std::sync::Arc<F>) -> std::io::Result<()>
where
    F: Fn(HttpRequest) + Send + Sync,
{
    let mut reader = BufReader::new(&stream);
    
    // Read request line
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    
    // Parse method and path
    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(()); // Invalid request, just close
    }
    
    let method = parts[0].to_string();
    let path = parts[1].to_string();
    
    // Read headers
    let mut headers = std::collections::HashMap::new();
    let mut content_length = 0;
    
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let line = line.trim();
        
        if line.is_empty() {
            break; // End of headers
        }
        
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_lowercase();
            let value = line[pos + 1..].trim().to_string();
            
            if key == "content-length" {
                content_length = value.parse().unwrap_or(0);
            }
            
            headers.insert(key, value);
        }
    }
    
    // Read body if present
    let mut body = Vec::new();
    if content_length > 0 {
        body = vec![0; content_length];
        reader.read_exact(&mut body)?;
    }
    
    // // Parse body as JSON
    // let json = if !body.is_empty() {
    //     match serde_json::from_slice::<Value>(&body) {
    //         Ok(value) => Some(value),
    //         Err(_) => None, // Invalid JSON, keep as None
    //     }
    // } else {
    //     None
    // };
    
    // Create request and pass to handler
    let request = HttpRequest {
        method,
        path,
        // headers,
        body,
        // json,
        stream,
    };
    
    handler(request);
    
    Ok(())
}

// Helper functions for sending standard responses
impl HttpRequest {
    pub fn send_200(&mut self, body: &str) -> std::io::Result<()> {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
            body.len(),
            body
        );
        self.stream.write_all(response.as_bytes())?;
        self.stream.flush()?;
        Ok(())
    }

    pub fn send_400(&mut self, body: &str) -> std::io::Result<()> {
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
            body.len(),
            body
        );
        self.stream.write_all(response.as_bytes())?;
        self.stream.flush()?;
        Ok(())
    }

    pub fn send_404(&mut self, body: &str) -> std::io::Result<()> {
        let response = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
            body.len(),
            body
        );
        self.stream.write_all(response.as_bytes())?;
        self.stream.flush()?;
        Ok(())
    }
}
