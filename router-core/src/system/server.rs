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

/// Module for handling dynamic TLS certificate selection based on SNI (Server Name Indication).
///
/// This module provides functionality to dynamically select TLS certificates based on
/// the hostname requested by clients through SNI. It supports both exact matches and
/// wildcard certificates.
mod boringssl_openssl {
    use async_trait::async_trait;
    use pingora::tls::pkey::{PKey, Private};
    use pingora::tls::ssl::{NameType, SslRef};
    use pingora::tls::x509::X509;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    pub(super) struct DynamicCert {
        certs: Vec<(Option<String>, X509, PKey<Private>)>,
        // Thread-safe cache for hostname lookups
        cache: Mutex<HashMap<String, (Arc<X509>, Arc<PKey<Private>>)>>,
        // Maximum number of entries to prevent unbounded growth
        max_cache_size: usize,
    }

    impl DynamicCert {
        pub(super) fn new() -> Box<Self> {
            Box::new(DynamicCert { 
                certs: Vec::new(),
                cache: Mutex::new(HashMap::new()),
                max_cache_size: 1000, // Default size, can be adjusted based on expected traffic patterns
            })
        }

        // Optional: Allow configuring the cache size
        #[allow(dead_code)]
        pub(super) fn with_cache_size(mut self, max_size: usize) -> Self {
            self.max_cache_size = max_size;
            self
        }

        pub(super) fn add_cert(
            &mut self,
            domain: String,
            cert: &str,
            key: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let cert_bytes = std::fs::read(cert)?;
            let cert = X509::from_pem(&cert_bytes)?;

            let key_bytes = std::fs::read(key)?;
            let key = PKey::private_key_from_pem(&key_bytes)?;

            self.certs.push((Some(domain), cert, key));
            Ok(())
        }

        fn domain_matches(pattern: &str, domain: &str) -> bool {
            // Existing implementation
            if pattern.starts_with("*.") {
                let suffix = &pattern[1..];
                domain.ends_with(suffix) && 
                domain[..domain.len() - suffix.len()].matches('.').count() == 0
            } else {
                pattern == domain
            }
        }

        // Find certificate for a hostname and cache the result
        fn find_cert_for_hostname(&self, hostname: &str) -> Option<(Arc<X509>, Arc<PKey<Private>>)> {
            // First check the cache
            {
                let cache = self.cache.lock().unwrap();
                if let Some(cached) = cache.get(hostname) {
                    return Some(cached.clone());
                }
            }

            // Not in cache, search for it
            let result = self.find_matching_cert(hostname);
            
            // If found, add to cache
            if let Some((cert, key)) = &result {
                let mut cache = self.cache.lock().unwrap();
                
                // Simple cache size management
                if cache.len() >= self.max_cache_size {
                    // Remove some entries if we're at capacity
                    // A more sophisticated approach would use LRU policy
                    if cache.len() > 10 {
                        // Remove approximately 10% of entries
                        let to_remove = (cache.len() / 10).max(1);
                        for _ in 0..to_remove {
                            if let Some(key) = cache.keys().next().cloned() {
                                cache.remove(&key);
                            }
                        }
                    } else {
                        cache.clear(); // Small cache, just clear it
                    }
                }
                
                cache.insert(hostname.to_string(), (cert.clone(), key.clone()));
            }
            
            result
        }
        
        // Search for matching certificate (exact or wildcard)
        fn find_matching_cert(&self, hostname: &str) -> Option<(Arc<X509>, Arc<PKey<Private>>)> {
            // First try exact matches
            for (domain, cert, key) in &self.certs {
                if let Some(domain_str) = domain {
                    if domain_str == hostname {
                        return Some((Arc::new(cert.clone()), Arc::new(key.clone())));
                    }
                }
            }

            // Then try wildcard matches
            for (domain, cert, key) in &self.certs {
                if let Some(domain_str) = domain {
                    if Self::domain_matches(domain_str, hostname) {
                        return Some((Arc::new(cert.clone()), Arc::new(key.clone())));
                    }
                }
            }
            
            // No match found, return the default if available
            if !self.certs.is_empty() {
                let (_, default_cert, default_key) = &self.certs[0];
                return Some((Arc::new(default_cert.clone()), Arc::new(default_key.clone())));
            }
            
            None
        }
    }

