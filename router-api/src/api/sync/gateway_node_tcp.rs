
use super::gateway_node_queries;
use crate::{
    api::sync::TCPDefaultResponse,
    client::{Client, ClientError, Result as TCPResult},
    config,
};
use log::{error, info, warn};

/// Syncs all gateway nodes to the registry server
///
/// This function retrieves all gateway nodes by joining data from multiple tables,
/// then sends the collected data to the gateway registry server via TCP.
///
/// # Returns
///
/// * `Ok(())` - If the data was successfully sent to the registry server
/// * `Err(ClientError)` - If there was an error retrieving or sending the data
///
/// # Errors
///
/// This function will return an error if:
/// - Database queries fail
/// - Connection to the registry server cannot be established
/// - Data transmission fails
pub async fn sync_gateway_nodes_to_registry() -> TCPResult<TCPDefaultResponse> {
    log::info!("Syncing gateway nodes to registry...");

    let gateway_nodes = match gateway_node_queries::get_all_gateway_nodes() {
        Ok(nodes) => nodes,
        Err(e) => {
            error!("Failed to retrieve gateway nodes from database: {}", e);
            return Err(ClientError::ProtocolError(format!("Database error: {}", e)));
        }
    };

    info!(
        "Retrieved {} gateway nodes from database",
        gateway_nodes.len()
    );
    info!("Gateway nodes: {:#?}", gateway_nodes);

    // Create the payload with the nodes
    let payload = gateway_nodes.clone();

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

    // Send the payload to the "gateway" endpoint without timeout
    match client.action::<_, TCPDefaultResponse>("gwnode", &payload).await {
        Ok(data) => {
            info!(
                "Successfully sent {} gateway nodes to registry",
                gateway_nodes.len()
            );
            // Close the current client
            if let Err(e) = client.close().await {
                warn!("Error closing client connection: {}", e);
            }
            Ok(data)
        }
        Err(e) => {
            error!("Failed to send gateway nodes to registry: {}", e);
            if let Err(close_err) = client.close().await {
                warn!("Error closing client connection: {}", close_err);
            }
            Err(e)
        }
    }
}

pub async fn sync_gateway_paths_to_registry() -> TCPResult<TCPDefaultResponse> {
    // Get the gateway nodes from the database using our JOIN query
    let gateway_path = match gateway_node_queries::get_all_gateway_paths() {
        Ok(nodes) => nodes,
        Err(e) => {
            error!("Failed to retrieve gateway paths from database: {}", e);
            return Err(ClientError::ProtocolError(format!("Database error: {}", e)));
        }
    };

    info!(
        "Retrieved {} gateway paths from database",
        gateway_path.len()
    );
    info!("Gateway paths: {:#?}", gateway_path);

    // Create the payload with the nodes for the second action
    let payload = gateway_path.clone();

    // Create a new client for the second action without timeout
    let mut new_client = Client::new();
    let server_address = config::Api::TCPAddress.get_str();
    
    match new_client.connect(server_address).await {
        Ok(_) => info!("Connected to registry server at {}", server_address),
        Err(e) => {
            error!("Failed to connect to registry server for second action: {}", e);
            return Err(e);
        }
    }

    // Set service for new client
    let mut new_client = new_client.service("registry");

    // Send the second payload without timeout
    match new_client.action::<_, TCPDefaultResponse>("gateway", &payload).await {
        Ok(data) => {
            info!(
                "Successfully sent {} gateway paths to registry",
                gateway_path.len()
            );
            if let Err(e) = new_client.close().await {
                warn!("Error closing client connection: {}", e);
            }
            Ok(data)
        }
        Err(e) => {
            error!("Failed to send gateway paths to registry: {}", e);
            if let Err(close_err) = new_client.close().await {
                warn!("Error closing client connection: {}", close_err);
            }
            Err(e)
        }
    }
}