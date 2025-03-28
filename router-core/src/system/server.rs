use pingora::{
    prelude::Opt,
    server::{RunArgs, Server},
};
use std::thread;

use crate::service;

use super::terminator;

pub fn init() {
    let mut server_threads = Vec::new();

    // sample of 10 second termination.
    {
        thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(10));
            terminator::service::init();
        });
    }

    // Non-TLS server thread
    {
        let handle = thread::spawn(|| {
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            let proxy = service::proxy::proxy_service("127.0.0.1:9010", "127.0.0.1:9000");
            my_server.bootstrap();
            my_server.add_service(proxy);
            // This call blocks until the process receives SIGINT (or another interrupt)
            my_server.run(RunArgs::default());
            log::warn!("Non-TLS server shutting down.");
        });
        server_threads.push(handle);
    }

    // TLS server thread
    {
        let handle = thread::spawn(|| {
            let opt = Some(Opt::default());
            let mut my_server = Server::new(opt).expect("Failed to create server");
            let proxy = service::proxy::proxy_service("127.0.0.1:9011", "127.0.0.1:8080");
            my_server.bootstrap();
            my_server.add_service(proxy);
            // This call blocks until the process receives SIGINT (or another interrupt)
            my_server.run(RunArgs::default());
            log::warn!("TLS server shutting down.");
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
