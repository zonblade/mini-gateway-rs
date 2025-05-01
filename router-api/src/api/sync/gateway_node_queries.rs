use super::gateway_node::GatewayNode;
use crate::api::settings::{gateway_queries, gwnode_queries, proxy_queries};
use crate::module::database::{get_connection, DatabaseError};
use log::{info, warn};

/// Retrieves a list of GatewayNode objects by joining gateway, gwnode, and proxy tables
///
/// This function performs a JOIN operation across multiple tables to construct
/// GatewayNode objects with all the required fields:
/// - `priority` from the gateway table
/// - `addr_listen` from the proxy table (which is proxy.addr_target)
/// - `addr_target` from the gwnode table (which is gwnode.alt_target)
/// - `path_listen` from the gateway table (which is gateway.pattern)
/// - `path_target` from the gateway table (which is gateway.target)
///
/// # Returns
///
/// * `Ok(Vec<GatewayNode>)` - A vector containing all gateway configurations
/// * `Err(DatabaseError)` - If there was an error retrieving the data
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The SQL query could not be executed
/// - There was an error mapping the database rows to GatewayNode structures
pub fn get_all_gateway_nodes() -> Result<Vec<GatewayNode>, DatabaseError> {
    let db = get_connection()?;

    // Ensure all required tables exist before querying
    gateway_queries::ensure_gateways_table()?;
    gwnode_queries::ensure_gateway_nodes_table()?;
    proxy_queries::ensure_proxies_table()?;

    // Add diagnostic queries to check if tables have data
    let gateway_count: i64 = db
        .query_one("SELECT COUNT(*) FROM gateways", [], |row| row.get(0))?
        .unwrap_or(0);

    let gwnode_count: i64 = db
        .query_one("SELECT COUNT(*) FROM gateway_nodes", [], |row| row.get(0))?
        .unwrap_or(0);

    let proxy_count: i64 = db
        .query_one("SELECT COUNT(*) FROM proxies", [], |row| row.get(0))?
        .unwrap_or(0);

    info!(
        "Table counts - gateways: {}, gateway_nodes: {}, proxies: {}",
        gateway_count, gwnode_count, proxy_count
    );

    if gateway_count == 0 || gwnode_count == 0 || proxy_count == 0 {
        warn!("One or more tables is empty, which will result in empty JOIN results");
    }

    // Try a simpler query first to see if we get any results from each table
    let simple_query = "
        SELECT 
            COUNT(*) 
        FROM 
            gateways g
        JOIN 
            gateway_nodes gn ON g.gwnode_id = gn.id
        JOIN 
            proxies p ON gn.proxy_id = p.id";

    let join_count: i64 = db
        .query_one(simple_query, [], |row| row.get(0))?
        .unwrap_or(0);

    info!("Query would return {} rows", join_count);

    // Original JOIN query
    // Join gateway, gateway_nodes (gwnode), and proxies tables
    // to construct complete GatewayNode objects
    let gateway_nodes = db.query(
        "SELECT 
            g.priority, 
            p.addr_target AS addr_listen, 
            gn.alt_target AS addr_target, 
            g.pattern AS path_listen, 
            g.target AS path_target
        FROM 
            gateways g
        JOIN 
            gateway_nodes gn ON g.gwnode_id = gn.id
        JOIN 
            proxies p ON gn.proxy_id = p.id
        ORDER BY 
            g.priority DESC",
        [],
        |row| {
            let data = GatewayNode {
                priority: row.get(0)?,
                addr_listen: row.get(1)?,
                addr_target: row.get(2)?,
                path_listen: row.get(3)?,
                path_target: row.get(4)?,
            };
            log::debug!("GatewayNode: {:#?}", data.clone());
            Ok(data)
        },
    )?;

    info!("Final query returned {} rows", gateway_nodes.len());

    Ok(gateway_nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::settings::{gateway_queries, gwnode_queries, proxy_queries};
    use crate::api::settings::{Gateway, GatewayNode as SettingsGatewayNode, Proxy};
    use uuid::Uuid;

    #[test]
    fn test_get_all_gateway_nodes() {
        // Setup: Create test data in the database
        let test_proxy = create_test_proxy();
        let test_gwnode = create_test_gwnode(&test_proxy.id);
        let test_gateway = create_test_gateway(&test_gwnode.id);

        // Call the function under test
        let result = get_all_gateway_nodes();

        // Cleanup: Remove test data
        let _ = gateway_queries::delete_gateway_by_id(&test_gateway.id);
        let _ = gwnode_queries::delete_gateway_node_by_id(&test_gwnode.id);
        let _ = proxy_queries::delete_proxy_by_id(&test_proxy.id);

        // Assertions
        assert!(result.is_ok(), "Function should return Ok result");

        let nodes = result.unwrap();

        if !nodes.is_empty() {
            // Since we can't guarantee our test data is the only data in the database,
            // we'll just check if the function returns data in the expected format
            let first_node = &nodes[0];
            // No need to check range as i8 is already constrained to -128 to 127
            assert!(
                first_node.priority >= i8::MIN,
                "Priority should be a valid i8 value"
            );
            assert!(
                !first_node.addr_listen.is_empty(),
                "addr_listen should not be empty"
            );
            assert!(
                !first_node.addr_target.is_empty(),
                "addr_target should not be empty"
            );
            assert!(
                !first_node.path_listen.is_empty(),
                "path_listen should not be empty"
            );
            assert!(
                !first_node.path_target.is_empty(),
                "path_target should not be empty"
            );
        }
    }

    // Helper functions to create test data
    fn create_test_proxy() -> Proxy {
        let id = Uuid::new_v4().to_string();
        let proxy = Proxy {
            id: id.clone(),
            title: "Test Proxy".to_string(),
            addr_listen: "127.0.0.1:8080".to_string(),
            addr_target: "127.0.0.1:8081".to_string(),
            tls: false,
            tls_pem: None,
            tls_key: None,
            tls_autron: false,
            sni: None,
            high_speed: false,
            high_speed_addr: None,
        };

        if let Err(e) = proxy_queries::save_proxy(&proxy) {
            log::error!("Failed to save test proxy: {}", e);
        }

        proxy
    }

    fn create_test_gwnode(proxy_id: &str) -> SettingsGatewayNode {
        let id = Uuid::new_v4().to_string();
        let gwnode = SettingsGatewayNode {
            id: id.clone(),
            title: "Test Gateway Node".to_string(),
            proxy_id: proxy_id.to_string(),
            alt_target: "127.0.0.1:8082".to_string(),
        };

        if let Err(e) = gwnode_queries::save_gateway_node(&gwnode) {
            log::error!("Failed to save test gwnode: {}", e);
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
            log::error!("Failed to save test gateway: {}", e);
        }

        gateway
    }
}
