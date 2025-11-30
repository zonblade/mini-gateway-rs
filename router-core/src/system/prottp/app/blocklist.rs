//! # Blocklist Module
//!
//! Handles IP blocklist updates from router-api and provides
//! blocking check functionality for Zero Trust security.

use crate::config::{self, BlocklistEntry};

/// Initialize/update blocklist from prottp payload
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

    // Set active flag based on list content
    let is_active = !blocklist_data.is_empty();

    log::info!(
        "Blocklist updated: {} entries (checksum: {}, active: {})",
        blocklist_data.len(),
        &checksum[..8],
        is_active
    );

    config::RoutingData::BlocklistID.set(&checksum);
    config::RoutingData::BlocklistData.xset(&blocklist_data);
    config::RoutingData::BlocklistActive.xset::<bool>(is_active);

    Ok(())
}

/// Check if an IP is blocked (with expiry support)
/// Returns Some(reason) if blocked, None if not blocked
pub fn is_blocked(ip: &str) -> Option<String> {
    let blocklist = config::RoutingData::BlocklistData.xget::<Vec<BlocklistEntry>>()?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    for entry in blocklist.iter() {
        if entry.ip != ip {
            continue;
        }

        // Check if expired
        match entry.expires_at {
            Some(expiry) if now >= expiry => continue, // Expired, skip
            _ => {
                // Not expired or permanent
                return Some(entry.reason.clone().unwrap_or_else(|| "blocked".to_string()));
            }
        }
    }

    None // Not blocked
}

/// Get current blocklist count (for monitoring)
pub fn count() -> usize {
    config::RoutingData::BlocklistData
        .xget::<Vec<BlocklistEntry>>()
        .map(|list| list.len())
        .unwrap_or(0)
}
