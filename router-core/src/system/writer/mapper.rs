use log::LevelFilter;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::RwLock;

use crate::system::writer::logger::TagBasedLogger;

// Get the default log directory based on the OS
#[cfg(target_os = "macos")]
fn get_default_log_dir() -> String {
    String::from("/tmp/gwrs/log")
}

#[cfg(target_os = "linux")]
fn get_default_log_dir() -> String {
    String::from("/tmp/gwrs/log")
}

#[cfg(target_os = "windows")]
fn get_default_log_dir() -> String {
    String::from("C:\\ProgramData\\gwrs")
}

/// Configure tag-based logging where logs are routed to different files based on message content
pub fn setup_tag_based_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Set global log level - enable more verbose logging for debugging
    std::env::set_var("RUST_LOG", "info");

    // Determine the log file path
    let log_dir = get_default_log_dir();

    // Create the log directory if it doesn't exist
    let log_dir_path = Path::new(&log_dir);
    if !log_dir_path.exists() {
        fs::create_dir_all(log_dir_path)?;
    }

    // Define log file paths
    let log_path_default    = format!("{}/core.log"         , log_dir);
    let log_path_proxy      = format!("{}/core.proxy.log"   , log_dir);
    let log_path_gateway    = format!("{}/core.gateway.log" , log_dir);
    let log_path_netlisten  = format!("{}/core.net.log"     , log_dir);

    // Open the log files for appending, create if they don't exist
    let file_default = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path_default)?;

    let file_proxy = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path_proxy)?;

    let file_gateway = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path_gateway)?;

    let file_netlisten = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path_netlisten)?;

    // Use buffered writers to improve performance
    let buffer_size = 64 * 1024; // 64KB buffer
    let writer_default      = BufWriter::with_capacity(buffer_size, file_default);
    let writer_proxy        = BufWriter::with_capacity(buffer_size, file_proxy);
    let writer_gateway      = BufWriter::with_capacity(buffer_size, file_gateway);
    let writer_netlisten    = BufWriter::with_capacity(buffer_size, file_netlisten);

    // Determine log level from environment or use default
    let log_level = std::env::var("RUST_LOG")
        .map(|level| match level.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn"  => LevelFilter::Warn,
            "info"  => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        })
        .unwrap_or(LevelFilter::Info);
    
    // Create and initialize the tag-based logger
    let tag_writers = vec![
        ("[PXY]"  , RwLock::new(writer_proxy), log_path_proxy),
        ("[GWX]"  , RwLock::new(writer_gateway), log_path_gateway),
        ("[NET]"  , RwLock::new(writer_netlisten), log_path_netlisten),
    ];

    let logger = Box::new(TagBasedLogger {
        default_writer: RwLock::new(writer_default),
        default_path: log_path_default,
        tag_writers,
        level_filter: log_level,
    });

    // Set the logger
    log::set_boxed_logger(logger).map(|()| log::set_max_level(log_level))?;

    // Write test log entries with various tags to verify the files are working
    log::info!("Tag-based logging system initialized");
    log::info!("[PXY] This is an proxy-related log message");
    log::info!("[GWX] This is a gateway-related log message");
    log::info!("[NET] This is a network-related log message");
    log::info!("This is a regular log message with no specific tag");

    Ok(())
}

/// Configure standard env_logger-based logging (fallback)
pub fn setup_standard_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Set global log level - enable more verbose logging for debugging
    std::env::set_var("RUST_LOG", "info");

    // Determine the log file path
    let log_dir = get_default_log_dir();

    // Create the log directory if it doesn't exist
    let log_dir_path = Path::new(&log_dir);
    if !log_dir_path.exists() {
        fs::create_dir_all(log_dir_path)?;
    }

    // Full path to the log file
    let log_file_path = format!("{}/core.log", log_dir);
    println!("Logging to: {}", log_file_path);

    // Open the log file for appending, create if it doesn't exist
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_file_path)?;

    // Use a buffered writer to improve performance
    let buffer_size = 64 * 1024; // Increased to 64KB buffer for better performance
    let buffered_writer = BufWriter::with_capacity(buffer_size, file);

    // Build a custom logger with a simpler format for better performance
    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(Box::new(buffered_writer)))
        .format(|buf, record| {
            // Minimalist log format for better performance
            writeln!(
                buf,
                "[{}] {} [{}] {}",
                buf.timestamp(),
                record.level(),
                record.module_path().unwrap_or("unknown"),
                record.args()
            )
        })
        .parse_filters(&std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".to_string()))
        .init();

    // Write test log entries at different levels to verify the file is working
    log::info!("Logging system initialized");
    log::debug!("Log file path: {}", log_file_path);

    Ok(())
}
