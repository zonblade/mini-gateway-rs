//! Signal handling utilities for the router core
//! 
//! This module provides functions for manually triggering signal-like
//! behavior in the application, useful for controlled shutdowns and
//! testing.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Global flag that can be used to trigger shutdown from any part of the code
static mut SHUTDOWN_TRIGGER: Option<Arc<AtomicBool>> = None;

/// Initialize signal handling with the main application's active state
/// 
/// # Safety
/// Not thread-safe. Should be called only during application initialization.
pub unsafe fn init(active_state: Arc<AtomicBool>) {
    SHUTDOWN_TRIGGER = Some(Arc::clone(&active_state));
}

/// Manually trigger a SIGINT-like shutdown sequence
/// 
/// This will log shutdown messages and set the active state to false,
/// simulating what happens when a SIGINT signal is received.
/// 
/// # Returns
/// `true` if the shutdown was triggered successfully, `false` otherwise
pub fn trigger_sigint() -> bool {
    unsafe {
        if let Some(trigger) = &SHUTDOWN_TRIGGER {
            log::debug!("SIGINT received, shutting down servers...");
            eprintln!("SIGINT received, shutting down servers...");
            trigger.store(false, Ordering::SeqCst);
            true
        } else {
            eprintln!("Cannot trigger SIGINT: signal handler not initialized");
            false
        }
    }
}
