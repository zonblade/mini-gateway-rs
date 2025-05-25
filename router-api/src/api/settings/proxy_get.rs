use super::{proxy_queries, proxydomain_queries};
use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;

/// Get a proxy by ID
///
/// This endpoint returns a specific proxy configuration by its ID,
/// along with all associated proxy domains.
#[get("/proxy/{id}")]
pub async fn get_proxy(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();

    match proxy_queries::get_proxy_by_id(&id) {
        Ok(Some(proxy)) => {
            // Fetch domains associated with this proxy
            match proxydomain_queries::get_proxy_domains_by_proxy_id(&id) {
                Ok(domains) => {
                    // Return combined proxy and domains
                    HttpResponse::Ok().json(json!({
                        "proxy": proxy,
                        "domains": domains
                    }))
                },
                Err(e) => {
                    log::error!("Error fetching domains for proxy {}: {}", id, e);
                    // Return proxy with empty domains and warning
                    HttpResponse::Ok().json(json!({
                        "proxy": proxy,
                        "domains": [],
                        "warning": "Could not fetch associated domains"
                    }))
                }
            }
        },
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Proxy with ID {} not found", id)
        })),
        Err(e) => {
            log::error!("Error fetching proxy {}: {}", id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch proxy"
            }))
        }
    }
}
