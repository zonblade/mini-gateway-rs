//! # Settings API Module
//!
//! This module provides a comprehensive API for managing gateway configuration settings, including:
//!
//! - **Proxies**: Basic forwarding configurations that listen on specific addresses and forward traffic
//! - **Gateway Nodes**: Extensions to proxies that provide alternative target paths
//! - **Gateways**: Routing rules tied to gateway nodes with pattern matching and prioritization
//!
//! The module is structured with a clear separation between data models, database queries, and HTTP endpoints.
//! Each component has dedicated submodules for listing, retrieving, creating, updating, and deleting resources.

pub mod gateway_get;
pub mod gateway_list;
pub mod gateway_queries;
pub mod gateway_set;
pub mod gwnode_get;
pub mod gwnode_list;
pub mod gwnode_queries;
pub mod gwnode_set;
pub mod proxy_get;
pub mod proxy_list;
pub mod proxy_queries;
pub mod proxy_set;

use serde::{Deserialize, Serialize};

// Import actix-web components for the configure function
use actix_web::web;
// Import authentication middleware
use crate::api::users::RoleAuth;

use super::users::JwtAuth;

/// Represents a proxy configuration in the system
///
/// A proxy is the most basic routing component that listens on a specific address and
/// forwards traffic to a target address. It can optionally use TLS for secure connections.
///
/// # Fields
///
/// * `id` - Unique identifier used to reference this proxy throughout the system
/// * `title` - Human-readable name for easy identification in user interfaces
/// * `addr_listen` - Network address where the proxy listens (format: "ip:port")
/// * `addr_target` - Destination address where traffic is forwarded (format: "ip:port")
/// * `tls` - Whether HTTPS/TLS is enabled for incoming connections
/// * `tls_pem` - PEM-encoded certificate when TLS is manually configured
/// * `tls_key` - Private key for the certificate when TLS is manually configured
/// * `tls_autron` - Whether automatic TLS certificate provisioning is enabled
/// * `sni` - Server Name Indication value for TLS negotiation
///
/// # Examples
///
/// Basic HTTP proxy:
/// ```
/// Proxy {
///     id: "550e8400-e29b-41d4-a716-446655440000",
///     title: "Web Server",
///     addr_listen: "0.0.0.0:80",
///     addr_target: "127.0.0.1:8080",
///     tls: false,
///     tls_pem: None,
///     tls_key: None,
///     tls_autron: false,
///     sni: None,
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proxy {
    /// Unique identifier for the proxy
    pub id: String,
    /// Human-readable title for the proxy
    pub title: String,
    /// Address where the proxy listens for incoming connections
    pub addr_listen: String,
    /// Target address where requests are forwarded to
    pub addr_target: String,
    /// Whether TLS is enabled for this proxy
    pub tls: bool,
    /// PEM certificate content for TLS
    pub tls_pem: Option<String>,
    /// Private key content for TLS
    pub tls_key: Option<String>,
    /// Whether automatic TLS is enabled
    pub tls_autron: bool,
    /// Server Name Indication value for TLS
    pub sni: Option<String>,
}

/// Represents a gateway node configuration in the system
///
/// A gateway node extends a proxy by providing an alternative target path.
/// It acts as an intermediary between proxies and gateways, allowing for more
/// complex routing scenarios.
///
/// # Fields
///
/// * `id` - Unique identifier for this gateway node
/// * `proxy_id` - The ID of the proxy this gateway node is associated with
/// * `alt_target` - An alternative target URL that can be used for routing
///
/// # Relationships
///
/// * Associated with exactly one `Proxy` via `proxy_id`
/// * Can have multiple `Gateway` routing rules attached to it
///
/// # Examples
///
/// ```
/// GatewayNode {
///     id: "7f9c24e5-1315-43a7-9f31-6eb9772cb46a",
///     proxy_id: "550e8400-e29b-41d4-a716-446655440000",
///     alt_target: "http://backup-server.internal:8080",
/// }
/// ```
///
/// When a proxy is deleted, its associated gateway nodes are not deleted but are marked
/// as "unbound" by setting their `proxy_id` to "unbound".
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GatewayNode {
    /// Unique identifier for the gateway node
    pub id: String,
    /// Reference to the proxy ID that this gateway node is associated with
    pub proxy_id: String,
    /// Alternative target URL
    pub alt_target: String,
}

