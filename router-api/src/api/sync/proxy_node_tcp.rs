use crate::config;
use crate::client::{Client, ClientError};
use super::{proxy_node_queries, TCPDefaultResponse};
use log::{error, info, warn};
use tokio::time::{timeout, Duration};

pub type TCPResult<T> = Result<T, ClientError>;

pub async fn sync_proxy_nodes_to_registry() -> TCPResult<TCPDefaultResponse> {
    log::info!("Syncing proxy nodes to registry...");

    // Get the gateway nodes from the database using our JOIN query
    let proxy_nodes = match proxy_node_queries::get_all_proxy_nodes() {
        Ok(nodes) => nodes,
        Err(e) => {
            error!("Failed to retrieve gateway nodes from database: {}", e);
            return Err(ClientError::ProtocolError(format!("Database error: {}", e)));
        }
    };

    info!(
        "Retrieved {} gateway nodes from database",
        proxy_nodes.len()
    );
    info!("Gateway nodes: {:#?}", proxy_nodes);

    // Create the payload with the nodes
    let payload = proxy_nodes.clone();

    // Create a new client instance
    let mut client = Client::new();

    let server_address = config::Api::TCPAddress.get_str();
    // Connect to the server with a timeout
    match timeout(Duration::from_secs(5), client.connect("127.0.0.1:30099")).await {
        Ok(connect_result) => {
            match connect_result {
                Ok(_) => info!("Connected to registry server at {}", server_address),
                Err(e) => {
                    error!("Failed to connect to registry server: {}", e);
                    return Err(e);
                }
            }
        },
        Err(_) => {
            warn!("Connection attempt to registry server timed out after 5 seconds");
            return Err(ClientError::ProtocolError("Connection timeout".to_string()));
        }
    }

    // Create a new client with the service set using builder pattern
    let mut client = client.service("registry");

    // Send the payload to the "gateway" endpoint with a timeout
    match timeout(
        Duration::from_secs(5),
        client.action::<_, TCPDefaultResponse>("proxy", &payload)
    ).await {
        Ok(action_result) => {
            match action_result {
                Ok(data) => {
                    info!(
                        "Successfully sent {} gateway nodes to registry",
                        proxy_nodes.len()
                    );
                    if let Err(e) = client.close().await {
                        warn!("Error while closing client connection: {}", e);
                    }
                    Ok(data)
                }
                Err(e) => {
                    error!("Failed to send gateway nodes to registry: {}", e);
                    if let Err(close_err) = client.close().await {
                        warn!("Error while closing client connection: {}", close_err);
                    }
                    Err(e)
                }
            }
        },
        Err(_) => {
            warn!("Registry server communication timed out after 5 seconds");
            if let Err(e) = client.close().await {
                warn!("Error while closing client connection: {}", e);
            }
            Err(ClientError::ProtocolError("Communication timeout".to_string()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::settings::proxy_queries;
    use crate::api::settings::Proxy;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_sync_proxy_nodes_to_registry() {
        // Setup: Create test data in the database
        let test_proxy = create_test_proxy();

        // Call the function under test
        let result = sync_proxy_nodes_to_registry().await;

        // Cleanup: Remove test data
        let _ = proxy_queries::delete_proxy_by_id(&test_proxy.id);

        // Assert that the result is Ok
        assert!(result.is_ok());
    }

    // Helper functions to create test data
    fn create_test_proxy() -> Proxy {
        let id = Uuid::new_v4().to_string();
        let proxy = Proxy {
            id: id.clone(),
            title: "Test Proxy".to_string(),
            addr_listen: "127.0.0.1:7070".to_string(),
            addr_target: "127.0.0.1:8081".to_string(),
            high_speed: false,
            high_speed_addr: None,
            high_speed_gwid: None,
        };

        if let Err(e) = proxy_queries::save_proxy(&proxy) {
            error!("Failed to save test proxy: {}", e);
        }

        proxy
    }
}
