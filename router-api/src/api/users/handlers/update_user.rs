use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::module::database::get_connection;
use crate::api::users::models::{User, UpdateUserRequest, UserResponse, Role};
use crate::api::users::helper::{ClaimsFromRequest, is_admin};
use std::sync::{Arc, Mutex};
use crate::client::Client;

pub async fn init(
    req: HttpRequest,
    path: web::Path<String>,
    update_req: web::Json<UpdateUserRequest>,
    _client: web::Data<Arc<Mutex<Client>>>
) -> impl Responder {
    let user_id = path.into_inner();
    
    // Extract authenticated user's claims
    let claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to get user authentication"})
            )
        }
    };
    
    // Check if user is updating their own account or is an admin
    let is_self = claims.sub == user_id;
    let is_admin_user = is_admin(&claims.role);
    
    if !is_self && !is_admin_user {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"error": "You can only update your own account"})
        );
    }
    
    // Create a vector to hold any dynamically created strings
    let mut constructed_values: Vec<String> = Vec::new();
    
    // Check if this is a role update attempt and if the user has permissions
    if update_req.role.is_some() {
        // Prevent users from upgrading their own role
        if is_self {
            return HttpResponse::Forbidden().json(
                serde_json::json!({"error": "Cannot update your own role"})
            );
        }
        
        // Only admins can update roles
        if !is_admin_user {
            return HttpResponse::Forbidden().json(
                serde_json::json!({"error": "Only administrators can update user roles"})
            );
        }
    }

    let db = match get_connection() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Failed to connect to database"})
        ),
    };

    // Check if user exists
    let existing_user = match db.query_one(
        "SELECT id, username, email, password_hash, role, created_at, updated_at FROM users WHERE id = ?",
        [&user_id],
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
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::NotFound().json(
                serde_json::json!({"error": "User not found"})
            );
        },
        Err(err) => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Database error: {}", err)})
            );
        }
    };

    // Check if the updated username is already taken
    if let Some(ref username) = update_req.username {
        if username != &existing_user.username {
            match db.query_one(
                "SELECT id FROM users WHERE username = ? AND id != ?",
                [username, &user_id],
                |row| row.get::<_, String>(0),
            ) {
                Ok(Some(_)) => {
                    return HttpResponse::BadRequest().json(
                        serde_json::json!({"error": "Username already exists"})
                    );
                },
                Ok(None) => {},
                Err(err) => {
                    return HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": format!("Database error: {}", err)})
                    );
                }
            }
        }
    }

    // Check if the updated email is already taken
    if let Some(ref email) = update_req.email {
        if email != &existing_user.email {
            match db.query_one(
                "SELECT id FROM users WHERE email = ? AND id != ?",
                [email, &user_id],
                |row| row.get::<_, String>(0),
            ) {
                Ok(Some(_)) => {
                    return HttpResponse::BadRequest().json(
                        serde_json::json!({"error": "Email already exists"})
                    );
                },
                Ok(None) => {},
                Err(err) => {
                    return HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": format!("Database error: {}", err)})
                    );
                }
            }
        }
    }

    // Build update parameters
    let mut params = Vec::new();
    let mut query_parts = Vec::new();

    if let Some(username) = &update_req.username {
        query_parts.push("username = ?");
        params.push(username as &dyn rusqlite::ToSql);
    }

    if let Some(email) = &update_req.email {
        query_parts.push("email = ?");
        params.push(email as &dyn rusqlite::ToSql);
    }

    if let Some(password) = &update_req.password {
        let password_hash = format!("hashed_{}", password); // Simulated hash
        constructed_values.push(password_hash);
        query_parts.push("password_hash = ?");
    }
    
    if let Some(role) = &update_req.role {
        let role_str = role.to_string();
        constructed_values.push(role_str);
        query_parts.push("role = ?");
    }

    for (i, part) in query_parts.iter().enumerate() {
        if part.contains("password_hash") || part.contains("role") {
            params.push(&constructed_values[i - (query_parts.len() - constructed_values.len())] as &dyn rusqlite::ToSql);
        }
    }

    // Always update the updated_at timestamp
    query_parts.push("updated_at = CURRENT_TIMESTAMP");

    if query_parts.is_empty() {
        return HttpResponse::BadRequest().json(
            serde_json::json!({"error": "No fields to update"})
        );
    }

    let query = format!(
        "UPDATE users SET {} WHERE id = ?",
        query_parts.join(", ")
    );
    
    // Add id parameter for WHERE clause
    params.push(&user_id as &dyn rusqlite::ToSql);

    match db.execute(&query, rusqlite::params_from_iter(params.iter())) {
        Ok(_) => {
            match db.query_one(
                "SELECT id, username, email, password_hash, role, created_at, updated_at FROM users WHERE id = ?",
                [&user_id],
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
                    HttpResponse::Ok().json(UserResponse::from(user))
                },
                Ok(None) => {
                    HttpResponse::NotFound().json(
                        serde_json::json!({"error": "User not found after update"})
                    )
                },
                Err(err) => {
                    HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": format!("Failed to fetch updated user: {}", err)})
                    )
                }
            }
        },
        Err(err) => {
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to update user: {}", err)})
            )
        }
    }
}