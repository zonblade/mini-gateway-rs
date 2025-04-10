mod logger;
mod mapper;

use mapper::{setup_standard_logging, setup_tag_based_logging};

pub fn writer_start() {
    // Try the tag-based logging first
    if let Err(e) = setup_tag_based_logging() {
        eprintln!("Failed to initialize tag-based logging: {}", e);

        // Fall back to standard file-based logging
        if let Err(e) = setup_standard_logging() {
            eprintln!("Failed to initialize file-based logging: {}", e);

            // Last resort: standard env_logger to stderr
            std::env::set_var("RUST_LOG", "info");
            env_logger::init();
            log::warn!("Using default logging configuration due to error: {}", e);
        } else {
            // Successfully set up standard file-based logging
            log::info!("Standard file-based logging initialized successfully");
        }
    } else {
        // Successfully set up tag-based logging
        log::info!("Tag-based logging initialized successfully");
    }
}
