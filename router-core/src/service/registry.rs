//! # Registry Service Module
//! 
//! This module provides functionality for a persistent registry service that monitors 
//! Redis streams for system command events and acts upon them. It implements a background
//! client thread that continuously watches for messages in a specified Redis stream and
//! triggers appropriate system actions in response.
//! 
//! ## Architecture
//! 
//! The service operates as a long-running background thread that:
//! 1. Connects to a Redis instance
//! 2. Subscribes to a specific stream key ("updates_stream")
//! 3. Blocks waiting for new messages
//! 4. Processes received messages and takes appropriate action
//! 
//! ## Supported Actions
//! 
//! Currently supported actions received via the stream:
//! - `restart`: Triggers system termination sequence via `system::terminator::service::init()`

use redis::Client;
use std::collections::HashMap;
use std::thread;

use crate::system;


/// # Registry Client Service
///
/// Initializes and starts a background thread that connects to Redis and listens for
/// system command messages on a predefined stream.
///
/// ## Behavior
///
/// This function spawns a non-blocking background thread that:
/// 1. Establishes a connection to the Redis server at 127.0.0.1:6379
/// 2. Subscribes to the "updates_stream" stream
/// 3. Processes incoming messages in an infinite loop
/// 4. Takes action based on message content (e.g., system restart)
///
/// ## Error Handling
///
/// The function handles connection errors and Redis query errors internally:
/// - If initial connection fails, it logs the error and returns
/// - If a Redis operation fails during the stream reading loop, it logs the error,
///   sleeps for 1 second, and retries
///
/// ## Stream Message Format
///
/// Expected message format is a key-value map where:
/// - `action`: Defines the command to execute (e.g., "restart")
/// - Additional fields may be present depending on the action
///
/// ## Thread Safety
///
/// This function uses `thread::spawn` to create a background thread that runs independently
/// of the calling context. The thread owns all its resources through move closures.
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

/// # Process Redis Stream Response
///
/// Parses the raw Redis XREAD command response into a structured format for easier processing.
///
/// ## Arguments
///
/// * `response` - A vector of Redis values returned from an XREAD command
///
/// ## Returns
///
/// * `Some(Vec<(String, HashMap<String, String>)>)` - A vector of tuples containing:
///   - Message ID as a String
///   - Message payload as a HashMap of field-value pairs
/// * `None` - If the response is empty or cannot be parsed
///
/// ## Data Structure Conversion
///
/// This function converts the nested Redis response structure into a more usable Rust format:
///
/// ```text
/// Redis XREAD response format:
/// [
///   [stream_name, [
///     [msg_id1, [field1, value1, field2, value2, ...]],
///     [msg_id2, [field1, value1, field2, value2, ...]]
///   ]]
/// ]
///
/// Converted to:
/// [
///   (msg_id1, {field1: value1, field2: value2, ...}),
///   (msg_id2, {field1: value1, field2: value2, ...})
/// ]
/// ```
///
/// ## Error Handling
///
/// If any parsing error occurs or the response structure doesn't match the expected format,
/// this function returns `None`.
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

// # Usage Example
//
// To publish to the stream from elsewhere in your code:
// redis::cmd("XADD").arg("updates_stream").arg("*").arg("action").arg("restart").arg("value").arg("some_value").execute(&mut con);
//
// Parameters:
// - "updates_stream": The stream key to publish to
// - "*": Auto-generate a message ID
// - Followed by alternating field-value pairs:
//   - "action" -> "restart": The command to execute
//   - "value" -> "some_value": Additional data for the command
