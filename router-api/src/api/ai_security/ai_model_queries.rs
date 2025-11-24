//! # AI Model Database Operations
//!
//! This module provides database operations for managing AI model configurations.
//! It handles creating the database table, querying, inserting, updating, and
//! deleting AI model records.

use crate::module::database::{get_connection, DatabaseError};
use chrono::Utc;
use uuid::Uuid;

/// Represents an AI model configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiModel {
    pub id: String,
    pub name: String,
    pub model_type: String,          // "xgboost" or "isolation"
    pub file_path: String,
    pub enabled: bool,
    pub version: Option<String>,
    pub uploaded_at: String,
    pub last_inference: Option<String>,
    pub inference_count: i64,
}

/// Creates the ai_models table in the database if it doesn't already exist
pub fn ensure_ai_models_table() -> Result<(), DatabaseError> {
    let db = get_connection()?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS ai_models (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            model_type TEXT NOT NULL,
            file_path TEXT NOT NULL,
            enabled INTEGER DEFAULT 1,
            version TEXT,
            uploaded_at TEXT NOT NULL,
            last_inference TEXT,
            inference_count INTEGER DEFAULT 0
        )",
        [],
    )?;

    Ok(())
}

/// Retrieves all AI models from the database
pub fn get_all_ai_models() -> Result<Vec<AiModel>, DatabaseError> {
    ensure_ai_models_table()?;
    let db = get_connection()?;

    db.query(
        "SELECT id, name, model_type, file_path, enabled, version, uploaded_at, last_inference, inference_count
         FROM ai_models ORDER BY uploaded_at DESC",
        [],
        |row| {
            Ok(AiModel {
                id: row.get(0)?,
                name: row.get(1)?,
                model_type: row.get(2)?,
                file_path: row.get(3)?,
                enabled: row.get::<_, i64>(4)? != 0,
                version: row.get(5)?,
                uploaded_at: row.get(6)?,
                last_inference: row.get(7)?,
                inference_count: row.get(8)?,
            })
        },
    )
}

/// Retrieves a specific AI model by ID
pub fn get_ai_model_by_id(id: &str) -> Result<Option<AiModel>, DatabaseError> {
    ensure_ai_models_table()?;
    let db = get_connection()?;

    let results = db.query(
        "SELECT id, name, model_type, file_path, enabled, version, uploaded_at, last_inference, inference_count
         FROM ai_models WHERE id = ?1",
        [id],
        |row| {
            Ok(AiModel {
                id: row.get(0)?,
                name: row.get(1)?,
                model_type: row.get(2)?,
                file_path: row.get(3)?,
                enabled: row.get::<_, i64>(4)? != 0,
                version: row.get(5)?,
                uploaded_at: row.get(6)?,
                last_inference: row.get(7)?,
                inference_count: row.get(8)?,
            })
        },
    )?;

    Ok(results.into_iter().next())
}

/// Inserts or updates an AI model in the database
pub fn upsert_ai_model(model: &AiModel) -> Result<(), DatabaseError> {
    ensure_ai_models_table()?;
    let db = get_connection()?;

    db.execute(
        "INSERT OR REPLACE INTO ai_models
         (id, name, model_type, file_path, enabled, version, uploaded_at, last_inference, inference_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            &model.id,
            &model.name,
            &model.model_type,
            &model.file_path,
            if model.enabled { 1 } else { 0 },
            &model.version,
            &model.uploaded_at,
            &model.last_inference,
            &model.inference_count,
        ],
    )?;

    Ok(())
}

/// Deletes an AI model from the database
pub fn delete_ai_model(id: &str) -> Result<(), DatabaseError> {
    ensure_ai_models_table()?;
    let db = get_connection()?;

    db.execute("DELETE FROM ai_models WHERE id = ?1", [id])?;

    Ok(())
}

/// Updates the enabled status of an AI model
pub fn update_ai_model_enabled(id: &str, enabled: bool) -> Result<(), DatabaseError> {
    ensure_ai_models_table()?;
    let db = get_connection()?;

    db.execute(
        "UPDATE ai_models SET enabled = ?1 WHERE id = ?2",
        rusqlite::params![if enabled { 1 } else { 0 }, id],
    )?;

    Ok(())
}

/// Updates inference statistics for an AI model
pub fn update_inference_stats(id: &str) -> Result<(), DatabaseError> {
    ensure_ai_models_table()?;
    let db = get_connection()?;

    let now = Utc::now().to_rfc3339();

    db.execute(
        "UPDATE ai_models
         SET last_inference = ?1, inference_count = inference_count + 1
         WHERE id = ?2",
        rusqlite::params![now, id],
    )?;

    Ok(())
}

/// Gets all enabled AI models
pub fn get_enabled_ai_models() -> Result<Vec<AiModel>, DatabaseError> {
    ensure_ai_models_table()?;
    let db = get_connection()?;

    db.query(
        "SELECT id, name, model_type, file_path, enabled, version, uploaded_at, last_inference, inference_count
         FROM ai_models WHERE enabled = 1 ORDER BY uploaded_at DESC",
        [],
        |row| {
            Ok(AiModel {
                id: row.get(0)?,
                name: row.get(1)?,
                model_type: row.get(2)?,
                file_path: row.get(3)?,
                enabled: row.get::<_, i64>(4)? != 0,
                version: row.get(5)?,
                uploaded_at: row.get(6)?,
                last_inference: row.get(7)?,
                inference_count: row.get(8)?,
            })
        },
    )
}
