use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::arp::{ArpPacket, ArpOperation};
use pnet::packet::Packet;
use log::{debug, error, info, warn};
use std::net::Ipv4Addr;

pub struct PacketHandler;

impl PacketHandler {
    /// Handle and process a captured Ethernet packet
    pub fn handle_packet(ethernet: &EthernetPacket, packet_id: u64) {
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
                    let sender_proto = arp.get_sender_proto_addr();
                    let target_proto = arp.get_target_proto_addr();
                    
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
                        "|NET| ARP Details: Hardware Type: {:?}, Protocol Type: {:?}, HW Addr Length: {}, Proto Addr Length: {}",
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
}
