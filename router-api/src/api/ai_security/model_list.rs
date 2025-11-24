//! # AI Model Listing Endpoint
//!
//! This module provides the HTTP endpoint for listing all AI models.

use super::ai_model_queries;
use actix_web::{get, HttpResponse, Responder};

/// Lists all AI models
///
/// # Endpoint
///
/// `GET /ai-security/models`
///
/// # Response
///
/// ## Success (200 OK)
/// Returns a JSON array of all AI models with their configurations.
///
/// ## Internal Server Error (500)
/// Returned when there is a database error.
#[get("/models")]
pub async fn list_ai_models() -> impl Responder {
    match ai_model_queries::get_all_ai_models() {
        Ok(models) => HttpResponse::Ok().json(models),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
