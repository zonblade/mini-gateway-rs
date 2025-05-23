use crate::module::{
    memory_log::core::{LogConsumer, MAX_MEMORY_SIZE, PROXY_LOGGER_NAME},
    temporary_log::{tlog_proxy, TemporaryLog},
};
use std::time::{Duration, Instant};

pub fn listen() {
    log::info!("Starting proxy log consumer...");
    println!("Starting proxy log consumer...");

    // Open shared memory
    let mut log_consumer =
        LogConsumer::new(PROXY_LOGGER_NAME, MAX_MEMORY_SIZE).expect("Failed to open shared memory");

    // Pre-allocate batch with capacity
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    const BATCH_SIZE: usize = 100;

    // Status tracking
    let mut consecutive_empty = 0;
    let mut last_health_check = Instant::now();
    let health_check_interval = Duration::from_secs(60); // Check health every minute

    log::info!(
        "Starting proxy log processing, queue size: {}",
        log_consumer.queue_size()
    );

    loop {
        // Periodic health check
        if last_health_check.elapsed() >= health_check_interval {
            // If we haven't received anything in a while, try to reconnect
            if consecutive_empty > 2000 {
                log::warn!(
                    "Too many consecutive empty results ({}), attempting to recreate consumer",
                    consecutive_empty
                );
                match LogConsumer::new(PROXY_LOGGER_NAME, MAX_MEMORY_SIZE) {
                    Ok(new_consumer) => {
                        log_consumer = new_consumer;
                        consecutive_empty = 0;
                    }
                    Err(e) => {
                        log::error!("Failed to recreate log consumer during health check: {}", e);
                        std::thread::sleep(Duration::from_millis(1000));
                        continue;
                    }
                }
            }

            last_health_check = Instant::now();
        }

        // Try to get a log entry with a timeout
        match log_consumer.get_log_with_timeout(10) {
            Ok(Some((timestamp, level, message))) => {
                consecutive_empty = 0;

                // Convert timestamp once
                let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
                    .unwrap_or(chrono::DateTime::UNIX_EPOCH);

                batch.push((datetime, level, message));

                // Process full batch
                if batch.len() >= BATCH_SIZE {
                    process_batch(&batch);
                    batch.clear();
                }
            }
            Ok(None) => {
                // Process any remaining logs first before incrementing consecutive_empty
                if !batch.is_empty() {
                    consecutive_empty = 0; // Reset counter when we process logs
                    process_batch(&batch);
                    batch.clear();
                }

                // Exponential backoff with max cap
                let wait_time = match consecutive_empty {
                    0..=4 => 10,
                    5..=19 => 50,
                    _ => 200,
                };

                std::thread::sleep(Duration::from_millis(wait_time));
                consecutive_empty += 1;
            }
            Err(_e) => {
                std::thread::sleep(Duration::from_millis(10));
                consecutive_empty += 1;
            }
        }
    }
}

// Extract batch processing to a separate function
fn process_batch(batch: &Vec<(chrono::DateTime<chrono::Utc>, u8, String)>) {
    // Replace with actual batch processing logic
    for (datetime, _level, message) in batch {
        // This log line is active - if you're not seeing this, there might be a log level issue
        // or no logs are being produced for the proxy
        // ID:14538016447660569718, TYPE:UPSTREAM, CONN:TCP, SIZE:195, STAT:UP@ON, SRC:0.0.0.0:3020, DST:127.0.0.1:3004
        // log::info!("PXY : Processing: {} - {}: {}", datetime, level, message);

        let message_inner = message.as_str();
        let message_inner = message_inner.split('|').collect::<Vec<&str>>();
        
        let message_inner = {
            if message_inner.len() > 1 {
                message_inner[1]
            } else {
                continue; // Skip if the message format is not as expected
            }
        };

        // Initialize variables to store extracted values
        let mut conn_id = String::new();
        let mut msg_type = "";
        let mut conn_type = "";
        let mut size: u64 = 0;
        let mut status = "";
        let mut source = String::new();
        let mut destination = String::new();

        // Direct field extraction
        for field in message_inner.split(',') {
            let field = field.trim();

            if let Some(colon_idx) = field.find(':') {
                let key = &field[..colon_idx].trim();
                let value = &field[colon_idx + 1..].trim();

                // Direct field matching without HashMap
                match *key {
                    "ID" => conn_id = value.to_string(),
                    "TYPE" => msg_type = value,
                    "CONN" => conn_type = value,
                    "SIZE" => size = value.parse().unwrap_or(0),
                    "STAT" => status = value,
                    "SRC" => source = value.to_string(),
                    "DST" => destination = value.to_string(),
                    _ => {} // Ignore unknown fields
                }
            }
        }

        // Determine request vs response
        let (conn_req, conn_res, bytes_in, bytes_out) = match msg_type {
            "DOWNSTREAM" => (1, 0, size, 0),
            "UPSTREAM" => (0, 1, 0, size),
            _ => (0, 0, 0, 0),
        };

        // Convert status to numeric code
        let status_code = if status == "N/A" {
            0
        } else {
            status.parse::<i32>().unwrap_or(0)
        };

        // Create and append the TemporaryLog
        let log_entry = TemporaryLog {
            date_time: datetime.clone(),
            conn_id,
            conn_type: conn_type.to_string(),
            peer: (source, destination),
            status_code,
            conn_req,
            conn_res,
            bytes_in: bytes_in as i32,
            bytes_out: bytes_out as i32,
        };

        let _ = tlog_proxy::append_data(log_entry);
    }
}
