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
    // Get the gateway nodes from the database using our JOIN query
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

    // Create the payload with the nodes
    let payload = gateway_nodes.clone();

    // Create a new client instance
    let mut client = Client::new();

    let server_address = config::Api::TCPAddress.get_str();
    // Connect to the server
    match client.connect("127.0.0.1:30099").await {
        Ok(_) => info!("Connected to registry server at {}", server_address),
        Err(e) => {
            error!("Failed to connect to registry server: {}", e);
            return Err(e);
        }
    }

    // Create a new client with the service set using builder pattern
    let mut client = client.service("registry");

    // Send the payload to the "gateway" endpoint
    match client
        .action::<_, TCPDefaultResponse>("gateway", &payload)
        .await
    {
        Ok(data) => {
            info!(
                "Successfully sent {} gateway nodes to registry",
                gateway_nodes.len()
            );
            client.close().await?;
            Ok(data)
        }
        Err(e) => {
            error!("Failed to send gateway nodes to registry: {}", e);
            client.close().await?;
            Err(e)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::settings::{gateway_queries, gwnode_queries, proxy_queries};
    use crate::api::settings::{Gateway, GatewayNode as SettingsGatewayNode, Proxy};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_sync_gateway_nodes_to_registry() {
        // Setup: Create test data in the database
        let test_proxy = create_test_proxy();
        let test_gwnode = create_test_gwnode(&test_proxy.id);
        let test_gateway = create_test_gateway(&test_gwnode.id);

        // Call the function under test
        let result = sync_gateway_nodes_to_registry().await;

        // Cleanup: Remove test data
        let _ = gateway_queries::delete_gateway_by_id(&test_gateway.id);
        let _ = gwnode_queries::delete_gateway_node_by_id(&test_gwnode.id);
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
            tls: false,
            tls_pem: None,
            tls_key: None,
            tls_autron: false,
            sni: None,
        };

        if let Err(e) = proxy_queries::save_proxy(&proxy) {
            error!("Failed to save test proxy: {}", e);
        }

        proxy
    }

    fn create_test_gwnode(proxy_id: &str) -> SettingsGatewayNode {
        let id = Uuid::new_v4().to_string();
        let gwnode = SettingsGatewayNode {
            id: id.clone(),
            proxy_id: proxy_id.to_string(),
            alt_target: "127.0.0.1:8082".to_string(),
        };

        if let Err(e) = gwnode_queries::save_gateway_node(&gwnode) {
            error!("Failed to save test gwnode: {}", e);
        }

        gwnode
    }

    fn create_test_gateway(gwnode_id: &str) -> Gateway {
        let id = Uuid::new_v4().to_string();
        let gateway = Gateway {
            id: id.clone(),
            gwnode_id: gwnode_id.to_string(),
            pattern: "/api/*".to_string(),
            target: "/".to_string(),
            priority: 10,
        };

        if let Err(e) = gateway_queries::save_gateway(&gateway) {
            error!("Failed to save test gateway: {}", e);
        }

        gateway
    }
}
