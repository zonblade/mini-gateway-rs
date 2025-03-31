use pingora::{
    listeners::tls::TlsSettings, prelude::Opt, proxy::HttpProxy, server::{RunArgs, Server}, services::Service
};
use std::{io::Write, thread};

use crate::{app::{gateway::GatewayApp, proxy::ProxyApp}, service};

use super::terminator;

pub fn init() {
    let mut server_threads = Vec::new();

    // // sample of 10 second termination.
    // {
    //     thread::spawn(|| {
    //         std::thread::sleep(std::time::Duration::from_secs(10));
    //         terminator::service::init();
    //     });
    // }

    // Gateway Service Thread
    {
        let handle = thread::spawn(|| {
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            my_server.bootstrap();

            let addr = vec![
                "127.0.0.1:30001",
                "127.0.0.1:30003"
            ];
            let mut my_gateway:Vec<Box<(dyn pingora::services::Service + 'static)>> = Vec::new();
            for addr in addr.iter() {
                let mut my_gateway_service = pingora::proxy::http_proxy_service(
                    &my_server.configuration,
                    GatewayApp::new(addr),
                );
                my_gateway_service.add_tcp(addr);
                my_gateway.push(Box::new(my_gateway_service));
            };

            my_server.add_services(my_gateway);
            my_server.run(RunArgs::default());
            log::warn!("Non-TLS server shutting down.");
        });
        server_threads.push(handle);
    }

    // Proxy server thread
    {
        let handle = thread::spawn(|| {
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            let proxy = service::proxy::proxy_service("0.0.0.0:2000");
            my_server.bootstrap();
            my_server.add_service(proxy);
            // This call blocks until the process receives SIGINT (or another interrupt)
            my_server.run(RunArgs::default());
            log::warn!("TLS server shutting down.");
        });
        server_threads.push(handle);
    }


    // Default Page
    {
        let handle: thread::JoinHandle<()> = thread::spawn(|| {// Create a TCP listener for the default 404 page
            let listener = std::net::TcpListener::bind("127.0.0.1:12871").expect("Failed to bind default page server");
            log::info!("Default 404 page server listening on 127.0.0.1:12871"); // Fixed port in log
            
            // HTML content for the 404 page
            let html_content = "<!DOCTYPE html>\
                               <html>\
                               <head><title>Mini Router</title></head>\
                               <body>\
                               <center><h1>Gateway.rs</h1></center>\
                               <hr>\
                               <center>mini-router</center>\
                               </body>\
                               </html>";
            
            // Calculate content length dynamically
            let content_length = html_content.len();
            
            // Build the full HTTP response
            let not_found_response = format!(
                "HTTP/1.1 404 Not Found\r\n\
                 Content-Type: text/html\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\
                 \r\n\
                 {}", 
                content_length, 
                html_content
            );
            
            // Accept connections in a loop
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        // Create a new owned response for each thread
                        let response = not_found_response.clone();
                        
                        // Handle each connection in a new thread
                        std::thread::spawn(move || {
                            // Write the 404 response
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
        });
        server_threads.push(handle);
    }

    for handle in server_threads {
        log::debug!("Waiting for server thread to finish...");
        if let Err(e) = handle.join() {
            log::debug!("Server thread failed: {:?}", e);
        }
    }
}
