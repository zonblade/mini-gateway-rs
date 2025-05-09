//! # Gateway Routing Rules Database Operations
//!
//! This module provides database operations for managing gateway routing configurations.
//! Gateways are the actual routing rules that define how incoming requests are matched
//! and forwarded based on patterns and priorities.
//!
//! The module handles creating the database table, querying, inserting, updating, and
//! deleting gateway records, as well as managing the relationship with gateway nodes.

use crate::module::database::{get_connection, DatabaseError};
use super::Gateway;
use uuid::Uuid;

/// Creates the gateways table in the database if it doesn't already exist
///
/// This function ensures that the database schema is properly initialized before
/// any operations are performed. It is automatically called by other functions
/// in this module, so there's usually no need to call it directly.
///
/// # Database Schema
///
/// Creates a table with the following structure:
/// - `id`: TEXT PRIMARY KEY - Unique identifier for the gateway
/// - `gwnode_id`: TEXT NOT NULL - Reference to the associated gateway node's ID
/// - `pattern`: TEXT NOT NULL - URL pattern for matching incoming requests
/// - `target`: TEXT NOT NULL - Target URL where matching requests should be routed
/// - `priority`: INTEGER NOT NULL - Priority level, with lower numbers having higher precedence
///
/// A foreign key constraint is established to ensure referential integrity with the
/// gateway_nodes table to ensure each gateway is associated with a valid gateway node.
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
pub fn ensure_gateways_table() -> Result<(), DatabaseError> {
    let db = get_connection()?;
    
    // Define the expected columns
    let expected_columns = ["id", "gwnode_id", "pattern", "target", "priority"];
    
    // Check if the table exists with the expected columns and is not corrupted
    if db.table_exists_with_columns("gateways", &expected_columns)? {
        log::debug!("gateways table exists and has expected structure");
        return Ok(());
    }
    
    log::info!("Creating or repairing gateways table");
    
    // Drop the table if it exists but is corrupted or missing columns
    db.execute("DROP TABLE IF EXISTS gateways", [])?;
    
    // Create the table with the full correct structure
    db.execute(
        "CREATE TABLE gateways (
            id TEXT PRIMARY KEY,
            gwnode_id TEXT NOT NULL,
            pattern TEXT NOT NULL,
            target TEXT NOT NULL,
            priority INTEGER NOT NULL,
            FOREIGN KEY(gwnode_id) REFERENCES gateway_nodes(id)
        )",
        [],
    )?;
    
    log::info!("Created gateways table with correct structure");
    Ok(())
}

/// Retrieves all gateway configurations from the database, ordered by priority
///
/// This function fetches all gateway records from the database, orders them by
/// priority (lower numbers first), and converts them into `Gateway` structures.
/// It automatically ensures the database table exists before performing the query.
///
/// # Returns
///
/// * `Ok(Vec<Gateway>)` - A vector containing all gateway configurations
/// * `Err(DatabaseError)` - If there was an error retrieving the gateways
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL query could not be executed
/// - There was an error mapping the database rows to `Gateway` structures
///
/// # Example
///
/// ```
/// use router_api::api::settings::gateway_queries;
///
/// match gateway_queries::get_all_gateways() {
///     Ok(gateways) => {
///         println!("Found {} gateways", gateways.len());
///         for gateway in gateways {
///             println!("Gateway: {} (pattern: {}, priority: {})", 
///                     gateway.id, gateway.pattern, gateway.priority);
///         }
///     },
///     Err(err) => // eprintln!!("Error retrieving gateways: {}", err),
/// }
/// ```
pub fn get_all_gateways() -> Result<Vec<Gateway>, DatabaseError> {
    let db = get_connection()?;
    
    // Ensure the table exists
    ensure_gateways_table()?;
    
    // Query all gateways, ordered by priority
    let gateways = db.query(
        "SELECT id, gwnode_id, pattern, target, priority FROM gateways ORDER BY priority ASC",
        [],
        |row| {
            Ok(Gateway {
                id: row.get(0)?,
                gwnode_id: row.get(1)?,
                pattern: row.get(2)?,
                target: row.get(3)?,
                priority: row.get(4)?,
            })
        },
    )?;
    
    Ok(gateways)
}

