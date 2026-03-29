use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Proxy {
    pub id: String,
    pub title: String,
    pub addr_listen: String,
    pub addr_target: String,
    pub high_speed: bool,
    pub high_speed_addr: Option<String>,
    pub high_speed_gwid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyDomain {
    pub id: String,
    pub proxy_id: Option<String>,
    pub tls: bool,
    pub tls_pem: Option<String>,
    pub tls_key: Option<String>,
    pub sni: Option<String>,
    pub tls_autron: Option<bool>,
    pub tls_mode: Option<String>,
    pub tls_email: Option<String>,
    pub expected_renew: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyWithDomains {
    pub proxy: Proxy,
    pub domains: Vec<ProxyDomainSummary>,
}

/// Abbreviated domain info returned in proxy list endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyDomainSummary {
    pub id: String,
    pub sni: Option<String>,
    pub tls: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyDetail {
    pub proxy: Proxy,
    pub domains: Vec<ProxyDomain>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyInput {
    pub proxy: Proxy,
    pub domains: Vec<ProxyDomain>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayNode {
    pub id: String,
    pub proxy_id: String,
    pub title: String,
    pub alt_target: String,
    pub priority: i32,
    pub domain_id: Option<String>,
    pub domain_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gateway {
    pub id: String,
    pub gwnode_id: String,
    pub pattern: String,
    pub target: String,
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CertGenerateRequest {
    pub domain: String,
    pub proxy_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staging: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CertGenerateResponse {
    pub status: String,
    pub message: String,
    pub domain: Option<String>,
    pub expected_renew: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SyncResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct IdBody {
    pub id: String,
}
