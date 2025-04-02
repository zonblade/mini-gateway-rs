use super::proxy_queries;
use actix_web::{get, HttpResponse, Responder};

/// List all proxies in the system
///
/// This endpoint returns a list of all configured proxies.
#[get("/proxies")]
pub async fn list_proxies() -> impl Responder {
    match proxy_queries::get_all_proxies() {
        Ok(proxies) => HttpResponse::Ok().json(proxies),
        Err(e) => {
            log::error!("Failed to list proxies: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch proxies")
        }
    }
}
