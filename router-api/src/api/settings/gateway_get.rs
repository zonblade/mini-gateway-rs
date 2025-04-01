/// service: gateway, action: get, queryparam: id
/// id is sha256 of target_path and source_path

// filepath: /Users/zonblade/Project/runegram/mini-gateway-rs/router-api/src/api/settings/gateway_get.rs
use actix_web::{get, web, HttpResponse, Responder};
use super::gateway_queries;

/// Get a gateway by ID
///
/// Returns the gateway configuration for the specified ID.
/// 
/// # Path Parameters
///
/// * `id` - The unique identifier of the gateway to retrieve
#[get("/gateway/{id}")]
pub async fn get_gateway(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    
    match gateway_queries::get_gateway_by_id(&id) {
        Ok(Some(gateway)) => HttpResponse::Ok().json(gateway),
        Ok(None) => HttpResponse::NotFound().json("Gateway not found"),
        Err(err) => {
            log::error!("Failed to get gateway: {}", err);
            HttpResponse::InternalServerError().json(format!("Error: {}", err))
        }
    }
}