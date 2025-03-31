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

use std::process::{id as pid, Command};

/// Initializes the termination process for the router.
///
/// # Functionality
///
/// This function triggers a self-termination of the current process by:
/// 1. Obtaining the current process ID
/// 2. Sending a SIGINT signal to itself using the system `kill` command
///
/// # Signal Details
///
/// SIGINT (Signal Interrupt) is used rather than SIGTERM or SIGKILL to allow
/// the process to perform necessary cleanup operations before terminating.
///
/// # Panics
///
/// This function will panic if:
/// - The `kill` command execution fails for any reason
/// - The system cannot convert the process ID to a string
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
/// This implementation uses the UNIX/Linux `kill` command and may not work
/// on all platforms, particularly Windows systems.
pub fn init(){
    let pid = pid();
    log::debug!(
        "Sample termination: sending SIGINT to process id: {}",
        pid
    );
    // Use the `kill` command to send SIGINT to self.
    let status = Command::new("kill")
        .arg("-SIGINT")
        .arg(pid.to_string())
        .status()
        .expect("Failed to execute kill command");
    log::debug!("Kill command exited with status: {}", status);
}