//! # Gateway Listing API Endpoints
//!
//! This module provides HTTP endpoints for listing gateway routing configurations,
//! either retrieving all gateways in the system or filtering by a specific gateway node.
//! These endpoints are read-only and do not modify any data.

use actix_web::{get, web, HttpResponse, Responder};
use super::gateway_queries;

/// Lists all gateway routing rules
///
/// This endpoint retrieves all gateway configurations from the database and returns
/// them as a JSON array, ordered by priority (lower number = higher priority).
///
/// # Endpoint
///
/// `GET /settings/gateway/list`
///
/// # Response
///
/// ## Success (200 OK)
/// Returns a JSON array of all gateway configurations with the following structure:
/// ```json
/// [
///   {
///     "id": "a1b2c3d4-e5f6-4321-8765-10293847abcd",
///     "gwnode_id": "7f9c24e5-1315-43a7-9f31-6eb9772cb46a",
///     "pattern": "/api/users/*",
///     "target": "http://user-service:8080",
///     "priority": 10
///   },
///   {
///     "id": "b2c3d4e5-f6a7-5432-8765-10293847dcba",
///     "gwnode_id": "8f9c24e5-1415-43a7-9f31-6eb9772cb46b",
///     "pattern": "/api/products/*",
///     "target": "http://product-service:8080",
///     "priority": 20
///   }
/// ]
/// ```
///
/// ## Internal Server Error (500)
/// Returned when there is a database or server error.
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
/// GET /settings/gateway/list
/// ```
#[get("/gateway/list")]
pub async fn list_gateways() -> impl Responder {
    match gateway_queries::get_all_gateways() {
        Ok(gateways) => HttpResponse::Ok().json(gateways),
        Err(err) => {
            log::error!("Failed to list gateways: {}", err);
            HttpResponse::InternalServerError().json(format!("Error: {}", err))
        }
    }
}

/// Lists all gateway routing rules for a specific gateway node
///
/// This endpoint retrieves all gateway configurations associated with a specific
/// gateway node and returns them as a JSON array, ordered by priority (lower number = higher priority).
///
/// # Endpoint
///
/// `GET /settings/gateway/list/{gwnode_id}`
///
/// # Path Parameters
///
/// * `gwnode_id` - The ID of the gateway node to list gateways for
///
/// # Response
///
/// ## Success (200 OK)
/// Returns a JSON array of gateway configurations that belong to the specified gateway node,
/// with the same structure as the `list_gateways` endpoint. If no gateways are found for the
/// gateway node, an empty array is returned.
///
/// ## Internal Server Error (500)
/// Returned when there is a database or server error.
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
/// GET /settings/gateway/list/7f9c24e5-1315-43a7-9f31-6eb9772cb46a
/// ```
#[get("/gateway/list/{gwnode_id}")]
pub async fn list_gateways_by_gwnode(path: web::Path<String>) -> impl Responder {
    let gwnode_id = path.into_inner();
    
    match gateway_queries::get_gateways_by_gwnode_id(&gwnode_id) {
        Ok(gateways) => HttpResponse::Ok().json(gateways),
        Err(err) => {
            log::error!("Failed to list gateways for gateway node {}: {}", gwnode_id, err);
            HttpResponse::InternalServerError().json(format!("Error: {}", err))
        }
    }
}