/// Retrieves a specific gateway configuration by its ID
///
/// This function fetches a single gateway record from the database based on
/// the provided ID. It automatically ensures the database table exists before
/// performing the query.
///
/// # Parameters
///
/// * `id` - The unique identifier of the gateway to retrieve
///
/// # Returns
///
/// * `Ok(Some(Gateway))` - If the gateway with the specified ID was found
/// * `Ok(None)` - If no gateway with the specified ID exists
/// * `Err(DatabaseError)` - If there was an error retrieving the gateway
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL query could not be executed
/// - There was an error mapping the database row to a `Gateway` structure
///
/// # Example
///
/// ```
/// use router_api::api::settings::gateway_queries;
///
/// let gateway_id = "a1b2c3d4-e5f6-4321-8765-10293847abcd";
/// match gateway_queries::get_gateway_by_id(gateway_id) {
///     Ok(Some(gateway)) => println!("Found gateway: {} (pattern: {}, priority: {})", 
///                                   gateway.id, gateway.pattern, gateway.priority),
///     Ok(None) => println!("No gateway found with ID: {}", gateway_id),
///     Err(err) => // eprintln!!("Error retrieving gateway: {}", err),
/// }
/// ```
pub fn get_gateway_by_id(id: &str) -> Result<Option<Gateway>, DatabaseError> {
    let db = get_connection()?;
    
    // Ensure the table exists
    ensure_gateways_table()?;
    
    // Query the gateway by ID
    let gateway = db.query_one(
        "SELECT id, gwnode_id, pattern, target, priority FROM gateways WHERE id = ?1",
        [id],
        |row| {
            Ok(Gateway {
                id: row.get(0)?,
                gwnode_id: row.get(1)?,
                pattern: row.get(2)?,
                target: row.get(3)?,
                priority: row.get(4)?,
            })
        },
    )?;
    
    Ok(gateway)
}

/// Retrieves all gateways associated with a specific gateway node
///
/// This function fetches all gateway records that reference the specified 
/// gateway node ID, ordered by priority (lower numbers first). It automatically
/// ensures the database table exists before performing the query.
///
/// # Parameters
///
/// * `gwnode_id` - The ID of the gateway node to find associated gateways for
///
/// # Returns
///
/// * `Ok(Vec<Gateway>)` - A vector containing all matching gateway configurations
/// * `Err(DatabaseError)` - If there was an error retrieving the gateways
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL query could not be executed
/// - There was an error mapping the database rows to `Gateway` structures
///
/// # Ordering
///
/// The returned gateways are ordered by their priority field in ascending order,
/// meaning that gateways with lower priority values (higher precedence) appear
/// first in the result set.
///
/// # Example
///
/// ```
/// use router_api::api::settings::gateway_queries;
///
/// let node_id = "7f9c24e5-1315-43a7-9f31-6eb9772cb46a";
/// match gateway_queries::get_gateways_by_gwnode_id(node_id) {
///     Ok(gateways) => {
///         println!("Found {} gateways for node {}", gateways.len(), node_id);
///         for gateway in gateways {
///             println!("Gateway: {} (pattern: {}, priority: {})", 
///                     gateway.id, gateway.pattern, gateway.priority);
///         }
///     },
///     Err(err) => // eprintln!!("Error retrieving gateways: {}", err),
/// }
/// ```
pub fn get_gateways_by_gwnode_id(gwnode_id: &str) -> Result<Vec<Gateway>, DatabaseError> {
    let db = get_connection()?;
    
    // Ensure the table exists
    ensure_gateways_table()?;
    
    // Query gateways by gateway node ID, ordered by priority
    let gateways = db.query(
        "SELECT id, gwnode_id, pattern, target, priority FROM gateways WHERE gwnode_id = ?1 ORDER BY priority ASC",
        [gwnode_id],
        |row| {
            Ok(Gateway {
                id: row.get(0)?,
                gwnode_id: row.get(1)?,
                pattern: row.get(2)?,
                target: row.get(3)?,
                priority: row.get(4)?,
            })
        },
    )?;
    
    Ok(gateways)
}

