//! # Router API
//! 
//! This module provides a RESTful API service for managing the mini-gateway-rs routing system.
//! The API allows for comprehensive management of users, proxies, gateway nodes, and configuration
//! settings, as well as monitoring service statistics.
//! 
//! The server uses Actix Web framework with CORS support and JWT-based authentication to secure
//! endpoints based on user roles (admin, staff, user).
//! 
//! ## Architecture
//! 
//! The API server is built with the following components:
//! - Actix Web HTTP server for handling REST requests
//! - SQLite database for storing configuration and user data
//! - Thread-safe client for managing shared state between requests
//! - CORS support for cross-origin requests
//! - JWT-based authentication and authorization
//! 
//! ## Usage
//! 
//! The server listens on port 24042 and provides endpoints for:
//! - User management (CRUD operations)
//! - Proxy configuration
//! - Gateway node management
//! - Routing rules and settings
//! - Service statistics

mod api;
mod client;
mod module;
mod config;

use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware, web, App, HttpServer};
use client::Client;
use std::sync::{Arc, Mutex};

/// Main entry point for the Router API server.
///
/// This function initializes the application by:
/// 1. Creating a thread-safe client instance that is shared across requests
/// 2. Configuring CORS settings for cross-origin requests
/// 3. Setting up middleware for logging and request processing
/// 4. Configuring API routes
/// 5. Starting the HTTP server with two worker threads
///
/// # Returns
///
/// Returns a Result that resolves to () if the server runs successfully,
/// or an error boxed as a trait object if any issues occur during startup
/// or execution.
///
/// # Errors
///
/// This function may return errors in the following situations:
/// - If the server fails to bind to the specified address/port
/// - If there are issues during server initialization
/// - If any critical runtime errors occur during server execution
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    config::init();
    log::info!("Starting API server...");
    // Create a thread-safe client wrapped in Arc<Mutex<>> to safely share
    // across multiple threads and request handlers
    let client = Arc::new(Mutex::new(Client::new()));

    // Configure and start actix-web server
    log::info!("Starting HTTP server on port 24042...");
    HttpServer::new(move || {
        // Configure CORS with permissive settings for development
        // In production, this should be restricted to specific origins
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            // Add client as app data to make it accessible in route handlers
            // via dependency injection
            .app_data(web::Data::new(client.clone()))
            // Enable logger middleware for request/response logging
            .wrap(middleware::Logger::default())
            // Enable CORS middleware with the configured settings
            .wrap(cors)
            // Configure routes using the function defined in the api module
            .configure(api::configure)
    })
    // Bind server to all network interfaces on port 24042
    .bind("0.0.0.0:24042")?
    // Set number of worker threads to 2 for handling concurrent requests
    .workers(2)
    // Start the HTTP server and keep it running until terminated
    .run()
    .await?;

    Ok(())
}
