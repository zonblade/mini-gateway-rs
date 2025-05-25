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
/// - `tls`: BOOLEAN NOT NULL DEFAULT 0 - Whether TLS is enabled
/// - `tls_pem`: TEXT - PEM certificate content
/// - `tls_key`: TEXT - Private key content
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
pub fn ensure_proxy_domains_table() -> Result<(), DatabaseError> {
    let db = get_connection()?;
    
    // Define the expected columns
    let expected_columns = ["id", "proxy_id", "tls", "tls_pem", "tls_key", "sni"];
    
    // Check if the table exists with the expected columns and is not corrupted
    if db.table_exists_with_columns("proxy_domains", &expected_columns)? {
        log::debug!("proxy_domains table exists and has expected structure");
        return Ok(());
    }
    
    log::info!("Creating or repairing proxy_domains table");
    
    // Drop the table if it exists but is corrupted or missing columns
    db.execute("DROP TABLE IF EXISTS proxy_domains", [])?;
    
    // Create the table with the full correct structure
    db.execute(
        "CREATE TABLE proxy_domains (
            id TEXT PRIMARY KEY,
            proxy_id TEXT NOT NULL,
            tls BOOLEAN NOT NULL DEFAULT 0,
            tls_pem TEXT,
            tls_key TEXT,
            sni TEXT
        )",
        [],
    )?;
    
    log::info!("Created proxy_domains table with correct structure");
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
#[allow(dead_code)]
pub fn get_all_proxy_domains() -> Result<Vec<ProxyDomain>, DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_proxy_domains_table()?;

    // Query all proxy domains
    let domains = db.query(
        "SELECT id, proxy_id, tls, tls_pem, tls_key, sni FROM proxy_domains",
        [],
        |row| {
            Ok(ProxyDomain {
                id: row.get(0)?,
                proxy_id: row.get(1)?,
                tls: row.get(2)?,
                tls_pem: row.get(3)?,
                tls_key: row.get(4)?,
                sni: row.get(5)?,
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
#[allow(dead_code)]
pub fn get_proxy_domain_by_id(id: &str) -> Result<Option<ProxyDomain>, DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_proxy_domains_table()?;

    // Query the proxy domain by ID
    let domain = db.query_one(
        "SELECT id, proxy_id, tls, tls_pem, tls_key, sni FROM proxy_domains WHERE id = ?1",
        [id],
        |row| {
            Ok(ProxyDomain {
                id: row.get(0)?,
                proxy_id: row.get(1)?,
                tls: row.get(2)?,
                tls_pem: row.get(3)?,
                tls_key: row.get(4)?,
                sni: row.get(5)?,
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
        "SELECT id, proxy_id, tls, tls_pem, tls_key, sni FROM proxy_domains WHERE proxy_id = ?1",
        [proxy_id],
        |row| {
            Ok(ProxyDomain {
                id: row.get(0)?,
                proxy_id: row.get(1)?,
                tls: row.get(2)?,
                tls_pem: row.get(3)?,
                tls_key: row.get(4)?,
                sni: row.get(5)?,
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
/// - The proxy_id is missing or empty (which would violate NOT NULL constraint)
pub fn save_proxy_domain(domain: &ProxyDomain) -> Result<(), DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_proxy_domains_table()?;
    
    // Validate that proxy_id is valid - return more specific error if not present
    let proxy_id = match &domain.proxy_id {
        Some(id) if !id.is_empty() => id.clone(),
        Some(_) => return Err(DatabaseError::from_msg("Proxy ID is empty")),
        None => return Err(DatabaseError::from_msg("Proxy ID is missing (null)"))
    };
    
    // Log the domain data we're trying to save
    log::debug!("Attempting to save domain: id={}, proxy_id={}, sni={:?}", 
               domain.id, proxy_id, domain.sni);
    
    // Insert or replace the proxy domain with validated proxy_id and proper NULL handling
    db.execute(
        "INSERT OR REPLACE INTO proxy_domains (id, proxy_id, tls, tls_pem, tls_key, sni) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            &domain.id,
            &proxy_id,
            &(if domain.tls { 1 } else { 0 }),
            &domain.tls_pem,
            &domain.tls_key,
            &domain.sni,
        ],
    ).map_err(|e| {
        log::error!("Database error when saving domain {}: {}", domain.id, e);
        DatabaseError::from(e)
    })?;

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

/// Deletes all proxy domain configurations from the database
///
/// This function removes all proxy domain records from the database.
/// It should be used with caution as it will remove all domain configurations.
///
/// # Returns
///
/// * `Ok(())` - If all proxy domains were successfully deleted
/// * `Err(DatabaseError)` - If there was an error deleting the proxy domains
pub fn delete_all_proxy_domains() -> Result<(), DatabaseError> {
    let db = get_connection()?;
    db.execute("DELETE FROM proxy_domains", [])?;
    Ok(())
}