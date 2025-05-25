//! # Proxy API Endpoints for Creating and Deleting
//!
//! This module provides HTTP endpoints for creating, updating, and deleting proxy configurations.
//! Proxies are the foundation of the gateway system, listening on specific addresses and forwarding
//! traffic to target destinations.

use actix_web::{delete, post, web, HttpResponse, Responder, HttpRequest};
use super::{Proxy, ProxyDomain, proxy_queries, proxydomain_queries};
use super::gwnode_queries;
use uuid::Uuid;
use crate::api::users::helper::{ClaimsFromRequest, is_staff_or_admin};
use crate::module::database::DatabaseError;
use serde::{Deserialize, Serialize};

/// Composite input structure for proxy creation/update with domains
///
/// This structure allows submitting a proxy configuration along with its
/// associated domains in a single request.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyInputObject {
    /// The core proxy configuration
    pub proxy: Proxy,
    /// Optional vector of associated domains
    pub domains: Option<Vec<ProxyDomain>>,
}

/// Creates or updates a proxy configuration
///
/// This endpoint processes HTTP POST requests to create new proxies or update existing ones.
/// When creating a new proxy, it automatically generates a unique ID and a target address
/// with a randomly available port.
///
/// # Endpoint
///
/// `POST /settings/proxy`
///
/// # Request Body
///
/// The request body should be a JSON object with the following fields:
/// - `id` (optional): The unique identifier for the proxy. If empty, a new UUID will be generated.
/// - `title`: Human-readable name for the proxy.
/// - `addr_listen`: Address where the proxy listens for connections (format: "ip:port").
/// - `high_speed` (optional): Whether speed mode is enabled for faster proxying (default: false).
/// - `high_speed_addr` (optional): Specific address to use for speed mode.
///
/// Note: TLS configuration has been moved to the ProxyDomain entity.
///
/// Note: The `addr_target` field does not need to be provided in the request as it is
/// automatically generated with an available port on 127.0.0.1.
///
/// # Response
///
/// ## Success (200 OK)
/// Returns the complete saved proxy configuration as a JSON object, including 
/// the generated ID and target address.
///
/// ## Internal Server Error (500)
/// Returned when there is a database error, port allocation failure, or other server error.
///
/// # Automatic Target Address
///
/// This endpoint automatically assigns a target address with:
/// - IP: 127.0.0.1 (localhost)
/// - Port: Randomly selected available port between 40000-49000
///
/// This ensures that each proxy has a unique target that can be safely used by other
/// components of the system.
///
/// # Examples
///
/// Create a new proxy:
/// ```
/// POST /settings/proxy
/// Content-Type: application/json
///
/// {
///   "title": "Web Server",
///   "addr_listen": "0.0.0.0:80",
///   "tls": false
/// }
/// ```
///
/// Update an existing proxy:
/// ```
/// POST /settings/proxy
/// Content-Type: application/json
///
/// {
///   "id": "550e8400-e29b-41d4-a716-446655440000",
///   "title": "Web Server (Updated)",
///   "addr_listen": "0.0.0.0:443",
///   "tls": true,
///   "tls_autron": true,
///   "sni": "example.com"
/// }
/// ```
#[post("/proxy")]
pub async fn set_proxy(
    req: HttpRequest,
    input: web::Json<ProxyInputObject>
) -> impl Responder {
    // Extract authenticated user's claims
    let claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Failed to get user authentication"})
            )
        }
    };
    
    // Verify user has admin or staff role
    if !is_staff_or_admin(&claims.role) {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"error": "Only administrators and staff can modify proxy settings"})
        );
    }
    
    let mut proxy = input.proxy.clone();
    let is_new_proxy = proxy.id.is_empty();
    
    // Generate an ID if none was provided
    if is_new_proxy {
        proxy.id = Uuid::new_v4().to_string();
    }

    // check if proxy.addr_listen is a valid ip address with port
    if !proxy.addr_listen.contains(":") {
        return HttpResponse::BadRequest().json(
            serde_json::json!({"error": "Addr listen must be a valid IP address with port"})
        );
    }

    // check if after : is a valid port 1 - 65535, return error if not
    if let Some(port) = proxy.addr_listen.split(":").nth(1) {
        if port.parse::<u16>().is_err() {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Addr listen must be a valid IP address with port"})
            );
        }
    }
    
    // Check for duplicate listen address - this check applies to all proxies regardless of mode
    match proxy_queries::has_duplicate_listen_address(&proxy.addr_listen, Some(&proxy.id)) {
        Ok(has_duplicate) => {
            if has_duplicate {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Cannot create/update proxy because there is already another proxy with the same listen address. Each proxy must have a unique listen address."
                }));
            }
        },
        Err(e) => {
            log::error!("Error checking for duplicate listen addresses: {}", e);
            return HttpResponse::BadRequest().body("Failed to check for duplicate listen addresses");
        }
    }
    
    // Store the proxy ID for potential cleanup if domain save fails
    let proxy_id = proxy.id.clone();
    
    // Generate a target address with random available port
    match proxy_queries::generate_target_address() {
        Ok(addr) => {
            proxy.addr_target = addr;
            
            // Handle high-speed mode configuration
            if proxy.high_speed {
                // If high_speed_gwid is provided, look up its alt_target to set as high_speed_addr
                if let Some(gwid) = &proxy.high_speed_gwid {
                    if !gwid.is_empty() {
                        match gwnode_queries::get_gateway_node_by_id(gwid) {
                            Ok(Some(gwnode)) => {
                                proxy.high_speed_addr = Some(gwnode.alt_target.clone());
                            },
                            Ok(None) => {
                                log::warn!("Gateway node {} not found for high_speed_gwid", gwid);
                                return HttpResponse::BadRequest().json(serde_json::json!({
                                    "error": "The specified gateway node for high-speed mode was not found"
                                }));
                            },
                            Err(e) => {
                                log::error!("Error retrieving gateway node {}: {}", gwid, e);
                                return HttpResponse::BadRequest().body("Failed to retrieve gateway node for high-speed mode");
                            }
                        }
                    }
                }
                
                // If high_speed is enabled but high_speed_addr is still empty, set it to the same as addr_target
                if proxy.high_speed_addr.is_none() || proxy.high_speed_addr.as_ref().unwrap().is_empty() {
                    proxy.high_speed_addr = Some(proxy.addr_target.clone());
                }
            } else {
                // If high_speed is disabled, set high_speed_addr and high_speed_gwid to None
                proxy.high_speed_addr = None;
                proxy.high_speed_gwid = None;
            }
            
            // Step 1: Save the proxy without verification
            if let Err(e) = proxy_queries::save_proxy(&proxy) {
                log::error!("Error saving proxy {}: {}", proxy.id, e);
                let error_message = match e {
                    DatabaseError::Sqlite(sqlite_error) => {
                        if let rusqlite::Error::SqliteFailure(err, _) = sqlite_error {
                            if err.code == rusqlite::ffi::ErrorCode::ConstraintViolation {
                                format!("Database constraint violation while saving proxy {}. Please check if the proxy ID is valid.", proxy.id)
                            } else {
                                format!("Database error while saving proxy {}: {}", proxy.id, sqlite_error)
                            }
                        } else {
                            format!("SQLite error: {}", sqlite_error)
                        }
                    },
                    _ => format!("Failed to save proxy {}: {}", proxy.id, e)
                };
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": error_message,
                    "proxy_id": proxy.id
                }));
            }
            
            log::debug!("Proxy {} saved successfully", proxy.id);
            
            // Step 2: Process domains if provided
            let mut saved_domain_ids = Vec::new(); // Track successfully saved domains for potential cleanup
            
            if let Some(incoming_domains) = &input.domains {
                // Check for duplicate domain names in the incoming domains
                let mut seen_domain_names = std::collections::HashSet::new();
                
                for domain in incoming_domains.iter() {
                    if let Some(domain_name) = &domain.sni {
                        if !domain_name.is_empty() {
                            // Check if this domain name has been seen before
                            if !seen_domain_names.insert(domain_name.clone()) {
                                // Cleanup the proxy we just created if this is a new proxy
                                if is_new_proxy {
                                    cleanup_proxy_and_domains(&proxy_id, &saved_domain_ids);
                                }
                                
                                return HttpResponse::BadRequest().json(serde_json::json!({
                                    "error": format!("Duplicate domain name '{}' found in the request. Each domain name must be unique.", domain_name)
                                }));
                            }
                        }
                    }
                }
                
                // Fetch existing domains for this proxy to identify domains to remove later
                let existing_domains = match proxydomain_queries::get_proxy_domains_by_proxy_id(&proxy.id) {
                    Ok(domains) => domains,
                    Err(e) => {
                        log::warn!("Warning: Failed to fetch existing domains: {}", e);
                        // Continue anyway, we just won't be able to remove old domains
                        Vec::new()
                    }
                };
                
                let mut existing_domain_ids_to_keep = Vec::new();
                
                // Process each domain individually
                for mut domain in incoming_domains.clone() {
                    // Ensure domain is associated with this proxy
                    domain.proxy_id = Some(proxy.id.clone());
                    
                    // Generate domain ID if not provided (empty string)
                    if domain.id.is_empty() {
                        domain.id = proxydomain_queries::generate_proxy_domain_id();
                    } else {
                        // If domain has an ID, it exists, so add it to the keep list
                        existing_domain_ids_to_keep.push(domain.id.clone());
                    }
                    
                    // Log domain data before saving
                    log::debug!("Saving proxy domain: id={}, proxy_id={:?}", domain.id, domain.proxy_id);
                    
                    // Save the domain with proper error handling
                    if let Err(e) = proxydomain_queries::save_proxy_domain(&domain) {
                        log::error!("Error saving proxy domain {}: {}", domain.id, e);
                        
                        // Cleanup the proxy and successfully saved domains if this is a new proxy
                        if is_new_proxy {
                            cleanup_proxy_and_domains(&proxy_id, &saved_domain_ids);
                        }
                        
                        // Return a detailed error message
                        let error_message = match e {
                            DatabaseError::Sqlite(sqlite_error) => {
                                if let rusqlite::Error::SqliteFailure(err, _) = sqlite_error {
                                    if err.code == rusqlite::ffi::ErrorCode::ConstraintViolation {
                                        format!("Foreign key constraint failed for domain {}. Make sure the proxy_id is valid.", domain.id)
                                    } else {
                                        format!("Database error while saving domain {}: {}", domain.id, sqlite_error)
                                    }
                                } else {
                                    format!("SQLite error: {}", sqlite_error)
                                }
                            },
                            _ => format!("Failed to save proxy domain {}: {}", domain.id, e)
                        };
                        
                        return HttpResponse::BadRequest().json(serde_json::json!({
                            "error": error_message,
                            "domain_id": domain.id,
                            "proxy_id": domain.proxy_id
                        }));
                    }
                    
                    // Add to the list of successfully saved domains
                    saved_domain_ids.push(domain.id.clone());
                }
                
                // Delete domains that exist in the database but are not in the incoming data
                if !is_new_proxy { // Only for existing proxies
                    let mut deleted_count = 0;
                    for existing_domain in existing_domains {
                        if !existing_domain_ids_to_keep.contains(&existing_domain.id) {
                            match proxydomain_queries::delete_proxy_domain_by_id(&existing_domain.id) {
                                Ok(_) => {
                                    deleted_count += 1;
                                    log::debug!("Deleted proxy domain {} as it was removed from frontend", existing_domain.id);
                                },
                                Err(e) => {
                                    log::error!("Error deleting removed proxy domain {}: {}", existing_domain.id, e);
                                    // Continue processing other domains despite this error
                                }
                            }
                        }
                    }
                    
                    if deleted_count > 0 {
                        log::info!("Deleted {} proxy domains that were removed from the frontend", deleted_count);
                    }
                }
            }
            
            // Step 3: Fetch all domains for this proxy to include in response
            let domains = match proxydomain_queries::get_proxy_domains_by_proxy_id(&proxy.id) {
                Ok(domains) => domains,
                Err(e) => {
                    log::error!("Error fetching proxy domains for {}: {}", proxy.id, e);
                    // Return success but with empty domains list and a warning
                    return HttpResponse::Ok().json(serde_json::json!({
                        "proxy": proxy,
                        "domains": Vec::<ProxyDomain>::new(),
                        "warning": "Proxy saved but could not fetch associated domains"
                    }));
                }
            };
            
            // Return the complete proxy with its domains
            HttpResponse::Ok().json(serde_json::json!({
                "proxy": proxy,
                "domains": domains
            }))
        },
        Err(e) => {
            log::error!("Error generating target address: {}", e);
            HttpResponse::BadRequest().body("Failed to generate target address")
        }
    }
}

