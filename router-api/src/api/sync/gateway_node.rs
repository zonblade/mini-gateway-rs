use std::sync::{Arc, Mutex};
use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{api::sync::gateway_node_tcp::{sync_gateway_nodes_to_registry, sync_gateway_paths_to_registry}, module::httpc::HttpC};

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
pub async fn gateway(client: web::Data<Arc<Mutex<HttpC>>>) -> HttpResponse {
    
    let result = match sync_gateway_nodes_to_registry(client.as_ref()).await {
        Ok(data) => {
            log::info!("Successfully synced gateway nodes to registry");
            let path_result = sync_gateway_paths_to_registry(client.as_ref()).await;
            match path_result {
                Ok(_paths) => {
                    log::info!("Successfully synced gateway paths to registry");
                    data
                }
                Err(e) => {
                    log::error!("Failed to sync gateway paths: {:?}", e);
                    return HttpResponse::BadRequest().body("Failed to sync gateway paths");
                }
            }
        }
        Err(e) => {
            log::error!("Failed to sync gateway nodes: {:?}", e);
            return HttpResponse::BadRequest().body("Failed to sync gateway nodes");
        }
    };

    HttpResponse::Ok().json(result)
}