use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::module::database::get_connection;
use crate::api::users::helper::{ClaimsFromRequest, can_modify_user};

// Delete a user
pub async fn init(
    req: HttpRequest,
    path: web::Path<String>
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
    
    // Check if the user is allowed to delete this user using the can_modify_user helper
    if !can_modify_user(&claims.sub, &claims.role, &user_id) {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"error": "You are not authorized to delete this user"})
        );
    }

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