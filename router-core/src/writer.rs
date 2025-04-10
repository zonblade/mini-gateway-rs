use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::RwLock;
use log::{LevelFilter, Record, Metadata, SetLoggerError, Level};
use regex::Regex;

// Define a custom logger that will direct logs to different files based on message content
struct TagBasedLogger {
    // Default writer for logs that don't match specific tags
    default_writer: RwLock<BufWriter<std::fs::File>>,
    // Map of tag-based writers (tag -> writer)
    tag_writers: Vec<(Regex, RwLock<BufWriter<std::fs::File>>)>,
    // Level filter
    level_filter: LevelFilter,
}

impl log::Log for TagBasedLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = record.args().to_string();
            
            // Check if message matches any of our tag patterns
            let mut matched = false;
            
            for (pattern, writer) in &self.tag_writers {
                if pattern.is_match(&message) {
                    if let Ok(mut writer) = writer.write() {
                        writeln!(
                            writer,
                            "[{}] {} [{}] {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            record.level(),
                            record.module_path().unwrap_or("unknown"),
                            record.args()
                        ).ok();
                        
                        // Flush high-priority logs immediately
                        if record.level() <= Level::Warn {
                            writer.flush().ok();
                        }
                    }
                    matched = true;
                    // We want logs to potentially go to multiple files if they match multiple patterns
                }
            }
            
            // If no specific tag matched or we want to log to the default file anyway
            if !matched {
                if let Ok(mut writer) = self.default_writer.write() {
                    writeln!(
                        writer,
                        "[{}] {} [{}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        record.module_path().unwrap_or("unknown"),
                        record.args()
                    ).ok();
                    
                    // Flush high-priority logs immediately
                    if record.level() <= Level::Warn {
                        writer.flush().ok();
                    }
                }
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut writer) = self.default_writer.write() {
            writer.flush().ok();
        }
        
        for (_, writer) in &self.tag_writers {
            if let Ok(mut writer) = writer.write() {
                writer.flush().ok();
            }
        }
    }
}

// Get the default log directory based on the OS
#[cfg(target_os = "macos")]
fn get_default_log_dir() -> String {
    dirs::home_dir()
        .map(|p| p.join("Library/Logs/gwrs").to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("/tmp/gwrs"))
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
fn setup_tag_based_logging() -> Result<(), Box<dyn std::error::Error>> {
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
    let default_log_path = format!("{}/core.log", log_dir);
    let app_log_path = format!("{}/app.log", log_dir);
    let listen_log_path = format!("{}/listen.log", log_dir);
    
    println!("Default logs: {}", default_log_path);
    println!("APP logs: {}", app_log_path);
    println!("LSTN logs: {}", listen_log_path);
    
    // Open the log files for appending, create if they don't exist
    let default_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&default_log_path)?;
    
    let app_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&app_log_path)?;
    
    let listen_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&listen_log_path)?;
    
    // Use buffered writers to improve performance
    let buffer_size = 64 * 1024; // 64KB buffer
    let default_writer = BufWriter::with_capacity(buffer_size, default_file);
    let app_writer = BufWriter::with_capacity(buffer_size, app_file);
    let listen_writer = BufWriter::with_capacity(buffer_size, listen_file);
    
    // Determine log level from environment or use default
    let log_level = std::env::var("RUST_LOG")
        .map(|level| match level.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        })
        .unwrap_or(LevelFilter::Info);
    
    // Create tag patterns (use case-insensitive regex for flexibility)
    let app_pattern = Regex::new(r"\[APP\]").unwrap();
    let listen_pattern = Regex::new(r"\[LSTN\]").unwrap();
    
    // Create and initialize the tag-based logger
    let tag_writers = vec![
        (app_pattern, RwLock::new(app_writer)),
        (listen_pattern, RwLock::new(listen_writer)),
    ];
    
    let logger = Box::new(TagBasedLogger {
        default_writer: RwLock::new(default_writer),
        tag_writers,
        level_filter: log_level,
    });
    
    // Set the logger
    log::set_boxed_logger(logger)
        .map(|()| log::set_max_level(log_level))?;
    
    // Write test log entries with various tags to verify the files are working
    log::info!("Tag-based logging system initialized");
    log::info!("[APP] This is an app-related log message");
    log::info!("[LSTN] This is a listener-related log message");
    log::info!("This is a regular log message with no specific tag");
    log::warn!("[APP] This is a warning in the app component");
    
    Ok(())
}

/// Configure standard env_logger-based logging (fallback)
fn setup_standard_logging() -> Result<(), Box<dyn std::error::Error>> {
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