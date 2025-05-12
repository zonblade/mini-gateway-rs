use crate::module::{
    memory_log::core::{LogConsumer, GATEWAY_LOGGER_NAME, MAX_MEMORY_SIZE},
    temporary_log::{tlog_gateway, TemporaryLog},
};
use std::time::{Duration, Instant};

pub fn listen() {
    log::info!("Starting log consumer...");

    // Open shared memory
    let mut log_consumer = LogConsumer::new(GATEWAY_LOGGER_NAME, MAX_MEMORY_SIZE)
        .expect("Failed to open shared memory");

    // Pre-allocate batch with capacity
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    const BATCH_SIZE: usize = 100;

    // Status tracking
    let mut consecutive_empty = 0;
    // let mut message_counter = 0;
    let mut last_health_check = Instant::now();
    let health_check_interval = Duration::from_secs(60); // Check health every minute
                                                         // let mut last_status_print = Instant::now();
                                                         // let status_interval = Duration::from_secs(10); // Print status once per second

    log::info!(
        "Starting log processing, queue size: {}",
        log_consumer.queue_size()
    );

    loop {
        // Periodic health check
        if last_health_check.elapsed() >= health_check_interval {
            log::info!("Health check - queue size: {}", log_consumer.queue_size());

            // If we haven't received anything in a while, try to reconnect
            if consecutive_empty > 2000 {
                log::warn!(
                    "Too many consecutive empty results ({}), attempting to recreate consumer",
                    consecutive_empty
                );
                match LogConsumer::new(GATEWAY_LOGGER_NAME, MAX_MEMORY_SIZE) {
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
                // message_counter += 1;
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
                // Process any remaining logs
                if !batch.is_empty() {
                    consecutive_empty = 0;
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
    for (datetime, level, message) in batch {
        // Process each log entry (commented out to avoid unnecessary prints)
        // Uncomment if processing is actually needed
        // ID:17936787362358910377, TYPE:REQ, CONN:HTTP, SIZE:0, STAT:N/A, SRC:127.0.0.1:42615, DST:127.0.0.1:3004 |
        // log::info!("GWX : Processing: {} - {}: {}", datetime, level, message);

        // let _ = tlog_gateway::append_data(TemporaryLog{
        //     date_time: datetime.clone(),
        //     conn_id: String::new(),
        //     status_code: 0,
        //     conn_req: 0,
        //     conn_res: 0,
        //     bytes_in: 0,
        //     bytes_out: 0,
        // });
    }
}
