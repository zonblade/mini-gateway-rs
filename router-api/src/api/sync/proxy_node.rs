use actix_web::{post, HttpResponse};
use serde::{Deserialize, Serialize};

use super::proxy_node_tcp::sync_proxy_nodes_to_registry;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyNode {
    /// Whether TLS is enabled for this proxy node
    pub tls: bool,
    
    /// Server Name Indication value for TLS connections
    pub sni: Option<String>,
    
    /// Path to the TLS certificate PEM file
    pub tls_pem: Option<String>,
    
    /// Path to the TLS private key file
    pub tls_key: Option<String>,
    
    /// Network address this proxy listens on (e.g., "0.0.0.0:443")
    pub addr_listen: String,
    
    /// Target address to forward traffic to (e.g., "127.0.0.1:8080")
    pub addr_target: String,
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