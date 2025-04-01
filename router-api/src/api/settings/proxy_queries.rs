use crate::module::database::{get_connection, DatabaseError};
use super::Proxy;
use std::net::{TcpListener, SocketAddr};
use rand::Rng;

/// Create the proxies table if it doesn't exist
pub fn ensure_proxies_table() -> Result<(), DatabaseError> {
    let db = get_connection()?;
    
    db.execute(
        "CREATE TABLE IF NOT EXISTS proxies (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            addr_listen TEXT NOT NULL,
            addr_target TEXT NOT NULL,
            tls BOOLEAN NOT NULL DEFAULT 0,
            tls_pem TEXT,
            tls_key TEXT,
            tls_autron BOOLEAN NOT NULL DEFAULT 0,
            sni TEXT
        )",
        [],
    )?;
    
    Ok(())
}

/// Get all proxies from the database
pub fn get_all_proxies() -> Result<Vec<Proxy>, DatabaseError> {
    let db = get_connection()?;
    
    // Ensure the table exists
    ensure_proxies_table()?;
    
    // Query all proxies
    let proxies = db.query(
        "SELECT id, title, addr_listen, addr_target, tls, tls_pem, tls_key, tls_autron, sni FROM proxies",
        [],
        |row| {
            Ok(Proxy {
                id: row.get(0)?,
                title: row.get(1)?,
                addr_listen: row.get(2)?,
                addr_target: row.get(3)?,
                tls: row.get(4)?,
                tls_pem: row.get(5)?,
                tls_key: row.get(6)?,
                tls_autron: row.get(7)?,
                sni: row.get(8)?,
            })
        },
    )?;
    
    Ok(proxies)
}

/// Get a proxy by ID from the database
pub fn get_proxy_by_id(id: &str) -> Result<Option<Proxy>, DatabaseError> {
    let db = get_connection()?;
    
    // Ensure the table exists
    ensure_proxies_table()?;
    
    // Query the proxy by ID
    let proxy = db.query_one(
        "SELECT id, title, addr_listen, addr_target, tls, tls_pem, tls_key, tls_autron, sni FROM proxies WHERE id = ?1",
        [id],
        |row| {
            Ok(Proxy {
                id: row.get(0)?,
                title: row.get(1)?,
                addr_listen: row.get(2)?,
                addr_target: row.get(3)?,
                tls: row.get(4)?,
                tls_pem: row.get(5)?,
                tls_key: row.get(6)?,
                tls_autron: row.get(7)?,
                sni: row.get(8)?,
            })
        },
    )?;
    
    Ok(proxy)
}

/// Save a proxy to the database
pub fn save_proxy(proxy: &Proxy) -> Result<(), DatabaseError> {
    let db = get_connection()?;
    
    // Ensure the table exists
    ensure_proxies_table()?;
    
    // Insert or replace the proxy
    db.execute(
        "INSERT OR REPLACE INTO proxies (id, title, addr_listen, addr_target, tls, tls_pem, tls_key, tls_autron, sni) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        [
            &proxy.id,
            &proxy.title,
            &proxy.addr_listen,
            &proxy.addr_target,
            &proxy.tls.to_string(),
            &proxy.tls_pem.clone().unwrap_or("\u{0000}".to_string()),
            &proxy.tls_key.clone().unwrap_or("\u{0000}".to_string()),
            &proxy.tls_autron.to_string(),
            &proxy.sni.clone().unwrap_or("\u{0000}".to_string()),
        ],
    )?;
    
    Ok(())
}

/// Delete a proxy by ID from the database
pub fn delete_proxy_by_id(id: &str) -> Result<bool, DatabaseError> {
    let db = get_connection()?;
    
    // Delete the proxy
    let affected_rows = db.execute(
        "DELETE FROM proxies WHERE id = ?1",
        [id],
    )?;
    
    Ok(affected_rows > 0)
}

/// Generate a target address with a random available port between 40000 and 49000
pub fn generate_target_address() -> Result<String, String> {
    let mut rng = rand::thread_rng();
    
    // Try up to 100 random ports to find an available one
    for _ in 0..100 {
        let port = rng.gen_range(40000..=49000);
        let addr = format!("127.0.0.1:{}", port);
        
        // Check if the port is available by trying to bind to it
        match TcpListener::bind(&addr) {
            Ok(_) => {
                // Successfully bound to the port, so it's available
                return Ok(addr);
            },
            Err(_) => {
                // Port is in use, try another one
                continue;
            }
        }
    }
    
    Err("Failed to find an available port after 100 attempts".to_string())
}