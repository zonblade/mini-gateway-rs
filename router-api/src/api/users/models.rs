use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Role {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "staff")]
    Staff,
    #[serde(rename = "user")]
    User,
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Role::Admin => "admin".to_string(),
            Role::Staff => "staff".to_string(),
            Role::User => "user".to_string(),
        }
    }
}

impl From<String> for Role {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "admin" => Role::Admin,
            "staff" => Role::Staff,
            _ => Role::User, // Default to user for any other value
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: Role,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl User {
    pub fn new(username: String, email: String, password: String, role: Role) -> Self {
        // In a real app, you would use a proper password hashing library like bcrypt
        // For this example, we'll just simulate a hash with a simple function
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            email,
            password_hash: format!("hashed_{}", password), // Simulated hash
            role,
            created_at: None,
            updated_at: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<Role>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<Role>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: Role,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}