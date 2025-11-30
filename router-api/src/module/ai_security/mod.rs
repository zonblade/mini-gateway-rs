//! # AI Security Module
//!
//! This module provides AI-based security and anomaly detection using ONNX models.
//! It supports XGBoost and Isolation Forest models for analyzing network traffic patterns.

pub mod models;
pub mod state;
pub mod thread_isolation;
pub mod thread_xgboost;
pub mod blocklist_store;
pub mod blocklist_sync;

use crate::api::ai_security::{ai_model_queries, ModelType};
use thread_isolation::spawn_isolation_thread;
use thread_xgboost::spawn_xgboost_thread;

// Re-export for external use
pub use state::{is_ai_enabled, send_to_inference};
pub use thread_xgboost::MlFeatureLog;

/// Initialize AI security - call at startup
/// Checks DB for existing models and spawns threads if found
pub fn init_ai_security() -> Result<(), String> {
    // Initialize ONNX Runtime globally (call once)
    ort::init()
        .with_name("mini-gateway-ai")
        .commit()
        .map_err(|e| format!("Failed to initialize ONNX Runtime: {}", e))?;

    // Ensure models directory exists
    let _ = std::fs::create_dir_all("./models");

    // Initialize blocklist store and sync
    {
        blocklist_store::init();
        blocklist_sync::init();
    }

    // Check DB for existing enabled models
    let models = ai_model_queries::get_enabled_ai_models().unwrap_or_default();

    if models.is_empty() {
        log::info!("No AI models found, AI security disabled");
        return Ok(());
    }

    // Spawn threads for existing models using deterministic match
    for model in &models {
        match model.model_type {
            ModelType::XGBoost => spawn_xgboost_with_sender(),
            ModelType::Isolation => spawn_isolation_with_sender(),
        }
    }

    // Enable AI
    state::enable_ai();
    log::info!("AI security initialized with {} model(s)", models.len());

    Ok(())
}

/// Spawn XGBoost thread and register sender
pub fn spawn_xgboost_with_sender() {
    if state::XGBOOST_TX.get().is_some() {
        return; // Already spawned
    }

    let (tx, rx) = std::sync::mpsc::channel();
    state::init_xgboost_sender(tx);
    spawn_xgboost_thread(rx);
    log::info!("XGBoost inference thread spawned");
}

/// Spawn Isolation thread and register sender
pub fn spawn_isolation_with_sender() {
    if state::ISOLATION_TX.get().is_some() {
        return; // Already spawned
    }

    let (tx, rx) = std::sync::mpsc::channel();
    state::init_isolation_sender(tx);
    spawn_isolation_thread(rx);
    log::info!("Isolation Forest inference thread spawned");
}
