use crate::module::memory_log::core::{LogConsumer, MAX_MEMORY_SIZE, GATEWAY_LOGGER_NAME};
use std::time::{Duration, Instant};

pub fn listen() {
    log::info!("Starting log consumer...");
    
    // Open shared memory
    let log_consumer =
        LogConsumer::new(GATEWAY_LOGGER_NAME, MAX_MEMORY_SIZE).expect("Failed to open shared memory");

    // Pre-allocate batch with capacity
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    const BATCH_SIZE: usize = 100;
    
    // Status tracking
    let mut consecutive_empty = 0;
    let mut message_counter = 0;
    let mut last_status_print = Instant::now();
    let status_interval = Duration::from_secs(10); // Print status once per second
    
    log::info!("Starting log processing, queue size: {}", log_consumer.queue_size());

    loop {
        // Only check queue size and print status periodically
        let now = Instant::now();
        if now.duration_since(last_status_print) >= status_interval {
            log::warn!(
                "Processed {} messages, Errors {}, queue size: {}\r",
                message_counter, consecutive_empty, log_consumer.queue_size()
            );
            last_status_print = now;
        }

        // Try to get a log entry with a timeout
        match log_consumer.get_log_with_timeout(10) {
            Ok(Some((timestamp, level, message))) => {
                consecutive_empty = 0;
                message_counter += 1;

                // Convert timestamp once
                let datetime = chrono::DateTime::from_timestamp(timestamp as i64,
                    0).unwrap_or(chrono::DateTime::UNIX_EPOCH);
                
                batch.push((datetime, level, message));

                // Process full batch
                if batch.len() >= BATCH_SIZE {
                    process_batch(&batch);
                    batch.clear();
                }
            }
            Ok(None) => {
                consecutive_empty += 1;

                // Process any remaining logs
                if !batch.is_empty() {
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
            }
            Err(e) => {
                consecutive_empty += 1;
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

// Extract batch processing to a separate function
fn process_batch(batch: &Vec<(chrono::DateTime<chrono::Utc>, u8, String)>) {
    // Replace with actual batch processing logic
    for (datetime, level, message) in batch {
        // Process each log entry (commented out to avoid unnecessary prints)
        // Uncomment if processing is actually needed
        // log::debug!("Processing: {} - {}: {}", datetime, level, message);
    }
}