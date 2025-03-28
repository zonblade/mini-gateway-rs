use std::thread::{self, sleep};
use std::time::Duration;

use pingora::server::configuration::Opt;
use pingora::server::{RunArgs, Server};
use service::registry::RedisMessage;
use tokio;

mod app;
mod service;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");

    env_logger::init();
    println!("Starting proxy server...");
    let redis_receiver = service::registry::run_redis_client();
    let mut active_state = false;

    loop {
        // create oneshot channel to send restart signal to the thread

        // detect if it's already on active state.
        if !active_state {
            let mut server_threads = Vec::new();

            // Spawn the server in a separate OS thread instead of a Tokio task
            let server_handle = thread::spawn(move || {
                let opt = Some(Opt::default());
                let mut my_server = Server::new(opt).expect("Failed to create server");

                let proxy = service::proxy::proxy_service("127.0.0.1:8088", "127.0.0.1:8080");

                my_server.add_service(proxy);
                my_server.run(RunArgs::default());
            });
            server_threads.push(server_handle);


            let server_handle_tls = thread::spawn(move || {
                let opt = Some(Opt::default());
                let mut my_server = Server::new(opt).expect("Failed to create server");

                let proxy = service::proxy::proxy_service("127.0.0.1:8088", "127.0.0.1:8080");

                my_server.add_service(proxy);
                my_server.run(RunArgs::default());
            });
            server_threads.push(server_handle_tls);


            // Join the thread to wait for the server to finish
            for handle in server_threads {
                if let Err(e) = handle.join() {
                    eprintln!("Server thread failed: {:?}", e);
                }
            }

            active_state = true;
            continue;
        }


        let signal = match redis_receiver.recv() {
            Ok(signal) => signal,
            Err(e) => {
                eprintln!("Redis client thread failed: {:?}", e);
                break;
            }
        };

        if signal == RedisMessage::TriggerRestart {
            println!("Received restart signal from Redis");
            active_state = false;
        }

        // if no update just delay it.
        sleep(Duration::from_millis(200));
    }
}
