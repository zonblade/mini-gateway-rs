use actix_web::{post, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::api::sync::gateway_node_tcp::sync_gateway_nodes_to_registry;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayNode {
    /// Processing priority (higher values = higher priority)
    pub priority: i8,
    
    /// Network address this gateway listens on (e.g., "0.0.0.0:80")
    pub addr_listen: String,
    
    /// Target address to forward traffic to (e.g., "127.0.0.1:8080")
    pub addr_target: String,
    
    /// URI path pattern to match incoming requests against (e.g., "/api/*")
    pub path_listen: String,
    
    /// Target path to rewrite matched paths to (e.g., "/")
    pub path_target: String,
}

#[post("/gateway")]
pub async fn gateway() -> HttpResponse {
    
    let result = sync_gateway_nodes_to_registry().await;

    match result {
        Ok(data)=> HttpResponse::Ok().json(data),
        Err(e) => {
            log::error!("Failed to sync gateway nodes: {}", e);
            HttpResponse::BadRequest().body("Failed to sync gateway nodes")
        }
    }
}