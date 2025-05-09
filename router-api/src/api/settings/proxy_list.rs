use super::{proxy_queries, proxydomain_queries};
use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

/// List all proxies in the system
///
/// This endpoint returns a list of all configured proxies
/// along with their associated domains (simplified to ID, SNI and TLS status only).
#[get("/proxies")]
pub async fn list_proxies() -> impl Responder {
    match proxy_queries::get_all_proxies() {
        Ok(proxies) => {
            // Create a vector to hold combined proxy+domains results
            let mut result = Vec::new();
            
            // For each proxy, fetch its domains
            for proxy in proxies {
                match proxydomain_queries::get_proxy_domains_by_proxy_id(&proxy.id) {
                    Ok(domains) => {
                        // Create simplified domain representations with only id, sni and tls status
                        let simplified_domains: Vec<_> = domains.iter().map(|domain| {
                            json!({
                                "id": domain.id,
                                "sni": domain.sni,
                                "tls": domain.tls
                            })
                        }).collect();
                        
                        // Add proxy with its simplified domains to result
                        result.push(json!({
                            "proxy": proxy,
                            "domains": simplified_domains
                        }));
                    },
                    Err(e) => {
                        log::error!("Error fetching domains for proxy {}: {}", proxy.id, e);
                        // Add proxy with empty domains and warning
                        result.push(json!({
                            "proxy": proxy,
                            "domains": [],
                            "warning": "Could not fetch associated domains"
                        }));
                    }
                }
            }
            
            HttpResponse::Ok().json(result)
        },
        Err(e) => {
            log::error!("Failed to list proxies: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch proxies")
        }
    }
}
