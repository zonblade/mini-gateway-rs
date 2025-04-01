mod api;
mod client;
mod module;

use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware, web, App, HttpServer};
use client::Client;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing thread-safe client...");
    let client = Arc::new(Mutex::new(Client::new()));

    // Configure and start actix-web server
    println!("Starting API server...");
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin("*")
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            // Add client as app data
            .app_data(web::Data::new(client.clone()))
            // Enable logger middleware
            .wrap(middleware::Logger::default())
            // Enable CORS
            .wrap(cors)
            // Configure routes
            .configure(api::configure)
    })
    .bind("0.0.0.0:24042")?
    .workers(2)
    .run()
    .await?;

    Ok(())
}
