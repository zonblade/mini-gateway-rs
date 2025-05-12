use crate::module::memory_log::core::{LogConsumer, MAX_MEMORY_SIZE, PROXY_LOGGER_NAME};
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
    
    log::info!("Starting proxy log processing, queue size: {}", log_consumer.queue_size());
    
    loop {
        // Periodic health check
        if last_health_check.elapsed() >= health_check_interval {
            
            // If we haven't received anything in a while, try to reconnect
            if consecutive_empty > 2000 {
                log::warn!("Too many consecutive empty results ({}), attempting to recreate consumer", consecutive_empty);
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
    for (datetime, level, message) in batch {
        // This log line is active - if you're not seeing this, there might be a log level issue
        // or no logs are being produced for the proxy
        // ID:14538016447660569718, TYPE:UPSTREAM, CONN:TCP, SIZE:195, STAT:UP@ON, SRC:0.0.0.0:3020, DST:127.0.0.1:3004
        // log::info!("PXY : Processing: {} - {}: {}", datetime, level, message);
        
        // Additional processing can be added here
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