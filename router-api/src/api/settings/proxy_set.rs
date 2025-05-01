//! # Proxy API Endpoints for Creating and Deleting
//!
//! This module provides HTTP endpoints for creating, updating, and deleting proxy configurations.
//! Proxies are the foundation of the gateway system, listening on specific addresses and forwarding
//! traffic to target destinations.

use actix_web::{delete, post, web, HttpResponse, Responder, HttpRequest};
use super::{Proxy, proxy_queries};
use super::gwnode_queries;
use uuid::Uuid;
use crate::api::users::helper::{ClaimsFromRequest, is_staff_or_admin};
use crate::module::database::DatabaseError;

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
/// - `tls` (optional): Whether TLS is enabled for incoming connections (default: false).
/// - `tls_pem` (optional): PEM certificate content when TLS is manually configured.
/// - `tls_key` (optional): Private key content when TLS is manually configured.
/// - `tls_autron` (optional): Whether automatic TLS certificate provisioning is enabled (default: false).
/// - `sni` (optional): Server Name Indication value for TLS negotiation.
/// - `high_speed` (optional): Whether speed mode is enabled for faster proxying (default: false).
/// - `high_speed_addr` (optional): Specific address to use for speed mode.
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
    proxy: web::Json<Proxy>
) -> impl Responder {
    // Extract authenticated user's claims
    let claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return HttpResponse::InternalServerError().json(
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
    
    let mut proxy = proxy.into_inner();
    
    // Generate an ID if none was provided
    if proxy.id.is_empty() {
        proxy.id = Uuid::new_v4().to_string();
    }
    
    // Generate a target address with random available port
    match proxy_queries::generate_target_address() {
        Ok(addr) => {
            proxy.addr_target = addr;
            
            // Check if high_speed can be enabled for this proxy
            if proxy.high_speed {
                match proxy_queries::has_duplicate_listen_address(&proxy.addr_listen, Some(&proxy.id)) {
                    Ok(has_duplicate) => {
                        if has_duplicate {
                            return HttpResponse::BadRequest().json(serde_json::json!({
                                "error": "Cannot enable high-speed mode for this proxy because there are multiple proxies with the same listen address."
                            }));
                        }
                        // If high_speed is enabled but high_speed_addr is empty, set it to the same as addr_target
                        if proxy.high_speed_addr.is_none() || proxy.high_speed_addr.as_ref().unwrap().is_empty() {
                            proxy.high_speed_addr = Some(proxy.addr_target.clone());
                        }
                    },
                    Err(e) => {
                        log::error!("Error checking for duplicate listen addresses: {}", e);
                        return HttpResponse::InternalServerError().body("Failed to check for duplicate listen addresses");
                    }
                }
            } else {
                // If high_speed is disabled, set high_speed_addr to None
                proxy.high_speed_addr = None;
            }
            
            match proxy_queries::save_proxy(&proxy) {
                Ok(()) => HttpResponse::Ok().json(proxy),
                Err(e) => {
                    log::error!("Error saving proxy {}: {}", proxy.id, e);
                    HttpResponse::InternalServerError().body("Failed to save proxy")
                }
            }
        },
        Err(e) => {
            log::error!("Error generating target address: {}", e);
            HttpResponse::InternalServerError().body("Failed to generate target address")
        }
    }
}

/// Deletes a proxy configuration by ID
///
/// This endpoint processes HTTP DELETE requests to remove proxy configurations.
/// Rather than cascading deletion, it implements a "soft unbinding" approach
/// for associated gateway nodes by marking them as "unbound" but preserving
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
/// information about how many gateway nodes were unbound in the process.
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
/// 1. First, it finds all gateway nodes associated with the proxy and marks them as "unbound"
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
            return HttpResponse::InternalServerError().json(
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
    
    // First unbind any gateway nodes associated with this proxy
    match gwnode_queries::unbind_gateway_nodes_by_proxy_id(&id) {
        Ok(unbound_count) => {
            // Now delete the proxy
            match proxy_queries::delete_proxy_by_id(&id) {
                Ok(deleted) => {
                    if deleted {
                        let message = if unbound_count > 0 {
                            format!("Proxy with ID {} deleted. {} gateway nodes were unbound.", id, unbound_count)
                        } else {
                            format!("Proxy with ID {} deleted.", id)
                        };
                        HttpResponse::Ok().body(message)
                    } else {
                        HttpResponse::NotFound().body(format!("Proxy with ID {} not found", id))
                    }
                },
                Err(e) => {
                    log::error!("Error deleting proxy {}: {}", id, e);
                    HttpResponse::InternalServerError().body("Failed to delete proxy")
                }
            }
        },
        Err(e) => {
            log::error!("Error unbinding gateway nodes for proxy {}: {}", id, e);
            HttpResponse::InternalServerError().body("Failed to unbind gateway nodes")
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
