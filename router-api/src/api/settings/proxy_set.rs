use actix_web::{delete, post, web, HttpResponse, Responder};
use super::{Proxy, proxy_queries};
use uuid::Uuid;

/// Create or update a proxy configuration
///
/// This endpoint creates a new proxy or updates an existing one.
/// If the provided proxy has an ID, it will update the existing proxy.
/// If no ID is provided, a new one will be generated.
/// The target address is automatically generated with IP 127.0.0.1 and a random available port.
#[post("/proxy")]
pub async fn set_proxy(proxy: web::Json<Proxy>) -> impl Responder {
    let mut proxy = proxy.into_inner();
    
    // Generate an ID if none was provided
    if proxy.id.is_empty() {
        proxy.id = Uuid::new_v4().to_string();
    }
    
    // Generate a target address with random available port
    match proxy_queries::generate_target_address() {
        Ok(addr) => {
            proxy.addr_target = addr;
            
            match proxy_queries::save_proxy(&proxy) {
                Ok(()) => HttpResponse::Ok().json(proxy),
                Err(e) => {
                    eprintln!("Error saving proxy: {}", e);
                    HttpResponse::InternalServerError().body("Failed to save proxy")
                }
            }
        },
        Err(e) => {
            eprintln!("Error generating target address: {}", e);
            HttpResponse::InternalServerError().body("Failed to generate target address")
        }
    }
}

/// Delete a proxy by ID
///
/// This endpoint deletes a proxy configuration by its ID.
#[delete("/proxy/{id}")]
pub async fn delete_proxy(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    
    match proxy_queries::delete_proxy_by_id(&id) {
        Ok(deleted) => {
            if deleted {
                HttpResponse::Ok().body(format!("Proxy with ID {} deleted", id))
            } else {
                HttpResponse::NotFound().body(format!("Proxy with ID {} not found", id))
            }
        },
        Err(e) => {
            eprintln!("Error deleting proxy {}: {}", id, e);
            HttpResponse::InternalServerError().body("Failed to delete proxy")
        }
    }
}
