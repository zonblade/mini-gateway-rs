use crate::{app::gateway::GatewayApp, service};
use pingora::{
    prelude::Opt,
    server::{RunArgs, Server},
};
use std::thread;

use super::default_page;


pub fn init() {
    let mut server_threads = Vec::new();

    // Gateway Service Thread
    {
        let handle = thread::spawn(|| {
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            my_server.bootstrap();

            let addr = vec!["127.0.0.1:30001", "127.0.0.1:30003"];
            let mut my_gateway: Vec<Box<(dyn pingora::services::Service + 'static)>> = Vec::new();
            for addr in addr.iter() {
                let mut my_gateway_service = pingora::proxy::http_proxy_service(
                    &my_server.configuration,
                    GatewayApp::new(addr),
                );
                my_gateway_service.add_tcp(addr);
                my_gateway.push(Box::new(my_gateway_service));
            }

            my_server.add_services(my_gateway);
            my_server.run(RunArgs::default());
        });
        server_threads.push(handle);
    }

    // non-TLS Proxy server thread
    {
        let handle = thread::spawn(|| {
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            let proxy = service::proxy::proxy_service("0.0.0.0:2000");
            my_server.bootstrap();
            my_server.add_service(proxy);
            // This call blocks until the process receives SIGINT (or another interrupt)
            my_server.run(RunArgs::default());
        });
        server_threads.push(handle);
    }

    // TLS Proxy server thread
    {
        let handle = thread::spawn(|| {
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            let proxy = service::proxy::proxy_service("0.0.0.0:2000");
            my_server.bootstrap();
            my_server.add_service(proxy);
            // This call blocks until the process receives SIGINT (or another interrupt)
            my_server.run(RunArgs::default());
        });
        server_threads.push(handle);
    }

    // Default Page
    {
        let handle404: thread::JoinHandle<()> = thread::spawn(|| {
            // Create a TCP listener for the default 404 page
            default_page::p404::init();
        });
        let handle500: thread::JoinHandle<()> = thread::spawn(|| {
            // Create a TCP listener for the default 500 page
            default_page::p500::init();
        });
        let handle_tls: thread::JoinHandle<()> = thread::spawn(|| {
            // Create a TCP listener for the default TLS page
            default_page::tls_honeypot::init();
        });
        server_threads.push(handle404);
        server_threads.push(handle500);
        server_threads.push(handle_tls);
    }

    for handle in server_threads {
        log::debug!("Waiting for server thread to finish...");
        if let Err(e) = handle.join() {
            log::debug!("Server thread failed: {:?}", e);
        }
    }
}
