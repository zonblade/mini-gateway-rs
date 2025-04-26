use std::net::UdpSocket;
use std::str;
use std::thread;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use crossbeam_channel::{Sender, Receiver, bounded, unbounded};

pub struct LogMessage {
    pub source_ip: String,
    pub source_port: u16,
    pub message: String,
    pub timestamp: std::time::SystemTime,
}

impl LogMessage {
    pub fn formatted(&self) -> String {
        // format!("[{}:{}] {}", self.source_ip, self.source_port, self.message)
        format!("{}", self.message)
    }
}

pub struct UdpLogFetcher {
    logs: Arc<RwLock<Vec<String>>>,
    socket: Option<UdpSocket>,
    running: Arc<RwLock<bool>>,
    tx: Sender<LogMessage>,
    rx: Receiver<LogMessage>,
    queue_size: usize,
    current_queue_size: Arc<RwLock<usize>>, // Track current queue size
}

impl UdpLogFetcher {
    /// Create a new UDP log fetcher with default queue size
    pub fn new() -> Self {
        // Default queue size is set to 1,000,000 as requested
        Self::with_queue_size(1_000_000)
    }

    /// Create a new UDP log fetcher with specified queue size
    pub fn with_queue_size(queue_size: usize) -> Self {
        // Create a bounded channel with the specified capacity
        let (tx, rx) = bounded(queue_size);
        
        UdpLogFetcher {
            logs: Arc::new(RwLock::new(Vec::new())),
            socket: None,
            running: Arc::new(RwLock::new(false)),
            tx,
            rx,
            queue_size,
            current_queue_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Get a new consumer channel to receive log messages
    pub fn get_consumer(&self) -> Receiver<LogMessage> {
        self.rx.clone()
    }

    /// Get the current number of messages in the queue
    pub fn queue_len(&self) -> usize {
        *self.current_queue_size.read().unwrap()
    }

    /// Get the maximum queue size
    pub fn max_queue_size(&self) -> usize {
        self.queue_size
    }

    /// Start the UDP listener on the specified address (e.g., "127.0.0.1:8888")
    pub fn start(&mut self, bind_address: &str) -> Result<(), String> {
        // Try to bind to the address
        let socket = match UdpSocket::bind(bind_address) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to bind to {}: {}", bind_address, e)),
        };
        
        // Set socket to non-blocking mode with a timeout
        if let Err(e) = socket.set_read_timeout(Some(Duration::from_millis(100))) {
            return Err(format!("Failed to set socket timeout: {}", e));
        }
        
        self.socket = Some(socket);
        
        // Set the running flag to true
        let mut running = self.running.write().unwrap();
        *running = true;
        drop(running);
        
        // Clone Arc references for the thread
        let logs = Arc::clone(&self.logs);
        let running = Arc::clone(&self.running);
        let queue_size_tracker = Arc::clone(&self.current_queue_size);
        let socket = self.socket.as_ref().unwrap().try_clone().unwrap();
        let tx = self.tx.clone();
        let queue_capacity = self.queue_size;
        
        // Start the listener thread (producer)
        thread::spawn(move || {
            let mut buffer = [0; 4096]; // 4KB buffer for incoming packets
            let mut dropped_count = 0;
            let mut last_warning_time = std::time::Instant::now();
            
            while *running.read().unwrap() {
                match socket.recv_from(&mut buffer) {
                    Ok((size, src)) => {
                        // Convert received bytes to a string
                        if let Ok(message) = str::from_utf8(&buffer[..size]) {
                            // Create a new log message
                            let log_message = LogMessage {
                                source_ip: src.ip().to_string(),
                                source_port: src.port(),
                                message: message.trim().to_string(),
                                timestamp: std::time::SystemTime::now(),
                            };
                            
                            // Format the message for the legacy logs storage
                            let formatted_message = log_message.formatted();
                            
                            // Try to send the message to any consumers through the channel
                            // If the channel is full or has no receivers, this will not block
                            match tx.try_send(log_message) {
                                Ok(_) => {
                                    // Update queue size tracker
                                    let mut size = queue_size_tracker.write().unwrap();
                                    *size = (*size + 1).min(queue_capacity);
                                },
                                Err(e) => {
                                    // Count dropped messages due to full queue
                                    dropped_count += 1;
                                    
                                    // Log a warning about dropped messages every 5 seconds
                                    let now = std::time::Instant::now();
                                    if now.duration_since(last_warning_time).as_secs() >= 5 {
                                        eprintln!("Queue full, dropped {} messages. Consider increasing queue size or adding more consumers.", dropped_count);
                                        dropped_count = 0; // Reset counter
                                        last_warning_time = now;
                                    }
                                }
                            }
                            
                            // Also add the message to our legacy logs vector with source information
                            let mut logs_guard = logs.write().unwrap();
                            logs_guard.push(formatted_message);
                            
                            // Limit log size to avoid memory issues (legacy storage)
                            if logs_guard.len() > 1000 {
                                logs_guard.remove(0);
                            }
                        }
                    },
                    Err(e) => {
                        // Ignore would-block errors which happen on timeout
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            eprintln!("Error receiving UDP data: {}", e);
                        }
                    }
                }
                
                // Small sleep to prevent CPU hogging
                thread::sleep(Duration::from_millis(10));
            }
        });
        
        // Start a thread to track consumer activity and adjust queue size
        let running_clone = Arc::clone(&self.running);
        let rx_clone = self.rx.clone();
        let queue_size_tracker = Arc::clone(&self.current_queue_size);
        
        thread::spawn(move || {
            while *running_clone.read().unwrap() {
                // Update queue size counter when messages are consumed
                if let Ok(_) = rx_clone.try_recv() {
                    let mut size = queue_size_tracker.write().unwrap();
                    if *size > 0 {
                        *size -= 1;
                    }
                }
                
                // Sleep a bit to avoid spinning
                thread::sleep(Duration::from_millis(10));
            }
        });
        
        Ok(())
    }
    
