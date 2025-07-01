mod app;
mod core;

pub fn init() {
    std::thread::spawn(|| {
        let server = core::HttpServer::new("127.0.0.1:30099");

        println!("[-PT-] Starting HTTP server on 30099");
        
        if let Err(e) = server.start(|mut request| {
            let body_string = {
                let string = String::from_utf8_lossy(&request.body); // Returns Cow<str>
                let string = string.to_string(); // Convert to owned String
                string
            };

            println!("[-PT-] Received request: {} {}", request.method, request.path);

            match (request.method.as_str(), request.path.as_str()) {
                ("GWRX", "/gateway/node") => {
                    let res = match app::gateway_node::init(body_string) {
                        Ok(_) => request.send_200("Gateway node data updated successfully"),
                        Err(e) => {
                            log::error!("Failed to update gateway node data: {}", e);
                            request.send_400("Failed to update gateway node data")
                        }
                    };
                    let _ = res;
                }
                ("GWRX", "/gateway/path") => {
                    let res = match app::gateway_path::init(body_string) {
                        Ok(_) => request.send_200("Gateway path data updated successfully"),
                        Err(e) => {
                            log::error!("Failed to update gateway path data: {}", e);
                            request.send_400("Failed to update gateway path data")
                        }
                    };
                    let _ = res;
                }
                ("GWRX", "/proxy/node") => {
                    let res = match app::proxy_node::init(body_string) {
                        Ok(_) => request.send_200("Proxy node data updated successfully"),
                        Err(e) => {
                            log::error!("Failed to update proxy node data: {}", e);
                            request.send_400("Failed to update proxy node data")
                        }
                    };
                    let _ = res;
                }
                _ => {
                    let _ =  request.send_404("");
                }
            }
        }) {
            log::error!("HTTP server error: {}", e);
        }
    });
}
