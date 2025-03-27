use pingora::server::configuration::Opt;
use pingora::server::Server;
use tokio;

mod app;
mod service;

#[tokio::main]
async fn main() {
    env_logger::init(); // For logging
    println!("Starting proxy server...");

    // Optional configuration
    let opt = Some(Opt::default());
    let mut my_server = Server::new(opt).expect("Failed to create server");

    // Single route: Forward requests from localhost:8080 to example.com
    let proxy = service::proxy::proxy_service(
        "127.0.0.1:8080", // listen
        "127.0.0.1:9982", // target
    );

    my_server.add_service(proxy);
    my_server.run_forever();
}
