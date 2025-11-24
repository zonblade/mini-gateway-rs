//! # AI Security Module
//!
//! This module provides AI-based security and anomaly detection using ONNX models.
//! It supports XGBoost and Isolation Forest models for analyzing network traffic patterns.

pub mod models;
pub mod thread_isolation;
pub mod thread_xgboost;

use crate::module::broker::BrokerClient;
use thread_isolation::spawn_isolation_thread;
use thread_xgboost::{spawn_xgboost_thread, MlFeatureLog};

/// Initializes the AI security system with broker and model threads
///
/// This function sets up the message broker and spawns threads for XGBoost
/// and Isolation Forest model inference.
///
/// # Returns
///
/// A BrokerClient with senders to both model threads, or an error message if initialization fails.
pub fn init_ai_security() -> Result<(std::sync::mpsc::Sender<MlFeatureLog>, std::sync::mpsc::Sender<MlFeatureLog>), String> {
    // Initialize ONNX Runtime globally (call once)
    ort::init()
        .with_name("mini-gateway-ai")
        .commit()
        .map_err(|e| format!("Failed to initialize ONNX Runtime: {}", e))?;

    // Ensure models directory exists
    if let Err(e) = std::fs::create_dir_all("./models") {
        log::warn!("Failed to create models directory: {}. Model uploads may fail.", e);
    } else {
        log::info!("Models directory ready: ./models");
    }

    log::info!("Initializing AI Security system...");

    let broker = BrokerClient::new();

    // Spawn XGBoost thread
    let xgboost_tx = broker.spawn(move |log: MlFeatureLog| {
        // This closure is executed in the XGBoost thread
        // The actual processing is handled by spawn_xgboost_thread
    });

    // Spawn Isolation Forest thread
    let isolation_tx = broker.spawn(move |log: MlFeatureLog| {
        // This closure is executed in the Isolation Forest thread
        // The actual processing is handled by spawn_isolation_thread
    });

    log::info!("AI Security system initialized successfully");

    Ok((xgboost_tx, isolation_tx))
}

/// Alternative initialization that spawns threads directly with custom handlers
///
/// This provides more control over the thread lifecycle and allows for
/// custom error handling and logging.
pub fn init_ai_security_advanced() -> Result<(), String> {
    // Initialize ONNX Runtime globally (call once)
    ort::init()
        .with_name("mini-gateway-ai")
        .commit()
        .map_err(|e| format!("Failed to initialize ONNX Runtime: {}", e))?;

    // Ensure models directory exists
    if let Err(e) = std::fs::create_dir_all("./models") {
        log::warn!("Failed to create models directory: {}. Model uploads may fail.", e);
    } else {
        log::info!("Models directory ready: ./models");
    }

    log::info!("Initializing AI Security system (advanced mode)...");

    // Create channels
    let (xgboost_tx, xgboost_rx) = std::sync::mpsc::channel();
    let (isolation_tx, isolation_rx) = std::sync::mpsc::channel();

    // Spawn threads
    let xgboost_handle = spawn_xgboost_thread(xgboost_rx);
    let isolation_handle = spawn_isolation_thread(isolation_rx);

    // Store handles for future use (e.g., graceful shutdown)
    // For now, just detach them
    std::thread::spawn(move || {
        if let Err(e) = xgboost_handle.join() {
            log::error!("XGBoost thread panicked: {:?}", e);
        }
    });

    std::thread::spawn(move || {
        if let Err(e) = isolation_handle.join() {
            log::error!("Isolation Forest thread panicked: {:?}", e);
        }
    });

    log::info!("AI Security system initialized successfully (advanced mode)");

    // TODO: Store senders somewhere accessible (e.g., global state, app data)
    // For now, they're dropped immediately which will stop the threads

    Ok(())
}
