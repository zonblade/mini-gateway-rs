//! Manages the initialization of the logging system.
//!
//! This module provides the entry point (`writer_start`) for setting up logging.
//! It attempts to initialize a tag-based UDP logger first and falls back to
//! standard logging mechanisms (`env_logger`) if the primary setup fails.

mod logger;
mod mapper;
pub mod rawid;

use mapper::{setup_standard_logging, setup_tag_based_logging};

/// Initializes the logging system for the application.
///
/// This function orchestrates the setup of the logging infrastructure.
/// It follows a priority order:
/// 1. Attempts to set up `TagBasedLogger` using `setup_tag_based_logging`.
///    If successful, logs an info message and returns.
/// 2. If tag-based logging fails, it prints an error and attempts to set up
///    standard file-based logging (assuming `setup_standard_logging` configures file output,
///    though its current implementation uses `env_logger` which typically logs to stderr).
///    If successful, logs an info message and returns.
/// 3. If both primary and fallback logging setups fail, it prints another error
///    and initializes the standard `env_logger` directly as a last resort,
///    logging a warning about using the default configuration.
pub fn writer_start() {
    // // Try the tag-based logging first
    eprintln!("[----] Initializing tag-based logging...");
    if setup_tag_based_logging().is_ok() {
        eprintln!("[----] Tag-based logging initialized successfully");
        return;
    }
    
    // Log failure to standard error as the logging system isn't fully up yet.
    eprintln!("[----] Failed to initialize tag-based logging");
    
    // Fall back to standard logging (currently env_logger)
    if setup_standard_logging().is_ok() {
        // This log might go to stderr depending on env_logger config.
        log::info!("Standard env_logger-based logging initialized successfully");
        return;
    }
    
    // Log failure to standard error.
    eprintln!("[----] Failed to initialize standard logging");
    
    // Last resort: standard env_logger to stderr
    // Ensure RUST_LOG is set for env_logger.
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    // This warning will go to stderr.
    log::warn!("Using default env_logger configuration as final fallback");
}
