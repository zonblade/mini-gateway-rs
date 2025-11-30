//! # Unified Stats Structs
//!
//! Data structures for the unified statistics SSE endpoint.

use serde::Serialize;
use std::collections::HashMap;

/// Unified statistics payload sent via SSE
#[derive(Debug, Clone, Serialize)]
pub struct UnifiedStats {
    /// ISO-8601 timestamp
    pub ts: String,
    /// Gateway statistics
    pub gateway: TargetStats,
    /// Proxy statistics
    pub proxy: TargetStats,
}

/// Statistics for a single target (gateway or proxy)
#[derive(Debug, Clone, Serialize, Default)]
pub struct TargetStats {
    /// Request count
    pub req: i64,
    /// Response count
    pub res: i64,
    /// Bytes received
    pub bytes_in: i64,
    /// Bytes sent
    pub bytes_out: i64,
    /// Status code counts (e.g., {"200": 35, "404": 3})
    pub status: HashMap<String, i64>,
}

impl TargetStats {
    pub fn new() -> Self {
        Self::default()
    }
}
