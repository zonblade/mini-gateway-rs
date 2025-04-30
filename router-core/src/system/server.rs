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
    app::gateway_fast::GatewayApp, config::{self, GatewayNode, ProxyNode}, service
};
use pingora::{
    prelude::Opt,
    server::{RunArgs, Server},
    services::Service,
};
use std::thread;

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
    let mut server_threads: Vec<thread::JoinHandle<()>> = Vec::new();

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

            println!("Gateway data: {:#?}", data);

            for node in data {
                // Check if the address is already in the list
                if !addr.contains(&node.addr_listen) {
                    addr.push(node.addr_listen);
                }
            }

            let mut my_gateway: Vec<Box<(dyn pingora::services::Service + 'static)>> = Vec::new();

            // Create a gateway service for each address
            for addr in addr.iter() {
                log::info!("Creating gateway service for address: {}", addr);
                let mut my_gateway_service = pingora::proxy::http_proxy_service(
                    &my_server.configuration,
                    GatewayApp::new(addr),
                );
                my_gateway_service.add_tcp(addr);
                
                my_gateway.push(Box::new(my_gateway_service));
            }
            log::info!("Gateway services created: {}", my_gateway.len());
            // Add all gateway services to the server and run
            my_server.add_services(my_gateway);
            my_server.run(RunArgs::default());
        });
        server_threads.push(handle);
    }

    // TLS and non-TLS proxy server thread - Handles TLS and non-TLS traffic
    {
        let handle = thread::spawn(|| {
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            my_server.bootstrap();
            let proxy = config::RoutingData::ProxyRouting
                .xget::<Vec<ProxyNode>>()
                .unwrap_or(vec![]);

            let mut proxies: Vec<Box<dyn Service>> = vec![];

            for px in proxy {

                println!("Proxy node: {:#?}", px);
                let addr_target = {
                    if px.high_speed {
                        px.high_speed_addr.unwrap_or(px.addr_target)
                    }else{
                        px.addr_target
                    }
                };
                println!("Proxy target address: {}", addr_target);

                if px.tls && px.sni.is_some() && px.tls_pem.is_some() && px.tls_key.is_some() {
                    let proxy_tls = service::proxy::proxy_service_tls_fast(
                        &px.addr_listen,
                        &addr_target,
                        &px.sni.as_ref().unwrap_or(&"localhost".to_string()),
                        px.high_speed,
                        &px.tls_pem.as_ref().unwrap(),
                        &px.tls_key.as_ref().unwrap(),
                    );

                    log::info!("Adding proxy TLS service");
                    proxies.push(Box::new(proxy_tls));
                    continue;
                }

                log::info!("Adding proxy fast service: {:?}", px.addr_listen);
                let proxy_set = service::proxy::proxy_service_fast(&px.addr_listen, &addr_target, px.high_speed);
                proxies.push(Box::new(proxy_set));

            }

            // Add all proxy services to the server
            my_server.add_services(proxies);

            println!("Starting server");
            // This call blocks until the process receives SIGINT (or another interrupt)
            my_server.run(RunArgs::default());
        });
        server_threads.push(handle);
    }

    // Default Page servers - Handle error conditions and security monitoring
    {
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

    // Wait for all server threads to complete (typically on shutdown)
    for handle in server_threads {
        log::debug!("Waiting for server thread to finish...");
        if let Err(e) = handle.join() {
            log::debug!("Server thread failed: {:?}", e);
        }
    }
}
