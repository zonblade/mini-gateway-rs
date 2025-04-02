use actix_web::{post, web, HttpResponse, Responder, HttpRequest};
use crate::module::database::get_connection;
use crate::api::users::models::{User, CreateUserRequest, UserResponse, Role};
use crate::api::users::helper::{ClaimsFromRequest, is_admin};

// Create a new user - only admins can perform this action
#[post("")]
pub async fn init(
    req: HttpRequest,
    create_req: web::Json<CreateUserRequest>
) -> impl Responder {
    // Extract authenticated user's claims and verify admin role
    let claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to get user authentication"})
            )
        }
    };
    
    // Only admins can create users
    if !is_admin(&claims.role) {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"error": "Only administrators can create users"})
        );
    }

    let db = match get_connection() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Failed to connect to database"})
        ),
    };

    // Check if username or email already exists
    match db.query_one(
        "SELECT id FROM users WHERE username = ? OR email = ?",
        [&create_req.username, &create_req.email],
        |row| row.get::<_, String>(0),
    ) {
        Ok(Some(_)) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Username or email already exists"})
            );
        },
        Ok(None) => {},
        Err(err) => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Database error: {}", err)})
            );
        }
    }

    // Create the new user
    let role = create_req.role.clone().unwrap_or(Role::User);
    let new_user = User::new(
        create_req.username.clone(),
        create_req.email.clone(),
        create_req.password.clone(),
        role,
    );

    match db.execute(
        "INSERT INTO users (id, username, email, password_hash, role) VALUES (?, ?, ?, ?, ?)",
        [
            &new_user.id,
            &new_user.username,
            &new_user.email,
            &new_user.password_hash,
            &new_user.role.to_string(),
        ],
    ) {
        Ok(_) => {
            // Fetch the inserted user to get created_at and updated_at
            match db.query_one(
                "SELECT id, username, email, password_hash, role, created_at, updated_at FROM users WHERE id = ?",
                [&new_user.id],
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
                    HttpResponse::Created().json(UserResponse::from(user))
                },
                _ => {
                    // If we can't fetch the full user, return a simplified response
                    let response = UserResponse {
                        id: new_user.id,
                        username: new_user.username,
                        email: new_user.email,
                        role: new_user.role,
                        created_at: None,
                        updated_at: None,
                    };
                    HttpResponse::Created().json(response)
                }
            }
        },
        Err(err) => {
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to create user: {}", err)})
            )
        }
    }
}