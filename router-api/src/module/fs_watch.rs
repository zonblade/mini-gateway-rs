use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use std::env;


#[cfg(target_os = "macos")]
fn get_default_log_dir() -> String {
    dirs::home_dir()
        .map(|p| p.join("Library/Logs/gwrs/core.log").to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("/tmp/gwrs/core.log"))
}

#[cfg(target_os = "linux")]
fn get_default_log_dir() -> String {
    String::from("/var/log/gwrs/core.log")
}

#[cfg(target_os = "windows")]
fn get_default_log_dir() -> String {
    String::from("C:\\ProgramData\\gwrs\\core.log")
}

const RETRY_INTERVAL: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

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
        
        // Try to open the file, create it if it doesn't exist
        let file = OpenOptions::new()
            .read(true)
            .write(true) // Add write permission to ensure proper file creation
            .create(true)
            .open(&self.path)?;
            
        // Get file metadata to check size
        let metadata = file.metadata()?;
        
        let mut reader = BufReader::new(file);
        // Only seek if the file has content
        let mut pos = if metadata.len() > 0 {
            reader.seek(SeekFrom::End(0))?
        } else {
            0
        };
        
        loop {
            // Check if file still exists
            if !self.path.exists() {
                return Ok(());
            }
            
            // Try to read new content
            let mut buffer = String::new();
            // Only seek if we have a valid position
            let new_pos = if pos > 0 {
                reader.seek(SeekFrom::Current(0))?
            } else {
                0
            };
            
            if new_pos < pos {
                // File was truncated, reset position
                println!("Log file was truncated, resetting position");
                pos = 0;
                reader.seek(SeekFrom::Start(0))?;
            } else {
                pos = new_pos;
            }
            
            let bytes_read = reader.read_line(&mut buffer)?;
            
            if bytes_read > 0 {
                // Remove the trailing newline
                if buffer.ends_with('\n') {
                    buffer.pop();
                    if buffer.ends_with('\r') {
                        buffer.pop();
                    }
                }
                
                // Only print the log line if it contains the "|ID:" pattern
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