//! # Server Management Module
//!
//! This module provides the initialization and management functionality for the router's
//! network servers. It handles creating and running multiple server instances in separate
//! threads, including gateway services, proxy servers (both TLS and non-TLS), and default
//! pages for error handling and security monitoring.
//!
//! ## Server Architecture
//!
//! The server system launches multiple components in parallel:
//! - Gateway servers for HTTP/HTTPS traffic routing based on path patterns
//! - Proxy servers for TLS termination and TCP traffic forwarding
//! - Default page servers for handling errors and security monitoring
//!
//! Each component runs in its own thread to provide isolation and parallel processing.

use super::default_page;
use crate::{
    app::gateway::GatewayApp,
    config::{self, GatewayNode, ProxyNode},
    service,
};
use pingora::{
    prelude::Opt,
    server::{RunArgs, Server},
    services::Service,
};
use std::thread;
use std::time::Duration;
use tokio::time::sleep;

/// Initialize and run all server components.
///
/// This function launches multiple server instances in separate threads:
/// 1. Gateway services for API routing (on ports 30001 and 30003)
/// 2. Non-TLS proxy service for HTTP traffic (on port 2001)
/// 3. TLS proxy service for HTTPS traffic (on port 2000)
/// 4. Default page servers for error handling and security:
///    - 404 error handler
///    - 500 error handler
///    - TLS honeypot for security monitoring
///
/// All servers run concurrently and this function blocks until all server
/// threads complete (typically on application shutdown).
///
/// # Thread Management
///
/// This function creates multiple threads and joins them at the end to ensure
/// proper cleanup. If any thread fails, the error is logged but the function
/// continues waiting for other threads.
///
/// # Implementation Details
///
/// Each server is created using the Pingora framework with default options.
/// The servers are bootstrapped individually and configured with appropriate
/// services before being launched with default run arguments.
pub fn init() {
    // Vector to store thread handles for later joining
    let mut server_threads = Vec::new();

    // Gateway Service Thread - Handles HTTP routing based on path patterns
    {
        let handle = thread::spawn(|| {
            // Create server with default options
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            my_server.bootstrap();

            // Configure listening addresses for gateway services
            let mut addr = vec![];
            let data = match config::RoutingData::GatewayRouting.xget::<Vec<GatewayNode>>() {
                Some(data)=>{
                    data
                },
                None=>{
                    vec![]
                }
            };

            log::debug!("Gateway data: {:#?}", data);

            for node in data {
                // Check if the address is already in the list
                if !addr.contains(&node.addr_listen) {
                    addr.push(node.addr_listen);
                }
            }

            let mut my_gateway: Vec<Box<(dyn pingora::services::Service + 'static)>> = Vec::new();

            // Create a gateway service for each address with staggered initialization
            for (idx, addr) in addr.iter().enumerate() {
                // Stagger the initialization to prevent resource spikes
                if idx > 0 {
                    // Small delay between service initializations to spread resource allocation
                    // Use a non-async sleep for this thread initialization context
                    std::thread::sleep(Duration::from_millis(200));
                }
                
                // Initialize the gateway service
                let service = init_gateway_service(addr, &my_server);
                my_gateway.push(service);
                
                log::info!("Gateway service {} initialized", idx + 1);
            }
            
            log::info!("Gateway services created: {}", my_gateway.len());
            
            // Add all gateway services to the server and run with optimal settings
            my_server.add_services(my_gateway);
            
            // Run the server with optimized settings
            let mut args = RunArgs::default();
            // Set graceful shutdown timeout to a reasonable value
            args.graceful_shutdown_timeout = Duration::from_secs(30);
            my_server.run(args);
        });
        server_threads.push(handle);
    }

    // Proxy Service Thread - Handles TCP proxying based on SNI/Host
    {
        // Add a small delay before initializing the proxy service
        // to avoid concurrent resource allocation with the gateway service
        std::thread::sleep(Duration::from_millis(500));
        
        let handle = thread::spawn(|| {
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            my_server.bootstrap();
            let mut node = config::RoutingData::ProxyRouting
                .xget::<Vec<ProxyNode>>()
                .unwrap_or(vec![]);
            // filter only for tls = false
            node.retain(|x| x.tls == false);
            
            // Pre-allocate the vector with the right capacity
            let mut proxies: Vec<Box<dyn Service>> = Vec::with_capacity(node.len());

            // Initialize services with staggered startup
            for (idx, proxy) in node.iter().enumerate() {
                // Stagger service initialization
                if idx > 0 {
                    std::thread::sleep(Duration::from_millis(200));
                }
                
                let proxy_app = ProxyApp::new(&proxy.addr_listen);
                let mut proxy_service = 
                    pingora::proxy::tcp_proxy_service(&my_server.configuration, proxy_app);
                
                // Use optimized buffer sizes when adding TCP listeners
                proxy_service.add_tcp_listener(&proxy.addr_listen, 
                    Some(8 * 1024 * 1024), // Send buffer
                    Some(8 * 1024 * 1024)  // Receive buffer
                );
                
                proxies.push(Box::new(proxy_service));
                log::info!("Proxy service initialized for: {}", proxy.addr_listen);
            }
            
            // Add all proxy services to the server
            my_server.add_services(proxies);
            
            // Run with optimized settings
            let mut args = RunArgs::default();
            args.graceful_shutdown_timeout = Duration::from_secs(30);
            my_server.run(args);
        });
        server_threads.push(handle);
    }

    // TLS Proxy Service Thread - Handles TLS termination and proxying
    {
        // Add another small delay before initializing the TLS service
        std::thread::sleep(Duration::from_millis(500));
        
        let handle = thread::spawn(|| {
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            my_server.bootstrap();
            let mut node = config::RoutingData::ProxyRouting
                .xget::<Vec<ProxyNode>>()
                .unwrap_or(vec![]);
            // filter only for tls = true
            node.retain(|x| x.tls == true);

            // Pre-allocate the vector with the right capacity
            let mut proxies: Vec<Box<dyn Service>> = Vec::with_capacity(node.len());

            // Initialize services with staggered startup
            for (idx, proxy) in node.iter().enumerate() {
                // Stagger service initialization
                if idx > 0 {
                    std::thread::sleep(Duration::from_millis(200));
                }
                
                let tls_pem = match proxy.tls_pem {
                    Some(ref pem) => pem,
                    None => {
                        log::error!("TLS PEM file not found for proxy: {}", proxy.addr_listen);
                        continue;
                    }
                };
                let tls_key = match proxy.tls_key {
                    Some(ref key) => key,
                    None => {
                        log::error!("TLS Key file not found for proxy: {}", proxy.addr_listen);
                        continue;
                    }
                };
                let proxy_set = service::proxy::proxy_service_tls(
                    &proxy.addr_listen,
                    &tls_pem,
                    &tls_key,
                );
                proxies.push(Box::new(proxy_set));
                log::info!("TLS Proxy service initialized for: {}", proxy.addr_listen);
            }
            
            // Add all proxy services to the server
            my_server.add_services(proxies);

            // Run with optimized settings
            let mut args = RunArgs::default();
            args.graceful_shutdown_timeout = Duration::from_secs(30);
            my_server.run(args);
        });
        server_threads.push(handle);
    }

    // Default Page servers - Handle error conditions and security monitoring
    {
        // Add a final delay before initializing the auxiliary services
        std::thread::sleep(Duration::from_millis(500));
        
        // 404 Not Found error page server
        let handle404: thread::JoinHandle<()> = thread::spawn(|| {
            // Create a TCP listener for the default 404 page
            default_page::p404::init();
        });

        // 500 Internal Server Error page server
        let handle500: thread::JoinHandle<()> = thread::spawn(|| {
            // Create a TCP listener for the default 500 page
            default_page::p500::init();
        });

        // TLS honeypot server for security monitoring
        let handle_tls: thread::JoinHandle<()> = thread::spawn(|| {
            // Create a TCP listener for the default TLS page
            default_page::tls_honeypot::init();
        });

        server_threads.push(handle404);
        server_threads.push(handle500);
        server_threads.push(handle_tls);
    }

    // Wait for all server threads to complete (this will block forever unless interrupted)
    for handle in server_threads {
        if let Err(e) = handle.join() {
            log::error!("Server thread panicked: {:?}", e);
        }
    }
}

/// # Initialize Gateway Service
/// 
/// Initializes a gateway service with optimized settings and staggered startup
/// to prevent concurrent resource allocation spikes.
/// 
/// ## Parameters
/// 
/// * `addr` - The address this gateway service will listen on
/// * `my_server` - The Pingora server instance to add the service to
/// 
/// ## Returns
/// 
/// Box containing the initialized gateway service
fn init_gateway_service(addr: &str, my_server: &pingora::server::Server) -> Box<dyn pingora::services::Service> {
    log::info!("Creating gateway service for address: {}", addr);
    
    // Create a new gateway app with the address
    let gateway_app = GatewayApp::new(addr);
    
    // Create the gateway service with optimized settings
    let mut gateway_service = pingora::proxy::http_proxy_service(
        &my_server.configuration,
        gateway_app,
    );
    
    // Add TCP listener with specific buffer sizes
    gateway_service.add_tcp_listener(addr, Some(8 * 1024 * 1024), Some(8 * 1024 * 1024));
    
    // Return the service as a boxed trait object
    Box::new(gateway_service)
}
