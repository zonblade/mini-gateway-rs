//! # Proxy Node Model
//!
//! This module defines the Proxy Node data structure, which represents 
//! the configuration of a proxy endpoint within the gateway system.
//! Proxy Nodes are serialized and sent to the registry service to sync
//! configuration across the distributed system.

use actix_web::{post, HttpResponse};
use serde::{Deserialize, Serialize};

use super::proxy_node_tcp::sync_proxy_nodes_to_registry;

/// Represents a proxy node configuration
///
/// This struct contains all the necessary information to set up a proxy endpoint,
/// including TLS configuration, listening address, target address, and other
/// connection-related parameters.
///
/// # Fields
///
/// * `addr_listen` - Address where the proxy listens for connections (e.g., "0.0.0.0:443")
/// * `addr_target` - Target address where requests are forwarded (e.g., "127.0.0.1:8080")
/// * `tls` - Whether TLS is enabled for incoming connections
/// * `tls_pem` - PEM certificate content when TLS is enabled (optional)
/// * `tls_key` - Private key content when TLS is enabled (optional)
/// * `sni` - Server Name Indication value for TLS negotiation (optional)
/// * `high_speed` - Whether speed mode is enabled for faster proxying 
/// * `high_speed_addr` - Specific address to use for speed mode (optional)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyNode {
    /// Address where the proxy listens for connections
    pub addr_listen: String,
    
    /// Target address where requests are forwarded
    pub addr_target: String,
    
    /// Whether TLS is enabled for this proxy
    pub tls: bool,
    
    /// PEM certificate content for TLS
    pub tls_pem: Option<String>,
    
    /// Private key content for TLS
    pub tls_key: Option<String>,
    
    /// Server Name Indication value for TLS
    pub sni: Option<String>,
    
    /// Whether speed mode is enabled for faster proxying
    pub high_speed: bool,
    
    /// Specific address to use for speed mode
    pub high_speed_addr: Option<String>,
}

#[post("/proxy")]
pub async fn gateway() -> HttpResponse {
    
    let result = sync_proxy_nodes_to_registry().await;

    match result {
        Ok(data)=> HttpResponse::Ok().json(data),
        Err(e) => {
            log::error!("Failed to sync gateway nodes: {}", e);
            HttpResponse::BadRequest().body("Failed to sync gateway nodes")
        }
    }
}