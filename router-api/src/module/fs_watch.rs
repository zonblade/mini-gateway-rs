use std::fs::{self, OpenOptions, File};
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::{PathBuf, Path};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::env;

#[cfg(target_os = "macos")]
fn get_default_log_dir() -> String {
    dirs::home_dir()
        .map(|p| p.join("Library/Logs/gwrs/core.log").to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("/tmp/gwrs/core.log"))
}

#[cfg(target_os = "linux")]
fn get_default_log_dir() -> String {
    String::from("/tmp/gwrs/log/core.log")
}

#[cfg(target_os = "windows")]
fn get_default_log_dir() -> String {
    String::from("C:\\ProgramData\\gwrs\\core.log")
}

const RETRY_INTERVAL: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

// Maximum log file size in bytes (3GB)
const MAX_LOG_SIZE: u64 = 3 * 1024 * 1024 * 1024;

// Maximum number of backup log files to keep
const MAX_BACKUP_FILES: usize = 5;

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

    /// Checks if the log file exceeds the maximum size and rotates it if necessary
    fn check_and_rotate_log(&self) -> io::Result<bool> {
        // Check if the log file exists
        if self.path.exists() {
            // Get the file size
            let metadata = fs::metadata(&self.path)?;
            let file_size = metadata.len();
            
            // If the file size exceeds the maximum, rotate the log
            if file_size >= MAX_LOG_SIZE {
                println!("Log file size ({} bytes) exceeds limit ({}), rotating logs", 
                    file_size, MAX_LOG_SIZE);
                self.rotate_logs()?;
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Rotates log files, creating backups with timestamps
    fn rotate_logs(&self) -> io::Result<()> {
        let log_dir = self.path.parent().unwrap_or(Path::new("."));
        let file_stem = self.path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let extension = self.path.extension().unwrap_or_default().to_string_lossy().to_string();
        
        // Get timestamp for uniqueness
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Create backup filename with timestamp
        let backup_filename = format!("{}.{}.{}", file_stem, timestamp, extension);
        let backup_path = log_dir.join(backup_filename);
        
        println!("Rotating log file to: {}", backup_path.display());
        
        // Rename current log file to backup
        fs::rename(&self.path, &backup_path)?;
        
        // Create a new empty log file
        File::create(&self.path)?;
        
        // Clean up old log files if there are too many
        self.cleanup_old_logs()?;
        
        Ok(())
    }

    /// Removes the oldest backup log files to keep only MAX_BACKUP_FILES
    fn cleanup_old_logs(&self) -> io::Result<()> {
        let log_dir = self.path.parent().unwrap_or(Path::new("."));
        let file_stem = self.path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let extension = self.path.extension().unwrap_or_default().to_string_lossy().to_string();
        
        // Collect all backup log files
        let mut backup_files: Vec<_> = fs::read_dir(log_dir)?
            .filter_map(Result::ok)
            .filter(|entry| {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        // Use simple string matching for our timestamp pattern
                        return name.starts_with(&format!("{}.",&file_stem)) && 
                               name.ends_with(&format!(".{}", &extension));
                    }
                }
                false
            })
            .collect();
        
        // Sort by modification time (oldest first)
        backup_files.sort_by(|a, b| {
            let time_a = a.metadata().and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::now());
            let time_b = b.metadata().and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::now());
            time_a.cmp(&time_b)
        });
        
        // Remove oldest files if we have too many
        if backup_files.len() > MAX_BACKUP_FILES {
            println!("Cleaning up old log files, keeping {} most recent backups", MAX_BACKUP_FILES);
            for old_file in backup_files.iter().take(backup_files.len() - MAX_BACKUP_FILES) {
                println!("Removing old log file: {}", old_file.path().display());
                let _ = fs::remove_file(old_file.path());
            }
        }
        
        Ok(())
    }

    fn watch_file(&self) -> io::Result<()> {
        // Ensure log directory exists
        self.ensure_log_directory()?;
        
        println!("Starting to watch log file: {}", self.path.display());
        
        // Check if we need to rotate the log before watching
        self.check_and_rotate_log()?;
        
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
        
        // Track when we last checked file size
        let mut last_size_check = Instant::now();
        let size_check_interval = Duration::from_secs(60); // Check size every minute
        
        loop {
            // Check if file still exists
            if !self.path.exists() {
                return Ok(());
            }
            
            // Periodically check file size and rotate if necessary
            if last_size_check.elapsed() >= size_check_interval {
                if self.check_and_rotate_log()? {
                    // File was rotated, need to reopen it
                    return Ok(());
                }
                last_size_check = Instant::now();
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