use serde::{Deserialize, Serialize};

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
