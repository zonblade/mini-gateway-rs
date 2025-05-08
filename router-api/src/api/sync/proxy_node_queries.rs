use serde::{Deserialize, Serialize};

use crate::{api::settings::proxydomain_queries, module::database::{get_connection, DatabaseError}};
use super::proxy_node::ProxyNode;
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
    
    // let proxy_nodes = db.query(
    //     "SELECT 
    //         id,
    //         title,
    //         addr_listen,
    //         addr_target,
    //         high_speed,
    //         high_speed_addr
    //     FROM 
    //         proxies",
    //     [],
    //     |row| {
            
    //         let data = ProxyNode {
    //             id: row.get(0)?,
    //             domains: vec![],
    //             addr_listen: row.get(2)?,
    //             addr_target: row.get(3)?,
    //             high_speed: row.get(4)?,
    //             high_speed_addr: match row.get::<_, String>(5) {
    //                 Ok(s) if s == "\u{0000}" => None,
    //                 Ok(s) => Some(s),
    //                 Err(_) => None,
    //             },
    //         };
    //         log::debug!("Proxy Node: {:#?}", data.clone());
    //         Ok(data)
    //     },
    // )?;
    
    // Ok(proxy_nodes)
    Ok(vec![])
}
