//! # AI Model Get Endpoint
//!
//! This module provides the HTTP endpoint for retrieving a specific AI model by ID.

use super::ai_model_queries;
use actix_web::{get, web, HttpResponse, Responder};

/// Gets a specific AI model by ID
///
/// # Endpoint
///
/// `GET /ai-security/model/{id}`
///
/// # Parameters
///
/// * `id` - The unique identifier of the AI model
///
/// # Response
///
/// ## Success (200 OK)
/// Returns the AI model configuration as JSON.
///
/// ## Not Found (404)
/// Returned when no model with the given ID exists.
///
/// ## Internal Server Error (500)
/// Returned when there is a database error.
#[get("/model/{id}")]
pub async fn get_ai_model(id: web::Path<String>) -> impl Responder {
    match ai_model_queries::get_ai_model_by_id(&id) {
        Ok(Some(model)) => HttpResponse::Ok().json(model),
        Ok(None) => HttpResponse::NotFound().body("AI model not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
