//! # ONNX Model Loading and Inference
//!
//! This module provides utilities for loading and running inference on ONNX models
//! using the ORT (ONNX Runtime) library version 2.0 with lazy loading support.

use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Errors that can occur during model operations
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("ONNX Runtime error: {0}")]
    OrtError(#[from] ort::Error),

    #[error("Model not loaded")]
    NotLoaded,

    #[error("Invalid input shape: {0}")]
    InvalidInputShape(String),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

/// Represents an ONNX model with lazy loading support
pub struct OnnxModel {
    file_path: String,
    session: Arc<Mutex<Option<Session>>>,
}

impl OnnxModel {
    /// Creates a new OnnxModel without loading it (lazy loading)
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            session: Arc::new(Mutex::new(None)),
        }
    }

    /// Loads the ONNX model into memory (if not already loaded)
    pub fn load(&self) -> Result<(), ModelError> {
        let mut session_guard = self.session.lock().unwrap();

        // Check if already loaded
        if session_guard.is_some() {
            return Ok(());
        }

        // Check if file exists
        if !Path::new(&self.file_path).exists() {
            return Err(ModelError::FileNotFound(self.file_path.clone()));
        }

        // Load the model using ORT 2.0 API
        log::info!("Loading ONNX model from: {}", self.file_path);
        let session = Session::builder()?
            .with_intra_threads(1)?
            .commit_from_file(&self.file_path)?;

        *session_guard = Some(session);
        log::info!("ONNX model loaded successfully: {}", self.file_path);

        Ok(())
    }

    /// Checks if the model is currently loaded
    pub fn is_loaded(&self) -> bool {
        self.session.lock().unwrap().is_some()
    }

    /// Runs inference on the model with the given input features
    ///
    /// # Arguments
    ///
    /// * `features` - Input feature vector as f32 values
    ///
    /// # Returns
    ///
    /// The model's output as a Vec<f32>
    pub fn infer(&self, features: Vec<f32>) -> Result<Vec<f32>, ModelError> {
        // Ensure model is loaded
        if !self.is_loaded() {
            self.load()?;
        }

        let mut session_guard = self.session.lock().unwrap();
        let session = session_guard.as_mut().ok_or(ModelError::NotLoaded)?;

        // Prepare input tensor - ORT 2.0 API: Tensor::from_array with shape tuple
        let num_features = features.len();
        let input = Tensor::from_array(([1, num_features], features))?;

        // Run inference - ORT 2.0 uses inputs! macro
        let outputs = session.run(ort::inputs![input])?;

        // Extract output - ORT 2.0 API
        let output_array = outputs[0].try_extract_array::<f32>()?;
        let output_slice = output_array.as_slice().ok_or_else(|| {
            ModelError::InvalidInputShape("Failed to get output as slice".to_string())
        })?;

        Ok(output_slice.to_vec())
    }

    /// Unloads the model from memory
    #[allow(dead_code)]
    pub fn unload(&self) {
        let mut session_guard = self.session.lock().unwrap();
        *session_guard = None;
        log::info!("ONNX model unloaded: {}", self.file_path);
    }
}

/// Convenience function to create and immediately load a model
#[allow(dead_code)]
pub fn load_onnx_model(file_path: &str) -> Result<OnnxModel, ModelError> {
    let model = OnnxModel::new(file_path.to_string());
    model.load()?;
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_loading() {
        let model = OnnxModel::new("./models/test.onnx".to_string());
        assert!(!model.is_loaded());
    }
}
