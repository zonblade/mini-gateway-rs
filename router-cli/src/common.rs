use serde::{Deserialize, Serialize};

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
