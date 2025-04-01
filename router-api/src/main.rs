mod client;
mod api;

use std::sync::{Arc, Mutex};
use client::Client;
use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use actix_web::http::header;

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
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
            .max_age(3600);
            
        App::new()
            // Add client as app data
            .app_data(web::Data::new(client.clone()))
            // Enable logger middleware
            .wrap(middleware::Logger::default())
            // Enable CORS
            .wrap(cors)
            // Configure routes here
            // .service(web::scope("/api/v1")
            //     .configure(api::configure))
    })
    .bind("0.0.0.0:24042")?
    .workers(2)
    .run()
    .await?;
    
    Ok(())
}
