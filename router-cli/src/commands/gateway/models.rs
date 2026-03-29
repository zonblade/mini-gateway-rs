use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Gateway {
    pub id: String,
    pub gwnode_id: String,
    pub pattern: String,
    pub target: String,
    pub priority: i32,
}
