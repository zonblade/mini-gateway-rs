//! # Terminator Service
//!
//! This module provides functionality for graceful termination of the router process.
//! 
//! The terminator service allows the router to self-terminate by sending appropriate
//! signals to itself, enabling clean shutdown procedures to be executed.
//!
//! ## Signal Handling
//!
//! SIGINT (Signal Interrupt) is typically sent when a user presses Ctrl+C in the terminal.
//! When received, this signal allows the application to perform cleanup operations
//! before shutting down.

use std::process::{id as pid, Command, exit};

/// Initializes the termination process for the router.
///
/// # Functionality
///
/// This function triggers a self-termination of the current process by:
/// 1. Obtaining the current process ID
/// 2. Sending a SIGINT signal to itself using the system `kill` command
/// 3. If the kill command isn't available, calls std::process::exit as a fallback
///
/// # Signal Details
///
/// SIGINT (Signal Interrupt) is used rather than SIGTERM or SIGKILL to allow
/// the process to perform necessary cleanup operations before terminating.
///
/// # Examples
///
/// ```
/// use router_core::system::terminator::service;
///
/// // Trigger graceful shutdown of the current process
/// service::init();
/// ```
///
/// # Platform Compatibility
///
/// This implementation primarily uses the UNIX/Linux `kill` command, but has a
/// fallback mechanism for platforms where it's not available (e.g., Windows).
pub fn init(){
    let pid = pid();
    log::debug!(
        "Sample termination: sending SIGINT to process id: {}",
        pid
    );
    
    // Try to use the `kill` command to send SIGINT to self
    match Command::new("kill")
        .arg("-SIGINT")
        .arg(pid.to_string())
        .status() {
            Ok(status) => {
                log::debug!("Kill command exited with status: {}", status);
            },
            Err(e) => {
                // Log the error but don't panic
                log::warn!("Failed to execute kill command: {}", e);
                log::debug!("Using process::exit as fallback termination method");
                
                // Use exit as a fallback
                exit(0);
            }
    }
}