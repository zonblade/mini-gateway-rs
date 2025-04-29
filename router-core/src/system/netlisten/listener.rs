use std::sync::Arc;
use std::time::Duration;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, Config};
use pnet::packet::ethernet::EthernetPacket;
use log::{debug, error, info, warn};

use super::types::NetworkListener;
use super::interfaces::InterfaceManager;
use super::permissions::PermissionChecker;
use super::packet_handler::PacketHandler;

impl NetworkListener {
    /// Start listening on the network interface
    pub fn start(&self) -> Result<(), String> {
        // Check for required permissions first
        if !PermissionChecker::has_capture_permissions() {
            return Err("Insufficient permissions for packet capture. This operation requires root privileges or CAP_NET_RAW capability. Try running with sudo or set the required capabilities on the binary.".to_string());
        }

        // Set the running state to true
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = true;
        drop(is_running);

        // Clone the running flag and packet counter for the listening thread
        let is_running = Arc::clone(&self.is_running);
        let packet_count = Arc::clone(&self.packet_count);
        
        // Find the interface with the name specified during initialization
        let interface = InterfaceManager::get_interface_by_name(&self.interface_name)
            .ok_or_else(|| format!("Network interface {} not found", self.interface_name))?;
        
        // Clone interface name for the thread
        let interface_name = self.interface_name.clone();
        
        // Log interface info for debugging
        info!("Starting listener on interface: {} (MAC: {:?})", 
              interface_name, 
              interface.mac.map(|m| m.to_string()).unwrap_or_else(|| "unknown".to_string()));
        
        // Create a new thread for packet capture
        std::thread::spawn(move || {
            // Configure the channel to use promiscuous mode
            let config = Config {
                read_timeout: Some(Duration::from_millis(100)),
                write_timeout: None,
                read_buffer_size: 65536,
                write_buffer_size: 65536,
                channel_type: datalink::ChannelType::Layer2,
                promiscuous: true,  // This enables promiscuous mode
                linux_fanout: None,
                bpf_fd_attempts: 1000,
            };
            
            // Create a channel to receive packets on the specified interface
            match datalink::channel(&interface, config) {
                Ok(Ethernet(tx, mut rx)) => {
                    info!("Successfully started listening on interface: {}", interface_name);
                    
                    // Continue capturing packets while the listener is running
                    while *is_running.lock().unwrap() {
                        match rx.next() {
                            Ok(packet) => {
                                // Increment packet counter
                                let mut count = packet_count.lock().unwrap();
                                *count += 1;
                                
                                // Process the captured packet
                                if let Some(ethernet) = EthernetPacket::new(packet) {
                                    PacketHandler::handle_packet(&ethernet, *count);
                                }
                            }
                            Err(e) => {
                                // Only log actual errors, not timeouts
                                if !e.to_string().contains("Timeout") {
                                    error!("An error occurred while reading packet: {}", e);
                                }
                                // Short pause to prevent CPU spinning on error
                                std::thread::sleep(Duration::from_millis(10));
                            }
                        }
                    }
                    info!("Stopped listening on interface: {}", interface_name);
                }
                Ok(_) => error!("Unhandled channel type"),
                Err(e) => error!("Failed to create datalink channel: {} (Interface: {})", e, interface_name),
            }
        });
        
        Ok(())
    }
}

/// Main function to start network capture
pub fn network_capture() {
    // List all interfaces with detailed information
    let interfaces_info = InterfaceManager::list_interfaces_with_info();
    println!("Available network interfaces:");
    for (name, info) in &interfaces_info {
        println!("- Interface: {}", name);
        for detail in info {
            println!("  {}", detail);
        }
    }
    
    // Get the list of available interfaces
    let available_interfaces = InterfaceManager::list_available_interfaces();
    
    if available_interfaces.is_empty() {
        println!("No suitable network interfaces found! Make sure you have active network interfaces.");
        std::process::exit(1);
    }
    
    // Choose the first available interface that's not loopback
    let default_interface = &available_interfaces[0];
    println!("Selected network interface: {} (from {} available interfaces)", 
             default_interface, available_interfaces.len());
    
    // Create the network listener with the selected interface
    let network_listener = NetworkListener::new(default_interface);
    
    // Try to start listening on the network interface
    match network_listener.start() {
        Ok(_) => println!("Successfully started network listener on interface: {}", default_interface),
        Err(e) => {
            println!("Failed to start network listener: {}", e);
            println!("Make sure you have the necessary permissions to capture packets.");
            println!("Try running with 'sudo' or set the required capabilities on the binary:");
            println!("sudo setcap cap_net_raw+ep /path/to/your-binary");
        }
    }
    
    // Print a message to generate some network traffic
    println!("Listening for network traffic...");
    println!("Try generating some traffic by opening a website or using a network utility");
}
