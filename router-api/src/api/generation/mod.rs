//! # Generation API Module
//!
//! This module provides utility endpoints for generating various types of identifiers
//! and tokens that frontend applications may need. This is particularly useful for
//! older browsers that may have limited or unreliable crypto API support.
//!
//! ## Available Endpoints
//!
//! - **UUID Generation**: Secure UUID v4 generation using system random number generator
//!
//! ## Security
//!
//! All endpoints in this module are protected by JWT authentication. Users must provide
//! a valid Bearer token to access these services. No additional role restrictions are
//! applied as UUID generation is a basic utility service.

mod uuid_generate;

use super::users::JwtAuth;
use actix_web::web;

/// Configure and mount all generation API routes
///
/// This function registers all generation utility endpoints under the "/generation"
/// path prefix. It's called during application startup to set up the routing
/// configuration.
///
/// # Security
///
/// All endpoints are protected by JWT authentication middleware. Users must provide
/// a valid Bearer token in the Authorization header to access these services.
///
/// # Arguments
///
/// * `cfg` - A mutable reference to the Actix Web service configuration
///
/// # API Endpoints
///
/// - GET /generation/uuid - Generate a new UUID v4
///
/// # Example
///
/// ```rust
/// // Inside main API configuration
/// cfg.service(
///     web::scope("/api/v1")
///         .configure(generation::configure)
/// );
/// ```
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/generation")
            // Apply JWT authentication to all generation endpoints
            // This ensures only authenticated users can generate UUIDs
            .wrap(JwtAuth::new())
            // UUID generation endpoint
            .service(uuid_generate::generate_uuid),
    );
}
