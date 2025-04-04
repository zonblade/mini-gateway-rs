//! # Configuration Module for Mini-Gateway Router Core
//! 
//! This module provides configuration structures and utilities for the router core component
//! of the mini-gateway system. It defines data structures for routing configuration, 
//! connection endpoints, and provides utilities for loading and managing configuration.
//! 
//! ## Configuration Categories
//! 
//! This module includes several key configuration components:
//! - Default port configurations for error handling and TLS honeypot
//! - Routing data enums for configuration storage and retrieval
//! - Proxy and Gateway node structures that define connection endpoints
//! - Utility functions for configuration initialization and data conversion
//! 
//! ## Usage
//! 
//! The configuration system uses the `mini-config` crate for settings management,
//! which provides the `Configure` trait for simple configuration storage and retrieval.

use mini_config::Configure;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Default port configuration for special service endpoints.
///
/// This structure defines the default ports for error handling and security services:
/// - 404 error handler service
/// - 500 error handler service
/// - TLS honeypot for security monitoring
pub struct DefaultPort {
    /// Port for handling 404 (Not Found) errors
    pub p404: &'static str,
    
    /// Port for handling 500 (Internal Server Error) errors
    pub p500: &'static str,
    
    /// Port for TLS honeypot service to monitor and log suspicious connection attempts
    pub tls_honeypot: &'static str,
}

/// Constant definition of default ports for special endpoints.
///
/// These values are used when no custom configuration is provided.
pub(crate) const DEFAULT_PORT: DefaultPort = DefaultPort {
    p404: "127.0.0.1:60404",
    p500: "127.0.0.1:60500",
    tls_honeypot: "127.0.0.1:60443",
};

/// Routing data configuration keys.
///
/// This enum defines the configuration keys used to store and retrieve 
/// routing-related data through the `mini-config` system. Each variant
/// corresponds to a specific piece of routing configuration.
///
/// # Examples
///
/// ```
/// // Get the current proxy ID
/// let proxy_id = RoutingData::ProxyID.get::<String>();
///
/// // Set the gateway routing configuration
/// RoutingData::GatewayRouting.xset::<Vec<GatewayNode>>(gateway_nodes);
/// ```
#[derive(Debug, Clone, Configure)]
pub enum RoutingData {
    /// Key for the current proxy identifier
    ProxyID,
    
    /// Key for proxy routing configuration data
    ProxyRouting,
    
    /// Key for the current gateway identifier
    GatewayID,
    
    /// Key for gateway routing configuration data
    GatewayRouting,
}

/// General configuration keys.
///
/// This enum defines configuration keys for general system settings.
/// These settings are typically used for connectivity and core system
/// behavior rather than routing rules.
///
/// # Examples
///
/// ```
/// // Get the storage connection string
/// let storage_uri = GeneralConfig::StorageURI.get::<String>();
/// ```
#[derive(Debug, Clone, Configure)]
pub enum GeneralConfig {
    /// Storage system connection URI for component communication and persistence
    StorageURI,
    
    /// Storage system type (e.g., "memory", "file", "database")
    StorageType,
    
    /// Connection timeout in milliseconds
    ConnectionTimeout
}

/// Proxy node configuration.
///
/// This structure defines the configuration for a proxy endpoint, including
/// TLS settings, network addresses, and processing priority.
///
/// # Fields
///
/// * `tls` - Whether TLS encryption is enabled for this proxy
/// * `sni` - Server Name Indication for TLS (if applicable)
/// * `tls_pem` - Path to the TLS certificate PEM file (if applicable)
/// * `tls_key` - Path to the TLS private key file (if applicable)
/// * `addr_listen` - Address and port the proxy listens on (e.g., "0.0.0.0:443")
/// * `addr_target` - Target address to proxy requests to (e.g., "127.0.0.1:8080")
/// * `priority` - Processing priority (higher values = higher priority)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyNode {
    /// Whether TLS is enabled for this proxy node
    pub tls: bool,
    
    /// Server Name Indication value for TLS connections
    pub sni: Option<String>,
    
    /// Path to the TLS certificate PEM file
    pub tls_pem: Option<String>,
    
    /// Path to the TLS private key file
    pub tls_key: Option<String>,
    
    /// Network address this proxy listens on (e.g., "0.0.0.0:443")
    pub addr_listen: String,
    
    /// Target address to forward traffic to (e.g., "127.0.0.1:8080")
    pub addr_target: String,
}

/// Gateway node configuration.
///
/// This structure defines the configuration for a gateway endpoint, including
/// network addresses, path matching, and processing priority.
///
/// # Fields
///
/// * `priority` - Processing priority (higher values = higher priority)
/// * `addr_listen` - Address and port the gateway listens on (e.g., "0.0.0.0:80")
/// * `addr_target` - Target address to proxy requests to (e.g., "127.0.0.1:8080")
/// * `path_listen` - URI path pattern to match incoming requests against (e.g., "/api/*")
/// * `path_target` - Target path to rewrite matched paths to (e.g., "/")
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayNode {
    /// Processing priority (higher values = higher priority)
    pub priority: u8,
    
    /// Network address this gateway listens on (e.g., "0.0.0.0:80")
    pub addr_listen: String,
    
    /// Target address to forward traffic to (e.g., "127.0.0.1:8080")
    pub addr_target: String,
    
    /// URI path pattern to match incoming requests against (e.g., "/api/*")
    pub path_listen: String,
    
    /// Target path to rewrite matched paths to (e.g., "/")
    pub path_target: String,
}

/// Convert a JSON string to a typed struct.
///
/// This helper function deserializes a JSON string into a typed Rust struct,
/// panicking if deserialization fails.
///
/// # Arguments
///
/// * `json_str` - JSON string to deserialize
///
/// # Returns
///
/// The deserialized data structure of type T
///
/// # Panics
///
/// This function will panic if the JSON string cannot be deserialized into
/// the requested type T.
///
/// # Examples
///
/// ```
/// let json = r#"{"priority": 10, "addr_listen": "0.0.0.0:80", ...}"#;
/// let node: GatewayNode = str_to_json(json);
/// ```
pub fn str_to_json<T>(json_str: &str) -> T
where
    T: DeserializeOwned,
{
    serde_json::from_str(json_str).unwrap()
}

/// Initialize the configuration system with default values.
///
/// This function sets up the initial configuration state by:
/// 1. Setting default proxy and gateway IDs
/// 2. Initializing empty routing tables for proxies and gateways
///
/// This should be called once during system startup before any
/// configuration is loaded or routing is performed.
pub fn init(){
    // initiate the routing id
    RoutingData::ProxyID.set("-");
    RoutingData::GatewayID.set("-");
    // initiate the routing data
    RoutingData::GatewayRouting.xset::<Vec<GatewayNode>>(vec![]);
    RoutingData::ProxyRouting.xset::<Vec<ProxyNode>>(vec![]);
}