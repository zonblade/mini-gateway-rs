//! UUID Generation Endpoint
//!
//! Provides a secure UUID generation service for frontend applications that need
//! unique identifiers but may have limited crypto support in older browsers.
//!
//! This endpoint generates UUIDs using the system's secure random number generator
//! instead of relying on client-side crypto APIs.

use actix_web::{get, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response structure for UUID generation
#[derive(Debug, Serialize, Deserialize)]
pub struct UuidResponse {
    /// The generated UUID in string format
    pub uuid: String,
    /// Timestamp when the UUID was generated (ISO 8601 format)
    pub generated_at: String,
}

/// Generate a new UUID
///
/// Generates a secure UUID v4 using the system's random number generator.
/// This endpoint is protected by JWT authentication and available to all
/// authenticated users regardless of role.
///
/// # Returns
///
/// * `200 OK` - JSON response containing the generated UUID and timestamp
/// * `500 Internal Server Error` - If UUID generation fails (unlikely)
///
/// # Example Response
///
/// ```json
/// {
///   "uuid": "550e8400-e29b-41d4-a716-446655440000",
///   "generated_at": "2024-01-01T12:00:00Z"
/// }
/// ```
#[get("/uuid")]
pub async fn generate_uuid() -> impl Responder {
    let uuid = Uuid::new_v4();
    let generated_at = chrono::Utc::now().to_rfc3339();

    let response = UuidResponse {
        uuid: uuid.to_string(),
        generated_at,
    };

    log::debug!("Generated UUID: {}", response.uuid);
    HttpResponse::Ok().json(response)
}