/// Saves a gateway configuration to the database
///
/// This function inserts a new gateway record or updates an existing one if a gateway
/// with the same ID already exists. It automatically ensures the database table exists
/// before performing the operation.
///
/// # Parameters
///
/// * `gateway` - The gateway configuration to save
///
/// # Returns
///
/// * `Ok(())` - If the gateway was successfully saved
/// * `Err(DatabaseError)` - If there was an error saving the gateway
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL statement could not be executed
/// - The foreign key constraint is violated (if the referenced gateway node does not exist)
///
/// # Security Notes
///
/// This function uses parameterized SQL queries to prevent SQL injection attacks.
///
/// # Example
///
/// ```
/// use router_api::api::settings::{Gateway, gateway_queries};
///
/// let gateway = Gateway {
///     id: "a1b2c3d4-e5f6-4321-8765-10293847abcd".to_string(),
///     gwnode_id: "7f9c24e5-1315-43a7-9f31-6eb9772cb46a".to_string(),
///     pattern: "/api/users/*".to_string(),
///     target: "http://user-service:8080".to_string(),
///     priority: 10,
/// };
///
/// match gateway_queries::save_gateway(&gateway) {
///     Ok(()) => println!("Gateway saved successfully"),
///     Err(err) => // eprintln!!("Error saving gateway: {}", err),
/// }
/// ```
pub fn save_gateway(gateway: &Gateway) -> Result<(), DatabaseError> {
    let db = get_connection()?;
    
    // Ensure the table exists
    ensure_gateways_table()?;
    
    // Insert or replace the gateway
    db.execute(
        "INSERT OR REPLACE INTO gateways (id, gwnode_id, pattern, target, priority) 
         VALUES (?1, ?2, ?3, ?4, ?5)",
        [
            &gateway.id,
            &gateway.gwnode_id,
            &gateway.pattern,
            &gateway.target,
            &gateway.priority.to_string(),
        ],
    )?;
    
    Ok(())
}

/// Deletes a gateway configuration from the database by its ID
///
/// This function removes a gateway record from the database based on its ID.
/// It returns a boolean indicating whether a record was actually deleted.
///
/// # Parameters
///
/// * `id` - The unique identifier of the gateway to delete
///
/// # Returns
///
/// * `Ok(true)` - If the gateway was found and deleted
/// * `Ok(false)` - If no gateway with the specified ID exists
/// * `Err(DatabaseError)` - If there was an error deleting the gateway
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
/// use router_api::api::settings::gateway_queries;
///
/// let gateway_id = "a1b2c3d4-e5f6-4321-8765-10293847abcd";
/// match gateway_queries::delete_gateway_by_id(gateway_id) {
///     Ok(true) => println!("Gateway deleted successfully"),
///     Ok(false) => println!("No gateway found with ID: {}", gateway_id),
///     Err(err) => // eprintln!!("Error deleting gateway: {}", err),
/// }
/// ```
pub fn delete_gateway_by_id(id: &str) -> Result<bool, DatabaseError> {
    let db = get_connection()?;
    
    // Delete the gateway
    let affected_rows = db.execute(
        "DELETE FROM gateways WHERE id = ?1",
        [id],
    )?;
    
    Ok(affected_rows > 0)
}

/// Generates a new unique identifier for a gateway
///
/// This function creates a UUID v4 (random) string that can be used as the ID
/// for a new gateway. UUIDs are globally unique identifiers that have an
/// extremely low probability of collision.
///
/// # Returns
///
/// * A string containing a new UUID v4 in canonical form (e.g., "a1b2c3d4-e5f6-4321-8765-10293847abcd")
///
/// # Example
///
/// ```
/// use router_api::api::settings::gateway_queries;
///
/// let new_id = gateway_queries::generate_gateway_id();
/// println!("Generated new gateway ID: {}", new_id);
/// ```
pub fn generate_gateway_id() -> String {
    Uuid::new_v4().to_string()
}