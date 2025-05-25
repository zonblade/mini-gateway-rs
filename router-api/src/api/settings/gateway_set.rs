//! # Gateway API Endpoints for Creating and Deleting
//!
//! This module provides HTTP endpoints for creating, updating, and deleting gateway routing configurations.
//! Gateways are the actual routing rules that define how incoming requests are matched and forwarded
//! based on patterns and priorities.

use actix_web::{post, web, HttpResponse, Responder, HttpRequest};
use super::{Gateway, gateway_queries, gwnode_queries};
use crate::api::users::helper::{ClaimsFromRequest, is_staff_or_admin};

/// Creates or updates a gateway routing rule
///
/// This endpoint processes HTTP POST requests to create new gateways or update
/// existing ones. It validates that the referenced gateway node exists before 
/// saving the gateway configuration.
///
/// # Endpoint
///
/// `POST /settings/gateway/set`
///
/// # Request Body
///
/// The request body should be a JSON object with the following fields:
/// - `id` (optional): The unique identifier for the gateway. If empty, a new ID will be generated.
/// - `gwnode_id`: The ID of the gateway node this gateway is associated with. Must reference an existing node.
/// - `pattern`: Pattern for URL matching (e.g., "/api/users/*", "^/users/[0-9]+").
/// - `target`: Target URL where matching requests should be routed.
/// - `priority`: Priority level, with lower numbers having higher precedence.
///
/// # Response
///
/// ## Success (200 OK)
/// Returns the saved gateway configuration as a JSON object, including any generated ID.
///
/// ## Bad Request (400)
/// Returned when the referenced gateway node does not exist.
///
/// ## Internal Server Error (500)
/// Returned when there is a database or server error.
///
/// # Pattern Matching
///
/// The `pattern` field supports various matching techniques:
/// - Exact path matching: "/api/users"
/// - Prefix matching with wildcard: "/api/*"
/// - Regex-like patterns: "^/users/[0-9]+"
///
/// # Priority System
///
/// When multiple patterns match an incoming request, the pattern with the lowest
/// `priority` value is selected. This allows for fine-grained control of routing
/// by creating specific rules (high priority/low number) that override more general
/// rules (low priority/high number).
///
/// # Examples
///
/// Create a new gateway:
/// ```
/// POST /settings/gateway/set
/// Content-Type: application/json
///
/// {
///   "gwnode_id": "7f9c24e5-1315-43a7-9f31-6eb9772cb46a",
///   "pattern": "/api/users/*",
///   "target": "http://user-service:8080",
///   "priority": 10
/// }
/// ```
///
/// Update an existing gateway:
/// ```
/// POST /settings/gateway/set
/// Content-Type: application/json
///
/// {
///   "id": "a1b2c3d4-e5f6-4321-8765-10293847abcd",
///   "gwnode_id": "7f9c24e5-1315-43a7-9f31-6eb9772cb46a",
///   "pattern": "/api/users/*",
///   "target": "http://new-user-service:9090",
///   "priority": 5
/// }
/// ```
#[post("/gateway/set")]
pub async fn set_gateway(
    req: HttpRequest,
    req_body: web::Json<Gateway>
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
            serde_json::json!({"error": "Only administrators and staff can modify gateway settings"})
        );
    }
    
    let mut gateway = req_body.into_inner();
    
    // If no ID provided, generate a new one
    if gateway.id.is_empty() {
        gateway.id = gateway_queries::generate_gateway_id();
    }
    
    // Verify that the referenced gateway node exists
    match gwnode_queries::get_gateway_node_by_id(&gateway.gwnode_id) {
        Ok(Some(_)) => {
            // Gateway node exists, proceed with saving the gateway
            match gateway_queries::save_gateway(&gateway) {
                Ok(_) => HttpResponse::Ok().json(gateway),
                Err(err) => {
                    log::error!("Failed to save gateway: {}", err);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Error: {}", err)
                    }))
                }
            }
        },
        Ok(None) => {
            // Gateway node does not exist
            log::error!("Cannot create gateway: Gateway Node ID {} not found", gateway.gwnode_id);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Gateway Node ID {} not found", gateway.gwnode_id)
            }))
        },
        Err(err) => {
            // Error retrieving gateway node
            log::error!("Failed to check gateway node existence: {}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Error: {}", err)
            }))
        }
    }
}

/// Deletes a gateway routing rule
///
/// This endpoint processes HTTP POST requests to delete gateway routing rules based
/// on their unique identifier.
///
/// # Endpoint
///
/// `POST /settings/gateway/delete`
///
/// # Request Body
///
/// The request body should be a JSON object with the following field:
/// - `id`: The unique identifier of the gateway to delete.
///
/// # Response
///
/// ## Success (200 OK)
/// Returns a success message if the gateway was found and deleted.
///
/// ## Not Found (404)
/// Returned when no gateway with the specified ID exists.
///
/// ## Internal Server Error (500)
/// Returned when there is a database or server error.
///
/// # Example
///
/// ```
/// POST /settings/gateway/delete
/// Content-Type: application/json
///
/// {
///   "id": "a1b2c3d4-e5f6-4321-8765-10293847abcd"
/// }
/// ```
#[post("/gateway/delete")]
pub async fn delete_gateway(
    req: HttpRequest,
    req_body: web::Json<DeleteRequest>
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
            serde_json::json!({"error": "Only administrators and staff can delete gateway settings"})
        );
    }
    
    let id = &req_body.id;
    
    match gateway_queries::delete_gateway_by_id(id) {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Gateway deleted successfully"
        })),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Gateway not found"
        })),
        Err(err) => {
            log::error!("Failed to delete gateway: {}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Error: {}", err)
            }))
        }
    }
}

/// Request body structure for delete operations
///
/// This structure defines the JSON schema for delete request bodies.
/// It is used by the `delete_gateway` endpoint to deserialize
/// the incoming JSON data.
#[derive(serde::Deserialize)]
pub struct DeleteRequest {
    /// The unique identifier of the item to delete
    pub id: String,
}