/// Represents a gateway configuration in the system
///
/// A gateway defines specific routing rules for a gateway node using pattern matching
/// and priority levels. Incoming requests are matched against patterns, and the rule
/// with the highest priority (lowest numeric value) is selected.
///
/// # Fields
///
/// * `id` - Unique identifier for this gateway
/// * `gwnode_id` - The ID of the gateway node this gateway is associated with
/// * `pattern` - URL pattern for matching incoming requests
/// * `target` - Target URL where matching requests should be routed
/// * `priority` - Priority level, with lower numbers having higher precedence
///
/// # Pattern Matching
///
/// The `pattern` field supports various matching techniques:
/// - Exact path matching: "/api/users"
/// - Prefix matching with wildcard: "/api/*"
/// - Regex-like patterns: "^/users/[0-9]+"
///
/// # Relationships
///
/// * Associated with exactly one `GatewayNode` via `gwnode_id`
/// * When a gateway node is deleted, all its associated gateways are also deleted
///
/// # Examples
///
/// ```
/// Gateway {
///     id: "a1b2c3d4-e5f6-4321-8765-10293847abcd",
///     gwnode_id: "7f9c24e5-1315-43a7-9f31-6eb9772cb46a",
///     pattern: "/api/users/*",
///     target: "http://user-service:8080",
///     priority: 10,
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Gateway {
    /// Unique identifier for the gateway
    pub id: String,
    /// Reference to the gateway node ID that this gateway is associated with
    pub gwnode_id: String,
    /// Pattern for URL matching
    pub pattern: String,
    /// Target URL
    pub target: String,
    /// Priority level (lower number = higher priority)
    pub priority: i32,
}

/// Configures the settings API routes
///
/// This function registers all endpoints for managing proxies, gateway nodes, and gateways
/// under the "/settings" path prefix. It's typically called during application startup
/// to set up the routing configuration.
///
/// # Parameters
///
/// * `cfg` - Mutable reference to a web service configuration where routes will be registered
///
/// # API Endpoints
///
/// ## Proxy endpoints:
/// - GET /settings/proxy - List all proxies
/// - GET /settings/proxy/{id} - Get a specific proxy by ID
/// - POST /settings/proxy - Create or update a proxy
/// - DELETE /settings/proxy/{id} - Delete a proxy
///
/// ## Gateway Node endpoints:
/// - GET /settings/gwnode/list - List all gateway nodes
/// - GET /settings/gwnode/list/{proxy_id} - List gateway nodes for a specific proxy
/// - GET /settings/gwnode/{id} - Get a specific gateway node by ID
/// - POST /settings/gwnode/set - Create or update a gateway node
/// - POST /settings/gwnode/delete - Delete a gateway node
///
/// ## Gateway endpoints:
/// - GET /settings/gateway/list - List all gateways
/// - GET /settings/gateway/list/{gwnode_id} - List gateways for a specific gateway node
/// - GET /settings/gateway/{id} - Get a specific gateway by ID
/// - POST /settings/gateway/set - Create or update a gateway
/// - POST /settings/gateway/delete - Delete a gateway
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settings")
            .wrap(JwtAuth::new())
            // Apply staff role authentication middleware to all settings endpoints
            // This allows both staff and admin users to access the settings (staff_or_admin)
            .wrap(RoleAuth::staff())
            .service(proxy_list::list_proxies)
            .service(proxy_get::get_proxy)
            .service(proxy_set::set_proxy)
            .service(proxy_set::delete_proxy)
            .service(gwnode_list::list_gateway_nodes)
            .service(gwnode_list::list_gateway_nodes_by_proxy)
            .service(gwnode_get::get_gateway_node)
            .service(gwnode_set::set_gateway_node)
            .service(gwnode_set::delete_gateway_node)
            .service(gateway_list::list_gateways)
            .service(gateway_list::list_gateways_by_gwnode)
            .service(gateway_get::get_gateway)
            .service(gateway_set::set_gateway)
            .service(gateway_set::delete_gateway),
    );
}
