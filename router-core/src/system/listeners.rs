use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, Config, NetworkInterface};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::arp::{ArpPacket, ArpOperation};
use pnet::packet::Packet;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::{debug, error, info, warn};
use std::process::Command;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct NetworkListener {
    interface_name: String,
    is_running: Arc<Mutex<bool>>,
    packet_count: Arc<Mutex<u64>>,
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

    /// List all available network interfaces
    pub fn list_available_interfaces() -> Vec<String> {
        let interfaces = datalink::interfaces();
        interfaces
            .into_iter()
            .filter(|iface| {
                // Filter out inactive interfaces
                !iface.ips.is_empty() && 
                !iface.is_loopback() && 
                iface.is_up() && 
                iface.is_running()
            })
            .map(|iface| iface.name)
            .collect()
    }

    /// List all available network interfaces with additional information
    pub fn list_interfaces_with_info() -> Vec<(String, Vec<String>)> {
        let interfaces = datalink::interfaces();
        interfaces
            .iter()
            .map(|iface| {
                let mut info = Vec::new();
                
                // Add IP addresses
                if !iface.ips.is_empty() {
                    let ips: Vec<String> = iface.ips.iter()
                        .map(|ip| ip.to_string())
                        .collect();
                    info.push(format!("IPs: {}", ips.join(", ")));
                } else {
                    info.push("No IP addresses".to_string());
                }
                
                // Add status information
                let status = format!("Status: {}{}{}",
                    if iface.is_up() { "UP" } else { "DOWN" },
                    if iface.is_running() { ", RUNNING" } else { "" },
                    if iface.is_loopback() { ", LOOPBACK" } else { "" }
                );
                info.push(status);
                
                // Add MAC address if available
                if let Some(mac) = iface.mac {
                    info.push(format!("MAC: {}", mac));
                }
                
                (iface.name.clone(), info)
            })
            .collect()
    }

    /// Start listening on the network interface
    pub fn start(&self) -> Result<(), String> {
        // Check for required permissions first
        if !Self::has_capture_permissions() {
            return Err("Insufficient permissions for packet capture. This operation requires root privileges or CAP_NET_RAW capability. Try running with sudo or set the required capabilities on the binary.".to_string());
        }

        // Set the running state to true
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = true;
        drop(is_running);

        // Clone the running flag and packet counter for the listening thread
        let is_running = Arc::clone(&self.is_running);
        let packet_count = Arc::clone(&self.packet_count);
        
        // Get all interfaces
        let interfaces = datalink::interfaces();
        
        // Find the interface with the name specified during initialization
        let interface = interfaces
            .into_iter()
            .filter(|iface| iface.name == self.interface_name)
            .next()
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
                                    Self::handle_packet(&ethernet, *count);
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

    /// Stop the network listener
    pub fn stop(&self) {
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = false;
        info!("Stopping network listener on interface: {}", self.interface_name);
    }

    /// Get the total number of packets captured
    pub fn get_packet_count(&self) -> u64 {
        *self.packet_count.lock().unwrap()
    }

    /// Check if the current process has the necessary permissions for packet capture
    fn has_capture_permissions() -> bool {
        // Check if running as root (effective user ID is 0)
        if unsafe { libc::geteuid() } == 0 {
            return true;
        }

        // Check if the binary has CAP_NET_RAW capability
        let output = Command::new("getcap")
            .arg(std::env::current_exe().unwrap_or_default().to_string_lossy().into_owned())
            .output();
        
        match output {
            Ok(output) => {
                let cap_output = String::from_utf8_lossy(&output.stdout);
                cap_output.contains("cap_net_raw")
            }
            Err(_) => false,
        }
    }

    /// Handle and process a captured Ethernet packet
    fn handle_packet(ethernet: &EthernetPacket, packet_id: u64) {
        match ethernet.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                    match ipv4.get_next_level_protocol() {
                        IpNextHeaderProtocols::Tcp => {
                            if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                                info!(
                                    "|NET| TCP Packet #{}: {}:{} -> {}:{} (len: {})",
                                    packet_id,
                                    ipv4.get_source(),
                                    tcp.get_source(),
                                    ipv4.get_destination(),
                                    tcp.get_destination(),
                                    tcp.payload().len()
                                );
                            }
                        }
                        IpNextHeaderProtocols::Udp => {
                            if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                                info!(
                                    "|NET| UDP Packet #{}: {}:{} -> {}:{} (len: {})",
                                    packet_id,
                                    ipv4.get_source(),
                                    udp.get_source(),
                                    ipv4.get_destination(),
                                    udp.get_destination(),
                                    udp.payload().len()
                                );
                            }
                        }
                        protocol => {
                            info!(
                                "|NET| IPv4 Packet #{} with protocol: {:?}",
                                packet_id, protocol
                            );
                        }
                    }
                }
            }
            EtherTypes::Ipv6 => {
                info!("IPv6 Packet #{} (not fully parsed)", packet_id);
            }
            EtherTypes::Arp => {
                // Parse ARP packet
                if let Some(arp) = ArpPacket::new(ethernet.payload()) {
                    // Extract source and destination hardware addresses (MAC)
                    let sender_hw = arp.get_sender_hw_addr();
                    let target_hw = arp.get_target_hw_addr();
                    
                    // Extract source and destination protocol addresses (IP)
                    let sender_proto = Self::parse_ipv4_bytes(arp.get_sender_proto_addr());
                    let target_proto = Self::parse_ipv4_bytes(arp.get_target_proto_addr());
                    
                    // Get ARP operation (request or reply)
                    let operation = match arp.get_operation() {
                        ArpOperation(1) => "REQUEST",
                        ArpOperation(2) => "REPLY",
                        _ => "UNKNOWN",
                    };
                    
                    // Log the ARP packet details
                    info!(
                        "|NET| ARP Packet #{}: {} {} -> {} (HW: {} -> {}) (Operation: {})",
                        packet_id,
                        operation,
                        sender_proto,
                        target_proto,
                        sender_hw,
                        target_hw,
                        operation
                    );
                    
                    // Additional details for better understanding
                    debug!(
                        "|NET| ARP Details: Hardware Type: {}, Protocol Type: {:?}, HW Addr Length: {}, Proto Addr Length: {}",
                        arp.get_hardware_type(),
                        arp.get_protocol_type(),
                        arp.get_hw_addr_len(),
                        arp.get_proto_addr_len()
                    );
                }
            }
            ethertype => {
                info!("Other packet #{} with ethertype: {:?}", packet_id, ethertype);
            }
        }
    }
    
    /// Helper function to convert raw bytes to an IPv4 address
    fn parse_ipv4_bytes(bytes: &[u8]) -> Ipv4Addr {
        if bytes.len() >= 4 {
            Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3])
        } else {
            Ipv4Addr::new(0, 0, 0, 0) // Return a default address if bytes are insufficient
        }
    }
}

pub fn network_capture(){
    // List all interfaces with detailed information
    let interfaces_info = NetworkListener::list_interfaces_with_info();
    println!("Available network interfaces:");
    for (name, info) in &interfaces_info {
        println!("- Interface: {}", name);
        for detail in info {
            println!("  {}", detail);
        }
    }
    
    // Get the list of available interfaces
    let available_interfaces = NetworkListener::list_available_interfaces();
    
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