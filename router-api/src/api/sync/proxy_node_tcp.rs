use crate::config;
use crate::client::{Client, ClientError};
use super::{proxy_node_queries, TCPDefaultResponse};
use log::{error, info, warn};

pub type TCPResult<T> = Result<T, ClientError>;

pub async fn sync_proxy_nodes_to_registry() -> TCPResult<TCPDefaultResponse> {
    log::info!("Syncing proxy nodes to registry...");

    // Get the proxy nodes from the database using our JOIN query
    let proxy_nodes = match proxy_node_queries::get_all_proxy_nodes() {
        Ok(nodes) => nodes,
        Err(e) => {
            error!("Failed to retrieve proxy nodes from database: {}", e);
            return Err(ClientError::ProtocolError(format!("Database error: {}", e)));
        }
    };

    info!(
        "Retrieved {} proxy nodes from database",
        proxy_nodes.len()
    );
    info!("Proxy nodes: {:#?}", proxy_nodes);

    // Create the payload with the nodes
    let payload = proxy_nodes.clone();

    // Create a new client instance
    let mut client = Client::new();

    let server_address = config::Api::TCPAddress.get_str();
    
    // Connect to the server without timeout
    match client.connect(server_address).await {
        Ok(_) => info!("Connected to registry server at {}", server_address),
        Err(e) => {
            error!("Failed to connect to registry server: {}", e);
            return Err(e);
        }
    }

    // Create a new client with the service set using builder pattern
    let mut client = client.service("registry");

    // Send the payload to the "proxy" endpoint without timeout
    match client.action::<_, TCPDefaultResponse>("proxy", &payload).await {
        Ok(data) => {
            info!(
                "Successfully sent {} proxy nodes to registry",
                proxy_nodes.len()
            );
            if let Err(e) = client.close().await {
                warn!("Error while closing client connection: {}", e);
            }
            Ok(data)
        }
        Err(e) => {
            error!("Failed to send proxy nodes to registry: {}", e);
            if let Err(close_err) = client.close().await {
                warn!("Error while closing client connection: {}", close_err);
            }
            Err(e)
        }
    }
}