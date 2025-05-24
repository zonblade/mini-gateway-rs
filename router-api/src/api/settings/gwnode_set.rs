//! # Gateway Node API Endpoints for Creating and Deleting
//!
//! This module provides HTTP endpoints for creating, updating, and deleting gateway node configurations.
//! It handles validating input data, checking dependencies, and performing cascading operations when needed.

use actix_web::{post, web, HttpResponse, Responder, HttpRequest};
use super::{GatewayNode, gwnode_queries};
use super::{proxy_queries, gateway_queries};
use crate::api::users::helper::{ClaimsFromRequest, is_staff_or_admin};
use crate::module::database::DatabaseError;

/// Creates or updates a gateway node configuration
///
/// This endpoint processes HTTP POST requests to create new gateway nodes or update
/// existing ones. It validates that the referenced proxy exists before saving the
/// gateway node.
///
/// # Endpoint
///
/// `POST /settings/gwnode/set`
///
/// # Request Body
///
/// The request body should be a JSON object with the following fields:
/// - `id` (optional): The unique identifier for the gateway node. If empty, a new ID will be generated.
/// - `proxy_id`: The ID of the proxy this gateway node is associated with. Must reference an existing proxy.
/// - `title`: Human-readable name for this gateway node
/// - `alt_target`: Alternative target URL for routing.
///
/// # Response
///
/// ## Success (200 OK)
/// Returns the saved gateway node configuration as a JSON object, including any generated ID.
///
/// ## Bad Request (400)
/// Returned when the referenced proxy does not exist.
///
/// ## Internal Server Error (500)
/// Returned when there is a database or server error.
///
/// # Examples
///
/// Create a new gateway node:
/// ```
/// POST /settings/gwnode/set
/// Content-Type: application/json
///
/// {
///   "proxy_id": "550e8400-e29b-41d4-a716-446655440000",
///   "title": "API Backup Gateway",
///   "alt_target": "http://backup-server:8080"
/// }
/// ```
///
/// Update an existing gateway node:
/// ```
/// POST /settings/gwnode/set
/// Content-Type: application/json
///
/// {
///   "id": "7f9c24e5-1315-43a7-9f31-6eb9772cb46a",
///   "proxy_id": "550e8400-e29b-41d4-a716-446655440000",
///   "title": "Updated Gateway Name",
///   "alt_target": "http://new-target:8080"
/// }
/// ```
#[post("/gwnode/set")]
pub async fn set_gateway_node(
    req: HttpRequest,
    req_body: web::Json<GatewayNode>
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
            serde_json::json!({"error": "Only administrators and staff can modify gateway nodes"})
        );
    }
    
    let mut node = req_body.into_inner();
    
    // If no ID provided, generate a new one
    if node.id.is_empty() {
        node.id = gwnode_queries::generate_gateway_node_id();
    }
    
    // If no title provided, set a default one
    if node.title.is_empty() {
        node.title = format!("Gateway Node {}", &node.id[..8]);
    }

    // check if ip address is with port, if not, return error
    if !node.alt_target.contains(":") {
        return HttpResponse::BadRequest().json(
            serde_json::json!({"error": "Alt target must be a valid IP address with port"})
        );
    }

    // check if after : is a valid port 1 - 65535, return error if not
    if let Some(port) = node.alt_target.split(":").nth(1) {
        if port.parse::<u16>().is_err() {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Alt target must be a valid IP address with port"})
            );
        }
    }
    
    // Get proxy details for better error messages
    let proxy_name = match proxy_queries::get_proxy_by_id(&node.proxy_id) {
        Ok(Some(proxy)) => proxy.title,
        Ok(None) => node.proxy_id.clone(),
        Err(e) => {
            log::error!("Error retrieving proxy {}: {}", node.proxy_id, e);
            return HttpResponse::BadRequest().json(
                serde_json::json!({
                    "error": format!("Failed to verify proxy existence: {}", e),
                    "proxy_id": node.proxy_id
                })
            );
        }
    };
    
    // Verify that the referenced proxy exists
    match proxy_queries::get_proxy_by_id(&node.proxy_id) {
        Ok(Some(_)) => {
            // Proxy exists, proceed with saving the gateway node
            match gwnode_queries::save_gateway_node(&node) {
                Ok(_) => HttpResponse::Ok().json(node),
                Err(err) => {
                    log::error!("Failed to save gateway node: {}", err);
                    let error_message = match err {
                        DatabaseError::Sqlite(sqlite_error) => {
                            if let rusqlite::Error::SqliteFailure(err, _) = sqlite_error {
                                if err.code == rusqlite::ffi::ErrorCode::ConstraintViolation {
                                    format!("Cannot save gateway node '{}' because of database constraints. Please check if the proxy '{}' exists and is valid.", node.title, proxy_name)
                                } else {
                                    format!("Database error while saving gateway node '{}': {}", node.title, sqlite_error)
                                }
                            } else {
                                format!("SQLite error: {}", sqlite_error)
                            }
                        },
                        _ => format!("Failed to save gateway node '{}': {}", node.title, err)
                    };
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "error": error_message,
                        "gateway_node_id": node.id
                    }))
                }
            }
        },
        Ok(None) => {
            // Proxy does not exist
            log::error!("Cannot create gateway node: Proxy '{}' not found", proxy_name);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Cannot create gateway node: Proxy '{}' not found", proxy_name),
                "proxy_id": node.proxy_id
            }))
        },
        Err(err) => {
            // Error retrieving proxy
            log::error!("Failed to check proxy existence: {}", err);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to verify proxy existence: {}", err),
                "proxy_id": node.proxy_id
            }))
        }
    }
}

