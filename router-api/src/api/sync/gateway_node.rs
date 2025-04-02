use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayNode {
    /// Processing priority (higher values = higher priority)
    pub priority: i8,
    
    /// Network address this gateway listens on (e.g., "0.0.0.0:80")
    pub addr_listen: String,
    
    /// Target address to forward traffic to (e.g., "127.0.0.1:8080")
    pub addr_target: String,
    
    /// URI path pattern to match incoming requests against (e.g., "/api/*")
    pub path_listen: String,
    
    /// Target path to rewrite matched paths to (e.g., "/")
    pub path_target: String,
}
