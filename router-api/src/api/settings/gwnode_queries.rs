//! # Gateway Node Database Operations
//!
//! This module provides database operations for managing gateway node configurations.
//! Gateway nodes act as intermediaries between proxies and gateways, providing alternative
//! routing paths and allowing for more complex routing scenarios.
//!
//! The module handles creating the database table, querying, inserting, updating, and
//! deleting gateway node records, as well as managing the relationship with proxies.

use super::GatewayNode;
use crate::module::database::{get_connection, DatabaseError};
use uuid::Uuid;

/// Creates the gateway_nodes table in the database if it doesn't already exist
///
/// This function ensures that the database schema is properly initialized before
/// any operations are performed. It is automatically called by other functions
/// in this module, so there's usually no need to call it directly.
///
/// # Database Schema
///
/// Creates a table with the following structure:
/// - `id`: TEXT PRIMARY KEY - Unique identifier for the gateway node
/// - `proxy_id`: TEXT NOT NULL - Reference to the associated proxy's ID
/// - `domain_id`: TEXT - Reference to the domain ID (can be null)
/// - `title`: TEXT NOT NULL - Human-readable name for this gateway node
/// - `alt_target`: TEXT NOT NULL - Alternative target URL for routing
/// - `priority`: INTEGER NOT NULL DEFAULT 100 - Processing priority
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
pub fn ensure_gateway_nodes_table() -> Result<(), DatabaseError> {
    let db = get_connection()?;
    
    // Define the expected columns
    let expected_columns = ["id", "proxy_id", "domain_id", "title", "alt_target", "priority"];
    
    // Check if the table exists with the expected columns and is not corrupted
    if db.table_exists_with_columns("gateway_nodes", &expected_columns)? {
        log::debug!("gateway_nodes table exists and has expected structure");
        return Ok(());
    }
    
    log::info!("Creating or repairing gateway_nodes table");
    
    // Drop the table if it exists but is corrupted or missing columns
    db.execute("DROP TABLE IF EXISTS gateway_nodes", [])?;
    
    // Create the table with the full correct structure
    db.execute(
        "CREATE TABLE gateway_nodes (
            id TEXT PRIMARY KEY,
            proxy_id TEXT NOT NULL,
            domain_id TEXT,
            title TEXT NOT NULL,
            alt_target TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 100,
            FOREIGN KEY(proxy_id) REFERENCES proxies(id),
            FOREIGN KEY(domain_id) REFERENCES proxy_domains(id)
        )",
        [],
    )?;
    
    log::info!("Created gateway_nodes table with correct structure");
    Ok(())
}

/// Retrieves all gateway node configurations from the database
///
/// This function fetches all gateway node records from the database and converts
/// them into `GatewayNode` structures. It automatically ensures the database table
/// exists before performing the query.
///
/// # Returns
///
/// * `Ok(Vec<GatewayNode>)` - A vector containing all gateway node configurations
/// * `Err(DatabaseError)` - If there was an error retrieving the gateway nodes
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL query could not be executed
/// - There was an error mapping the database rows to `GatewayNode` structures
///
/// # Example
///
/// ```
/// use router_api::api::settings::gwnode_queries;
///
/// match gwnode_queries::get_all_gateway_nodes() {
///     Ok(nodes) => {
///         println!("Found {} gateway nodes", nodes.len());
///         for node in nodes {
///             println!("Gateway node: {} (title: {}, proxy: {})", node.id, node.title, node.proxy_id);
///         }
///     },
///     Err(err) => eprintln!("Error retrieving gateway nodes: {}", err),
/// }
/// ```
pub fn get_all_gateway_nodes() -> Result<Vec<GatewayNode>, DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_gateway_nodes_table()?;

    // Query all gateway nodes with a LEFT JOIN that properly handles NULL values
    // Using GROUP BY to avoid duplicate gateway nodes due to multiple associated proxy domains
    // Use GROUP_CONCAT to include domain information in a single row per gateway node
    let nodes = db.query(
        "
        SELECT 
            n.id, 
            n.proxy_id, 
            n.domain_id,
            n.title, 
            n.alt_target, 
            n.priority,
            (SELECT d.sni FROM proxy_domains d WHERE d.id = n.domain_id LIMIT 1) as domain_name
        FROM gateway_nodes as n",
        [],
        |row| {
            Ok(GatewayNode {
                id: row.get(0)?,
                proxy_id: row.get(1)?,
                domain_id: row.get::<_, Option<String>>(2)?,
                title: row.get(3)?,
                alt_target: row.get(4)?,
                priority: row.get(5)?,
                domain_name: row.get::<_, Option<String>>(6)?,
            })
        },
    )?;

    log::info!("Retrieved {} gateway nodes from the database", nodes.len());

    Ok(nodes)
}

