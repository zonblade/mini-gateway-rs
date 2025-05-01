//! # Proxy Database Operations
//!
//! This module provides database operations for managing proxy configurations.
//! It handles creating the database table, querying, inserting, updating, and
//! deleting proxy records.

use super::Proxy;
use crate::module::database::{get_connection, DatabaseError};
use rand::Rng;
use std::net::TcpListener;

/// Creates the proxies table in the database if it doesn't already exist
///
/// This function ensures that the database schema is properly initialized before
/// any operations are performed. It is automatically called by other functions
/// in this module, so there's usually no need to call it directly.
///
/// # Database Schema
///
/// Creates a table with the following structure:
/// - `id`: TEXT PRIMARY KEY - Unique identifier for the proxy
/// - `title`: TEXT NOT NULL - Human-readable name for the proxy
/// - `addr_listen`: TEXT NOT NULL - Address where the proxy listens for connections
/// - `addr_target`: TEXT NOT NULL - Target address where requests are forwarded
/// - `tls`: BOOLEAN NOT NULL DEFAULT 0 - Whether TLS is enabled
/// - `tls_pem`: TEXT - PEM certificate content
/// - `tls_key`: TEXT - Private key content
/// - `tls_autron`: BOOLEAN NOT NULL DEFAULT 0 - Whether automatic TLS is enabled
/// - `sni`: TEXT - Server Name Indication value
///
/// # Returns
///
/// * `Ok(())` if the table exists or was created successfully
/// * `Err(DatabaseError)` if there was an error creating the table
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The SQL statement to create the table could not be executed
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
            sni TEXT,
            high_speed BOOLEAN NOT NULL DEFAULT 0,
            high_speed_addr TEXT
        )",
        [],
    )?;

    Ok(())
}

/// Retrieves all proxy configurations from the database
///
/// This function fetches all proxy records from the database and converts
/// them into `Proxy` structures. It automatically ensures the database table
/// exists before performing the query.
///
/// # Returns
///
/// * `Ok(Vec<Proxy>)` - A vector containing all proxy configurations
/// * `Err(DatabaseError)` - If there was an error retrieving the proxies
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL query could not be executed
/// - There was an error mapping the database rows to `Proxy` structures
///
/// # Example
///
/// ```
/// use router_api::api::settings::proxy_queries;
///
/// match proxy_queries::get_all_proxies() {
///     Ok(proxies) => {
///         println!("Found {} proxies", proxies.len());
///         for proxy in proxies {
///             println!("Proxy: {} ({})", proxy.title, proxy.addr_listen);
///         }
///     },
///     Err(err) => eprintln!("Error retrieving proxies: {}", err),
/// }
/// ```
pub fn get_all_proxies() -> Result<Vec<Proxy>, DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_proxies_table()?;

    // Query all proxies
    let proxies = db.query(
        "SELECT id, title, addr_listen, addr_target, tls, tls_pem, tls_key, tls_autron, sni, high_speed, high_speed_addr FROM proxies",
        [],
        |row| {
            Ok(Proxy {
                id: row.get(0)?,
                title: row.get(1)?,
                addr_listen: row.get(2)?,
                addr_target: row.get(3)?,
                tls: row.get(4)?,
                tls_pem: match row.get::<_, String>(5) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                tls_key: match row.get::<_, String>(6) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                tls_autron: row.get(7)?,
                sni: match row.get::<_, String>(8) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                high_speed: row.get(9)?,
                high_speed_addr: match row.get::<_, String>(10) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
            })
        },
    )?;

    Ok(proxies)
}

