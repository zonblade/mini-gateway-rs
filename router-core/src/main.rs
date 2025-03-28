use std::thread;

use pingora::server::configuration::Opt;
use pingora::server::{RunArgs, Server};
use tokio;

mod app;
mod service;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");

    env_logger::init();
    println!("Starting proxy server...");

    let opt = Some(Opt::default());
    let mut my_server = Server::new(opt).expect("Failed to create server");

    let proxy = service::proxy::proxy_service("127.0.0.1:8088", "127.0.0.1:8080");

    my_server.add_service(proxy);
    
    // Spawn the server in a separate OS thread instead of a Tokio task
    let server_handle = thread::spawn(move || {
        my_server.run(RunArgs::default());
    });
    
    // Your main async operations can continue here
    // ...
    
    // Join the thread to wait for the server to finish
    if let Err(e) = server_handle.join() {
        eprintln!("Server thread failed: {:?}", e);
    }
}