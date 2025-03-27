use pingora::server::configuration::Opt;
use pingora::server::Server;
use tokio;

mod app;
mod service;

#[tokio::main]
async fn main() {
    env_logger::init();
    println!("Starting proxy server...");

    let opt = Some(Opt::default());
    let mut my_server = Server::new(opt).expect("Failed to create server");

    // Define SNI (target server's hostname)
    let proxy_sni = "example.com"; // Set this to your target's hostname

    let cert_path = "path/to/cert.pem"; // Replace with real cert path
    let key_path = "path/to/key.pem";   // Replace with real key path

    let proxy = service::proxy::proxy_service_tls(
        "127.0.0.1:8080", // listen address
        "127.0.0.1:9982", // target address
        proxy_sni,        // SNI for upstream connection
        cert_path,
        key_path,
    );

    my_server.add_service(proxy);
    my_server.run_forever();
}