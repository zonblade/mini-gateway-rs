//! # Blocklist Module
//!
//! Handles IP blocklist updates from router-api and provides
//! blocking check functionality for Zero Trust security.
//!
//! Uses an in-memory cache (`BLOCKLIST_CACHE`) to avoid deserializing
//! JSON on every request. The cache is rebuilt on init (every ~5s sync)
//! and cleanup (every 5min). Hot-path lookups are O(1) HashMap gets
//! behind a read lock, plus an `AtomicBool` for the active flag.

use crate::config::{self, BlocklistEntry};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, RwLock};

struct BlocklistValue {
    reason: Option<String>,
    expires_at: Option<u64>,
}

static BLOCKLIST_CACHE: LazyLock<RwLock<HashMap<String, Vec<BlocklistValue>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static BLOCKLIST_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Build a `HashMap<String, Vec<BlocklistValue>>` from a flat entry list.
fn build_cache(entries: &[BlocklistEntry]) -> HashMap<String, Vec<BlocklistValue>> {
    let mut map: HashMap<String, Vec<BlocklistValue>> = HashMap::new();
    for e in entries {
        map.entry(e.ip.clone()).or_default().push(BlocklistValue {
            reason: e.reason.clone(),
            expires_at: e.expires_at,
        });
    }
    map
}

/// Initialize/update blocklist from prottp payload.
///
/// Called by the prottp handler every ~5 seconds. Parses the JSON once,
/// rebuilds the in-memory cache, and persists to mini_config for
/// compatibility with cleanup and other consumers.
pub fn init(payload: String) -> Result<(), serde_json::Error> {
    let checksum = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    // Check if blocklist has changed
    let old_checksum = config::RoutingData::BlocklistID.get();
    if checksum == old_checksum {
        log::debug!("Blocklist unchanged, skipping update");
        return Ok(());
    }

    let blocklist_data = serde_json::from_str::<Vec<BlocklistEntry>>(&payload)?;

    let is_active = !blocklist_data.is_empty();

    log::info!(
        "Blocklist updated: {} entries (checksum: {}, active: {})",
        blocklist_data.len(),
        &checksum[..8],
        is_active
    );

    // Rebuild in-memory cache
    let new_cache = build_cache(&blocklist_data);
    if let Ok(mut cache) = BLOCKLIST_CACHE.write() {
        *cache = new_cache;
    }
    BLOCKLIST_ACTIVE.store(is_active, Ordering::Release);

    // Persist to mini_config (canonical store for cleanup compatibility)
    config::RoutingData::BlocklistID.set(&checksum);
    config::RoutingData::BlocklistData.xset(&blocklist_data);
    config::RoutingData::BlocklistActive.xset::<bool>(is_active);

    Ok(())
}

/// Fast O(1) check whether the blocklist is active.
///
/// Single atomic load (~1ns). Used by the hot path in `gateway_fast.rs`
/// to skip the heavier `is_blocked()` call entirely when no blocklist
/// is configured.
pub fn is_active() -> bool {
    BLOCKLIST_ACTIVE.load(Ordering::Acquire)
}

/// Check if an IP is blocked (with expiry support).
///
/// Returns `Some(reason)` if blocked, `None` if not blocked.
/// Uses the in-memory HashMap cache — O(1) lookup + tiny inner-Vec
/// scan for per-IP entries.
pub fn is_blocked(ip: &str) -> Option<String> {
    let cache = BLOCKLIST_CACHE.read().ok()?;

    let entries = cache.get(ip)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    for entry in entries {
        match entry.expires_at {
            Some(expiry) if now >= expiry => continue, // Expired, skip
            _ => {
                // Not expired or permanent
                return Some(
                    entry
                        .reason
                        .clone()
                        .unwrap_or_else(|| "blocked".to_string()),
                );
            }
        }
    }

    None
}

/// Cleanup expired blocklist entries.
///
/// Reads from mini_config (canonical persistent store), filters out
/// expired entries, rebuilds the in-memory cache, and writes cleaned
/// data back to mini_config.
pub fn cleanup() {
    let blocklist = match config::RoutingData::BlocklistData.xget::<Vec<BlocklistEntry>>() {
        Some(list) => list,
        None => return,
    };

    let count_before = blocklist.len();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Filter out expired entries
    let cleaned: Vec<BlocklistEntry> = blocklist
        .into_iter()
        .filter(|e| e.expires_at.map(|exp| exp > now).unwrap_or(true))
        .collect();

    let count_after = cleaned.len();
    let is_active = !cleaned.is_empty();

    // Rebuild in-memory cache from cleaned data
    let new_cache = build_cache(&cleaned);
    if let Ok(mut cache) = BLOCKLIST_CACHE.write() {
        *cache = new_cache;
    }
    BLOCKLIST_ACTIVE.store(is_active, Ordering::Release);

    // Write cleaned data back to mini_config
    config::RoutingData::BlocklistData.xset(&cleaned);

    if cleaned.is_empty() {
        config::RoutingData::BlocklistActive.xset::<bool>(false);
        log::info!("Blocklist cleanup: all entries expired, deactivated");
    } else if count_before != count_after {
        log::info!(
            "Blocklist cleanup: removed {} expired, {} remaining",
            count_before - count_after,
            count_after
        );
    }
}
