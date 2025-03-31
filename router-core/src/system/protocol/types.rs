// Type definitions for protocol module

use std::collections::HashMap;

/// # Connection Parameters
///
/// This struct represents the parsed parameters from a client handshake message
/// in the protocol implementation. It holds all the information needed to process
/// a connection after successful handshake negotiation.
///
/// ## Fields
///
/// * `service` - The target service name that the client wants to connect to.
///   This identifies which service in the Gateway should handle the request.
///
/// * `action` - The specific action or method to be performed on the specified service.
///   This allows services to implement different behaviors based on the requested action.
///
/// * `parameters` - A collection of key-value pairs that provide additional information
///   for the service to process the request. These are parsed from the query string
///   portion of the handshake message.
///
/// ## Format
///
/// The connection parameters are extracted from a handshake following the format:
/// ```
/// gate://service_name/action?param1=value1&param2=value2
/// ```
///
/// ## Examples
///
/// A typical handshake message might look like:
/// ```
/// gate://user_service/authenticate?username=johndoe&session=abc123
/// ```
///
/// This would be parsed into a `ConnectionParams` with:
/// * service = "user_service"
/// * action = "authenticate"
/// * parameters = {"username": "johndoe", "session": "abc123"}
///
#[derive(Debug, Clone)]
pub struct ConnectionParams {
    /// The target service name to connect to
    pub service: String,
    
    /// The action to perform on the service
    pub action: String,
    
    /// Additional parameters passed with the request
    pub parameters: HashMap<String, String>,
}