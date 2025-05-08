use crate::api::settings::{gateway_queries, gwnode_queries, proxy_queries, proxydomain_queries};
use crate::module::database::{get_connection, DatabaseError};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QGatewayNode {
    pub priority: u8,               // from gateway_node table set to 0 since we dont use it
    pub addr_listen: String,        // from proxy table
    pub addr_bind: String,          // from proxy table (proxy.addr_target)
    pub tls: Vec<QGatewayNodeSNI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QGatewayNodeSNI {
    pub tls: bool,               // from proxy_domain table associated with the proxy used in gateway_node
    pub sni: Option<String>,     // from proxy_domain table associated with the proxy used in gateway_node
    pub tls_pem: Option<String>, // from proxy_domain table associated with the proxy used in gateway_node
    pub tls_key: Option<String>, // from proxy_domain table associated with the proxy used in gateway_node
}

/// sync all path
/// 
/// table infomation
/// ```sql
/// CREATE TABLE gateway_nodes (
///   id TEXT PRIMARY KEY,
///   proxy_id TEXT NOT NULL,
///   domain_id TEXT,
///   title TEXT NOT NULL,
///   alt_target TEXT NOT NULL,
///   priority INTEGER NOT NULL DEFAULT 100,
///   FOREIGN KEY (proxy_id) REFERENCES proxies (id),
///   FOREIGN KEY (domain_id) REFERENCES proxy_domains (id)
/// )
/// ```
/// ```sql
/// CREATE TABLE gateways (
///   id TEXT PRIMARY KEY,
///   gwnode_id TEXT NOT NULL,
///   pattern TEXT NOT NULL,
///   target TEXT NOT NULL,
///   priority INTEGER NOT NULL,
///   FOREIGN KEY (gwnode_id) REFERENCES gateway_nodes (id)
/// )
/// ```
/// ```sql
/// CREATE TABLE proxy_domains (
///   id TEXT PRIMARY KEY,
///   proxy_id TEXT NOT NULL,
///   tls BOOLEAN NOT NULL DEFAULT 0,
///   tls_pem TEXT,
///   tls_key TEXT,
///   sni TEXT
/// )
///```
/// ```sql
/// CREATE TABLE proxies (
///   id TEXT PRIMARY KEY,
///   title TEXT NOT NULL,
///   addr_listen TEXT NOT NULL,
///   addr_target TEXT NOT NULL,
///   high_speed BOOLEAN NOT NULL DEFAULT 0,
///   high_speed_addr TEXT
/// )
/// ```
/// 
// sync all nodes
pub fn get_all_gateway_nodes() -> Result<Vec<QGatewayNode>, DatabaseError> {
    let db = get_connection()?;

    // Ensure all required tables exist before querying
    gateway_queries::ensure_gateways_table()?;
    gwnode_queries::ensure_gateway_nodes_table()?;
    proxy_queries::ensure_proxies_table()?;
    proxydomain_queries::ensure_proxy_domains_table()?;

    // Get all unique listening addresses with their target (bind) addresses
    let addr_query = "
        SELECT DISTINCT 
            p.addr_listen,
            p.addr_target
        FROM 
            gateway_nodes gn
        JOIN 
            proxies p ON gn.proxy_id = p.id
    ";

    let listening_addresses = db.query(addr_query, [], |row| {
        Ok((
            row.get::<_, String>(0)?, // addr_listen
            row.get::<_, String>(1)?  // addr_target (addr_bind)
        ))
    })?;

    let mut gateway_nodes = Vec::new();
    
    // For each unique listening address
    for (addr_listen, addr_bind) in listening_addresses {
        // Find all gateway nodes using this listening address
        let nodes_query = "
            SELECT 
                gn.id AS node_id
            FROM 
                gateway_nodes gn
            JOIN 
                proxies p ON gn.proxy_id = p.id
            WHERE 
                p.addr_listen = ?
        ";

        let node_ids = db.query(nodes_query, [&addr_listen], |row| {
            row.get::<_, String>(0)
        })?;

        // Collect all TLS configurations for all nodes with this listening address
        let mut tls_configs = Vec::new();
        let mut seen_snis = std::collections::HashSet::new();

        for node_id in node_ids {
            let tls_query = "
                SELECT 
                    pd.tls,
                    pd.sni,
                    pd.tls_pem,
                    pd.tls_key
                FROM 
                    proxy_domains pd
                JOIN 
                    gateway_nodes gn ON pd.id = gn.domain_id
                WHERE 
                    gn.id = ?
                UNION
                SELECT 
                    pd.tls,
                    pd.sni,
                    pd.tls_pem,
                    pd.tls_key
                FROM 
                    proxy_domains pd
                JOIN 
                    gateway_nodes gn ON pd.proxy_id = gn.proxy_id
                WHERE 
                    gn.id = ? 
                    AND (gn.domain_id IS NULL OR pd.id != gn.domain_id)
            ";

            let node_tls_configs = db.query(tls_query, [&node_id, &node_id], |row| {
                Ok(QGatewayNodeSNI {
                    tls: row.get(0)?,
                    sni: row.get::<_, Option<String>>(1)?,
                    tls_pem: row.get(2)?,
                    tls_key: row.get(3)?,
                })
            })?;

            // Add only unique TLS configurations (based on SNI)
            for config in node_tls_configs {
                let sni_key = config.sni.clone().unwrap_or_default();
                if !seen_snis.contains(&sni_key) {
                    seen_snis.insert(sni_key);
                    tls_configs.push(config);
                }
            }
        }

        // Create a single gateway node for this listening address with combined TLS configs
        gateway_nodes.push(QGatewayNode {
            priority: 0,  // set to 0 as specified
            addr_listen,
            addr_bind,    // Added addr_bind from proxy.addr_target
            tls: tls_configs,
        });
    }

    Ok(gateway_nodes)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QGatewayPath {
    pub priority: u8,        // from gateway table
    pub sni: Option<String>, // from proxy_domain table
    pub addr_bind: String, // from proxy table
    pub addr_target: String, // from gateway node table
    pub path_listen: String, // from gateway table
    pub path_target: String, // from gateway table
}
/// sync all path
/// 
/// table infomation
/// ```sql
/// CREATE TABLE gateway_nodes (
///   id TEXT PRIMARY KEY,
///   proxy_id TEXT NOT NULL,
///   domain_id TEXT,
///   title TEXT NOT NULL,
///   alt_target TEXT NOT NULL,
///   priority INTEGER NOT NULL DEFAULT 100,
///   FOREIGN KEY (proxy_id) REFERENCES proxies (id),
///   FOREIGN KEY (domain_id) REFERENCES proxy_domains (id)
/// )
/// ```
/// ```sql
/// CREATE TABLE gateways (
///   id TEXT PRIMARY KEY,
///   gwnode_id TEXT NOT NULL,
///   pattern TEXT NOT NULL,
///   target TEXT NOT NULL,
///   priority INTEGER NOT NULL,
///   FOREIGN KEY (gwnode_id) REFERENCES gateway_nodes (id)
/// )
/// ```
/// ```sql
/// CREATE TABLE proxy_domains (
///   id TEXT PRIMARY KEY,
///   proxy_id TEXT NOT NULL,
///   tls BOOLEAN NOT NULL DEFAULT 0,
///   tls_pem TEXT,
///   tls_key TEXT,
///   sni TEXT
/// )
///```
/// ```sql
/// CREATE TABLE proxies (
///   id TEXT PRIMARY KEY,
///   title TEXT NOT NULL,
///   addr_listen TEXT NOT NULL,
///   addr_target TEXT NOT NULL,
///   high_speed BOOLEAN NOT NULL DEFAULT 0,
///   high_speed_addr TEXT
/// )
/// ```
/// 
pub fn get_all_gateway_paths() -> Result<Vec<QGatewayPath>, DatabaseError> {
    let db = get_connection()?;

    // Ensure all required tables exist before querying
    gateway_queries::ensure_gateways_table()?;
    gwnode_queries::ensure_gateway_nodes_table()?;
    proxy_queries::ensure_proxies_table()?;
    proxydomain_queries::ensure_proxy_domains_table()?;

    let query = "SELECT 
        g.priority,
        pd.sni,
        p.addr_target AS addr_bind,
        gn.alt_target AS addr_target,
        g.pattern AS path_listen,
        g.target AS path_target
    FROM gateways g
    JOIN gateway_nodes gn ON g.gwnode_id = gn.id
    JOIN proxies p ON gn.proxy_id = p.id
    LEFT JOIN proxy_domains pd ON gn.domain_id = pd.id
    ORDER BY g.priority DESC";

    let rows = db.query(query, [], |row| {
        Ok(QGatewayPath {
            priority: row.get(0)?,
            sni: row.get(1)?,
            addr_bind: row.get(2)?,
            addr_target: row.get(3)?,
            path_listen: row.get(4)?,
            path_target: row.get(5)?,
        })
    })?;
    
    Ok(rows)
}