/// Retrieves a specific proxy configuration by its ID
///
/// This function fetches a single proxy record from the database based on
/// the provided ID. It automatically ensures the database table exists before
/// performing the query.
///
/// # Parameters
///
/// * `id` - The unique identifier of the proxy to retrieve
///
/// # Returns
///
/// * `Ok(Some(Proxy))` - If the proxy with the specified ID was found
/// * `Ok(None)` - If no proxy with the specified ID exists
/// * `Err(DatabaseError)` - If there was an error retrieving the proxy
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL query could not be executed
/// - There was an error mapping the database row to a `Proxy` structure
///
/// # Example
///
/// ```
/// use router_api::api::settings::proxy_queries;
///
/// let proxy_id = "550e8400-e29b-41d4-a716-446655440000";
/// match proxy_queries::get_proxy_by_id(proxy_id) {
///     Ok(Some(proxy)) => println!("Found proxy: {} ({})", proxy.title, proxy.addr_listen),
///     Ok(None) => println!("No proxy found with ID: {}", proxy_id),
///     Err(err) => eprintln!("Error retrieving proxy: {}", err),
/// }
/// ```
pub fn get_proxy_by_id(id: &str) -> Result<Option<Proxy>, DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_proxies_table()?;

    // Query the proxy by ID
    let proxy = db.query_one(
        "SELECT id, title, addr_listen, addr_target, tls, tls_pem, tls_key, tls_autron, sni, high_speed, high_speed_addr FROM proxies WHERE id = ?1",
        [id],
        |row| {
            Ok(Proxy {
                id: row.get(0)?,
                title: row.get(1)?,
                addr_listen: row.get(2)?,
                addr_target: row.get(3)?,
                tls: row.get(4)?,
                tls_pem: match row.get::<_, String>(5) {
                    Ok(s) if s == "\\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                tls_key: match row.get::<_, String>(6) {
                    Ok(s) if s == "\\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                tls_autron: row.get(7)?,
                sni: match row.get::<_, String>(8) {
                    Ok(s) if s == "\\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                high_speed: row.get(9)?,
                high_speed_addr: match row.get::<_, String>(10) {
                    Ok(s) if s == "\\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
            })
        },
    )?;

    Ok(proxy)
}

/// Saves a proxy configuration to the database
///
/// This function inserts a new proxy record or updates an existing one if a proxy
/// with the same ID already exists. It automatically ensures the database table
/// exists before performing the operation.
///
/// # Parameters
///
/// * `proxy` - The proxy configuration to save
///
/// # Returns
///
/// * `Ok(())` - If the proxy was successfully saved
/// * `Err(DatabaseError)` - If there was an error saving the proxy
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL statement could not be executed
///
/// # Security Notes
///
/// This function uses parameterized SQL queries to prevent SQL injection attacks.
/// The `tls_pem`, `tls_key`, and `sni` fields are stored as `\u{0000}` (NULL character)
/// when they are `None` to maintain consistent storage.
///
/// # Example
///
/// ```
/// use router_api::api::settings::{Proxy, proxy_queries};
///
/// let proxy = Proxy {
///     id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
///     title: "Web Server".to_string(),
///     addr_listen: "0.0.0.0:80".to_string(),
///     addr_target: "127.0.0.1:8080".to_string(),
///     tls: false,
///     tls_pem: None,
///     tls_key: None,
///     tls_autron: false,
///     sni: None,
/// };
///
/// match proxy_queries::save_proxy(&proxy) {
///     Ok(()) => println!("Proxy saved successfully"),
///     Err(err) => eprintln!("Error saving proxy: {}", err),
/// }
/// ```
pub fn save_proxy(proxy: &Proxy) -> Result<(), DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_proxies_table()?;

    // Insert or replace the proxy
    db.execute(
        "INSERT OR REPLACE INTO proxies (id, title, addr_listen, addr_target, tls, tls_pem, tls_key, tls_autron, sni, high_speed, high_speed_addr) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        [
            &proxy.id,
            &proxy.title,
            &proxy.addr_listen,
            &proxy.addr_target,
            &(if proxy.tls { "1" } else { "0" }).to_string(),
            &proxy.tls_pem.clone().unwrap_or("\u{0000}".to_string()),
            &proxy.tls_key.clone().unwrap_or("\u{0000}".to_string()),
            &(if proxy.tls_autron { "1" } else { "0" }).to_string(),
            &proxy.sni.clone().unwrap_or("\u{0000}".to_string()),
            &(if proxy.high_speed { "1" } else { "0" }).to_string(),
            &proxy.high_speed_addr.clone().unwrap_or("\u{0000}".to_string()),
        ],
    )?;

    Ok(())
}

