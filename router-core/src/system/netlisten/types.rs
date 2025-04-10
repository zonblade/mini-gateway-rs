use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct NetworkListener {
    pub interface_name: String,
    pub is_running: Arc<Mutex<bool>>,
    pub packet_count: Arc<Mutex<u64>>,
}

impl NetworkListener {
    /// Create a new network listener for the specified interface
    pub fn new(interface_name: &str) -> Self {
        Self {
            interface_name: interface_name.to_string(),
            is_running: Arc::new(Mutex::new(false)),
            packet_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Get the total number of packets captured
    pub fn get_packet_count(&self) -> u64 {
        *self.packet_count.lock().unwrap()
    }
    
    /// Stop the network listener
    pub fn stop(&self) {
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = false;
        log::info!("Stopping network listener on interface: {}", self.interface_name);
    }
}
