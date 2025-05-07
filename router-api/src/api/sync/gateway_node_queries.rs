use super::gateway_node::GatewayNode;
use crate::api::settings::{gateway_queries, gwnode_queries, proxy_queries};
use crate::module::database::{get_connection, DatabaseError};
use log::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QGatewayPath {
    pub priority: u8,
    pub sni: Option<String>,
    pub addr_listen: String,
    pub addr_target: String,
    pub path_listen: String,
    pub path_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QGatewayNode {
    pub priority: u8,
    pub addr_listen: String,
    pub tls: Vec<QGatewayNodeSNI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QGatewayNodeSNI {
    pub tls: bool,
    pub sni: Option<String>,
    pub tls_pem: Option<String>,
    pub tls_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QDomainGateway {}

pub fn get_all_gateway_nodes() -> Result<Vec<QGatewayNode>, DatabaseError> {
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
    let mut gateway_nodes = db.query(
        "SELECT 
            p.addr_listen
        FROM
            proxies p
        WHERE 
            p.high_speed = 0
        GROUP BY 
            p.addr_listen",
        [],
        |row| {
            let data = QGatewayNode {
                priority: row.get(0)?,
                addr_listen: row.get(1)?,
                tls: vec![],
            };
            log::debug!("GatewayPath: {:#?}", data.clone());
            Ok(data)
        },
    )?;

    Ok(gateway_nodes)
}

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
pub fn get_all_gateway_paths() -> Result<Vec<QGatewayPath>, DatabaseError> {
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
            gn.sni as sni,
            p.addr_target AS addr_listen, 
            CASE 
                WHEN d.id IS NOT NULL THEN d.target 
                ELSE gn.alt_target 
            END AS addr_target,
            g.pattern AS path_listen, 
            g.target AS path_target
        FROM 
            gateways g
        JOIN 
            gateway_nodes gn ON g.gwnode_id = gn.id
        JOIN 
            proxies p ON gn.proxy_id = p.id
        LEFT JOIN 
            domains d ON gn.domain_id = d.id
        WHERE 
            p.high_speed = 0
        ORDER BY 
            g.priority DESC",
        [],
        |row| {
            let data = QGatewayPath {
                priority: row.get(0)?,
                sni: row.get::<_, Option<String>>(1)?,
                addr_listen: row.get(2)?,
                addr_target: row.get(3)?,
                path_listen: row.get(4)?,
                path_target: row.get(5)?,
            };
            log::debug!("GatewayNode: {:#?}", data.clone());
            Ok(data)
        },
    )?;

    info!("Final query returned {} rows", gateway_nodes.len());

    Ok(gateway_nodes)
}
