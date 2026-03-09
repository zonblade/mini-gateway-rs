//! # Blocklist Store Module
//!
//! Stores pending blocklist entries detected by AI inference.
//! Supports deduplication when both XGBoost and Isolation Forest detect same IP.

use mini_config::Configure;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Configuration keys for blocklist storage
#[derive(Debug, Clone, Configure)]
pub enum BlocklistStore {
    /// Pending entries awaiting sync to router-core
    PendingEntries,
    /// Unix timestamp of last successful sync
    LastSyncTime,
}

/// A pending blocklist entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PendingEntry {
    pub ip: String,
    pub reason: String,
    pub expires_at: Option<u64>,
    pub created_at: u64,
}

/// Mutex for thread-safe deduplication
static DEDUP_LOCK: Mutex<()> = Mutex::new(());

/// Initialize blocklist store
pub fn init() {
    BlocklistStore::PendingEntries.xset::<Vec<PendingEntry>>(vec![]);
    BlocklistStore::LastSyncTime.xset::<u64>(0);
    log::info!("Blocklist store initialized");
}

/// Add a blocked IP with deduplication
/// Returns true if added, false if already exists (deduplicated)
pub fn add_blocked(ip: String, reason: String, ttl_secs: Option<u64>) -> bool {
    let _guard = DEDUP_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut entries = BlocklistStore::PendingEntries
        .xget::<Vec<PendingEntry>>()
        .unwrap_or_default();

    // Check for duplicate IP in pending list
    if entries.iter().any(|e| e.ip == ip) {
        log::debug!("Blocklist dedupe: {} already pending", ip);
        return false;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let expires_at = ttl_secs.map(|ttl| now + ttl);

    let entry = PendingEntry {
        ip: ip.clone(),
        reason,
        expires_at,
        created_at: now,
    };

    entries.push(entry);
    BlocklistStore::PendingEntries.xset(&entries);

    log::info!("Blocklist: added {} ({} pending)", ip, entries.len());
    true
}

/// Get all pending entries for sync
pub fn get_pending() -> Vec<PendingEntry> {
    BlocklistStore::PendingEntries
        .xget::<Vec<PendingEntry>>()
        .unwrap_or_default()
}

/// Clear pending entries after successful sync
pub fn clear_pending() {
    BlocklistStore::PendingEntries.xset::<Vec<PendingEntry>>(vec![]);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    BlocklistStore::LastSyncTime.xset(now);
}

/// Get count of pending entries
#[allow(dead_code)]
pub fn pending_count() -> usize {
    get_pending().len()
}
