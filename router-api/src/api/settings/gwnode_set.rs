//! # Gateway Node API Endpoints for Creating and Deleting
//!
//! This module provides HTTP endpoints for creating, updating, and deleting gateway node configurations.
//! It handles validating input data, checking dependencies, and performing cascading operations when needed.

use actix_web::{post, web, HttpResponse, Responder};
use super::{GatewayNode, gwnode_queries};
use super::{proxy_queries, gateway_queries};

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
///   "alt_target": "http://new-target:8080"
/// }
/// ```
#[post("/gwnode/set")]
pub async fn set_gateway_node(req_body: web::Json<GatewayNode>) -> impl Responder {
    let mut node = req_body.into_inner();
    
    // If no ID provided, generate a new one
    if node.id.is_empty() {
        node.id = gwnode_queries::generate_gateway_node_id();
    }
    
    // Verify that the referenced proxy exists
    match proxy_queries::get_proxy_by_id(&node.proxy_id) {
        Ok(Some(_)) => {
            // Proxy exists, proceed with saving the gateway node
            match gwnode_queries::save_gateway_node(&node) {
                Ok(_) => HttpResponse::Ok().json(node),
                Err(err) => {
                    log::error!("Failed to save gateway node: {}", err);
                    HttpResponse::InternalServerError().json(format!("Error: {}", err))
                }
            }
        },
        Ok(None) => {
            // Proxy does not exist
            log::error!("Cannot create gateway node: Proxy ID {} not found", node.proxy_id);
            HttpResponse::BadRequest().json(format!("Error: Proxy ID {} not found", node.proxy_id))
        },
        Err(err) => {
            // Error retrieving proxy
            log::error!("Failed to check proxy existence: {}", err);
            HttpResponse::InternalServerError().json(format!("Error: {}", err))
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
/// 1. First, it retrieves and deletes all gateways that reference the gateway node
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
pub async fn delete_gateway_node(req_body: web::Json<DeleteRequest>) -> impl Responder {
    let id = &req_body.id;
    
    // First, get all gateways associated with this gateway node
    match gateway_queries::get_gateways_by_gwnode_id(id) {
        Ok(gateways) => {
            let gateway_count = gateways.len();
            
            // Delete all associated gateways first
            for gateway in &gateways {
                if let Err(err) = gateway_queries::delete_gateway_by_id(&gateway.id) {
                    log::error!("Failed to delete associated gateway {}: {}", gateway.id, err);
                    return HttpResponse::InternalServerError()
                        .json(format!("Error deleting associated gateway {}: {}", gateway.id, err));
                }
            }
            
            // Now delete the gateway node itself
            match gwnode_queries::delete_gateway_node_by_id(id) {
                Ok(true) => {
                    if gateway_count > 0 {
                        HttpResponse::Ok().json(format!("Gateway node deleted successfully along with {} associated gateways", gateway_count))
                    } else {
                        HttpResponse::Ok().json("Gateway node deleted successfully")
                    }
                },
                Ok(false) => HttpResponse::NotFound().json("Gateway node not found"),
                Err(err) => {
                    log::error!("Failed to delete gateway node: {}", err);
                    HttpResponse::InternalServerError().json(format!("Error: {}", err))
                }
            }
        },
        Err(err) => {
            log::error!("Failed to retrieve associated gateways: {}", err);
            HttpResponse::InternalServerError().json(format!("Error: {}", err))
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