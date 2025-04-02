use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyNode {
    /// Whether TLS is enabled for this proxy node
    pub tls: bool,
    
    /// Server Name Indication value for TLS connections
    pub sni: Option<String>,
    
    /// Path to the TLS certificate PEM file
    pub tls_pem: Option<String>,
    
    /// Path to the TLS private key file
    pub tls_key: Option<String>,
    
    /// Network address this proxy listens on (e.g., "0.0.0.0:443")
    pub addr_listen: String,
    
    /// Target address to forward traffic to (e.g., "127.0.0.1:8080")
    pub addr_target: String,
}