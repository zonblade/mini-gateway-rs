use crossbeam_channel::Receiver;
use std::thread;
use std::time::Duration;
use crate::module::udp_log_fetcher::LogMessage;
use crate::module::udp_log_db::UdpLogDb;
use log;

/// A processor that receives UDP log messages and saves them to the database
pub struct UdpLogProcessor {
    receiver: Receiver<LogMessage>,
    db_pool: UdpLogDb,
    running: bool,
}

impl UdpLogProcessor {
    /// Create a new UDP log processor
    pub fn new(receiver: Receiver<LogMessage>, db_pool: UdpLogDb) -> Self {
        UdpLogProcessor {
            receiver,
            db_pool,
            running: false,
        }
    }

    /// Start processing messages in the current thread (blocking)
    pub fn start_processing(&mut self) {
        self.running = true;
        log::info!("Starting UDP log processor");

        while self.running {
            // Try to receive a message with a timeout
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(log_message) => {
                    // Process the message
                    if let Some(formatted) = self.db_pool.process_message(&log_message) {
                        // Check for connection events (11 for new connection, 00 for disconnection)
                        if formatted.id.ends_with("11") || formatted.id.ends_with("00") {
                            log::info!(
                                "Connection event: {} - {}",
                                formatted.id,
                                if formatted.id.ends_with("11") { "connected" } else { "disconnected" }
                            );
                        }

                        // Add the log to the pool
                        self.db_pool.add_log(&log_message, &formatted);
                    }
                },
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // No message received, just continue
                },
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    // Channel disconnected, stop processing
                    log::warn!("UDP log channel disconnected, stopping processor");
                    self.running = false;
                },
            }
        }
    }

    /// Start processing messages in a separate thread (non-blocking)
    pub fn start_processing_thread(mut self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            self.start_processing();
        })
    }

    /// Stop the processor
    pub fn stop(&mut self) {
        self.running = false;
    }
}

/// Initialize the database pooling system and start a processor
pub fn start_log_processing(receiver: Receiver<LogMessage>) -> thread::JoinHandle<()> {
    // Initialize the database pooling system
    let db_pool = crate::module::udp_log_db::init();
    
    // Create a processor
    let processor = UdpLogProcessor::new(receiver, db_pool);
    
    // Start processing in a separate thread
    processor.start_processing_thread()
}