//! # XGBoost Model Thread Handler
//!
//! This module provides the thread handler for XGBoost-based anomaly detection.
//! It receives ML feature logs via a channel and runs inference using an ONNX model.

use super::models::{ModelError, OnnxModel};
use crate::api::ai_security::ai_model_queries;
use crate::api::ai_security::ModelType;
use std::sync::mpsc::Receiver;

/// Represents a parsed ML feature log for inference
#[derive(Debug, Clone)]
pub struct MlFeatureLog {
    // Identification
    pub conn_id: String,

    // Timing
    pub duration_ms: f32,

    // HTTP
    pub http_status: f32,
    pub http_method: String,
    pub protocol: String,

    // Size metrics
    pub size_in: f32,
    pub size_out: f32,

    // TCP metrics (10 fields)
    pub tcp_rtt: f32,
    pub tcp_retrans: f32,
    pub tcp_lost: f32,
    pub tcp_send_wnd: f32,
    pub tcp_recv_wnd: f32,
    pub tcp_send_mss: f32,
    pub tcp_recv_mss: f32,
    pub tcp_bytes_acked: f32,
    pub tcp_segs_in: f32,
    pub tcp_segs_out: f32,

    // TLS
    pub tls_version: String,

    // Network
    pub client_ip: String,
    pub client_port: f32,
    #[allow(dead_code)]
    pub server_ip: String,
    pub server_port: f32,
}

impl MlFeatureLog {
    /// Converts the log into a feature vector for model inference
    pub fn to_feature_vector(&self) -> Vec<f32> {
        vec![
            self.duration_ms,
            self.http_status,
            self.size_in,
            self.size_out,
            self.tcp_rtt,
            self.tcp_retrans,
            self.tcp_lost,
            self.tcp_send_wnd,
            self.tcp_recv_wnd,
            self.tcp_send_mss,
            self.tcp_recv_mss,
            self.tcp_bytes_acked,
            self.tcp_segs_in,
            self.tcp_segs_out,
            self.client_port,
            self.server_port,
            self.encode_protocol(),
            self.encode_method(),
            self.encode_tls(),
        ]
    }

    fn encode_protocol(&self) -> f32 {
        match self.protocol.as_str() {
            "HTTP/1.0" => 0.0,
            "HTTP/1.1" => 1.0,
            "HTTP/2" => 2.0,
            "HTTP/3" => 3.0,
            _ => 0.0,
        }
    }

    fn encode_method(&self) -> f32 {
        match self.http_method.as_str() {
            "GET" => 0.0,
            "POST" => 1.0,
            "PUT" => 2.0,
            "DELETE" => 3.0,
            "PATCH" => 4.0,
            "HEAD" => 5.0,
            "OPTIONS" => 6.0,
            _ => 7.0,
        }
    }

    fn encode_tls(&self) -> f32 {
        if self.tls_version == "-" || self.tls_version.is_empty() {
            0.0
        } else {
            1.0
        }
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

        let xgboost_model = models
            .iter()
            .find(|m| m.model_type == ModelType::XGBoost)
            .ok_or_else(|| {
                ModelError::FileNotFound("No enabled XGBoost model found".to_string())
            })?;

        log::info!(
            "Loading XGBoost model: {} from {}",
            xgboost_model.name,
            xgboost_model.file_path
        );

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

                    // Check anomaly threshold and add to blocklist
                    if let Some(score) = output.first() {
                        if *score > 0.8 {
                            log::warn!(
                                "ANOMALY DETECTED by XGBoost: conn_id={}, ip={}, score={}",
                                log.conn_id,
                                log.client_ip,
                                score
                            );

                            // Add to blocklist (1 hour TTL)
                            let reason = format!("xgboost:{:.2}", score);
                            super::blocklist_store::add_blocked(
                                log.client_ip.clone(),
                                reason,
                                Some(3600), // 1 hour TTL
                            );
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
