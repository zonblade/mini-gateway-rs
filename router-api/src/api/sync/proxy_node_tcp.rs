
use std::sync::{Arc, Mutex};

use crate::module::httpc::HttpC;

use super::{proxy_node_queries, HTTPCResponse};
use log::{error, info, warn};

pub async fn sync_proxy_nodes_to_registry(client: &Arc<Mutex<HttpC>>) -> Result<HTTPCResponse, HTTPCResponse> {
    log::info!("Syncing proxy nodes to registry...");

    // Get the proxy nodes from the database using our JOIN query
    let proxy_nodes = match proxy_node_queries::get_all_proxy_nodes() {
        Ok(nodes) => nodes,
        Err(e) => {
            error!("Failed to retrieve proxy nodes from database: {}", e);
            return Err(HTTPCResponse{
                status: "error".to_string(),
                message: format!("Database error: {}", e),
            });
        }
    };

    info!(
        "Retrieved {} proxy nodes from database",
        proxy_nodes.len()
    );
    info!("Proxy nodes: {:#?}", proxy_nodes);

    // Create the payload with the nodes
    let payload = proxy_nodes.clone();
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
            let _ = client.post_text("/proxy/node", &payload_str);
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
        message: format!("Successfully sync proxy nodes"),
    })
}