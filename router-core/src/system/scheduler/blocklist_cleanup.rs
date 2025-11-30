//! # Blocklist Cleanup Scheduler
//!
//! Background worker that periodically removes expired blocklist entries.
//! Runs every 5 minutes and updates BlocklistActive flag when list becomes empty.

use crate::config::{BlocklistEntry, RoutingData};
use std::time::Duration;

const CLEANUP_INTERVAL_SECS: u64 = 300; // 5 minutes

pub fn init() {
    std::thread::spawn(|| {
        log::info!("Blocklist cleanup scheduler started (interval: {}s)", CLEANUP_INTERVAL_SECS);

        loop {
            // Sleep first
            std::thread::sleep(Duration::from_secs(CLEANUP_INTERVAL_SECS));

            // Skip if blocklist not active
            let is_active = RoutingData::BlocklistActive.xget::<bool>().unwrap_or(false);
            if !is_active {
                log::debug!("Blocklist cleanup: skipped (inactive)");
                continue;
            }

            // Get current blocklist
            let blocklist = match RoutingData::BlocklistData.xget::<Vec<BlocklistEntry>>() {
                Some(list) => list,
                None => continue,
            };

            let count_before = blocklist.len();

            // Get current timestamp
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

            // Update blocklist
            RoutingData::BlocklistData.xset(&cleaned);

            // Update active flag if empty
            if cleaned.is_empty() {
                RoutingData::BlocklistActive.xset::<bool>(false);
                log::info!("Blocklist cleanup: all entries expired, deactivated");
            } else if count_before != count_after {
                log::info!(
                    "Blocklist cleanup: removed {} expired, {} remaining",
                    count_before - count_after,
                    count_after
                );
            }
        }
    });
}