/// Helper function to clean up a proxy and all its domains when an error occurs
fn cleanup_proxy_and_domains(proxy_id: &str, domain_ids: &[String]) {
    // First delete the domains
    for domain_id in domain_ids {
        if let Err(e) = proxydomain_queries::delete_proxy_domain_by_id(domain_id) {
            log::error!("Error deleting domain {} during cleanup: {}", domain_id, e);
        }
    }
    
    // Then delete the proxy
    if let Err(e) = proxy_queries::delete_proxy_by_id(proxy_id) {
        log::error!("Error deleting proxy {} during cleanup: {}", proxy_id, e);
    }
}

/// Deletes a proxy configuration by ID
///
/// This endpoint processes HTTP DELETE requests to remove proxy configurations.
/// Rather than cascading deletion, it implements a "soft unbinding" approach
/// for associated  proxy nodes by marking them as "unbound" but preserving
/// their configuration.
///
/// # Endpoint
///
/// `DELETE /settings/proxy/{id}`
///
/// # Path Parameters
///
/// * `id` - The unique identifier of the proxy to delete
///
/// # Response
///
/// ## Success (200 OK)
/// Returns a success message indicating the proxy was deleted, along with
/// information about how many  proxy nodes were unbound in the process.
///
/// ## Not Found (404)
/// Returned when no proxy with the specified ID exists.
///
/// ## Internal Server Error (500)
/// Returned when there is a database or server error during the deletion or unbinding process.
///
/// # Gateway Node Handling
///
/// This endpoint implements a two-step process:
/// 1. First, it finds all  proxy nodes associated with the proxy and marks them as "unbound"
///    (sets their proxy_id to "unbound") rather than deleting them
/// 2. Then, it deletes the proxy itself
///
/// This approach preserves gateway node configurations even when their associated proxy
/// is removed, allowing them to be reassigned to different proxies later.
///
/// # Example
///
/// ```
/// DELETE /settings/proxy/550e8400-e29b-41d4-a716-446655440000
/// ```
#[delete("/proxy/{id}")]
pub async fn delete_proxy(
    req: HttpRequest,
    path: web::Path<String>
) -> impl Responder {
    // Extract authenticated user's claims
    let claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Failed to get user authentication"})
            )
        }
    };
    
    // Verify user has admin or staff role
    if !is_staff_or_admin(&claims.role) {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"error": "Only administrators and staff can delete proxy settings"})
        );
    }
    
    let id = path.into_inner();
    
    // Get proxy details for better error messages
    let proxy_name = match proxy_queries::get_proxy_by_id(&id) {
        Ok(Some(proxy)) => proxy.title,
        _ => id.clone() // Fallback to ID if proxy not found
    };
    
    // First delete any proxy domains associated with this proxy
    let domains_deleted = match proxydomain_queries::delete_proxy_domains_by_proxy_id(&id) {
        Ok(count) => count,
        Err(e) => {
            log::error!("Error deleting proxy domains for proxy {}: {}", id, e);
            return HttpResponse::BadRequest().body(format!("Failed to delete proxy domains for '{}'", proxy_name));
        }
    };
    
    // Then unbind any  proxy nodes associated with this proxy
    match gwnode_queries::unbind_gateway_nodes_by_proxy_id(&id) {
        Ok(unbound_count) => {
            // Now delete the proxy
            match proxy_queries::delete_proxy_by_id(&id) {
                Ok(deleted) => {
                    if deleted {
                        let mut message = format!("Proxy '{}' deleted.", proxy_name);
                        
                        if domains_deleted > 0 {
                            message.push_str(&format!(" {} proxy domains were removed.", domains_deleted));
                        }
                        
                        if unbound_count > 0 {
                            message.push_str(&format!(" {}  proxy nodes were unbound.", unbound_count));
                        }
                        
                        HttpResponse::Ok().body(message)
                    } else {
                        HttpResponse::NotFound().body(format!("Proxy '{}' not found", proxy_name))
                    }
                },
                Err(e) => {
                    log::error!("Error deleting proxy {}: {}", id, e);
                    let error_message = match e {
                        DatabaseError::Sqlite(sqlite_error) => {
                            if let rusqlite::Error::SqliteFailure(err, _) = sqlite_error {
                                if err.code == rusqlite::ffi::ErrorCode::ConstraintViolation {
                                    format!("Cannot delete proxy '{}' because it is still referenced by other entities. Please unbind all  proxy nodes first.", proxy_name)
                                } else {
                                    format!("Database error while deleting proxy '{}': {}", proxy_name, sqlite_error)
                                }
                            } else {
                                format!("SQLite error: {}", sqlite_error)
                            }
                        },
                        _ => format!("Failed to delete proxy '{}': {}", proxy_name, e)
                    };
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "error": error_message,
                        "proxy_id": id
                    }))
                }
            }
        },
        Err(e) => {
            log::error!("Error unbinding  proxy nodes for proxy {}: {}", id, e);
            let error_message = match e {
                DatabaseError::Sqlite(sqlite_error) => {
                    if let rusqlite::Error::SqliteFailure(err, _) = sqlite_error {
                        if err.code == rusqlite::ffi::ErrorCode::ConstraintViolation {
                            format!("Cannot  unbind proxy nodes for proxy '{}' because of existing  proxy nodes. Please remove or modify to different proxy for all gateway node configurations before deleting the proxy.", proxy_name)
                        } else {
                            format!("Database error while unbinding  proxy nodes for '{}': {}", proxy_name, sqlite_error)
                        }
                    } else {
                        format!("SQLite error: {}", sqlite_error)
                    }
                },
                _ => format!("Failed to  unbind proxy nodes for '{}': {}", proxy_name, e)
            };
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": error_message,
                "proxy_id": id
            }))
        }
    }
}

/// Checks if a proxy can use high-speed mode by verifying there are no duplicates
/// with the same listen address.
///
/// This is a helper function for the set_proxy endpoint to ensure the constraint
/// that high-speed mode can only be enabled when there is exactly one proxy
/// with a given listen address.
///
/// # Parameters
///
/// * `proxy` - The proxy being created or updated
///
/// # Returns
///
/// * `Ok(true)` - If high-speed mode is allowed for this proxy
/// * `Ok(false)` - If high-speed mode is not allowed due to duplicates
/// * `Err(DatabaseError)` - If there was a database error
#[allow(dead_code)]
async fn can_use_high_speed(proxy: &Proxy) -> Result<bool, DatabaseError> {
    // If high_speed is not enabled, we don't need to check
    if !proxy.high_speed {
        return Ok(true);
    }
    
    // Check if there are other proxies with the same listen address
    let has_duplicate = proxy_queries::has_duplicate_listen_address(
        &proxy.addr_listen,
        Some(&proxy.id)
    )?;
    
    Ok(!has_duplicate)
}