/// Retrieves a specific gateway node configuration by its ID
///
/// This function fetches a single gateway node record from the database based on
/// the provided ID. It automatically ensures the database table exists before
/// performing the query.
///
/// # Parameters
///
/// * `id` - The unique identifier of the gateway node to retrieve
///
/// # Returns
///
/// * `Ok(Some(GatewayNode))` - If the gateway node with the specified ID was found
/// * `Ok(None)` - If no gateway node with the specified ID exists
/// * `Err(DatabaseError)` - If there was an error retrieving the gateway node
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL query could not be executed
/// - There was an error mapping the database row to a `GatewayNode` structure
///
/// # Example
///
/// ```
/// use router_api::api::settings::gwnode_queries;
///
/// let node_id = "7f9c24e5-1315-43a7-9f31-6eb9772cb46a";
/// match gwnode_queries::get_gateway_node_by_id(node_id) {
///     Ok(Some(node)) => println!("Found gateway node: {} (title: {}, alt_target: {})",
///                                node.id, node.title, node.alt_target),
///     Ok(None) => println!("No gateway node found with ID: {}", node_id),
///     Err(err) => eprintln!("Error retrieving gateway node: {}", err),
/// }
/// ```
pub fn get_gateway_node_by_id(id: &str) -> Result<Option<GatewayNode>, DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_gateway_nodes_table()?;

    // Query the gateway node by ID
    // Using subqueries to avoid duplicates from proxy domain relationships
    let node = db.query_one(
        "
        SELECT 
            n.id, 
            n.proxy_id, 
            n.domain_id,
            n.title, 
            n.alt_target, 
            n.priority,
            (SELECT d.sni FROM proxy_domains d WHERE d.id = n.domain_id LIMIT 1) as domain_name
        FROM gateway_nodes as n 
        WHERE n.id = ?1",
        [id],
        |row| {
            Ok(GatewayNode {
                id: row.get(0)?,
                proxy_id: row.get(1)?,
                domain_id: row.get::<_, Option<String>>(2)?,
                title: row.get(3)?,
                alt_target: row.get(4)?,
                priority: row.get(5)?,
                domain_name: row.get::<_, Option<String>>(6)?,
            })
        },
    )?;

    Ok(node)
}

/// Retrieves all gateway nodes associated with a specific proxy
///
/// This function fetches all gateway node records that reference the specified
/// proxy ID. It automatically ensures the database table exists before performing
/// the query.
///
/// # Parameters
///
/// * `proxy_id` - The ID of the proxy to find associated gateway nodes for
///
/// # Returns
///
/// * `Ok(Vec<GatewayNode>)` - A vector containing all matching gateway node configurations
/// * `Err(DatabaseError)` - If there was an error retrieving the gateway nodes
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL query could not be executed
/// - There was an error mapping the database rows to `GatewayNode` structures
///
/// # Example
///
/// ```
/// use router_api::api::settings::gwnode_queries;
///
/// let proxy_id = "550e8400-e29b-41d4-a716-446655440000";
/// match gwnode_queries::get_gateway_nodes_by_proxy_id(proxy_id) {
///     Ok(nodes) => {
///         println!("Found {} gateway nodes for proxy {}", nodes.len(), proxy_id);
///         for node in nodes {
///             println!("Gateway node: {} (title: {}, alt_target: {})",
///                      node.id, node.title, node.alt_target);
///         }
///     },
///     Err(err) => eprintln!("Error retrieving gateway nodes: {}", err),
/// }
/// ```
pub fn get_gateway_nodes_by_proxy_id(proxy_id: &str) -> Result<Vec<GatewayNode>, DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_gateway_nodes_table()?;

    // Query gateway nodes by proxy ID
    // Using subqueries to avoid duplicates from proxy domain relationships
    let nodes = db.query(
        "
        SELECT 
            n.id, 
            n.proxy_id, 
            n.domain_id,
            n.title, 
            n.alt_target, 
            n.priority,
            (SELECT d.sni FROM proxy_domains d WHERE d.id = n.domain_id LIMIT 1) as domain_name
        FROM gateway_nodes as n
        WHERE n.proxy_id = ?1
        ORDER BY priority DESC",
        [proxy_id],
        |row| {
            Ok(GatewayNode {
                id: row.get(0)?,
                proxy_id: row.get(1)?,
                domain_id: row.get::<_, Option<String>>(2)?,
                title: row.get(3)?,
                alt_target: row.get(4)?,
                priority: row.get(5)?,
                domain_name: row.get::<_, Option<String>>(6)?,
            })
        },
    )?;

    Ok(nodes)
}

