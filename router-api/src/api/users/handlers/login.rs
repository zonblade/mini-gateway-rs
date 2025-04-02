use actix_web::{post, web, HttpResponse, Responder};
use crate::module::database::get_connection;
use crate::api::users::models::{User, Role};
use crate::api::users::helper::{AuthConfig, generate_token};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::client::Client;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub role: Option<String>,
    pub message: String,
}

#[post("/login")]
pub async fn init(
    login_req: web::Json<LoginRequest>,
    _client: web::Data<Arc<Mutex<Client>>>
) -> impl Responder {
    let db = match get_connection() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Failed to connect to database"})
        ),
    };

    // Find user by username
    match db.query_one(
        "SELECT id, username, email, password_hash, role, created_at, updated_at FROM users WHERE username = ?",
        [&login_req.username],
        |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                password_hash: row.get(3)?,
                role: Role::from(row.get::<_, String>(4)?),
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    ) {
        Ok(Some(user)) => {
            // In a real application, you would use a proper password verification
            // Here we just compare with our simple hashed password
            let expected_hash = format!("hashed_{}", login_req.password);
            
            if user.password_hash == expected_hash {
                // Create JWT token
                let auth_config = AuthConfig::default();
                match generate_token(&user, &auth_config) {
                    Ok(token) => {
                        let response = LoginResponse {
                            success: true,
                            token: Some(token),
                            user_id: Some(user.id),
                            username: Some(user.username),
                            role: Some(user.role.to_string()),
                            message: "Login successful".to_string(),
                        };
                        HttpResponse::Ok().json(response)
                    },
                    Err(_) => {
                        HttpResponse::InternalServerError().json(
                            serde_json::json!({"error": "Failed to generate token"})
                        )
                    }
                }
            } else {
                // Password doesn't match
                let response = LoginResponse {
                    success: false,
                    token: None,
                    user_id: None,
                    username: None,
                    role: None,
                    message: "Invalid username or password".to_string(),
                };
                HttpResponse::Unauthorized().json(response)
            }
        },
        Ok(None) => {
            // User not found
            let response = LoginResponse {
                success: false,
                token: None,
                user_id: None,
                username: None,
                role: None,
                message: "Invalid username or password".to_string(),
            };
            HttpResponse::Unauthorized().json(response)
        },
        Err(err) => {
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Database error: {}", err)})
            )
        }
    }
}