/// Deletes a gateway node and its associated gateways
///
/// This endpoint processes HTTP POST requests to delete gateway nodes. It implements
/// cascading deletion by first removing all gateways associated with the specified
/// gateway node before deleting the node itself.
///
/// # Endpoint
///
/// `POST /settings/gwnode/delete`
///
/// # Request Body
///
/// The request body should be a JSON object with the following field:
/// - `id`: The unique identifier of the gateway node to delete.
///
/// # Response
///
/// ## Success (200 OK)
/// Returns a success message, including information about how many associated gateways
/// were also deleted.
///
/// ## Not Found (404)
/// Returned when no gateway node with the specified ID exists.
///
/// ## Internal Server Error (500)
/// Returned when there is a database or server error, including failures during
/// the cascading deletion process.
///
/// # Cascading Deletion
///
/// This endpoint implements a two-step deletion process:
/// 1. First, it retrieves and deletes all gateways associated with the specified
/// gateway node
/// 2. Then, it deletes the gateway node itself
///
/// If any part of this process fails, the operation is aborted and an error is returned.
///
/// # Example
///
/// ```
/// POST /settings/gwnode/delete
/// Content-Type: application/json
///
/// {
///   "id": "7f9c24e5-1315-43a7-9f31-6eb9772cb46a"
/// }
/// ```
#[post("/gwnode/delete")]
pub async fn delete_gateway_node(
    req: HttpRequest,
    req_body: web::Json<DeleteRequest>
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
            serde_json::json!({"error": "Only administrators and staff can delete gateway nodes"})
        );
    }
    
    let id = &req_body.id;

    // Get gateway node details for better error messages
    let node_name = match gwnode_queries::get_gateway_node_by_id(id) {
        Ok(Some(node)) => node.title,
        Ok(None) => id.clone(),
        Err(e) => {
            log::error!("Error retrieving gateway node {}: {}", id, e);
            return HttpResponse::BadRequest().json(
                serde_json::json!({
                    "error": format!("Failed to verify gateway node existence: {}", e),
                    "gateway_node_id": id
                })
            );
        }
    };
    
    // First, get all gateways associated with this gateway node
    match gateway_queries::get_gateways_by_gwnode_id(id) {
        Ok(gateways) => {
            let gateway_count = gateways.len();
            
            // Delete all associated gateways first
            for gateway in &gateways {
                if let Err(err) = gateway_queries::delete_gateway_by_id(&gateway.id) {
                    log::error!("Failed to delete associated gateway {}: {}", gateway.id, err);
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": format!("Failed to delete associated gateway for '{}': {}", node_name, err),
                        "gateway_node_id": id,
                        "gateway_id": gateway.id
                    }));
                }
            }
            
            // Now delete the gateway node itself
            match gwnode_queries::delete_gateway_node_by_id(id) {
                Ok(true) => {
                    let message = if gateway_count > 0 {
                        format!("Gateway node '{}' deleted successfully along with {} associated gateways", node_name, gateway_count)
                    } else {
                        format!("Gateway node '{}' deleted successfully", node_name)
                    };
                    HttpResponse::Ok().json(serde_json::json!({
                        "message": message
                    }))
                },
                Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Gateway node '{}' not found", node_name),
                    "gateway_node_id": id
                })),
                Err(err) => {
                    log::error!("Failed to delete gateway node: {}", err);
                    let error_message = match err {
                        DatabaseError::Sqlite(sqlite_error) => {
                            if let rusqlite::Error::SqliteFailure(err, _) = sqlite_error {
                                if err.code == rusqlite::ffi::ErrorCode::ConstraintViolation {
                                    format!("Cannot delete gateway node '{}' because it is still referenced by other entities", node_name)
                                } else {
                                    format!("Database error while deleting gateway node '{}': {}", node_name, sqlite_error)
                                }
                            } else {
                                format!("SQLite error: {}", sqlite_error)
                            }
                        },
                        _ => format!("Failed to delete gateway node '{}': {}", node_name, err)
                    };
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "error": error_message,
                        "gateway_node_id": id
                    }))
                }
            }
        },
        Err(err) => {
            log::error!("Failed to retrieve associated gateways: {}", err);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to retrieve associated gateways for '{}': {}", node_name, err),
                "gateway_node_id": id
            }))
        }
    }
}

/// Request body structure for delete operations
///
/// This structure defines the JSON schema for delete request bodies.
/// It is used by the `delete_gateway_node` endpoint to deserialize
/// the incoming JSON data.
#[derive(serde::Deserialize)]
pub struct DeleteRequest {
    /// The unique identifier of the item to delete
    pub id: String,
}