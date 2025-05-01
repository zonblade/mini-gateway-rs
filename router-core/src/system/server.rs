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
    app::gateway_fast::GatewayApp,
    config::{self, GatewayNode, ProxyNode},
    service,
};
use pingora::{
    listeners::tls::TlsSettings,
    prelude::Opt,
    server::{RunArgs, Server},
    services::Service,
};
use std::ops::DerefMut;
use std::thread;

mod boringssl_openssl {
    use async_trait::async_trait;
    use pingora::tls::pkey::{PKey, Private};
    use pingora::tls::x509::X509;

    pub(super) struct DynamicCert {
        cert: X509,
        key: PKey<Private>,
    }

    impl DynamicCert {
        pub(super) fn new(cert: &str, key: &str) -> Box<Self> {
            let cert_bytes = std::fs::read(cert).unwrap();
            let cert = X509::from_pem(&cert_bytes).unwrap();

            let key_bytes = std::fs::read(key).unwrap();
            let key = PKey::private_key_from_pem(&key_bytes).unwrap();
            Box::new(DynamicCert { cert, key })
        }
    }

    #[async_trait]
    impl pingora::listeners::TlsAccept for DynamicCert {
        async fn certificate_callback(&self, ssl: &mut pingora::tls::ssl::SslRef) {
            use pingora::tls::ext;
            ext::ssl_use_certificate(ssl, &self.cert).unwrap();
            ext::ssl_use_private_key(ssl, &self.key).unwrap();
        }
    }
}

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
    // cases are, if the proxy have a multiple address, it converted to normal gateway, either it's HTTPS or HTTP
    {
        let handle = thread::spawn(|| {
            //
            // 3010 -> x
            //      -> y
            // 3010 -> z
            //      populated as
            //     3010 -> x | y | z
            // if there is any high speed setup, remove all of the associated
            // because it will be handled by the high speed proxy

            let gateway = config::RoutingData::GatewayRouting
                .xget::<Vec<GatewayNode>>()
                .unwrap_or(vec![]);
            let proxy = config::RoutingData::ProxyRouting
                .xget::<Vec<ProxyNode>>()
                .unwrap_or(vec![]);

            let proxy_with_high_speed = proxy.iter().filter(|px| px.high_speed).collect::<Vec<_>>();

            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            // my_server.bootstrap();
            let mut my_gateway: Vec<Box<(dyn pingora::services::Service + 'static)>> = Vec::new();

            let mut already_listened: Vec<String> = vec![];

            log::info!("Gateway Added: {:#?}", &gateway);

            for gw in gateway {
                let listen_addr = gw.addr_listen.clone();

                // check if the listen address is already listened
                if already_listened.contains(&listen_addr) {
                    log::warn!("Gateway service {} is already listened", &listen_addr);
                    continue;
                }
                already_listened.push(listen_addr.clone());

                // find the proxy setup for the listen address
                let proxy_setup = proxy
                    .iter()
                    .filter(|px| px.addr_target == listen_addr)
                    .collect::<Vec<_>>();
                if proxy_setup.len() == 0 {
                    continue;
                }
                let proxy_setup = proxy_setup[0].clone();

                // check if the proxy listen is in the high speed proxy list
                let is_high_speed = proxy_with_high_speed
                    .iter()
                    .any(|px| px.addr_listen == proxy_setup.addr_listen);
                if is_high_speed {
                    log::warn!(
                        "Gateway service {} is already handled by high speed proxy",
                        &proxy_setup.addr_listen
                    );
                    continue;
                }

                log::info!("Setting up gateway service for {}", &gw.addr_target);

                // setup the gateway service
                let mut my_gateway_service = pingora::proxy::http_proxy_service(
                    &my_server.configuration,
                    GatewayApp::new(&gw.addr_listen),
                );

                log::warn!("Gateway Added: {:#?}", &proxy_setup.addr_listen);

                // setup if there any SSL
                let proxy_sni = proxy_setup.sni;
                let proxy_tls = proxy_setup.tls;
                let proxy_tls_pem = proxy_setup.tls_pem;
                let proxy_tls_key = proxy_setup.tls_key;

                // setup tls if needed
                if proxy_tls
                    && proxy_sni.is_some()
                    && proxy_tls_pem.is_some()
                    && proxy_tls_key.is_some()
                {
                    let cert_path = proxy_tls_pem.as_ref().unwrap();
                    let key_path = proxy_tls_key.as_ref().unwrap();

                    let dynamic_cert = boringssl_openssl::DynamicCert::new(&cert_path, &key_path);
                    let mut tls_settings = TlsSettings::with_callbacks(dynamic_cert).unwrap();
                    // // by default intermediate supports both TLS 1.2 and 1.3. We force to tls 1.2 just for the demo

                    tls_settings
                        .deref_mut()
                        .deref_mut()
                        .set_max_proto_version(Some(pingora::tls::ssl::SslVersion::TLS1_2))
                        .unwrap();

                    tls_settings.enable_h2();

                    my_gateway_service.add_tls_with_settings(
                        &proxy_setup.addr_listen,
                        None,
                        tls_settings,
                    );
                } else {
                    my_gateway_service.add_tcp(&proxy_setup.addr_listen);
                }

                // setup the proxy service
                my_gateway.push(Box::new(my_gateway_service));
            }

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
                .unwrap_or(vec![])
                .into_iter()
                .filter(|px| px.high_speed)
                .collect::<Vec<_>>();

            let mut proxies: Vec<Box<dyn Service>> = vec![];

            for px in proxy {
                let addr_target = px.high_speed_addr.unwrap_or(px.addr_target);
                log::warn!("Proxy Added: {}", &px.addr_listen);

                if px.tls && px.sni.is_some() && px.tls_pem.is_some() && px.tls_key.is_some() {
                    let proxy_tls = service::proxy::proxy_service_tls_fast(
                        &px.addr_listen,
                        &addr_target,
                        &px.sni.as_ref().unwrap_or(&"localhost".to_string()),
                        &px.tls_pem.as_ref().unwrap(),
                        &px.tls_key.as_ref().unwrap(),
                    );

                    log::info!("Adding proxy TLS service");
                    proxies.push(Box::new(proxy_tls));
                    continue;
                }

                log::info!("Adding proxy fast service: {:?}", px.addr_listen);
                let proxy_set = service::proxy::proxy_service_fast(
                    &px.addr_listen,
                    &addr_target,
                );
                proxies.push(Box::new(proxy_set));
            }

            // Add all proxy services to the server
            my_server.add_services(proxies);

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
