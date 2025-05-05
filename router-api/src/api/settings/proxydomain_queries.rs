//! # Proxy Domain Database Operations
//!
//! This module provides database operations for managing proxy domain configurations.
//! It handles creating the database table, querying, inserting, updating, and
//! deleting proxy domain records.

use crate::module::database::{get_connection, DatabaseError};
use super::ProxyDomain;
use uuid::Uuid;

/// Creates the proxy_domains table in the database if it doesn't already exist
///
/// This function ensures that the database schema is properly initialized before
/// any operations are performed. It is automatically called by other functions
/// in this module, so there's usually no need to call it directly.
///
/// # Database Schema
///
/// Creates a table with the following structure:
/// - `id`: TEXT PRIMARY KEY - Unique identifier for the proxy domain
/// - `proxy_id`: TEXT NOT NULL - Reference to the proxy this domain is associated with
/// - `gwnode_id`: TEXT - Reference to an optional gateway node for routing
/// - `tls`: BOOLEAN NOT NULL DEFAULT 0 - Whether TLS is enabled
/// - `tls_pem`: TEXT - PEM certificate content
/// - `tls_key`: TEXT - Private key content
/// - `sni`: TEXT - Server Name Indication value
///
/// Foreign key constraints are established to ensure referential integrity with the
/// proxies and gateway_nodes tables.
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
pub fn ensure_proxy_domains_table() -> Result<(), DatabaseError> {
    let db = get_connection()?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS proxy_domains (
            id TEXT PRIMARY KEY,
            proxy_id TEXT NOT NULL,
            gwnode_id TEXT NOT NULL,
            tls BOOLEAN NOT NULL DEFAULT 0,
            tls_pem TEXT,
            tls_key TEXT,
            sni TEXT,
            FOREIGN KEY(proxy_id) REFERENCES proxies(id),
            FOREIGN KEY(gwnode_id) REFERENCES gateway_nodes(id)
        )",
        [],
    )?;

    Ok(())
}

/// Generates a unique ID for a new proxy domain
///
/// This is a utility function that generates a UUID v4 string to use
/// as the identifier for a new proxy domain record.
///
/// # Returns
///
/// A string containing a random UUID v4 value
pub fn generate_proxy_domain_id() -> String {
    Uuid::new_v4().to_string()
}

/// Retrieves all proxy domain configurations from the database
///
/// This function fetches all proxy domain records from the database and converts
/// them into `ProxyDomain` structures. It automatically ensures the database table
/// exists before performing the query.
///
/// # Returns
///
/// * `Ok(Vec<ProxyDomain>)` - A vector containing all proxy domain configurations
/// * `Err(DatabaseError)` - If there was an error retrieving the proxy domains
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL query could not be executed
/// - There was an error mapping the database rows to `ProxyDomain` structures
pub fn get_all_proxy_domains() -> Result<Vec<ProxyDomain>, DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_proxy_domains_table()?;

    // Query all proxy domains
    let domains = db.query(
        "SELECT id, proxy_id, gwnode_id, tls, tls_pem, tls_key, sni FROM proxy_domains",
        [],
        |row| {
            Ok(ProxyDomain {
                id: row.get(0)?,
                proxy_id: row.get(1)?,
                gwnode_id: row.get(2)?,
                tls: row.get(3)?,
                tls_pem: match row.get::<_, String>(4) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                tls_key: match row.get::<_, String>(5) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                sni: match row.get::<_, String>(6) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
            })
        },
    )?;

    Ok(domains)
}

/// Retrieves a specific proxy domain configuration by its ID
///
/// This function fetches a single proxy domain record from the database based on
/// the provided ID. It automatically ensures the database table exists before
/// performing the query.
///
/// # Parameters
///
/// * `id` - The unique identifier of the proxy domain to retrieve
///
/// # Returns
///
/// * `Ok(Some(ProxyDomain))` - If the proxy domain with the specified ID was found
/// * `Ok(None)` - If no proxy domain with the specified ID exists
/// * `Err(DatabaseError)` - If there was an error retrieving the proxy domain
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL query could not be executed
/// - There was an error mapping the database row to a `ProxyDomain` structure
pub fn get_proxy_domain_by_id(id: &str) -> Result<Option<ProxyDomain>, DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_proxy_domains_table()?;

    // Query the proxy domain by ID
    let domain = db.query_one(
        "SELECT id, proxy_id, gwnode_id, tls, tls_pem, tls_key, sni FROM proxy_domains WHERE id = ?1",
        [id],
        |row| {
            Ok(ProxyDomain {
                id: row.get(0)?,
                proxy_id: row.get(1)?,
                gwnode_id: row.get(2)?,
                tls: row.get(3)?,
                tls_pem: match row.get::<_, String>(4) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                tls_key: match row.get::<_, String>(5) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                sni: match row.get::<_, String>(6) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
            })
        },
    )?;

    Ok(domain)
}

