/// Provides functions for setting up different logging configurations.
// filepath: /Users/zonblade/Project/runegram/mini-gateway-rs/router-core/src/system/writer/mapper.rs
use log::LevelFilter;

use crate::system::writer::logger::TagBasedLogger;

/// Configures and initializes the `TagBasedLogger`.
///
/// This function sets up a logging system where log messages are routed based on tags
/// embedded within the message content. It determines the log level from the `RUST_LOG`
/// environment variable (defaulting to `Info` if not set or invalid) and configures
/// the `TagBasedLogger` with predefined tags: `[PXY]`, `[GWX]`, and `[NET]`.
///
/// After initialization, it logs several test messages to verify the setup.
///
/// # Returns
///
/// Returns `Ok(())` if the logger was successfully initialized and set as the global logger.
/// Returns an `Err` containing the underlying error if setting the logger fails.
pub fn setup_tag_based_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Set global log level - enable more verbose logging for debugging
    // Consider making this configurable or removing if not needed for production.
    std::env::set_var("RUST_LOG", "info");

    // Determine log level from environment or use default
    let log_level = std::env::var("RUST_LOG")
        .map(|level| match level.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn"  => LevelFilter::Warn,
            "info"  => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info, // Default to Info for unrecognized values
        })
        .unwrap_or(LevelFilter::Info); // Default to Info if RUST_LOG is not set
    
    // Define the tags that the logger will recognize and route.
    let tag_writers = vec![
        "[PXY]", // Tag for proxy-related messages
        "[GWX]", // Tag for gateway-specific messages
        "[NET]", // Tag for network-related messages
    ];

    // Create the TagBasedLogger instance.
    let logger = Box::new(TagBasedLogger {
        tag_writers,
        level_filter: log_level,
    });

    // Set the created logger as the global logger for the `log` facade.
    // Also sets the maximum log level to filter messages early.
    log::set_boxed_logger(logger).map(|()| log::set_max_level(log_level))?;

    // Log informational messages to confirm initialization and test tags.
    log::info!("Tag-based logging system initialized");
    log::info!("[PXY] This is an proxy-related log message");
    log::info!("[GWX] This is a gateway-related log message");
    log::info!("[NET] This is a network-related log message");
    // This message might not be routed if no tag matches, depending on udp_sender logic.
    log::info!("This is a regular log message with no specific tag");

    Ok(())
}

/// Configures and initializes the standard `env_logger`.
///
/// This function serves as a fallback logging mechanism. It initializes the `env_logger`,
/// which reads the `RUST_LOG` environment variable to configure logging levels and filters.
/// Logs will typically be written to standard error.
///
/// # Returns
///
/// Returns `Ok(())` after initializing `env_logger`.
/// Note: `env_logger::init()` might panic if it fails internally, though this is rare.
pub fn setup_standard_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Set RUST_LOG for env_logger if not already set, defaulting to info.
    // This ensures some logging output even if RUST_LOG wasn't previously defined.
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    Ok(())
}
