use actix_web::{get, web, HttpResponse, Responder};
use crate::module::database::get_connection;
use crate::api::users::models::{User, UserResponse, Role};

// Get a specific user by ID
#[get("/{user_id}")]
pub async fn init(
    path: web::Path<String>
) -> impl Responder {
    let user_id = path.into_inner();

    let db = match get_connection() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Failed to connect to database"})
        ),
    };

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
                serde_json::json!({"error": "User not found"})
            )
        },
        Err(err) => {
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Database error: {}", err)})
            )
        }
    }
}