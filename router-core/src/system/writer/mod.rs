mod logger;
mod mapper;

use mapper::{setup_standard_logging, setup_tag_based_logging};

pub fn writer_start() {
    // Try the tag-based logging first
    if setup_tag_based_logging().is_ok() {
        log::info!("Tag-based logging initialized successfully");
        return;
    }
    
    eprintln!("Failed to initialize tag-based logging");
    
    // Fall back to standard file-based logging
    if setup_standard_logging().is_ok() {
        log::info!("Standard file-based logging initialized successfully");
        return;
    }
    
    eprintln!("Failed to initialize file-based logging");
    
    // Last resort: standard env_logger to stderr
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    log::warn!("Using default logging configuration");
}
