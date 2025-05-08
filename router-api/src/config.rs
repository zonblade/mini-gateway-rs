use mini_config::Configure;
use std::sync::{Arc, RwLock};
use std::sync::Once;

#[derive(Debug, Clone, Configure)]
pub enum Api {
    TCPAddress
}

// Define a struct for configuration entries
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LogGatewayEntry {
    pub path: String,
    pub status: String,
    pub count: i32,
    pub timestamp: String,
}

// Global append-only vector with RwLock for thread safety
pub static GLOBAL_LOG_GATEWAY: RwLock<Option<Arc<Vec<LogGatewayEntry>>>> = RwLock::new(None);
pub static GLOBAL_LOG_PROXY: RwLock<Option<Arc<Vec<LogGatewayEntry>>>> = RwLock::new(None);
static INIT: Once = Once::new();

// Helper function to append a value to the global config
pub fn append_config(_key: &str, _value: &str) {
    // if let Ok(mut config) = GLOBAL_LOG_GATEWAY.write() {
    //     let new_entry = LogGatewayEntry {
    //         key: key.to_string(),
    //         value: value.to_string(),
    //     };
        
    //     if config.is_none() {
    //         let mut vec = Vec::new();
    //         vec.push(new_entry);
    //         *config = Some(Arc::new(vec));
    //     } else {
    //         // Need to create a new vector when appending to maintain immutability
    //         let old_vec = config.as_ref().unwrap();
    //         let mut new_vec = old_vec.as_ref().clone();
    //         new_vec.push(new_entry);
    //         *config = Some(Arc::new(new_vec));
    //     }
    // }
}

pub fn init(){
    Api::TCPAddress.set("127.0.0.1:30099");
    
    // Initialize the global config only once
    INIT.call_once(|| {
        if let Ok(mut gateway_config) = GLOBAL_LOG_GATEWAY.write() {
            *gateway_config = Some(Arc::new(Vec::new()));
        }
        
        if let Ok(mut proxy_config) = GLOBAL_LOG_PROXY.write() {
            *proxy_config = Some(Arc::new(Vec::new()));
        }
    });
    
    // Add initial values
    append_config("tcp_address", "127.0.0.1:30099");
}