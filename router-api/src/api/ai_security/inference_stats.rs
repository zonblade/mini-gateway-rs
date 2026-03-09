//! # Inference Statistics Endpoint
//!
//! This module provides the HTTP endpoint for viewing AI model inference statistics.

use super::ai_model_queries;
use actix_web::{get, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

/// Inference statistics summary
#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceStats {
    pub total_models: usize,
    pub enabled_models: usize,
    pub total_inferences: i64,
    pub models: Vec<ModelStats>,
}

/// Per-model statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelStats {
    pub id: String,
    pub name: String,
    pub model_type: String,
    pub enabled: bool,
    pub inference_count: i64,
    pub last_inference: Option<String>,
}

/// Gets inference statistics for all AI models
///
/// # Endpoint
///
/// `GET /ai-security/stats`
///
/// # Response
///
/// ## Success (200 OK)
/// Returns inference statistics as JSON.
///
/// ## Internal Server Error (500)
/// Returned when there is a database error.
#[get("/stats")]
pub async fn get_inference_stats() -> impl Responder {
    let models = match ai_model_queries::get_all_ai_models() {
        Ok(models) => models,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Database error: {}", e))
        }
    };

    let enabled_count = models.iter().filter(|m| m.enabled).count();
    let total_inferences: i64 = models.iter().map(|m| m.inference_count).sum();

    let model_stats: Vec<ModelStats> = models
        .iter()
        .map(|m| ModelStats {
            id: m.id.clone(),
            name: m.name.clone(),
            model_type: m.model_type.to_string(),
            enabled: m.enabled,
            inference_count: m.inference_count,
            last_inference: m.last_inference.clone(),
        })
        .collect();

    let stats = InferenceStats {
        total_models: models.len(),
        enabled_models: enabled_count,
        total_inferences,
        models: model_stats,
    };

    HttpResponse::Ok().json(stats)
}
