use crate::module::{
    ai_security::{self, MlFeatureLog},
    memory_log::core::{LogConsumer, GATEWAY_LOGGER_NAME, MAX_MEMORY_SIZE},
    temporary_log::{tlog_gateway, TemporaryLog},
};
use std::time::{Duration, Instant};

pub async fn listen() {
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
                batch.shrink_to_fit();
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
                    process_batch(&batch).await;
                    batch.clear();
                    batch.shrink_to_fit();
                }
            }
            Ok(None) => {
                // Process any remaining logs
                if !batch.is_empty() {
                    consecutive_empty = 0;
                    process_batch(&batch).await;
                    batch.clear();
                    batch.shrink_to_fit();
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
async fn process_batch(batch: &Vec<(chrono::DateTime<chrono::Utc>, u8, String)>) {
    // FAST PATH: Check if AI is enabled ONCE per batch
    let ai_enabled = ai_security::is_ai_enabled();

    // Replace with actual batch processing logic
    for (datetime, _level, message) in batch {
        // Process each log entry (commented out to avoid unnecessary prints)
        // Uncomment if processing is actually needed
        // | ID:17936787362358910377, TYPE:REQ, CONN:HTTP, SIZE:0, STAT:N/A, SRC:127.0.0.1:42615, DST:127.0.0.1:3004 |
        // log::info!("GWX : Processing: {} - {}: {}", datetime, level, message);
        let message_inner = message.as_str();
        let message_vector = message_inner.split('|').collect::<Vec<&str>>();

        let message_inner = {
            if message_vector.len() > 1 {
                message_vector[1]
            } else {
                continue; // Skip if the message format is not as expected
            }
        };

        // UNUSED: destructured for pattern match only
        let _header_inner = {
            if message_vector.len() > 2 {
                message_vector[2]
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
        let mut path_src = String::new();
        let mut path_dst = String::new();

        // ML feature fields (only used when AI enabled)
        let mut duration_ms: f32 = 0.0;
        let mut tcp_rtt: u32 = 0;
        let mut tcp_retrans: u32 = 0;
        let mut tcp_lost: u32 = 0;
        let mut protocol = String::new();
        let mut http_method = String::new();
        let mut tcp_send_wnd: u32 = 0;
        let mut tcp_recv_wnd: u32 = 0;
        let mut tcp_send_mss: u32 = 0;
        let mut tcp_recv_mss: u32 = 0;
        let mut tcp_bytes_acked: u64 = 0;
        let mut tcp_segs_in: u32 = 0;
        let mut tcp_segs_out: u32 = 0;
        let mut tls_version = String::new();
        let mut client_ip = String::new();
        let mut client_port: u16 = 0;
        let mut server_ip = String::new();
        let mut server_port: u16 = 0;

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
                    "PTH_SRC" => path_src = value.to_string(),
                    "PTH_DST" => path_dst = value.to_string(),

                    // ML fields - only parse if AI enabled
                    "DUR" if ai_enabled => duration_ms = value.parse().unwrap_or(0.0),
                    "TCP_RTT" if ai_enabled => tcp_rtt = value.parse().unwrap_or(0),
                    "TCP_RETRANS" if ai_enabled => tcp_retrans = value.parse().unwrap_or(0),
                    "TCP_LOST" if ai_enabled => tcp_lost = value.parse().unwrap_or(0),
                    "PROTO" if ai_enabled => protocol = value.to_string(),
                    "METHOD" if ai_enabled => http_method = value.to_string(),
                    "TCP_SND_WND" if ai_enabled => tcp_send_wnd = value.parse().unwrap_or(0),
                    "TCP_RCV_WND" if ai_enabled => tcp_recv_wnd = value.parse().unwrap_or(0),
                    "TCP_SND_MSS" if ai_enabled => tcp_send_mss = value.parse().unwrap_or(0),
                    "TCP_RCV_MSS" if ai_enabled => tcp_recv_mss = value.parse().unwrap_or(0),
                    "TCP_BYTES_ACKED" if ai_enabled => tcp_bytes_acked = value.parse().unwrap_or(0),
                    "TCP_SEGS_IN" if ai_enabled => tcp_segs_in = value.parse().unwrap_or(0),
                    "TCP_SEGS_OUT" if ai_enabled => tcp_segs_out = value.parse().unwrap_or(0),
                    "TLS_VER" if ai_enabled => tls_version = value.to_string(),
                    "CLIENT" if ai_enabled => {
                        if let Some((ip, port)) = value.rsplit_once(':') {
                            client_ip = ip.to_string();
                            client_port = port.parse().unwrap_or(0);
                        }
                    }
                    "SERVER" if ai_enabled => {
                        if let Some((ip, port)) = value.rsplit_once(':') {
                            server_ip = ip.to_string();
                            server_port = port.parse().unwrap_or(0);
                        }
                    }

                    // All ML fields are now parsed above when ai_enabled
                    _ if !ai_enabled
                        && matches!(
                            *key,
                            "DUR"
                                | "PROTO"
                                | "METHOD"
                                | "TCP_RTT"
                                | "TCP_RETRANS"
                                | "TCP_LOST"
                                | "TCP_SND_WND"
                                | "TCP_RCV_WND"
                                | "TCP_SND_MSS"
                                | "TCP_RCV_MSS"
                                | "TCP_BYTES_ACKED"
                                | "TCP_SEGS_IN"
                                | "TCP_SEGS_OUT"
                                | "TLS_VER"
                                | "CLIENT"
                                | "SERVER"
                        ) => {}

                    _ => {} // Ignore unknown fields
                }
            }
        }

        // Determine request vs response
        let (conn_req, conn_res, bytes_in, bytes_out) = match msg_type {
            "REQ" => (1, 0, size, 0),
            "RES" => (0, 1, 0, size),
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
            date_time: *datetime,
            conn_id: conn_id.clone(),
            conn_type: conn_type.to_string(),
            peer: (source, destination),
            status_code,
            conn_req,
            conn_res,
            bytes_in: bytes_in as i32,
            bytes_out: bytes_out as i32,
            path_src: if path_src.is_empty() {
                None
            } else {
                Some(path_src)
            },
            path_dst: if path_dst.is_empty() {
                None
            } else {
                Some(path_dst)
            },
        };

        let _ = tlog_gateway::append_data(log_entry);

        // Send to AI inference if enabled (only on response which has all metrics)
        if ai_enabled && msg_type == "RES" {
            let ml_log = MlFeatureLog {
                conn_id: conn_id.clone(),
                duration_ms,
                http_status: status_code as f32,
                http_method,
                protocol,
                size_in: bytes_in as f32,
                size_out: bytes_out as f32,
                tcp_rtt: tcp_rtt as f32,
                tcp_retrans: tcp_retrans as f32,
                tcp_lost: tcp_lost as f32,
                tcp_send_wnd: tcp_send_wnd as f32,
                tcp_recv_wnd: tcp_recv_wnd as f32,
                tcp_send_mss: tcp_send_mss as f32,
                tcp_recv_mss: tcp_recv_mss as f32,
                tcp_bytes_acked: tcp_bytes_acked as f32,
                tcp_segs_in: tcp_segs_in as f32,
                tcp_segs_out: tcp_segs_out as f32,
                tls_version,
                client_ip,
                client_port: client_port as f32,
                server_ip,
                server_port: server_port as f32,
            };
            ai_security::send_to_inference(ml_log);
        }
    }
}
