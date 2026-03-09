//! # Isolation Forest Model Thread Handler
//!
//! This module provides the thread handler for Isolation Forest-based anomaly detection.
//! It receives ML feature logs via a channel and runs inference using an ONNX model.

use super::models::{ModelError, OnnxModel};
use super::thread_xgboost::MlFeatureLog; // Reuse the same feature log struct
use crate::api::ai_security::ai_model_queries;
use crate::api::ai_security::ModelType;
use std::sync::mpsc::Receiver;

/// Isolation Forest thread handler that processes ML logs and runs inference
pub struct IsolationForestHandler {
    model: Option<OnnxModel>,
    model_id: Option<String>,
}

impl IsolationForestHandler {
    /// Creates a new Isolation Forest handler
    pub fn new() -> Self {
        Self {
            model: None,
            model_id: None,
        }
    }

    /// Loads the Isolation Forest model from database config
    pub fn load_model(&mut self) -> Result<(), ModelError> {
        // Get enabled Isolation Forest model from database
        let models = ai_model_queries::get_enabled_ai_models()
            .map_err(|e| ModelError::FileNotFound(format!("Database error: {}", e)))?;

        let isolation_model = models
            .iter()
            .find(|m| m.model_type == ModelType::Isolation)
            .ok_or_else(|| {
                ModelError::FileNotFound("No enabled Isolation Forest model found".to_string())
            })?;

        log::info!(
            "Loading Isolation Forest model: {} from {}",
            isolation_model.name,
            isolation_model.file_path
        );

        let model = OnnxModel::new(isolation_model.file_path.clone());
        self.model_id = Some(isolation_model.id.clone());
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
        log::info!("Isolation Forest thread started");

        // Load model (lazy loading - will load on first inference)
        if let Err(e) = self.load_model() {
            log::error!("Failed to initialize Isolation Forest model: {}", e);
            return;
        }

        // Process incoming logs
        while let Ok(log) = receiver.recv() {
            match self.run_inference(&log) {
                Ok(output) => {
                    log::info!(
                        "Isolation Forest inference for {}: result={:?}",
                        log.conn_id,
                        output
                    );

                    // Update inference stats
                    if let Some(model_id) = &self.model_id {
                        let _ = ai_model_queries::update_inference_stats(model_id);
                    }

                    // Check anomaly threshold and add to blocklist
                    if let Some(score) = output.first() {
                        if *score > 0.8 {
                            log::warn!(
                                "ANOMALY DETECTED by Isolation Forest: conn_id={}, ip={}, score={}",
                                log.conn_id,
                                log.client_ip,
                                score
                            );

                            // Add to blocklist (1 hour TTL)
                            let reason = format!("isolation:{:.2}", score);
                            super::blocklist_store::add_blocked(
                                log.client_ip.clone(),
                                reason,
                                Some(3600), // 1 hour TTL
                            );
                        }
                    }
                }
                Err(e) => {
                    log::error!(
                        "Isolation Forest inference error for {}: {}",
                        log.conn_id,
                        e
                    );
                }
            }
        }

        log::info!("Isolation Forest thread stopped");
    }
}

/// Spawns the Isolation Forest handler thread
pub fn spawn_isolation_thread(receiver: Receiver<MlFeatureLog>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let handler = IsolationForestHandler::new();
        handler.run(receiver);
    })
}
