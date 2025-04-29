use crate::module::database::{get_connection, DatabaseError};
use super::proxy_node::ProxyNode;
use log::error;
use crate::api::settings::proxy_queries;

/// Retrieves a list of ProxyNode objects from the database
///
/// This function queries the proxies table to construct ProxyNode objects with all required fields:
/// - `tls` - Whether TLS is enabled
/// - `sni` - Server Name Indication value for TLS connections (if any)
/// - `tls_pem` - Path to the TLS certificate PEM file (if any)
/// - `tls_key` - Path to the TLS private key file (if any)
/// - `addr_listen` - Network address this proxy listens on
/// - `addr_target` - Target address to forward traffic to
/// - `priority` - Processing priority
///
/// # Returns
///
/// * `Ok(Vec<ProxyNode>)` - A vector containing all proxy node configurations
/// * `Err(DatabaseError)` - If there was an error retrieving the data
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The SQL query could not be executed
/// - There was an error mapping the database rows to ProxyNode structures
pub fn get_all_proxy_nodes() -> Result<Vec<ProxyNode>, DatabaseError> {
    let db = get_connection()?;
    
    // Ensure the proxies table exists before querying it
    proxy_queries::ensure_proxies_table()?;
    
    let proxy_nodes = db.query(
        "SELECT 
            id,
            title,
            addr_listen,
            addr_target,
            tls,
            tls_pem,
            tls_key,
            tls_autron,
            sni
        FROM 
            proxies",
        [],
        |row| {
            // Handle Option<String> fields properly
            // For tls_pem, tls_key, and sni, ensure empty strings and null characters become None
            let tls_pem: Option<String> = row.get(5)?;
            let tls_key: Option<String> = row.get(6)?;
            let sni: Option<String> = row.get(8)?;
            
            // Convert empty strings or null character strings to None
            let clean_string_option = |opt: Option<String>| -> Option<String> {
                opt.and_then(|s| {
                    if s.is_empty() || s == "\u{0000}" {
                        None
                    } else {
                        Some(s)
                    }
                })
            };
            
            let data = ProxyNode {
                addr_listen: row.get(2)?,
                addr_target: row.get(3)?,
                tls: row.get(4).unwrap_or(false),
                tls_pem: clean_string_option(tls_pem),
                tls_key: clean_string_option(tls_key),
                sni: clean_string_option(sni)
            };
            log::debug!("Proxy Node: {:#?}", data.clone());
            Ok(data)
        },
    )?;
    
    Ok(proxy_nodes)
}
