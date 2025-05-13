//! # Statistics API Module
//! 
//! This module provides endpoints for collecting and reporting gateway statistics.
//! 
//! ## Features
//! 
//! - **Recent Metrics**: Provides recent (last 120 minutes) gateway statistics.
//! - **Status Code Filtering**: Allows filtering statistics by HTTP status code.
//! - **Traffic Volume**: Reports total bytes in/out for the recent period.
//! 
//! ## Endpoints (Implemented)
//! 
//! - `GET /api/v1/statistics/default` - Returns default gateway statistics for the last 120 minutes.
//! - `GET /api/v1/statistics/status/{status}` - Returns gateway statistics filtered by HTTP status code for the last 120 minutes.
//! - `GET /api/v1/statistics/bytes` - Returns total bytes in/out for the last 120 minutes.
//! 
//! ### Query Parameters
//! 
//! All endpoints accept the following optional query parameter:
//! 
//! - `target`: string, optional. Determines the data source:
//!     - `domain` (default): Returns statistics for gateway domains.
//!     - `proxy`: Returns statistics for proxies.
//! 
//! ## Authorization
//! 
//! Statistics endpoints may require authentication and are typically restricted to users
//! with admin or staff roles, as they provide sensitive operational data about the system.
//! 
//! ## Data Collection
//! 
//! Statistics are collected through background processes that:
//! 
//! 1. Gather metrics from running gateway nodes
//! 2. Store this data in a time-series format
//! 3. Aggregate and analyze the data for reporting
//! 4. Provide recent views (last 120 minutes)
// mod logs;
// mod logs_broadcast;
mod log_default;
mod log_bytesio;
mod log_status_code;

use actix_web::web;
// use logs_broadcast::LogsBroadcaster;

/// Configure statistics API routes
/// 
/// This function will set up the routes for statistics endpoints when implemented.
/// Currently a placeholder as the statistics functionality has not yet been implemented.
/// 
/// # Arguments
/// 
/// * `cfg` - A mutable reference to the service configuration
pub fn configure(cfg: &mut web::ServiceConfig) {
    // Statistics endpoints will be implemented here in the future
    // Example route configuration:

    // let sse_logs = LogsBroadcaster::create();
    // let sse_logs = web::Data::from(sse_logs);

    cfg.service(
        web::scope("/statistics")
            // .wrap(JwtAuth::new())
            // .wrap(RoleAuth::admin())
            .service(log_default::init)
            .service(log_status_code::init)
            .service(log_bytesio::init)
    //         .route("/gateways/{id}", web::get().to(handlers::get_gateway_stats))
    //         .route("/proxies/{id}", web::get().to(handlers::get_proxy_stats))
    //         .route("/traffic", web::get().to(handlers::get_traffic_stats))
    //         .route("/errors", web::get().to(handlers::get_error_stats))
    );
}