    #[async_trait]
    impl pingora::listeners::TlsAccept for DynamicCert {
        async fn certificate_callback(&self, ssl: &mut SslRef) {
            use pingora::tls::ext;

            if self.certs.is_empty() {
                panic!("No certificates configured for TLS!");
            }

            if let Some(server_name) = ssl.servername(NameType::HOST_NAME) {
                // Use the cache to efficiently look up certificates
                if let Some((cert, key)) = self.find_cert_for_hostname(server_name) {
                    ext::ssl_use_certificate(ssl, &cert).unwrap();
                    ext::ssl_use_private_key(ssl, &key).unwrap();
                    return;
                }
            }

            // No SNI or no matching certificate found, use default (index 0)
            let (_, default_cert, default_key) = &self.certs[0];
            ext::ssl_use_certificate(ssl, default_cert).unwrap();
            ext::ssl_use_private_key(ssl, default_key).unwrap();
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

            let gateway = config::RoutingData::GatewayNodeListen
                .xget::<Vec<GatewayNode>>()
                .unwrap_or(vec![]);

            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            // my_server.bootstrap();
            let mut my_gateway: Vec<Box<(dyn pingora::services::Service + 'static)>> = Vec::new();

            let mut already_listened: Vec<String> = vec![];

            eprintln!("[----] Gateway Loaded: {:#?}", &gateway);

            for gw in gateway {
                let listen_addr = gw.addr_listen.clone();

                // check if the listen address is already listened
                if already_listened.contains(&listen_addr) {
                    eprintln!("[----] Gateway service {} is already listened", &listen_addr);
                    continue;
                }
                already_listened.push(listen_addr.clone());

                // setup the gateway service
                let mut my_gateway_service = pingora::proxy::http_proxy_service(
                    &my_server.configuration,
                    GatewayApp::new(&gw.addr_bind),
                );

                eprintln!("[----] Gateway Added: {:#?}", &gw.addr_listen);

                let mut dynamic_cert = boringssl_openssl::DynamicCert::new();
                for tls in gw.tls.clone() {
                    let proxy_sni = tls.sni;
                    let proxy_tls = tls.tls;
                    if !proxy_tls {
                        eprintln!(
                            "[----] Gateway service {:?} [{}] is not TLS, skipping.",
                            proxy_sni, &gw.addr_listen
                        );
                        continue;
                    }

                    let proxy_tls_pem = tls.tls_pem;
                    let proxy_tls_key = tls.tls_key;

                    let cert_path = proxy_tls_pem.as_ref().unwrap();
                    let key_path = proxy_tls_key.as_ref().unwrap();

                    match dynamic_cert.add_cert(
                        proxy_sni.unwrap_or("localhost".to_string()),
                        &cert_path,
                        &key_path,
                    ) {
                        Ok(_) => {
                            eprintln!("[----] Gateway service {} added TLS cert", &gw.addr_listen);
                        }
                        Err(e) => {
                            eprintln!(
                                "[----] Gateway service {} failed to add TLS cert: {:?}",
                                &gw.addr_listen, e
                            );
                        }
                    };
                }

                if gw.tls.is_empty() {
                    // No TLS settings, add TCP service
                    my_gateway_service.add_tcp(&gw.addr_listen);
                } else {
                    // TLS settings are present, add TLS service
                    let mut tls_settings = TlsSettings::with_callbacks(dynamic_cert).unwrap();
                    tls_settings
                        .deref_mut()
                        .deref_mut()
                        .set_max_proto_version(Some(pingora::tls::ssl::SslVersion::TLS1_2))
                        .unwrap();

                    tls_settings.enable_h2();

                    my_gateway_service.add_tls_with_settings(&gw.addr_listen, None, tls_settings);
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

            eprintln!("[----] Proxy Loaded: {:#?}", &proxy);

            let mut proxies: Vec<Box<dyn Service>> = vec![];

            for px in proxy {
                let addr_target = px.high_speed_addr.unwrap_or(px.addr_target);
                eprintln!("[----] Proxy Added: {}", &px.addr_listen);

                if px.tls && px.sni.is_some() && px.tls_pem.is_some() && px.tls_key.is_some() {
                    let proxy_tls = service::proxy::proxy_service_tls_fast(
                        &px.addr_listen,
                        &addr_target,
                        &px.sni.as_ref().unwrap_or(&"localhost".to_string()),
                        &px.tls_pem.as_ref().unwrap(),
                        &px.tls_key.as_ref().unwrap(),
                    );

                    eprintln!("[----] Adding proxy TLS service");
                    proxies.push(Box::new(proxy_tls));
                    continue;
                }

                eprintln!("[----] Adding proxy fast service: {:?}", px.addr_listen);
                let proxy_set = service::proxy::proxy_service_fast(&px.addr_listen, &addr_target);
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
        eprintln!("[----] Waiting for server thread to finish...");
        if let Err(e) = handle.join() {
            eprintln!("[----] Server thread failed: {:?}", e);
        }
    }
}
