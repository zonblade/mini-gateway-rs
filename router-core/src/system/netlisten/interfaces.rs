use pnet::datalink::{self, NetworkInterface};

/// Functions for managing and listing network interfaces
pub struct InterfaceManager;

impl InterfaceManager {
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

    /// Get a specific interface by name
    pub fn get_interface_by_name(interface_name: &str) -> Option<NetworkInterface> {
        let interfaces = datalink::interfaces();
        interfaces.into_iter().find(|iface| iface.name == interface_name)
    }
}
