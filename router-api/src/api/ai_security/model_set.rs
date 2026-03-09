//! # AI Model Set/Update Endpoint
//!
//! This module provides the HTTP endpoint for updating AI model configurations.

use super::ai_model_queries;
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

/// Request body for updating an AI model
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateModelRequest {
    pub id: String,
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub version: Option<String>,
}

/// Updates an AI model configuration
///
/// # Endpoint
///
/// `POST /ai-security/model/set`
///
/// # Request Body
///
/// JSON object with the following fields:
/// - `id` - The unique identifier of the AI model to update
/// - `name` (optional) - New name for the model
/// - `enabled` (optional) - Enable or disable the model
/// - `version` (optional) - Update version string
///
/// # Response
///
/// ## Success (200 OK)
/// Returns the updated AI model configuration as JSON.
///
/// ## Not Found (404)
/// Returned when no model with the given ID exists.
///
/// ## Internal Server Error (500)
/// Returned when there is a database error.
#[post("/model/set")]
pub async fn set_ai_model(req: web::Json<UpdateModelRequest>) -> impl Responder {
    // Get existing model
    let mut model = match ai_model_queries::get_ai_model_by_id(&req.id) {
        Ok(Some(model)) => model,
        Ok(None) => return HttpResponse::NotFound().body("AI model not found"),
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Database error: {}", e))
        }
    };

    // Update fields if provided
    if let Some(name) = &req.name {
        model.name = name.clone();
    }

    if let Some(enabled) = req.enabled {
        model.enabled = enabled;
    }

    if let Some(version) = &req.version {
        model.version = Some(version.clone());
    }

    // Save updated model
    match ai_model_queries::upsert_ai_model(&model) {
        Ok(_) => HttpResponse::Ok().json(model),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
