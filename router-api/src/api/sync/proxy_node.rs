//! # Proxy Node Model
//!
//! This module defines the Proxy Node data structure, which represents 
//! the configuration of a proxy endpoint within the gateway system.
//! Proxy Nodes are serialized and sent to the registry service to sync
//! configuration across the distributed system.

use std::sync::{Arc, Mutex};

use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::module::httpc::HttpC;

use super::proxy_node_tcp::sync_proxy_nodes_to_registry;

/// Represents a proxy node configuration
///
/// This struct contains all the necessary information to set up a proxy endpoint,
/// including listening address, target address, and other connection-related parameters.
/// Note: TLS information is now stored separately in ProxyDomain.
///
/// # Fields
///
/// * `id` - Unique identifier for this proxy
/// * `addr_listen` - Address where the proxy listens for connections (e.g., "0.0.0.0:443")
/// * `addr_target` - Target address where requests are forwarded (e.g., "127.0.0.1:8080")
/// * `high_speed` - Whether speed mode is enabled for faster proxying 
/// * `high_speed_addr` - Specific address to use for speed mode (optional)
/// * `domains` - List of domain configurations with TLS settings (if any)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyNode {
    /// Unique identifier for this proxy
    pub id: String,
    
    /// Address where the proxy listens for connections
    pub addr_listen: String,
    
    /// Target address where requests are forwarded
    pub addr_target: String,
    
    /// Whether speed mode is enabled for faster proxying
    pub high_speed: bool,
    
    /// Specific address to use for speed mode
    pub high_speed_addr: Option<String>,
    
    /// List of domain configurations with TLS settings
    pub domains: Vec<ProxyDomain>,
}

/// Represents a proxy domain configuration with TLS settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyDomain {
    /// Unique identifier for this proxy domain
    pub id: String,
    
    /// Reference to the proxy this domain is associated with
    pub proxy_id: String,
    
    /// Reference to a specific gateway node (optional)
    pub gwnode_id: String,
    
    /// Whether TLS is enabled for this domain
    pub tls: bool,
    
    /// PEM certificate content for TLS
    pub tls_pem: Option<String>,
    
    /// Private key content for TLS
    pub tls_key: Option<String>,
    
    /// Server Name Indication value for TLS
    pub sni: Option<String>,
}

#[post("/proxy")]
pub async fn gateway(client: web::Data<Arc<Mutex<HttpC>>>) -> HttpResponse {
    let result = sync_proxy_nodes_to_registry(client.as_ref()).await;

    match result {
        Ok(data)=> HttpResponse::Ok().json(data),
        Err(e) => {
            log::error!("Failed to sync gateway nodes: {:?}", e);
            HttpResponse::BadRequest().body("Failed to sync gateway nodes")
        }
    }
}