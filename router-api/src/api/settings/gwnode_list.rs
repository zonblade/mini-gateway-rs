// filepath: /Users/zonblade/Project/runegram/mini-gateway-rs/router-api/src/api/settings/gwnode_list.rs
use actix_web::{get, web, HttpResponse, Responder};
use super::gwnode_queries;

/// List all gateway nodes
///
/// Returns a JSON array of all configured gateway nodes.
#[get("/gwnode/list")]
pub async fn list_gateway_nodes() -> impl Responder {
    match gwnode_queries::get_all_gateway_nodes() {
        Ok(nodes) => HttpResponse::Ok().json(nodes),
        Err(err) => {
            log::error!("Failed to list gateway nodes: {}", err);
            HttpResponse::InternalServerError().json(format!("Error: {}", err))
        }
    }
}

/// List all gateway nodes for a specific proxy
///
/// Returns a JSON array of all gateway nodes associated with the specified proxy ID.
///
/// # Path Parameters
///
/// * `proxy_id` - The ID of the proxy to list gateway nodes for
#[get("/gwnode/list/{proxy_id}")]
pub async fn list_gateway_nodes_by_proxy(path: web::Path<String>) -> impl Responder {
    let proxy_id = path.into_inner();
    
    match gwnode_queries::get_gateway_nodes_by_proxy_id(&proxy_id) {
        Ok(nodes) => HttpResponse::Ok().json(nodes),
        Err(err) => {
            log::error!("Failed to list gateway nodes for proxy {}: {}", proxy_id, err);
            HttpResponse::InternalServerError().json(format!("Error: {}", err))
        }
    }
}