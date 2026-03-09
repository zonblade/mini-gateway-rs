//! # Blocklist Cleanup Scheduler
//!
//! Background worker that periodically removes expired blocklist entries.
//! Runs every 5 minutes. Delegates all filtering and cache-rebuild logic
//! to `blocklist::cleanup()`.

use crate::system::prottp::app::blocklist;
use std::time::Duration;

const CLEANUP_INTERVAL_SECS: u64 = 300; // 5 minutes

pub fn init() {
    std::thread::spawn(|| {
        log::info!(
            "Blocklist cleanup scheduler started (interval: {}s)",
            CLEANUP_INTERVAL_SECS
        );

        loop {
            std::thread::sleep(Duration::from_secs(CLEANUP_INTERVAL_SECS));

            if !blocklist::is_active() {
                log::debug!("Blocklist cleanup: skipped (inactive)");
                continue;
            }

            blocklist::cleanup();
        }
    });
}
