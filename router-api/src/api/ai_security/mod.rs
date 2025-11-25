//! # AI Security API Module
//!
//! This module provides a comprehensive API for managing AI security models, including:
//!
//! - **AI Models**: Upload, configure, enable/disable ONNX models for anomaly detection
//! - **Inference Stats**: View inference statistics and results
//!
//! The module uses ONNX Runtime for model inference with XGBoost and Isolation Forest models.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// AI Model types supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    XGBoost,
    Isolation,
}

impl ModelType {
    /// All supported model types
    pub const ALL: &'static [ModelType] = &[ModelType::XGBoost, ModelType::Isolation];
}

impl fmt::Display for ModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelType::XGBoost => write!(f, "xgboost"),
            ModelType::Isolation => write!(f, "isolation"),
        }
    }
}

impl FromStr for ModelType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "xgboost" => Ok(ModelType::XGBoost),
            "isolation" => Ok(ModelType::Isolation),
            _ => Err(format!("Invalid model type: '{}'. Must be 'xgboost' or 'isolation'", s)),
        }
    }
}

pub mod ai_model_queries;
mod model_list;
mod model_get;
mod model_upload;
mod model_set;
mod model_delete;
mod inference_stats;

// Re-export AiModel for external use
pub use ai_model_queries::AiModel;

use actix_web::web;
use crate::api::users::{JwtAuth, RoleAuth};

/// Configures the AI security API routes
///
/// This function registers all endpoints for managing AI models and viewing inference statistics
/// under the "/ai-security" path prefix.
///
/// # Parameters
///
/// * `cfg` - Mutable reference to a web service configuration where routes will be registered
///
/// # API Endpoints
///
/// ## AI Model endpoints:
/// - GET /ai-security/models - List all AI models
/// - GET /ai-security/model/{id} - Get a specific AI model by ID
/// - POST /ai-security/model/upload - Upload a new ONNX model file
/// - POST /ai-security/model/set - Update or enable/disable a model
/// - DELETE /ai-security/model/{id} - Delete an AI model
///
/// ## Inference Stats endpoints:
/// - GET /ai-security/stats - Get inference statistics
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ai-security")
            .wrap(JwtAuth::new())
            .wrap(RoleAuth::admin())
            // AI Model endpoints
            .service(model_list::list_ai_models)
            .service(model_get::get_ai_model)
            .service(model_upload::upload_ai_model)
            .service(model_set::set_ai_model)
            .service(model_delete::delete_ai_model)
            // Inference Stats endpoints
            .service(inference_stats::get_inference_stats),
    );
}
