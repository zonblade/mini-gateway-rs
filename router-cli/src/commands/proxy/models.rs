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
