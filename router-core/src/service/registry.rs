use redis::Client;
use std::collections::HashMap;
use std::thread;

use crate::system;


pub fn client() {
    log::info!("Starting Watcher client thread...");

    thread::spawn(move || {
        log::info!("Stream Thread Starting...");

        // Connect to Redis
        let client = match Client::open("redis://127.0.0.1:6379") {
            Ok(client) => client,
            Err(e) => {
                log::error!("Failed to connect to Redis: {:?}", e);
                return;
            }
        };

        // Get a connection
        let mut con = match client.get_connection() {
            Ok(con) => con,
            Err(e) => {
                log::error!("Failed to get Redis connection: {:?}", e);
                return;
            }
        };

        log::info!("Connected to Database");

        // Create a stream key if it doesn't exist
        let stream_key = "updates_stream";

        // Initial stream ID ($ means we only want new messages from now on)
        let mut last_id = "$".to_string();

        loop {
            // Block and wait for new messages in the stream
            let result: Result<Vec<redis::Value>, redis::RedisError> = redis::cmd("XREAD")
                .arg("BLOCK")
                .arg(0) // Block indefinitely (0 = no timeout)
                .arg("STREAMS")
                .arg(stream_key)
                .arg(&last_id)
                .query(&mut con);

            let result = match result {
                Ok(response) => response,
                Err(e) => {
                    log::error!("Redis error: {:?}", e);
                    // Sleep briefly before reconnecting
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }
            };

            // Process stream entries
            if let Some(entries) = process_stream_response(result) {
                for (id, data) in entries {
                    // Update last_id for next iteration
                    last_id = id;

                    // Process the data
                    log::debug!("Redis stream data: {:?}", data);

                    // If the data indicates we need a restart
                    if let Some(value) = data.get("action") {
                        if value == "restart" {
                            system::terminator::service::init();
                        }

                        // other action in the future
                    }
                }
            }
        }
    });
}

// Helper function to parse Redis stream response
fn process_stream_response(
    response: Vec<redis::Value>,
) -> Option<Vec<(String, std::collections::HashMap<String, String>)>> {
    // Ensure there's at least one element in the response.
    if response.is_empty() {
        return None;
    }

    // Convert the first element of the response into our expected structure:
    // (stream_name, messages)
    // where messages is Vec<(message_id, Vec<field_and_value_strings>)>
    let stream_info: (String, Vec<(String, Vec<String>)>) =
        match redis::from_redis_value(&response[0]) {
            Ok(info) => info,
            Err(_) => return None,
        };

    let messages = stream_info.1;
    let mut result = Vec::new();

    // Iterate over each message.
    for (msg_id, flat_fields) in messages {
        let mut data = HashMap::new();
        let mut iter = flat_fields.into_iter();

        // The flat vector contains alternating field and value.
        while let (Some(field), Some(value)) = (iter.next(), iter.next()) {
            data.insert(field, value);
        }

        result.push((msg_id, data));
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

// To publish to the stream from elsewhere in your code:
// redis::cmd("XADD").arg("updates_stream").arg("*").arg("action").arg("restart").arg("value").arg("some_value").execute(&mut con);