/// Saves a gateway node configuration to the database
///
/// This function inserts a new gateway node record or updates an existing one if a gateway node
/// with the same ID already exists. It automatically ensures the database table exists
/// before performing the operation.
///
/// # Parameters
///
/// * `node` - The gateway node configuration to save
///
/// # Returns
///
/// * `Ok(())` - If the gateway node was successfully saved
/// * `Err(DatabaseError)` - If there was an error saving the gateway node
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection could not be established
/// - The table does not exist and could not be created
/// - The SQL statement could not be executed
/// - The foreign key constraint is violated (if the referenced proxy does not exist)
///
/// # Security Notes
///
/// This function uses parameterized SQL queries to prevent SQL injection attacks.
///
/// # Example
///
/// ```
/// use router_api::api::settings::{GatewayNode, gwnode_queries};
///
/// let node = GatewayNode {
///     id: "7f9c24e5-1315-43a7-9f31-6eb9772cb46a".to_string(),
///     proxy_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
///     title: "API Backup Gateway".to_string(),
///     alt_target: "http://backup-server.internal:8080".to_string(),
///     priority: 150, // Higher priority than default
/// };
///
/// match gwnode_queries::save_gateway_node(&node) {
///     Ok(()) => println!("Gateway node saved successfully"),
///     Err(err) => eprintln!("Error saving gateway node: {}", err),
/// }
/// ```
pub fn save_gateway_node(node: &GatewayNode) -> Result<(), DatabaseError> {
    let db = get_connection()?;

    // Ensure the table exists
    ensure_gateway_nodes_table()?;

    // Insert or update the gateway node
    db.execute(
        "INSERT INTO gateway_nodes (id, proxy_id, domain_id, title, alt_target, priority)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
         proxy_id = ?2,
         domain_id = ?3,
         title = ?4,
         alt_target = ?5,
         priority = ?6",
        rusqlite::params![
            node.id,
            node.proxy_id,
            node.domain_id,
            node.title,
            node.alt_target,
            node.priority,
        ],
    )?;

    Ok(())
}

/// Deletes a gateway node configuration from the database by its ID
///
/// This function removes a gateway node record from the database based on its ID.
/// It returns a boolean indicating whether a record was actually deleted.
///
/// Note: When a gateway node is deleted, all associated gateways should also be deleted,
/// but this function doesn't handle that automatically. The API endpoint that calls this
/// function should handle the cascading deletion.
///
/// # Parameters
///
/// * `id` - The unique identifier of the gateway node to delete
///
/// # Returns
///
/// * `Ok(true)` - If the gateway node was found and deleted
/// * `Ok(false)` - If no gateway node with the specified ID exists
/// * `Err(DatabaseError)` - If there was an error deleting the gateway node
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
/// use router_api::api::settings::gwnode_queries;
///
/// let node_id = "7f9c24e5-1315-43a7-9f31-6eb9772cb46a";
/// match gwnode_queries::delete_gateway_node_by_id(node_id) {
///     Ok(true) => println!("Gateway node deleted successfully"),
///     Ok(false) => println!("No gateway node found with ID: {}", node_id),
///     Err(err) => eprintln!("Error deleting gateway node: {}", err),
/// }
/// ```
pub fn delete_gateway_node_by_id(id: &str) -> Result<bool, DatabaseError> {
    let db = get_connection()?;

    // Delete the gateway node
    let affected_rows = db.execute("DELETE FROM gateway_nodes WHERE id = ?1", [id])?;

    Ok(affected_rows > 0)
}

/// Generates a new unique identifier for a gateway node
///
/// This function creates a UUID v4 (random) string that can be used as the ID
/// for a new gateway node. UUIDs are globally unique identifiers that have an
/// extremely low probability of collision.
///
/// # Returns
///
/// * A string containing a new UUID v4 in canonical form (e.g., "7f9c24e5-1315-43a7-9f31-6eb9772cb46a")
///
/// # Example
///
/// ```
/// use router_api::api::settings::gwnode_queries;
///
/// let new_id = gwnode_queries::generate_gateway_node_id();
/// println!("Generated new gateway node ID: {}", new_id);
/// ```
pub fn generate_gateway_node_id() -> String {
    Uuid::new_v4().to_string()
}

/// Updates gateway nodes to be unbound when their associated proxy is deleted
///
/// Rather than deleting gateway nodes when their associated proxy is removed,
/// this function marks them as "unbound" by setting their proxy_id field to the
/// special value "unbound". This preserves the gateway node configuration while
/// indicating that it's no longer tied to a valid proxy.
///
/// # Parameters
///
/// * `proxy_id` - The ID of the proxy that is being deleted
///
/// # Returns
///
/// * `Ok(usize)` - The number of gateway nodes that were updated
/// * `Err(DatabaseError)` - If there was an error updating the gateway nodes
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
/// use router_api::api::settings::gwnode_queries;
///
/// let proxy_id = "550e8400-e29b-41d4-a716-446655440000";
/// match gwnode_queries::unbind_gateway_nodes_by_proxy_id(proxy_id) {
///     Ok(count) => println!("{} gateway nodes were marked as unbound", count),
///     Err(err) => eprintln!("Error unbinding gateway nodes: {}", err),
/// }
/// ```
pub fn unbind_gateway_nodes_by_proxy_id(proxy_id: &str) -> Result<usize, DatabaseError> {
    let db = get_connection()?;

    // Update all gateway nodes associated with this proxy to mark them as unbound
    let affected_rows = db.execute(
        "UPDATE gateway_nodes SET proxy_id = 'unbound' WHERE proxy_id = ?1",
        [proxy_id],
    )?;

    Ok(affected_rows)
}
