use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use std::env;

#[cfg(target_os = "macos")]
fn get_default_log_dir() -> String {
    dirs::home_dir()
        .map(|p| p.join("Library/Logs/gwrs/core.proxy.log").to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("/tmp/gwrs/core.proxy.log"))
}

#[cfg(target_os = "linux")]
fn get_default_log_dir() -> String {
    String::from("/tmp/gwrs/log/core.proxy.log")
}

#[cfg(target_os = "windows")]
fn get_default_log_dir() -> String {
    String::from("C:\\ProgramData\\gwrs\\core.proxy.log")
}

const RETRY_INTERVAL: Duration = Duration::from_secs(1);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub struct LogWatcher {
    path: PathBuf,
}

impl LogWatcher {
    pub fn new() -> Self {
        // Resolve the log path
        let log_path = env::var("GWRS_LOG_PATH").unwrap_or_else(|_| get_default_log_dir());
        
        // Expand tilde if present (for macOS home directory)
        let expanded_path = if log_path.starts_with("~/") {
            if let Ok(home) = env::var("HOME") {
                log_path.replace("~", &home)
            } else {
                log_path
            }
        } else {
            log_path
        };
        
        Self {
            path: PathBuf::from(expanded_path),
        }
    }

    // Create log directory if it doesn't exist
    fn ensure_log_directory(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                println!("Creating log directory: {}", parent.display());
                std::fs::create_dir_all(parent)?;
            }
        }
        Ok(())
    }

    fn watch_file(&self) -> io::Result<()> {
        // Ensure log directory exists
        self.ensure_log_directory()?;
        
        println!("Starting to watch log file: {}", self.path.display());
        
        // Store the initial inode/file ID to detect when the file is replaced
        let initial_metadata = fs::metadata(&self.path).ok();
        let mut current_metadata = initial_metadata.clone();
        let mut reader_needs_reset = true;
        let mut reader = None;
        let mut pos: u64 = 0;
        
        // Track when we last checked for file metadata changes
        let mut last_metadata_check = Instant::now();
        let metadata_check_interval = Duration::from_millis(100); // Check metadata every 100ms
        
        loop {
            // Check if file still exists
            if !self.path.exists() {
                println!("Log file no longer exists at {}", self.path.display());
                return Ok(());
            }
            
            // Check if the file has been replaced (different inode/ID)
            // Only check metadata periodically to reduce filesystem calls
            if last_metadata_check.elapsed() >= metadata_check_interval {
                let new_metadata = fs::metadata(&self.path).ok();
                
                // Compare modification times to detect file replacement
                let file_changed = match (current_metadata.as_ref(), new_metadata.as_ref()) {
                    (Some(old_meta), Some(new_meta)) => {
                        if let (Ok(old_time), Ok(new_time)) = (old_meta.modified(), new_meta.modified()) {
                            old_time != new_time
                        } else {
                            // If we can't get modification times, assume file changed
                            true
                        }
                    },
                    // If either metadata is missing, assume file changed
                    _ => true
                };
                
                if file_changed {
                    println!("Log file was replaced or modified, switching to new file");
                    current_metadata = new_metadata;
                    reader_needs_reset = true;
                    pos = 0;
                }
                
                last_metadata_check = Instant::now();
            }
            
            // Reset reader if needed (first time or after file replacement)
            if reader_needs_reset {
                // Try to open the file, create it if it doesn't exist
                let file = match OpenOptions::new()
                    .read(true)
                    .write(true) // Add write permission to ensure proper file creation
                    .create(true)
                    .open(&self.path) {
                        Ok(f) => f,
                        Err(e) => {
                            println!("Failed to open log file: {}", e);
                            thread::sleep(RETRY_INTERVAL);
                            continue;
                        }
                    };
                    
                reader = Some(BufReader::new(file));
                reader_needs_reset = false;
                pos = 0;
            }
            
            // Get a reference to our reader
            let reader_ref = match reader.as_mut() {
                Some(r) => r,
                None => {
                    println!("No reader available");
                    thread::sleep(RETRY_INTERVAL);
                    continue;
                }
            };
            
            // Only seek if we have a valid position
            if pos > 0 {
                match reader_ref.seek(SeekFrom::Start(pos)) {
                    Ok(new_pos) => {
                        if new_pos < pos {
                            // File was truncated, reset position
                            println!("Log file was truncated, resetting position");
                            pos = 0;
                            reader_ref.seek(SeekFrom::Start(0))?;
                        }
                    },
                    Err(e) => {
                        println!("Seek error: {}, resetting reader", e);
                        reader_needs_reset = true;
                        thread::sleep(POLL_INTERVAL);
                        continue;
                    }
                }
            }
            
            // Try to read new content
            let mut buffer = String::new();
            
            let bytes_read = match reader_ref.read_line(&mut buffer) {
                Ok(n) => n,
                Err(e) => {
                    println!("Error reading line: {}, resetting reader", e);
                    reader_needs_reset = true;
                    thread::sleep(POLL_INTERVAL);
                    continue;
                }
            };
            
            if bytes_read > 0 {
                // Update position for next read
                pos += bytes_read as u64;
                
                // Remove the trailing newline
                if buffer.ends_with('\n') {
                    buffer.pop();
                    if buffer.ends_with('\r') {
                        buffer.pop();
                    }
                }
                
                // Only print the log line if it contains the "|ID:" pattern
                // traffic log
                if buffer.contains("|ID:") {
                    println!("{}", buffer);
                }
            } else {
                // No new data, sleep for a short period
                thread::sleep(POLL_INTERVAL);
            }
        }
    }
}

/// Start watching the log file in a separate thread
///
/// # Returns
/// 
/// A JoinHandle to the spawned thread, which can be used to wait for the
/// thread to complete or to detach it.
/// 
/// # Example
/// 
/// ```
/// let _handle = start_log_watcher();
/// 
/// // Continue with other operations, the log watcher runs in the background
/// // The handle can be ignored if you don't need to join the thread later
/// ```
pub fn start_log_watcher() -> thread::JoinHandle<()> {
    let watcher = LogWatcher::new();
    
    // Spawn a dedicated thread for the log watcher
    thread::spawn(move || {
        let mut last_check = Instant::now();
        
        loop {
            if !watcher.path.exists() {
                if let Err(e) = watcher.ensure_log_directory() {
                    println!("Failed to create log directory: {}", e);
                }
                
                if last_check.elapsed() >= RETRY_INTERVAL {
                    println!("Log file not found at {}, retrying in {} seconds", 
                        watcher.path.display(), RETRY_INTERVAL.as_secs());
                    last_check = Instant::now();
                }
                thread::sleep(RETRY_INTERVAL);
                continue;
            }

            match watcher.watch_file() {
                Ok(_) => {
                    // If we get here, the file was removed or had an error
                    println!("Stopped watching file: {}", watcher.path.display());
                }
                Err(e) => {
                    println!("Error watching file {}: {}", watcher.path.display(), e);
                }
            }

            // Wait before trying again
            thread::sleep(RETRY_INTERVAL);
        }
    })
}