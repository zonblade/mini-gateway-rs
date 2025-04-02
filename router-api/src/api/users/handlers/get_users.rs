use actix_web::{get, HttpResponse, Responder};
use crate::module::database::get_connection;
use crate::api::users::models::{User, UserResponse, Role};

// Get all users
#[get("")]
pub async fn init() -> impl Responder {
    let db = match get_connection() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Failed to connect to database"})
        ),
    };

    match db.query(
        "SELECT id, username, email, password_hash, role, created_at, updated_at FROM users",
        [],
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
        Ok(users) => {
            let user_responses: Vec<UserResponse> = users.into_iter()
                .map(UserResponse::from)
                .collect();
            HttpResponse::Ok().json(user_responses)
        },
        Err(err) => {
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Database error: {}", err)})
            )
        }
    }
}