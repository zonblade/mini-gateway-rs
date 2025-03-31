// Parsing utilities for protocol module

use super::types::ConnectionParams;
use std::collections::HashMap;

/// # Parse Connection Parameters
///
/// Parses a handshake message into structured connection parameters.
///
/// This function takes a raw handshake string from a client connection and extracts
/// structured information according to the protocol format. It processes messages in
/// the format `gate://service_name/action?param1=value1&param2=value2` and converts
/// them into a structured `ConnectionParams` object.
///
/// ## Format Specification
///
/// The protocol handshake follows the URI-like format:
///
/// ```text
/// <prefix><service_name>/<action>?<parameters>
/// ```
///
/// Where:
/// - `prefix` is the protocol identifier (e.g., "gate://")
/// - `service_name` is the name of the target service
/// - `action` is the operation to perform on the service
/// - `parameters` are optional key-value pairs in query string format
///
/// ## Parameters
///
/// * `handshake` - The raw handshake message received from the client
/// * `prefix` - The protocol prefix to strip from the beginning of the message (e.g., "gate://")
///
/// ## Returns
///
/// A [`ConnectionParams`] struct containing the parsed service name, action, and parameters.
///
/// ## Examples
///
/// ```
/// let handshake = "gate://auth_service/login?username=admin&password=secret";
/// let params = parse_connection_params(handshake, "gate://");
///
/// assert_eq!(params.service, "auth_service");
/// assert_eq!(params.action, "login");
/// assert_eq!(params.parameters.get("username"), Some(&"admin".to_string()));
/// assert_eq!(params.parameters.get("password"), Some(&"secret".to_string()));
/// ```
///
/// ## Edge Cases
///
/// - If the handshake doesn't start with the specified prefix, an empty `ConnectionParams` is returned
/// - If there's no service or action, those fields will be empty strings
/// - Parameters without a value will get an empty string as value
///
pub fn parse_connection_params(handshake: &str, prefix: &str) -> ConnectionParams {
    // Default implementation - can be expanded to parse more parameters
    // Format: gate://service_name/action?param1=value1&param2=value2
    let mut params = ConnectionParams {
        service: String::new(),
        action: String::new(),
        parameters: HashMap::new(),
    };
    
    // Remove prefix
    if let Some(rest) = handshake.strip_prefix(prefix) {
        // Parse service name and action
        if let Some(service_end) = rest.find('/') {
            params.service = rest[..service_end].trim().to_string();
            
            // Parse action
            let after_service = &rest[service_end + 1..];
            if let Some(query_start) = after_service.find('?') {
                params.action = after_service[..query_start].trim().to_string();
                
                // Parse query parameters
                let query_str = &after_service[query_start + 1..];
                for param_pair in query_str.split('&') {
                    if let Some(eq_pos) = param_pair.find('=') {
                        let key = param_pair[..eq_pos].trim().to_string();
                        let value = param_pair[eq_pos + 1..].trim().to_string();
                        params.parameters.insert(key, value);
                    }
                }
            } else {
                params.action = after_service.trim().to_string();
            }
        }
    }
    
    params
}