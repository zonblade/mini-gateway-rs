use serde::{Deserialize, Serialize};

use crate::{api::settings::{gwnode_queries, proxydomain_queries}, module::database::{get_connection, DatabaseError}};
use crate::api::settings::proxy_queries;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QProxyNode {
    pub tls: bool,                      // from proxy table
    pub sni: Option<String>,            // from proxy table
    pub tls_pem: Option<String>,        // from proxy table
    pub tls_key: Option<String>,        // from proxy table
    pub addr_listen: String,            // from proxy table
    pub addr_target: String,            // from proxy table
    pub high_speed: bool,               // always true
    pub high_speed_addr: Option<String>,// always Some
    pub buffer_size: Option<usize>,     // always None, because unused now
    pub timeout_secs: Option<u64>,      // always None, because unused now
    pub adaptive_buffer: bool,          // always false, because unused now
}


/// 
/// table infomation
/// 
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
///   high_speed_addr TEXT,
///   high_speed_gwid TEXT,
/// )
/// ```
pub fn get_all_proxy_nodes() -> Result<Vec<QProxyNode>, DatabaseError> {
    let db = get_connection()?;
    
    // Ensure the proxies table exists before querying it
    proxy_queries::ensure_proxies_table()?;
    proxydomain_queries::ensure_proxy_domains_table()?;
    gwnode_queries::ensure_gateway_nodes_table()?;
    
    // Query to retrieve proxy nodes with TLS information via gateway_nodes
    // Filtering for proxies where high_speed is enabled (true/1)
    let query = "
        SELECT 
            COALESCE(pd.tls, 0) AS tls,
            pd.sni,
            pd.tls_pem,
            pd.tls_key,
            p.addr_listen,
            p.addr_target,
            1 AS high_speed,
            p.high_speed_addr,
            NULL AS buffer_size,
            NULL AS timeout_secs,
            0 AS adaptive_buffer
        FROM 
            proxies p
        LEFT JOIN 
            gateway_nodes gn ON p.high_speed_gwid = gn.id
        LEFT JOIN 
            proxy_domains pd ON gn.domain_id = pd.id
        WHERE
            p.high_speed = 1
    ";
    
    let proxy_nodes = db.query(query, [], |row| {
        Ok(QProxyNode {
            tls: row.get(0)?,
            sni: row.get(1)?,
            tls_pem: row.get(2)?,
            tls_key: row.get(3)?,
            addr_listen: row.get(4)?,
            addr_target: row.get(5)?,
            high_speed: row.get(6)?,
            high_speed_addr: row.get(7)?,
            buffer_size: row.get(8)?,
            timeout_secs: row.get(9)?,
            adaptive_buffer: row.get(10)?,
        })
    })?;
    
    Ok(proxy_nodes)
}
