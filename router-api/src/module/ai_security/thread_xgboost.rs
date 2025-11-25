//! # XGBoost Model Thread Handler
//!
//! This module provides the thread handler for XGBoost-based anomaly detection.
//! It receives ML feature logs via a channel and runs inference using an ONNX model.

use super::models::{OnnxModel, ModelError};
use crate::api::ai_security::ai_model_queries;
use crate::api::ai_security::ModelType;
use std::sync::mpsc::Receiver;

/// Represents a parsed ML feature log for inference
#[derive(Debug, Clone)]
pub struct MlFeatureLog {
    pub conn_id: String,
    pub duration_ms: f32,
    pub tcp_rtt: f32,
    pub tcp_retrans: f32,
    pub tcp_lost: f32,
    pub http_status: f32,
    // Add more features as needed to match your 43 features
}

impl MlFeatureLog {
    /// Converts the log into a feature vector for model inference
    pub fn to_feature_vector(&self) -> Vec<f32> {
        vec![
            self.duration_ms,
            self.tcp_rtt,
            self.tcp_retrans,
            self.tcp_lost,
            self.http_status,
            // Add all 43 features here in the correct order
        ]
    }
}

/// XGBoost thread handler that processes ML logs and runs inference
pub struct XGBoostHandler {
    model: Option<OnnxModel>,
    model_id: Option<String>,
}

impl XGBoostHandler {
    /// Creates a new XGBoost handler
    pub fn new() -> Self {
        Self {
            model: None,
            model_id: None,
        }
    }

    /// Loads the XGBoost model from database config
    pub fn load_model(&mut self) -> Result<(), ModelError> {
        // Get enabled XGBoost model from database
        let models = ai_model_queries::get_enabled_ai_models()
            .map_err(|e| ModelError::FileNotFound(format!("Database error: {}", e)))?;

        let xgboost_model = models.iter()
            .find(|m| m.model_type == ModelType::XGBoost)
            .ok_or_else(|| ModelError::FileNotFound("No enabled XGBoost model found".to_string()))?;

        log::info!("Loading XGBoost model: {} from {}", xgboost_model.name, xgboost_model.file_path);

        let model = OnnxModel::new(xgboost_model.file_path.clone());
        self.model_id = Some(xgboost_model.id.clone());
        self.model = Some(model);

        Ok(())
    }

    /// Runs inference on a single ML log
    pub fn run_inference(&self, log: &MlFeatureLog) -> Result<Vec<f32>, ModelError> {
        let model = self.model.as_ref().ok_or(ModelError::NotLoaded)?;
        let features = log.to_feature_vector();
        model.infer(features)
    }

    /// Main loop that processes ML logs from the receiver channel
    pub fn run(mut self, receiver: Receiver<MlFeatureLog>) {
        log::info!("XGBoost thread started");

        // Load model (lazy loading - will load on first inference)
        if let Err(e) = self.load_model() {
            log::error!("Failed to initialize XGBoost model: {}", e);
            return;
        }

        // Process incoming logs
        while let Ok(log) = receiver.recv() {
            match self.run_inference(&log) {
                Ok(output) => {
                    log::info!("XGBoost inference for {}: result={:?}", log.conn_id, output);

                    // Update inference stats
                    if let Some(model_id) = &self.model_id {
                        let _ = ai_model_queries::update_inference_stats(model_id);
                    }

                    // TODO: Process output (e.g., check anomaly threshold, alert, store result)
                    // For now, just log it
                    if let Some(score) = output.first() {
                        if *score > 0.8 {
                            log::warn!("ANOMALY DETECTED by XGBoost: conn_id={}, score={}", log.conn_id, score);
                        }
                    }
                }
                Err(e) => {
                    log::error!("XGBoost inference error for {}: {}", log.conn_id, e);
                }
            }
        }

        log::info!("XGBoost thread stopped");
    }
}

/// Spawns the XGBoost handler thread
pub fn spawn_xgboost_thread(receiver: Receiver<MlFeatureLog>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let handler = XGBoostHandler::new();
        handler.run(receiver);
    })
}
