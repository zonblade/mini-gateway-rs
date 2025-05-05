use super::proxydomain_queries;
use actix_web::{get, web, HttpResponse, Responder};

/// Get a proxy domain by ID
///
/// This endpoint returns a specific proxy domain configuration by its ID.
#[get("/proxydomain/{id}")]
pub async fn get_proxy_domain(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();

    match proxydomain_queries::get_proxy_domain_by_id(&id) {
        Ok(Some(domain)) => HttpResponse::Ok().json(domain),
        Ok(None) => HttpResponse::NotFound().body(format!("Proxy domain with ID {} not found", id)),
        Err(e) => {
            log::error!("Error fetching proxy domain {}: {}", id, e);
            HttpResponse::InternalServerError().body("Failed to fetch proxy domain")
        }
    }
}
