use actix_web::{get, web, HttpResponse, Responder};
use super::{Proxy, proxy_queries};

/// Get a proxy by ID
///
/// This endpoint returns a specific proxy configuration by its ID.
#[get("/proxy/{id}")]
pub async fn get_proxy(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    
    match proxy_queries::get_proxy_by_id(&id) {
        Ok(Some(proxy)) => HttpResponse::Ok().json(proxy),
        Ok(None) => HttpResponse::NotFound().body(format!("Proxy with ID {} not found", id)),
        Err(e) => {
            eprintln!("Error fetching proxy {}: {}", id, e);
            HttpResponse::InternalServerError().body("Failed to fetch proxy")
        }
    }
}