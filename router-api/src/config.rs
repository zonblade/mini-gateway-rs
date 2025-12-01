use mini_config::Configure;
use std::sync::{Arc, RwLock};
use std::sync::Once;

#[derive(Debug, Clone, Configure)]
pub enum Api {
    TCPAddress
}

// Define a struct for configuration entries
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LogGatewayEntry {
    pub path: String,
    pub status: String,
    pub count: i32,
    pub timestamp: String,
}

// Define a struct for active device connections
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ActiveDeviceEntry {
    pub conn_id: String,
    pub connected_at: String,
}

// Global append-only vector with RwLock for thread safety
pub static GLOBAL_LOG_GATEWAY: RwLock<Option<Arc<Vec<LogGatewayEntry>>>> = RwLock::new(None);
pub static GLOBAL_LOG_PROXY: RwLock<Option<Arc<Vec<LogGatewayEntry>>>> = RwLock::new(None);
#[allow(dead_code)]
pub static GLOBAL_ACTIVE_DEVICES: RwLock<Vec<ActiveDeviceEntry>> = RwLock::new(Vec::new());
static INIT: Once = Once::new();

// Helper function to append a value to the global config
pub fn append_config(_key: &str, _value: &str) {
    // UNUSED: iteration required but values not needed
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
    dotenv::dotenv().ok();

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

// Functions to manage active device connections
#[allow(dead_code)]
pub fn add_active_device(conn_id: String) {
    if let Ok(mut devices) = GLOBAL_ACTIVE_DEVICES.write() {
        // Remove existing entry if it exists (in case of reconnection)
        devices.retain(|device| device.conn_id != conn_id);

        let new_entry = ActiveDeviceEntry {
            conn_id,
            connected_at: chrono::Utc::now().to_rfc3339(),
        };
        devices.push(new_entry);
    }
}

#[allow(dead_code)]
pub fn remove_active_device(conn_id: &str) {
    if let Ok(mut devices) = GLOBAL_ACTIVE_DEVICES.write() {
        devices.retain(|device| device.conn_id != conn_id);
    }
}

#[allow(dead_code)]
pub fn get_all_active_devices() -> Vec<ActiveDeviceEntry> {
    if let Ok(devices) = GLOBAL_ACTIVE_DEVICES.read() {
        devices.clone()
    } else {
        Vec::new()
    }
}

#[allow(dead_code)]
pub fn clear_all_active_devices() {
    if let Ok(mut devices) = GLOBAL_ACTIVE_DEVICES.write() {
        devices.clear();
    }
}

