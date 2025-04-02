//! # API Module
//!
//! This module contains the RESTful API endpoints and handlers for the router-api service.
//! It organizes the API into logical modules for different functionality domains,
//! including user management, system settings, statistics, and synchronization between
//! gateway and proxy nodes.
//!
//! ## Module Structure
//!
//! The API is organized into the following submodules:
//!
//! - `settings`: Configuration endpoints for the gateway and proxy settings
//! - `users`: User management, authentication, and authorization
//! - `statistics`: Performance and usage metrics collection and reporting
//! - `sync`: Gateway and proxy node synchronization and status reporting
//!
//! ## API Configuration
//!
//! All API endpoints are mounted under the `/api/v1` prefix and follow RESTful conventions.
//! Authentication is applied globally through JWT middleware, with specific permissions
//! enforced at the individual endpoint level.

pub mod settings;
pub mod statistics;
pub mod sync;
pub mod users;

use actix_web::web;
use users::init_database;

/// Configure and mount all API routes for the application.
///
/// This function is called during application startup to register all API routes
/// and middleware with the Actix Web service configuration. It initializes the user
/// database and mounts all API endpoints under the `/api/v1` prefix.
///
/// # Arguments
///
/// * `cfg` - A mutable reference to the Actix Web service configuration
///
/// # Example
///
/// ```rust
/// // Inside main.rs or similar file
/// App::new()
///     .configure(api::configure)
/// ```
pub fn configure(cfg: &mut web::ServiceConfig) {
    // Initialize the users database
    if let Err(e) = init_database() {
        eprintln!("Error initializing users database: {}", e);
    }

    cfg.service(
        web::scope("/api/v1")
            // Apply JWT authentication to all API routes
            // This middleware only verifies that the token is valid
            // Specific endpoints can enforce additional role requirements
            .configure(settings::configure)
            .configure(users::configure), // Statistics and sync modules are empty now, but will be protected when implemented
                                          // .configure(statistics::configure)
                                          // .configure(sync::configure)
    );
}
