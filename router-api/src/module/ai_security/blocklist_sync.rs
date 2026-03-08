//! # Blocklist Sync Module
//!
//! Background thread that periodically syncs pending blocklist entries
//! to router-core via prottp HTTP endpoint.

use super::blocklist_store::{self, PendingEntry};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Sync interval in seconds
const SYNC_INTERVAL_SECS: u64 = 5;

/// Router-core prottp endpoint
const PROTTP_ADDR: &str = "127.0.0.1:30099";

/// Initialize blocklist sync background thread
pub fn init() {
    std::thread::spawn(|| {
        eprintln!("[BLKS] Blocklist sync thread started (interval: {}s)", SYNC_INTERVAL_SECS);

        loop {
            std::thread::sleep(Duration::from_secs(SYNC_INTERVAL_SECS));

            let pending = blocklist_store::get_pending();
            if pending.is_empty() {
                continue;
            }

            log::debug!("Blocklist sync: {} entries pending", pending.len());

            match sync_to_router_core(&pending) {
                Ok(_) => {
                    log::info!("Blocklist sync: sent {} entries to router-core", pending.len());
                    blocklist_store::clear_pending();
                }
                Err(e) => {
                    log::warn!("Blocklist sync failed: {} (will retry)", e);
                }
            }
        }
    });
}

/// Sync pending entries to router-core via prottp
fn sync_to_router_core(entries: &[PendingEntry]) -> Result<(), String> {
    // Convert to router-core BlocklistEntry format
    let blocklist_payload: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "ip": e.ip,
                "reason": e.reason,
                "expires_at": e.expires_at,
                "created_at": e.created_at
            })
        })
        .collect();

    let body = serde_json::to_string(&blocklist_payload)
        .map_err(|e| format!("JSON serialize error: {}", e))?;

    // Connect to prottp server
    let mut stream = TcpStream::connect(PROTTP_ADDR)
        .map_err(|e| format!("Connection error: {}", e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("Set timeout error: {}", e))?;

    // Build prottp request (method: GWRX)
    let request = format!(
        "GWRX /gateway/blocklist HTTP/1.1\r\n\
         Host: {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         \r\n\
         {}",
        PROTTP_ADDR,
        body.len(),
        body
    );

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("Write error: {}", e))?;

    // Read response
    let mut response = vec![0u8; 1024];
    let n = stream
        .read(&mut response)
        .map_err(|e| format!("Read error: {}", e))?;

    let response_str = String::from_utf8_lossy(&response[..n]);

    // Check for 200 OK
    if response_str.contains("200") {
        Ok(())
    } else {
        Err(format!("Non-200 response: {}", response_str.lines().next().unwrap_or("")))
    }
}
