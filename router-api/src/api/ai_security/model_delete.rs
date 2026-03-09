//! # AI Model Delete Endpoint
//!
//! This module provides the HTTP endpoint for deleting AI models.

use super::ai_model_queries;
use actix_web::{delete, web, HttpResponse, Responder};
use std::fs;

/// Deletes an AI model
///
/// # Endpoint
///
/// `DELETE /ai-security/model/{id}`
///
/// # Parameters
///
/// * `id` - The unique identifier of the AI model to delete
///
/// # Response
///
/// ## Success (200 OK)
/// Returns a success message.
///
/// ## Not Found (404)
/// Returned when no model with the given ID exists.
///
/// ## Internal Server Error (500)
/// Returned when there is a database or file system error.
#[delete("/model/{id}")]
pub async fn delete_ai_model(id: web::Path<String>) -> impl Responder {
    // Get model to retrieve file path
    let model = match ai_model_queries::get_ai_model_by_id(&id) {
        Ok(Some(model)) => model,
        Ok(None) => return HttpResponse::NotFound().body("AI model not found"),
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Database error: {}", e))
        }
    };

    // Delete from database
    if let Err(e) = ai_model_queries::delete_ai_model(&id) {
        return HttpResponse::InternalServerError().body(format!("Database error: {}", e));
    }

    // Delete model file from disk (best effort, don't fail if file doesn't exist)
    if let Err(e) = fs::remove_file(&model.file_path) {
        log::warn!("Failed to delete model file {}: {}", model.file_path, e);
    }

    // Check if any models remain
    let remaining = ai_model_queries::get_enabled_ai_models().unwrap_or_default();
    if remaining.is_empty() {
        crate::module::ai_security::state::disable_ai();
        log::info!("All AI models deleted, AI security disabled");
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "AI model deleted successfully",
        "id": id.to_string()
    }))
}
