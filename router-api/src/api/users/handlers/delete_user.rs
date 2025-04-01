use actix_web::{delete, web, HttpResponse, Responder};
use crate::module::database::get_connection;
use std::sync::{Arc, Mutex};
use crate::client::Client;

// Delete a user
#[delete("/{user_id}")]
pub async fn init(
    path: web::Path<String>,
    _client: web::Data<Arc<Mutex<Client>>>
) -> impl Responder {
    let user_id = path.into_inner();

    let db = match get_connection() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Failed to connect to database"})
        ),
    };

    // First check if the user exists
    match db.query_one(
        "SELECT id FROM users WHERE id = ?",
        [&user_id],
        |row| row.get::<_, String>(0),
    ) {
        Ok(Some(_)) => {},
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
    }

    // Delete the user
    match db.execute(
        "DELETE FROM users WHERE id = ?",
        [&user_id],
    ) {
        Ok(_) => {
            HttpResponse::Ok().json(
                serde_json::json!({"message": "User successfully deleted"})
            )
        },
        Err(err) => {
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to delete user: {}", err)})
            )
        }
    }
}