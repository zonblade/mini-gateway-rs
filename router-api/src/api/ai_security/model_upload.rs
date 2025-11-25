//! # AI Model Upload Endpoint
//!
//! This module provides the HTTP endpoint for uploading ONNX model files.

use super::{ai_model_queries::{self, AiModel}, ModelType};
use actix_multipart::Multipart;
use actix_web::{post, HttpResponse, Responder};
use chrono::Utc;
use futures_util::stream::StreamExt as _;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

/// Uploads a new ONNX model file
///
/// # Endpoint
///
/// `POST /ai-security/model/upload`
///
/// # Request Body
///
/// Multipart form data with the following fields:
/// - `name` - Human-readable name for the model
/// - `model_type` - Type of model ("xgboost" or "isolation")
/// - `version` (optional) - Version string
/// - `file` - The ONNX model file
///
/// # Response
///
/// ## Success (200 OK)
/// Returns the created AI model configuration as JSON.
///
/// ## Bad Request (400)
/// Returned when required fields are missing or invalid.
///
/// ## Internal Server Error (500)
/// Returned when there is a file system or database error.
#[post("/model/upload")]
pub async fn upload_ai_model(mut payload: Multipart) -> impl Responder {
    let mut name: Option<String> = None;
    let mut model_type: Option<String> = None;
    let mut version: Option<String> = None;
    let mut file_saved = false;
    let model_id = Uuid::new_v4().to_string();

    // Create models directory if it doesn't exist
    let models_dir = PathBuf::from("./models");
    if let Err(e) = fs::create_dir_all(&models_dir) {
        return HttpResponse::InternalServerError()
            .body(format!("Failed to create models directory: {}", e));
    }

    // Process multipart fields
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(e) => return HttpResponse::BadRequest().body(format!("Multipart error: {}", e)),
        };

        let content_disposition = field.content_disposition();
        let field_name = content_disposition.and_then(|cd| cd.get_name()).unwrap_or("");

        match field_name {
            "name" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(e) => return HttpResponse::BadRequest().body(format!("Error reading field: {}", e)),
                    };
                    data.extend_from_slice(&chunk);
                }
                name = Some(String::from_utf8_lossy(&data).to_string());
            }
            "model_type" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(e) => return HttpResponse::BadRequest().body(format!("Error reading field: {}", e)),
                    };
                    data.extend_from_slice(&chunk);
                }
                model_type = Some(String::from_utf8_lossy(&data).to_string());
            }
            "version" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(e) => return HttpResponse::BadRequest().body(format!("Error reading field: {}", e)),
                    };
                    data.extend_from_slice(&chunk);
                }
                version = Some(String::from_utf8_lossy(&data).to_string());
            }
            "file" => {
                // Save file
                let file_path = models_dir.join(format!("{}.onnx", model_id));
                let mut file = match fs::File::create(&file_path) {
                    Ok(file) => file,
                    Err(e) => return HttpResponse::InternalServerError()
                        .body(format!("Failed to create file: {}", e)),
                };

                while let Some(chunk) = field.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(e) => return HttpResponse::BadRequest().body(format!("Error reading file: {}", e)),
                    };

                    if let Err(e) = file.write_all(&chunk) {
                        return HttpResponse::InternalServerError()
                            .body(format!("Failed to write file: {}", e));
                    }
                }

                file_saved = true;
            }
            _ => {}
        }
    }

    // Validate required fields
    let name = match name {
        Some(n) if !n.trim().is_empty() => n.trim().to_string(),
        _ => return HttpResponse::BadRequest().body("Missing or empty 'name' field"),
    };

    let model_type: ModelType = match model_type {
        Some(mt) => match mt.parse::<ModelType>() {
            Ok(t) => t,
            Err(e) => return HttpResponse::BadRequest().body(e),
        },
        None => return HttpResponse::BadRequest().body("Missing 'model_type' field"),
    };

    if !file_saved {
        return HttpResponse::BadRequest().body("Missing 'file' field");
    }

    // Create AI model record
    let ai_model = AiModel {
        id: model_id.clone(),
        name,
        model_type,
        file_path: format!("./models/{}.onnx", model_id),
        enabled: true,
        version,
        uploaded_at: Utc::now().to_rfc3339(),
        last_inference: None,
        inference_count: 0,
    };

    // Save to database
    match ai_model_queries::upsert_ai_model(&ai_model) {
        Ok(_) => {
            // Spawn inference thread if this is the first model of this type
            match ai_model.model_type {
                ModelType::XGBoost => crate::module::ai_security::spawn_xgboost_with_sender(),
                ModelType::Isolation => crate::module::ai_security::spawn_isolation_with_sender(),
            }
            // Enable AI inference
            crate::module::ai_security::state::enable_ai();

            HttpResponse::Ok().json(ai_model)
        }
        Err(e) => {
            // Clean up file on database error
            let _ = fs::remove_file(format!("./models/{}.onnx", model_id));
            HttpResponse::InternalServerError().body(format!("Database error: {}", e))
        }
    }
}
