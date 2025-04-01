// filepath: /Users/zonblade/Project/runegram/mini-gateway-rs/router-api/src/api/settings/gwnode_get.rs
use actix_web::{get, web, HttpResponse, Responder};
use super::gwnode_queries;

/// Get a gateway node by ID
///
/// Returns the gateway node configuration for the specified ID.
/// 
/// # Path Parameters
///
/// * `id` - The unique identifier of the gateway node to retrieve
#[get("/gwnode/{id}")]
pub async fn get_gateway_node(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    
    match gwnode_queries::get_gateway_node_by_id(&id) {
        Ok(Some(node)) => HttpResponse::Ok().json(node),
        Ok(None) => HttpResponse::NotFound().json("Gateway node not found"),
        Err(err) => {
            log::error!("Failed to get gateway node: {}", err);
            HttpResponse::InternalServerError().json(format!("Error: {}", err))
        }
    }
}