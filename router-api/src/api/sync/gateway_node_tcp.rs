
use std::sync::{Arc, Mutex};

use super::gateway_node_queries;
use crate::{
    api::sync::HTTPCResponse,
    config, module::httpc::HttpC,
};
use log::{error, info, warn};

pub async fn sync_gateway_nodes_to_registry(client: &Arc<Mutex<HttpC>>) -> Result<HTTPCResponse, HTTPCResponse> {
    log::info!("Syncing gateway nodes to registry...");

    let gateway_nodes = match gateway_node_queries::get_all_gateway_nodes() {
        Ok(nodes) => nodes,
        Err(e) => {
            error!("Failed to retrieve gateway nodes from database: {}", e);
            return Err(HTTPCResponse{
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            });
        }
    };

    info!(
        "Retrieved {} gateway nodes from database",
        gateway_nodes.len()
    );
    info!("Gateway nodes: {:#?}", gateway_nodes);

    // Create the payload with the nodes
    let payload = gateway_nodes.clone();
    let payload_str = match serde_json::to_string(&payload) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize proxy nodes to JSON: {}", e);
            return Err(HTTPCResponse{
                status: "error".to_string(),
                message: format!("Serialization error: {}", e),
            });
        }
    };

    let _ = match client.lock() {
        Ok(client)=>{
            let _ = client.post_text("/gateway/node", &payload_str);
            info!("Successfully sent proxy nodes to registry");
        },
        Err(e)=>{
            error!("Failed to lock HTTP client: {}", e);
            return Err(HTTPCResponse{
                status: "error".to_string(),
                message: format!("Client lock error: {}", e),
            });
        }
    };
    Ok(HTTPCResponse {
        status: "success".to_string(),
        message: format!("Successfully synced gateway nodes"),
    })
}

pub async fn sync_gateway_paths_to_registry(client: &Arc<Mutex<HttpC>>) -> Result<HTTPCResponse, HTTPCResponse> {
    // Get the gateway nodes from the database using our JOIN query
    let gateway_path = match gateway_node_queries::get_all_gateway_paths() {
        Ok(nodes) => nodes,
        Err(e) => {
            error!("Failed to retrieve gateway paths from database: {}", e);
            return Err(HTTPCResponse{
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            });
        }
    };

    info!(
        "Retrieved {} gateway paths from database",
        gateway_path.len()
    );
    info!("Gateway paths: {:#?}", gateway_path);

    // Create the payload with the nodes for the second action
    let payload = gateway_path.clone();
    let payload_str = match serde_json::to_string(&payload) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize proxy nodes to JSON: {}", e);
            return Err(HTTPCResponse{
                status: "error".to_string(),
                message: format!("Serialization error: {}", e),
            });
        }
    };

    let _ = match client.lock() {
        Ok(client)=>{
            let _ = client.post_text("/gateway/path", &payload_str);
            info!("Successfully sent proxy nodes to registry");
        },
        Err(e)=>{
            error!("Failed to lock HTTP client: {}", e);
            return Err(HTTPCResponse{
                status: "error".to_string(),
                message: format!("Client lock error: {}", e),
            });
        }
    };

    Ok(HTTPCResponse {
        status: "success".to_string(),
        message: format!("Successfully synced gateway paths"),
    })
}