/// Retrieves all proxy domains associated with a specific proxy
///
/// This function fetches all proxy domain records from the database that are
/// associated with the given proxy ID. It automatically ensures the database table
/// exists before performing the query.
///
/// # Parameters
///
/// * `proxy_id` - The ID of the proxy to find associated domains for
///
/// # Returns
///
/// * `Ok(Vec<ProxyDomain>)` - A vector containing all matching proxy domain configurations
/// * `Err(DatabaseError)` - If there was an error retrieving the proxy domains
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL query could not be executed
/// - There was an error mapping the database rows to `ProxyDomain` structures
pub fn get_proxy_domains_by_proxy_id(proxy_id: &str) -> Result<Vec<ProxyDomain>, DatabaseError> {
    let db = get_connection()?;
    
    // Ensure the table exists
    ensure_proxy_domains_table()?;
    
    // Query proxy domains by proxy ID
    let domains = db.query(
        "SELECT id, proxy_id, gwnode_id, tls, tls_pem, tls_key, sni FROM proxy_domains WHERE proxy_id = ?1",
        [proxy_id],
        |row| {
            Ok(ProxyDomain {
                id: row.get(0)?,
                proxy_id: row.get(1)?,
                gwnode_id: row.get(2)?,
                tls: row.get(3)?,
                tls_pem: match row.get::<_, String>(4) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                tls_key: match row.get::<_, String>(5) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
                sni: match row.get::<_, String>(6) {
                    Ok(s) if s == "\u{0000}" => None,
                    Ok(s) => Some(s),
                    Err(_) => None,
                },
            })
        },
    )?;
    
    Ok(domains)
}

/// Saves a proxy domain configuration to the database
///
/// This function inserts a new proxy domain record or updates an existing one if a domain
/// with the same ID already exists. It automatically ensures the database table
/// exists before performing the operation.
///
/// # Parameters
///
/// * `domain` - The proxy domain configuration to save
///
/// # Returns
///
/// * `Ok(())` - If the proxy domain was successfully saved
/// * `Err(DatabaseError)` - If there was an error saving the proxy domain
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL statement could not be executed
/// - The foreign key constraint is violated (if the referenced proxy or gateway node does not exist)
pub fn save_proxy_domain(domain: &ProxyDomain) -> Result<(), DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_proxy_domains_table()?;

    // Insert or replace the proxy domain
    db.execute(
        "INSERT OR REPLACE INTO proxy_domains (id, proxy_id, gwnode_id, tls, tls_pem, tls_key, sni) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        [
            &domain.id,
            &domain.proxy_id,
            &domain.gwnode_id,
            &(if domain.tls { "1" } else { "0" }).to_string(),
            &domain.tls_pem.clone().unwrap_or("\u{0000}".to_string()),
            &domain.tls_key.clone().unwrap_or("\u{0000}".to_string()),
            &domain.sni.clone().unwrap_or("\u{0000}".to_string()),
        ],
    )?;

    Ok(())
}

/// Deletes a proxy domain configuration from the database by its ID
///
/// This function removes a proxy domain record from the database based on its ID.
/// It returns a boolean indicating whether a record was actually deleted.
///
/// # Parameters
///
/// * `id` - The unique identifier of the proxy domain to delete
///
/// # Returns
///
/// * `Ok(true)` - If the proxy domain was found and deleted
/// * `Ok(false)` - If no proxy domain with the specified ID exists
/// * `Err(DatabaseError)` - If there was an error deleting the proxy domain
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL statement could not be executed
pub fn delete_proxy_domain_by_id(id: &str) -> Result<bool, DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_proxy_domains_table()?;

    // Delete the proxy domain
    let affected_rows = db.execute("DELETE FROM proxy_domains WHERE id = ?1", [id])?;

    Ok(affected_rows > 0)
}

/// Deletes all proxy domains associated with a specific proxy
///
/// This function removes all proxy domain records from the database that are
/// associated with the given proxy ID. It returns the number of records deleted.
///
/// # Parameters
///
/// * `proxy_id` - The ID of the proxy whose domains should be deleted
///
/// # Returns
///
/// * `Ok(usize)` - The number of proxy domains that were deleted
/// * `Err(DatabaseError)` - If there was an error deleting the proxy domains
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL statement could not be executed
pub fn delete_proxy_domains_by_proxy_id(proxy_id: &str) -> Result<usize, DatabaseError> {
    let db = get_connection()?;
    
    // Ensure the table exists
    ensure_proxy_domains_table()?;
    
    // Delete all proxy domains associated with this proxy
    let affected_rows = db.execute(
        "DELETE FROM proxy_domains WHERE proxy_id = ?1",
        [proxy_id],
    )?;
    
    Ok(affected_rows)
}