/// Deletes a proxy configuration from the database by its ID
///
/// This function removes a proxy record from the database based on its ID.
/// It returns a boolean indicating whether a record was actually deleted.
///
/// # Parameters
///
/// * `id` - The unique identifier of the proxy to delete
///
/// # Returns
///
/// * `Ok(true)` - If the proxy was found and deleted
/// * `Ok(false)` - If no proxy with the specified ID exists
/// * `Err(DatabaseError)` - If there was an error deleting the proxy
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The SQL statement could not be executed
///
/// # Example
///
/// ```
/// use router_api::api::settings::proxy_queries;
///
/// let proxy_id = "550e8400-e29b-41d4-a716-446655440000";
/// match proxy_queries::delete_proxy_by_id(proxy_id) {
///     Ok(true) => println!("Proxy deleted successfully"),
///     Ok(false) => println!("No proxy found with ID: {}", proxy_id),
///     Err(err) => eprintln!("Error deleting proxy: {}", err),
/// }
/// ```
pub fn delete_proxy_by_id(id: &str) -> Result<bool, DatabaseError> {
    let db = get_connection()?;

    // Delete the proxy
    let affected_rows = db.execute("DELETE FROM proxies WHERE id = ?1", [id])?;

    Ok(affected_rows > 0)
}

/// Generates a target address with a random available port
///
/// This function creates a localhost address (127.0.0.1) with a randomly selected
/// port between 40000 and 49000 that is currently available on the system. It tries
/// up to 100 different ports before giving up.
///
/// The generated address is typically used as the `addr_target` for new proxy configurations.
///
/// # Returns
///
/// * `Ok(String)` - A string in the format "127.0.0.1:PORT" with an available port
/// * `Err(String)` - An error message if no available port could be found
///
/// # Error Conditions
///
/// This function will return an error if:
/// - It fails to find an available port after 100 attempts
///
/// # Implementation Details
///
/// Port availability is determined by attempting to bind a TCP listener to the port.
/// If binding is successful, the port is considered available. This approach ensures
/// that the port is genuinely available at the time of checking.
///
/// # Example
///
/// ```
/// use router_api::api::settings::proxy_queries;
///
/// match proxy_queries::generate_target_address() {
///     Ok(addr) => println!("Generated target address: {}", addr),
///     Err(err) => eprintln!("Error generating address: {}", err),
/// }
/// ```
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
            }
            Err(_) => {
                // Port is in use, try another one
                continue;
            }
        }
    }

    Err("Failed to find an available port after 100 attempts".to_string())
}

/// Checks if there are multiple proxies using the same listen address
///
/// This function counts how many proxies are configured to listen on a given address.
/// It's used to enforce constraints for high-speed mode, which requires that each
/// listen address is unique across all proxies.
///
/// # Arguments
///
/// * `listen_addr` - The listen address to check (e.g., "0.0.0.0:8080")
/// * `exclude_id` - Optional proxy ID to exclude from the check (used when updating a proxy)
///
/// # Returns
///
/// * `Ok(true)` - If there are multiple proxies with the same listen address
/// * `Ok(false)` - If there is only one or zero proxies with the listen address
/// * `Err(DatabaseError)` - If there was an error performing the check
///
/// # Example
///
/// ```
/// match has_duplicate_listen_address("0.0.0.0:443", Some("proxy-1")) {
///     Ok(true) => println!("Cannot enable high-speed mode for this address"),
///     Ok(false) => println!("High-speed mode can be enabled"),
///     Err(e) => eprintln!("Database error: {}", e),
/// }
/// ```
pub fn has_duplicate_listen_address(listen_addr: &str, exclude_id: Option<&str>) -> Result<bool, DatabaseError> {
    ensure_proxies_table()?;
    let db = get_connection()?;
    
    let count: i64;
    
    if let Some(id) = exclude_id {
        // Count proxies with the same listen address, excluding the specified proxy
        let sql = "SELECT COUNT(*) FROM proxies WHERE addr_listen = ? AND id != ?";
        count = db.query_one(sql, [listen_addr, id], |row| row.get(0))?.unwrap_or(0);
    } else {
        // Count all proxies with the given listen address
        let sql = "SELECT COUNT(*) FROM proxies WHERE addr_listen = ?";
        count = db.query_one(sql, [listen_addr], |row| row.get(0))?.unwrap_or(0);
    }
    
    Ok(count > 0)
}