    /// Stop the UDP listener
    pub fn stop(&self) {
        let mut running = self.running.write().unwrap();
        *running = false;
    }
    
    /// Get the collected logs (legacy method)
    pub fn get_logs(&self) -> Vec<String> {
        let logs = self.logs.read().unwrap();
        logs.clone()
    }
    
    /// Clear all logs from the legacy storage
    pub fn clear_logs(&self) {
        let mut logs = self.logs.write().unwrap();
        logs.clear();
    }
}

/// A UDP log fetcher that handles multiple ports with separate sockets,
/// ensuring that each port only receives messages specifically addressed to it.
pub struct MultiPortUdpLogFetcher {
    fetchers: Vec<UdpLogFetcher>,
    port_map: Vec<u16>,
    running: Arc<RwLock<bool>>,
}

impl MultiPortUdpLogFetcher {
    /// Create a new multi-port UDP log fetcher
    pub fn new() -> Self {
        MultiPortUdpLogFetcher {
            fetchers: Vec::new(),
            port_map: Vec::new(),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Add a port to listen on with a specific fetcher
    pub fn add_port(&mut self, port: u16, queue_size: usize) -> Result<(), String> {
        if self.port_map.contains(&port) {
            return Err(format!("Port {} already added", port));
        }
        
        let mut fetcher = UdpLogFetcher::with_queue_size(queue_size);
        self.port_map.push(port);
        self.fetchers.push(fetcher);
        
        Ok(())
    }
    
    /// Start listening on all added ports
    pub fn start_all(&mut self, ip: &str) -> Result<(), String> {
        if *self.running.read().unwrap() {
            return Err("Already running".to_string());
        }
        
        let mut failed_ports = Vec::new();
        
        // Start each fetcher with its dedicated port
        for (index, port) in self.port_map.iter().enumerate() {
            let bind_address = format!("{}:{}", ip, port);
            log::info!("Starting UDP listener on {}", bind_address);
            
            if let Err(e) = self.fetchers[index].start(&bind_address) {
                failed_ports.push((port, e));
            }
        }
        
        // If any ports failed to start, stop all and return an error
        if !failed_ports.is_empty() {
            // Stop any that did start
            for fetcher in &self.fetchers {
                fetcher.stop();
            }
            
            let error_msg = failed_ports
                .iter()
                .map(|(port, err)| format!("Port {}: {}", port, err))
                .collect::<Vec<_>>()
                .join(", ");
                
            return Err(format!("Failed to start on some ports: {}", error_msg));
        }
        
        // Set running flag to true
        let mut running = self.running.write().unwrap();
        *running = true;
        
        Ok(())
    }
    
    /// Stop all fetchers
    pub fn stop_all(&self) {
        let mut running = self.running.write().unwrap();
        *running = false;
        
        for fetcher in &self.fetchers {
            fetcher.stop();
        }
    }
    
    /// Get a consumer for a specific port
    pub fn get_consumer_for_port(&self, port: u16) -> Option<Receiver<LogMessage>> {
        if let Some(index) = self.port_map.iter().position(|&p| p == port) {
            Some(self.fetchers[index].get_consumer())
        } else {
            None
        }
    }
    
    /// Get consumers for all ports
    pub fn get_all_consumers(&self) -> Vec<(u16, Receiver<LogMessage>)> {
        self.port_map
            .iter()
            .enumerate()
            .map(|(index, &port)| (port, self.fetchers[index].get_consumer()))
            .collect()
    }
    
    /// Get queue length for a specific port
    pub fn queue_len_for_port(&self, port: u16) -> Option<usize> {
        if let Some(index) = self.port_map.iter().position(|&p| p == port) {
            Some(self.fetchers[index].queue_len())
        } else {
            None
        }
    }
    
    /// Get logs for a specific port (legacy method)
    pub fn get_logs_for_port(&self, port: u16) -> Option<Vec<String>> {
        if let Some(index) = self.port_map.iter().position(|&p| p == port) {
            Some(self.fetchers[index].get_logs())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_udp_log_fetcher() {
        // Create and start the fetcher
        let mut fetcher = UdpLogFetcher::new();
        fetcher.start("127.0.0.1:8899").unwrap();
        
        // Allow some time for the listener to start
        thread::sleep(Duration::from_millis(100));
        
        // Get a consumer and spawn a thread to listen for messages
        let consumer = fetcher.get_consumer();
        let received = Arc::new(RwLock::new(Vec::new()));
        let received_clone = received.clone();
        
        let consumer_thread = thread::spawn(move || {
            // Try to receive a message within a timeout
            if let Ok(message) = consumer.recv_timeout(Duration::from_millis(500)) {
                let mut received = received_clone.write().unwrap();
                received.push(message);
            }
        });
        
        // Send a test message
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let test_message = "Hello, UDP!";
        sender.send_to(test_message.as_bytes(), "127.0.0.1:8899").unwrap();
        
        // Wait for the consumer thread to finish
        consumer_thread.join().unwrap();
        
        // Check if we received the message in the queue
        let received = received.read().unwrap();
        assert!(!received.is_empty(), "No message received in the queue");
        if !received.is_empty() {
            assert_eq!(received[0].message, test_message);
        }
        
        // Also check the legacy method
        let logs = fetcher.get_logs();
        assert!(logs.iter().any(|log| log.contains(test_message)));
        
        // Stop the fetcher
        fetcher.stop();
    }
    
    #[test]
    fn test_multiple_consumers() {
        // Create and start the fetcher
        let mut fetcher = UdpLogFetcher::new();
        fetcher.start("127.0.0.1:8898").unwrap();
        
        // Get two consumers
        let consumer1 = fetcher.get_consumer();
        let consumer2 = fetcher.get_consumer();
        
        // Allow some time for the listener to start
        thread::sleep(Duration::from_millis(100));
        
        // Send a test message
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let test_message = "Multiple consumers test!";
        sender.send_to(test_message.as_bytes(), "127.0.0.1:8898").unwrap();
        
        // Allow some time for the message to be processed
        thread::sleep(Duration::from_millis(100));
        
        // Both consumers should receive the message (first come, first served)
        // At least one of them should receive it
        let result1 = consumer1.try_recv();
        let result2 = consumer2.try_recv();
        
        assert!(result1.is_ok() || result2.is_ok(), 
                "Neither consumer received the message");
        
        if let Ok(msg) = result1 {
            assert_eq!(msg.message, test_message);
        }
        
        if let Ok(msg) = result2 {
            assert_eq!(msg.message, test_message);
        }
        
        // Stop the fetcher
        fetcher.stop();
    }
    
    #[test]
    fn test_queue_size_tracking() {
        // Create a fetcher with a small queue size for testing
        let mut fetcher = UdpLogFetcher::with_queue_size(10);
        fetcher.start("127.0.0.1:8897").unwrap();
        
        // Check that initial queue size is 0
        assert_eq!(fetcher.queue_len(), 0);
        assert_eq!(fetcher.max_queue_size(), 10);
        
        // Send several test messages
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        for i in 0..5 {
            let test_message = format!("Test message {}", i);
            sender.send_to(test_message.as_bytes(), "127.0.0.1:8897").unwrap();
        }
        
        // Allow some time for messages to be processed
        thread::sleep(Duration::from_millis(100));
        
        // Queue size should increase (might not be exactly 5 due to timing)
        assert!(fetcher.queue_len() > 0, "Queue size should be greater than 0");
        
        // Consume some messages
        let consumer = fetcher.get_consumer();
        for _ in 0..3 {
            let _ = consumer.recv_timeout(Duration::from_millis(100));
        }
        
        // Allow some time for queue counter to update
        thread::sleep(Duration::from_millis(100));
        
        // Queue size should decrease
        assert!(fetcher.queue_len() < 5, "Queue size should decrease after consumption");
        
        fetcher.stop();
    }
}

#[cfg(test)]
mod multi_port_tests {
    use super::*;
    use std::net::UdpSocket;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_multi_port_receiver() {
        // Create a multi-port fetcher with 3 ports
        let mut multi_fetcher = MultiPortUdpLogFetcher::new();
        
        // Add three test ports
        let test_port1 = 8891;
        let test_port2 = 8892;
        let test_port3 = 8893;
        
        multi_fetcher.add_port(test_port1, 100).unwrap();
        multi_fetcher.add_port(test_port2, 100).unwrap();
        multi_fetcher.add_port(test_port3, 100).unwrap();
        
        // Start all fetchers
        multi_fetcher.start_all("127.0.0.1").unwrap();
        
        // Get consumers for each port
        let consumer1 = multi_fetcher.get_consumer_for_port(test_port1).unwrap();
        let consumer2 = multi_fetcher.get_consumer_for_port(test_port2).unwrap();
        let consumer3 = multi_fetcher.get_consumer_for_port(test_port3).unwrap();
        
        // Allow time for listeners to start
        thread::sleep(Duration::from_millis(100));
        
        // Create a sender socket
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        
        // Send messages to each port
        let msg1 = "Message for port 1";
        let msg2 = "Message for port 2";
        let msg3 = "Message for port 3";
        
        sender.send_to(msg1.as_bytes(), format!("127.0.0.1:{}", test_port1)).unwrap();
        sender.send_to(msg2.as_bytes(), format!("127.0.0.1:{}", test_port2)).unwrap();
        sender.send_to(msg3.as_bytes(), format!("127.0.0.1:{}", test_port3)).unwrap();
        
        // Allow time for messages to be processed
        thread::sleep(Duration::from_millis(200));
        
        // Check that each consumer received only its own message
        let received1 = consumer1.try_recv();
        let received2 = consumer2.try_recv();
        let received3 = consumer3.try_recv();
        
        assert!(received1.is_ok(), "Consumer 1 should receive a message");
        assert!(received2.is_ok(), "Consumer 2 should receive a message");
        assert!(received3.is_ok(), "Consumer 3 should receive a message");
        
        if let Ok(msg) = received1 {
            assert_eq!(msg.message, msg1);
        }
        
        if let Ok(msg) = received2 {
            assert_eq!(msg.message, msg2);
        }
        
        if let Ok(msg) = received3 {
            assert_eq!(msg.message, msg3);
        }
        
        // Stop all fetchers
        multi_fetcher.stop_all();
    }
    
    #[test]
    fn test_port_isolation() {
        // Create a multi-port fetcher with 2 ports
        let mut multi_fetcher = MultiPortUdpLogFetcher::new();
        
        // Add two test ports
        let test_port1 = 8894;
        let test_port2 = 8895;
        
        multi_fetcher.add_port(test_port1, 100).unwrap();
        multi_fetcher.add_port(test_port2, 100).unwrap();
        
        // Start all fetchers
        multi_fetcher.start_all("127.0.0.1").unwrap();
        
        // Get consumer for port 1 only
        let consumer1 = multi_fetcher.get_consumer_for_port(test_port1).unwrap();
        
        // Allow time for listeners to start
        thread::sleep(Duration::from_millis(100));
        
        // Create a sender socket
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        
        // Send message to port 2 only
        let msg2 = "Message for port 2 only";
        sender.send_to(msg2.as_bytes(), format!("127.0.0.1:{}", test_port2)).unwrap();
        
        // Allow time for message to be processed
        thread::sleep(Duration::from_millis(200));
        
        // Consumer 1 should NOT receive port 2's message
        let received1 = consumer1.try_recv();
        assert!(received1.is_err(), "Consumer 1 should NOT receive port 2's message");
        
        // Now send a message to port 1
        let msg1 = "Message for port 1";
        sender.send_to(msg1.as_bytes(), format!("127.0.0.1:{}", test_port1)).unwrap();
        
        // Allow time for message to be processed
        thread::sleep(Duration::from_millis(200));
        
        // Now consumer 1 should receive a message
        let received1 = consumer1.try_recv();
        assert!(received1.is_ok(), "Consumer 1 should receive port 1's message");
        
        if let Ok(msg) = received1 {
            assert_eq!(msg.message, msg1);
        }
        
        // Stop all fetchers
        multi_fetcher.stop_all();
    }
}