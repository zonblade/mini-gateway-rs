use actix_web::{get, web, HttpResponse, Responder};
use super::{Proxy, proxy_queries};

/// List all proxies in the system
///
/// This endpoint returns a list of all configured proxies.
#[get("/proxies")]
pub async fn list_proxies() -> impl Responder {
    match proxy_queries::get_all_proxies() {
        Ok(proxies) => HttpResponse::Ok().json(proxies),
        Err(e) => {
            eprintln!("Error fetching proxies: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch proxies")
        }
    }
}
