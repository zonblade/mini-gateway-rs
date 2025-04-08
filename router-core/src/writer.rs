use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

#[cfg(target_os = "macos")]
fn get_default_log_dir() -> String {
    dirs::home_dir()
        .map(|p| p.join("Library/Logs/gwrs").to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("/tmp/gwrs"))
}

#[cfg(target_os = "linux")]
fn get_default_log_dir() -> String {
    String::from("/var/log/gwrs")
}

#[cfg(target_os = "windows")]
fn get_default_log_dir() -> String {
    String::from("C:\\ProgramData\\gwrs")
}

/// Configure file-based logging to the specified directory
fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
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

pub fn writer_start() {
    if let Err(e) = setup_logging() {
        eprintln!("Failed to initialize file-based logging: {}", e);
        // Fall back to standard env_logger to stderr
        std::env::set_var("RUST_LOG", "info");
        env_logger::init();
        log::warn!("Using default logging configuration due to error: {}", e);
    } else {
        // Successfully set up file-based logging
        log::info!("File-based logging initialized successfully");
    }
}