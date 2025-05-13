//! # Router API
//!
//! This module provides a RESTful API service for managing the mini-gateway-rs routing system.
//! The API allows for comprehensive management of users, proxies, gateway nodes, and configuration
//! settings, as well as monitoring service statistics and health.
//!
//! ## Architecture
//!
//! The API server is built with the following components:
//! - **Actix Web**: High-performance HTTP server framework for handling REST requests
//! - **SQLite Database**: Persistent storage for configuration, user data, and routing rules
//! - **Thread-safe Client**: Arc<Mutex<Client>> for managing shared state between requests
//! - **CORS Support**: Configurable cross-origin request security
//! - **JWT Authentication**: Role-based access control (admin, staff, user)
//! - **Registry Synchronization**: Automatic sync of proxy and gateway nodes with central registry
//!
//! ## API Endpoints
//!
//! The server provides the following endpoint categories:
//! - `/api/v1/users` - User management (create, read, update, delete)
//! - `/api/v1/proxies` - Proxy configuration and status
//! - `/api/v1/gateways` - Gateway node management
//! - `/api/v1/routes` - Routing rules and policies
//! - `/api/v1/stats` - Service performance and usage metrics
//! - `/api/v1/health` - Health checks and system status
//!
//! ## Authentication
//!
//! The API uses JWT tokens for authentication with the following roles:
//! - **Admin**: Full access to all endpoints and operations
//! - **Staff**: Access to monitoring and limited configuration
//! - **User**: Access only to assigned resources and read operations
//!
//! ## Configuration
//!
//! Server configuration is loaded from:
//! - Environment variables
//! - Configuration files in the working directory
//! - Default values for development environments
//!
//! ## Network
//!
//! By default, the service listens on port 24042 on all network interfaces (0.0.0.0).
//! This can be configured through environment variables or config files.

mod api;
mod client;
mod config;
mod module;

use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware, web, App, HttpServer};
use api::sync;
use client::Client;
use module::memory_log;
use std::sync::{Arc, Mutex};

/// Main entry point for the Router API server.
///
/// This function initializes the application by:
/// 1. Loading configuration from environment variables and config files
/// 2. Creating a thread-safe client instance that is shared across requests
/// 3. Synchronizing proxy and gateway nodes with the central registry
/// 4. Configuring CORS settings for cross-origin requests
/// 5. Setting up middleware for logging, authentication and request processing
/// 6. Configuring API routes for all endpoint categories
/// 7. Starting the HTTP server with worker threads for concurrent request handling
///
/// # Network Configuration
///
/// The server binds to 0.0.0.0:24042 by default, making it accessible from any network interface.
/// This can be customized through the ROUTER_API_HOST and ROUTER_API_PORT environment variables.
///
/// # Performance
///
/// The server uses 2 worker threads by default to handle concurrent requests efficiently.
/// This value can be adjusted based on available system resources and expected load.
///
/// # Synchronization
///
/// During startup, the server synchronizes proxy and gateway node configurations with
/// the central registry to ensure consistency across the routing system.
///
/// # Returns
///
/// Returns a Result that resolves to () if the server runs successfully,
/// or an error boxed as a trait object if any issues occur.
///
/// # Errors
///
/// This function may return errors in the following situations:
/// - Configuration loading failures
/// - Registry synchronization issues
/// - Network binding failures (e.g., port already in use)
/// - Critical runtime errors during server execution
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    {
        std::env::set_var("RUST_LOG", "warn");
        env_logger::init();
        config::init();
    }

    {
        log::info!("Starting memory log spawner...");
        memory_log::spawner::spawn_all();
    }

    // Parse command line arguments using clap
    let matches = clap::Command::new("Router API")
        .version("0.0.1-pre")
        .author("mini-gateway-rs")
        .about("RESTful API service for managing the Mini Gateway routing system")
        .arg(
            clap::Arg::new("ip")
                .long("ip")
                .help("IP address to bind the server to")
                .value_name("IP")
                .default_value("0.0.0.0"),
        )
        .arg(
            clap::Arg::new("port")
                .long("port")
                .help("Port number to bind the server to")
                .value_name("PORT")
                .default_value("24042")
                .value_parser(clap::value_parser!(u16)),
        )
        .get_matches();

    // Extract values with fallbacks
    let ip = matches.get_one::<String>("ip").unwrap();
    let port = matches.get_one::<u16>("port").unwrap();
    let bind_address = format!("{}:{}", ip, port);

    log::info!("Starting API server on {}...", bind_address);

    // Create a thread-safe client wrapped in Arc<Mutex<>> to safely share
    // across multiple threads and request handlers
    let client = Arc::new(Mutex::new(Client::new()));

    log::info!("Initializing sync...");
    {
        // Try to sync with registry but don't fail startup if it doesn't work
        match sync::proxy_node_tcp::sync_proxy_nodes_to_registry().await {
            Ok(_) => log::info!("Successfully synced proxy nodes to registry"),
            Err(e) => log::warn!("Failed to sync proxy nodes to registry: {}. Continuing startup anyway.", e),
        }
        
        match sync::gateway_node_tcp::sync_gateway_nodes_to_registry().await {
            Ok(_) => log::info!("Successfully synced gateway nodes to registry"),
            Err(e) => log::warn!("Failed to sync gateway nodes to registry: {}. Continuing startup anyway.", e),
        }

        match sync::gateway_node_tcp::sync_gateway_paths_to_registry().await {
            Ok(_) => log::info!("Successfully synced gateway paths to registry"),
            Err(e) => log::warn!("Failed to sync gateway paths to registry: {}. Continuing startup anyway.", e),
        }
    }

    // Configure and start actix-web server
    log::info!("Starting HTTP server on {}...", bind_address);
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
    // Bind server to the specified address and port
    .bind(&bind_address)?
    // Set number of worker threads to 2 for handling concurrent requests
    .workers(2)
    // Start the HTTP server and keep it running until terminated
    .run()
    .await?;

    Ok(())
}
