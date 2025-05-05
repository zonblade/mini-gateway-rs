use actix_web::{delete, post, web, HttpResponse, Responder, HttpRequest};
use super::{ProxyDomain, proxydomain_queries};
use super::{proxy_queries, gwnode_queries};
use uuid::Uuid;
use crate::api::users::helper::{ClaimsFromRequest, is_staff_or_admin};

/// Creates or updates a proxy domain configuration
///
/// This endpoint processes HTTP POST requests to create new proxy domains or update
/// existing ones. It validates that the referenced proxy and gateway node exist.
///
/// # Endpoint
///
/// `POST /settings/proxydomain`
///
/// # Request Body
///
/// The request body should be a JSON object with the following fields:
/// - `id` (optional): The unique identifier for the proxy domain. If empty, a new ID will be generated.
/// - `proxy_id`: The ID of the proxy this domain is associated with. Must reference an existing proxy.
/// - `gwnode_id`: The ID of the gateway node to route requests to (optional).
/// - `tls`: Whether TLS is enabled for this domain.
/// - `tls_pem` (optional): PEM certificate content when TLS is manually configured.
/// - `tls_key` (optional): Private key content when TLS is manually configured.
/// - `sni` (optional): Server Name Indication value for TLS negotiation.
///
/// # Response
///
/// ## Success (200 OK)
/// Returns the saved proxy domain configuration as a JSON object, including any generated ID.
///
/// ## Bad Request (400)
/// Returned when the referenced proxy or gateway node does not exist.
///
/// ## Internal Server Error (500)
/// Returned when there is a database or server error.
#[post("/proxydomain")]
pub async fn set_proxy_domain(
    req: HttpRequest,
    domain: web::Json<ProxyDomain>
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
            serde_json::json!({"error": "Only administrators and staff can modify proxy domains"})
        );
    }
    
    let mut domain = domain.into_inner();
    
    // Generate an ID if none was provided
    if domain.id.is_empty() {
        domain.id = Uuid::new_v4().to_string();
    }
    
    // Verify that the referenced proxy exists
    match proxy_queries::get_proxy_by_id(&domain.proxy_id) {
        Ok(Some(_)) => {}, // Proxy exists, continue
        Ok(None) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": format!("Proxy with ID {} does not exist", domain.proxy_id)})
            );
        },
        Err(e) => {
            log::error!("Error checking proxy existence: {}", e);
            return HttpResponse::InternalServerError().body("Failed to verify proxy");
        }
    }
    
    // If gwnode_id is provided, verify that it exists
    if !domain.gwnode_id.is_empty() && domain.gwnode_id != "default" {
        match gwnode_queries::get_gateway_node_by_id(&domain.gwnode_id) {
            Ok(Some(_)) => {}, // Gateway node exists, continue
            Ok(None) => {
                return HttpResponse::BadRequest().json(
                    serde_json::json!({"error": format!("Gateway node with ID {} does not exist", domain.gwnode_id)})
                );
            },
            Err(e) => {
                log::error!("Error checking gateway node existence: {}", e);
                return HttpResponse::InternalServerError().body("Failed to verify gateway node");
            }
        }
    }
    
    // Save the proxy domain
    match proxydomain_queries::save_proxy_domain(&domain) {
        Ok(()) => HttpResponse::Ok().json(domain),
        Err(e) => {
            log::error!("Error saving proxy domain {}: {}", domain.id, e);
            HttpResponse::InternalServerError().body("Failed to save proxy domain")
        }
    }
}

/// Deletes a proxy domain configuration by ID
///
/// This endpoint processes HTTP DELETE requests to remove proxy domain configurations.
///
/// # Endpoint
///
/// `DELETE /settings/proxydomain/{id}`
///
/// # Path Parameters
///
/// * `id` - The unique identifier of the proxy domain to delete
///
/// # Response
///
/// ## Success (200 OK)
/// Returns a message confirming the proxy domain was deleted.
///
/// ## Not Found (404)
/// Returned when the specified proxy domain ID does not exist.
///
/// ## Internal Server Error (500)
/// Returned when there is a database or server error.
#[delete("/proxydomain/{id}")]
pub async fn delete_proxy_domain(
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
            serde_json::json!({"error": "Only administrators and staff can delete proxy domains"})
        );
    }
    
    let id = path.into_inner();
    
    // Delete the proxy domain
    match proxydomain_queries::delete_proxy_domain_by_id(&id) {
        Ok(true) => HttpResponse::Ok().body(format!("Proxy domain with ID {} deleted", id)),
        Ok(false) => HttpResponse::NotFound().body(format!("Proxy domain with ID {} not found", id)),
        Err(e) => {
            log::error!("Error deleting proxy domain {}: {}", id, e);
            HttpResponse::InternalServerError().body("Failed to delete proxy domain")
        }
    }
}
