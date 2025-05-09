//! # Router Core
//!
//! The router core is the central component of the mini-gateway system. It handles network
//! traffic routing, proxying, and gateway functionality for the entire system.
//!
//! ## Architecture
//!
//! The router core is built with the following components:
//! - **System Layer**: Provides server components, protocol handling, and termination controls
//! - **Service Layer**: Manages service discovery, registration, and inter-service communication
//! - **App Layer**: Implements application-specific logic for gateways and proxies
//! - **Config**: Handles configuration management and dynamic updates
//!
//! ## Communication
//!
//! The router core communicates with other components using a message-based architecture,
//! allowing for configuration updates and state synchronization without service interruption.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use system::memory_log;
use tokio::{self};

mod app;
mod config;
mod service;
mod system;

/// Main entry point for the router core application.
///
/// This function initializes the core components of the routing system:
/// 1. Sets up logging configuration
/// 2. Initializes the service registry for inter-service communication
/// 3. Starts the custom protocol server for control messages
/// 4. Sets up signal handlers for graceful shutdown
/// 5. Starts the main server in a separate thread
/// 6. Enters a control loop for monitoring and management
///
/// The application can be terminated by:
/// - SIGINT (Ctrl+C) signal
/// - Ctrl+X keyboard shortcut via the terminator CLI
///
/// # Lifecycle
///
/// The router runs continuously until terminated, monitoring for configuration
/// changes and adjusting routing behavior dynamically.
#[tokio::main]
async fn main() {
    // Configure file-based logging
    config::init();
    // std::env::set_var("RUST_LOG", "info");
    // env_logger::init();
    eprintln!("[----] Starting proxy server...");

    // Create atomic flag to track server active state
    let active_state = Arc::new(AtomicBool::new(false));

    eprintln!("[----] Starting protocol server...");
    // Initialize custom protocol server for control and management interface
    {
        system::protocol::start_interface();
    }
    
    eprintln!("[----] Starting service registry...");
    // initialize global sender
    {
        system::writer::writer_start();
    }

    eprintln!("[----] Starting CTRL+C Listener...");
    // Set up interrupt handler for graceful shutdown on SIGINT (Ctrl+C)
    {
        let running_clone = Arc::clone(&active_state);
        ctrlc::set_handler(move || {
            log::debug!("SIGINT received, shutting down servers...");
            eprintln!("\n[----] SIGINT received, shutting down servers...");
            eprintln!("[----] Cleaning up memory log...");
            let _ = memory_log::log_cleanup();
            eprintln!("[----] Cleaning up memory log done.");
            eprintln!("[----] Finish...\n\n");
            running_clone.store(false, Ordering::SeqCst);
            eprintln!("[----] Restarting the Proxy and Gateway...");
        })
        .expect("Error setting Ctrl-C handler");
    }

    eprintln!("[----] Starting Main Loop...");

    // Main application loop - continues until termination signal
    loop {

        // Check for Ctrl+X termination signal via CLI interface
        if system::terminator::cli::init(Duration::from_millis(0)) {
            let _ = memory_log::log_cleanup();
            eprintln!("[----] Ctrl+X received, exiting...");
            break;
        }

        // Start server if not already active
        if !active_state.load(std::sync::atomic::Ordering::Relaxed) {
            // Set active state flag
            active_state.store(true, std::sync::atomic::Ordering::Relaxed);

            // Launch server in separate thread to avoid blocking the control loop
            std::thread::spawn(|| {
                system::server::init();
            });

            continue;
        }

        // Small sleep to prevent CPU spinning in the control loop
        sleep(Duration::from_millis(50));
    }
}
