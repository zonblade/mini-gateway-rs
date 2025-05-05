use super::proxydomain_queries;
use actix_web::{get, web, HttpResponse, Responder};

/// List all proxy domains in the system
///
/// This endpoint returns a list of all configured proxy domains.
#[get("/proxydomains")]
pub async fn list_proxy_domains() -> impl Responder {
    match proxydomain_queries::get_all_proxy_domains() {
        Ok(domains) => HttpResponse::Ok().json(domains),
        Err(e) => {
            log::error!("Failed to list proxy domains: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch proxy domains")
        }
    }
}

/// List proxy domains for a specific proxy
///
/// This endpoint returns a list of proxy domains filtered by the proxy ID.
#[get("/proxydomains/proxy/{proxy_id}")]
pub async fn list_proxy_domains_by_proxy(proxy_id: web::Path<String>) -> impl Responder {
    let proxy_id = proxy_id.into_inner();
    
    match proxydomain_queries::get_proxy_domains_by_proxy_id(&proxy_id) {
        Ok(domains) => HttpResponse::Ok().json(domains),
        Err(e) => {
            log::error!("Failed to list proxy domains for proxy {}: {}", proxy_id, e);
            HttpResponse::InternalServerError().body("Failed to fetch proxy domains")
        }
    }
}
