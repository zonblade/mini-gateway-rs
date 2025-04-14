use std::time::Duration;

pub const RETRY_INTERVAL: Duration = Duration::from_secs(1);
pub const POLL_INTERVAL: Duration = Duration::from_millis(10);
pub const SCAN_INTERVAL: Duration = Duration::from_millis(100); // How often to scan for rotated files
pub const DB_FLUSH_INTERVAL: Duration = Duration::from_secs(5); // How often to flush logs to database