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
    config::{self, GatewayNode, ProxyNode},
    service,
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
    let mut server_threads = Vec::new();

    // TLS and non-TLS proxy server thread - Handles TLS and non-TLS traffic
    {
        let handle = thread::spawn(|| {
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            my_server.bootstrap();
            let proxy = config::RoutingData::ProxyRouting
                .xget::<Vec<ProxyNode>>()
                .unwrap_or(vec![]);
            let gateway = config::RoutingData::GatewayRouting
                .xget::<Vec<GatewayNode>>()
                .unwrap_or(vec![]);

            println!("Gateway: {:?}", gateway);
            println!("Proxy: {:?}", proxy);

            let mut proxies: Vec<Box<dyn Service>> = vec![];

            for gw in gateway {
                // find gw.addr_listen in proxy
                let proxy_node = proxy.iter().find(|p| p.addr_target == gw.addr_listen);
                if let Some(proxy_node) = proxy_node {
                    log::info!("Found proxy node: {:?}", proxy_node);

                    if proxy_node.tls {
                        let proxy_tls = service::proxy::proxy_service_tls_fast(
                            &proxy_node.addr_listen,
                            &gw.addr_target,
                            &proxy_node.sni.as_ref().unwrap_or(&"localhost".to_string()),
                            &proxy_node.tls_pem.as_ref().unwrap(),
                            &proxy_node.tls_key.as_ref().unwrap(),
                        );

                        log::info!("Adding proxy TLS service");
                        proxies.push(Box::new(proxy_tls));
                        continue;
                    }

                    log::info!("Adding proxy fast service: {:?}", proxy_node.addr_listen);
                    let proxy_set = service::proxy::proxy_service_fast(&proxy_node.addr_listen, &gw.addr_target);
                    proxies.push(Box::new(proxy_set));
                }
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
