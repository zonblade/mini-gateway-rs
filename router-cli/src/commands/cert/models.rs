use serde::{Deserialize, Serialize};

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
