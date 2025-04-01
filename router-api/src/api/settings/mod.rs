pub mod proxy_get;
pub mod proxy_list;
pub mod proxy_set;
pub mod proxy_queries;

use serde::{Deserialize, Serialize};

/// Represents a proxy configuration in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proxy {
    /// Unique identifier for the proxy
    pub id: String,
    /// Human-readable title for the proxy
    pub title: String,
    /// Address where the proxy listens for incoming connections
    pub addr_listen: String,
    /// Target address where requests are forwarded to
    pub addr_target: String,
    /// Whether TLS is enabled for this proxy
    pub tls: bool,
    /// PEM certificate content for TLS
    pub tls_pem: Option<String>,
    /// Private key content for TLS
    pub tls_key: Option<String>,
    /// Whether automatic TLS is enabled
    pub tls_autron: bool,
    /// Server Name Indication value for TLS
    pub sni: Option<String>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settings")
            .service(proxy_list::list_proxies)
            .service(proxy_get::get_proxy)
            .service(proxy_set::set_proxy)
            .service(proxy_set::delete_proxy),
    );
}

// Import actix-web components for the configure function
use actix_web::web;