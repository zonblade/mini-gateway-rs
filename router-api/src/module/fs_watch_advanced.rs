use std::fs::{self, OpenOptions, File};
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write, Read};
use std::path::{PathBuf, Path};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::env;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

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

// State storage location
#[cfg(target_os = "macos")]
fn get_state_path() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join("Library/Application Support/gwrs/logstate.json"))
        .unwrap_or_else(|| PathBuf::from("/tmp/gwrs/logstate.json"))
}

#[cfg(target_os = "linux")]
fn get_state_path() -> PathBuf {
    PathBuf::from("/tmp/gwrs/log/logstate.json")
}

#[cfg(target_os = "windows")]
fn get_state_path() -> PathBuf {
    PathBuf::from("C:\\ProgramData\\gwrs\\logstate.json")
}

const RETRY_INTERVAL: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(100);
const STATE_SAVE_INTERVAL: Duration = Duration::from_secs(60);

// Maximum log file size in bytes (3GB)
const MAX_LOG_SIZE: u64 = 3 * 1024 * 1024 * 1024;

// Maximum number of backup log files to keep
const MAX_BACKUP_FILES: usize = 5;

// Structure to store the last processed log timestamp
#[derive(Serialize, Deserialize, Debug, Clone)]
struct LogState {
    last_processed_timestamp: Option<String>,
    processed_files: HashMap<String, bool>,
}

impl Default for LogState {
    fn default() -> Self {
        Self {
            last_processed_timestamp: None,
            processed_files: HashMap::new(),
        }
    }
}

pub struct LogWatcher {
    path: PathBuf,
    state: Arc<Mutex<LogState>>,
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
        
        // Load or initialize state
        let state = Self::load_state();
        
        Self {
            path: PathBuf::from(expanded_path),
            state,
        }
    }

    // Load log processing state from disk
    fn load_state() -> Arc<Mutex<LogState>> {
        let state_path = get_state_path();
        
        // Ensure directory exists
        if let Some(parent) = state_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        let state = if state_path.exists() {
            match fs::read_to_string(&state_path) {
                Ok(content) => {
                    match serde_json::from_str::<LogState>(&content) {
                        Ok(state) => {
                            println!("Loaded log processing state. Last timestamp: {:?}", 
                                state.last_processed_timestamp);
                            state
                        },
                        Err(e) => {
                            println!("Failed to parse state file, creating new state: {}", e);
                            LogState::default()
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to read state file, creating new state: {}", e);
                    LogState::default()
                }
            }
        } else {
            println!("No state file found, starting with new state");
            LogState::default()
        };
        
        Arc::new(Mutex::new(state))
    }
    
    // Save the current log processing state to disk
    fn save_state(&self) -> io::Result<()> {
        let state_path = get_state_path();
        
        // Ensure directory exists
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Get state and serialize to JSON
        let state = self.state.lock().unwrap();
        let state_json = match serde_json::to_string_pretty(&*state) {
            Ok(json) => json,
            Err(e) => {
                println!("Failed to serialize state: {}", e);
                return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
            }
        };
        
        // Write to temporary file first
        let temp_path = state_path.with_extension("tmp");
        fs::write(&temp_path, state_json)?;
        
        // Rename to actual state file (atomic operation)
        fs::rename(temp_path, state_path)?;
        
        Ok(())
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

    // Extract timestamp from log line
    fn extract_timestamp(&self, log_line: &str) -> Option<String> {
        // Assuming timestamp is in ISO 8601 format like [2025-04-09T12:34:56.789Z]
        if log_line.len() > 20 {
            if let Some(start) = log_line.find('[') {
                if let Some(end) = log_line[start..].find(']') {
                    if end > start + 1 {
                        return Some(log_line[start+1..start+end].to_string());
                    }
                }
            }
        }
        None
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
        let backup_path = log_dir.join(&backup_filename);
        
        println!("Rotating log file to: {}", backup_path.display());
        
        // Rename current log file to backup
        fs::rename(&self.path, &backup_path)?;
        
        // Create a new empty log file
        File::create(&self.path)?;
        
        // Update state to remember we've rotated the file
        {
            let mut state = self.state.lock().unwrap();
            state.processed_files.insert(backup_filename, false);
            // Don't update the last processed timestamp since we want to continue from where we left off
        }
        
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
            
            // Lock state once for the entire operation
            let mut state = self.state.lock().unwrap();
            
            for old_file in backup_files.iter().take(backup_files.len() - MAX_BACKUP_FILES) {
                let filename = old_file.file_name().to_string_lossy().to_string();
                println!("Removing old log file: {}", old_file.path().display());
                
                // Remove the file entry from our processed state
                state.processed_files.remove(&filename);
                
                // Remove the actual file
                let _ = fs::remove_file(old_file.path());
            }
        }
        
        Ok(())
    }
    
    // Process all unprocessed backup files in chronological order
    fn process_backup_files(&self) -> io::Result<()> {
        let log_dir = self.path.parent().unwrap_or(Path::new("."));
        let file_stem = self.path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let extension = self.path.extension().unwrap_or_default().to_string_lossy().to_string();
        
        // Get all backup log files
        let mut backup_files: Vec<_> = fs::read_dir(log_dir)?
            .filter_map(Result::ok)
            .filter(|entry| {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        return name.starts_with(&format!("{}.",&file_stem)) && 
                               name.ends_with(&format!(".{}", &extension));
                    }
                }
                false
            })
            .collect();
        
        // Sort by timestamp in the filename (chronological order)
        backup_files.sort_by(|a, b| {
            let name_a = a.file_name().to_string_lossy().to_string();
            let name_b = b.file_name().to_string_lossy().to_string();
            
            // Extract timestamp part from filename
            let ts_a = name_a.split('.').nth(1).unwrap_or("0");
            let ts_b = name_b.split('.').nth(1).unwrap_or("0");
            
            // Parse as numbers and compare
            let num_a = ts_a.parse::<u64>().unwrap_or(0);
            let num_b = ts_b.parse::<u64>().unwrap_or(0);
            
            num_a.cmp(&num_b)
        });
        
        // Process each unprocessed backup file
        let mut state = self.state.lock().unwrap();
        
        // Track if any file was processed
        let mut any_processed = false;
        
        for file_entry in backup_files {
            let filename = file_entry.file_name().to_string_lossy().to_string();
            
            // Check if this file has been processed already
            if *state.processed_files.get(&filename).unwrap_or(&false) {
                println!("Skipping already processed backup file: {}", filename);
                continue;
            }
            
            println!("Processing backup log file: {}", filename);
            
            // Process this backup file from the last processed timestamp
            match self.process_log_file(&file_entry.path(), &state.last_processed_timestamp) {
                Ok(last_ts) => {
                    // Update the last processed timestamp if one was found
                    if let Some(ts) = last_ts {
                        state.last_processed_timestamp = Some(ts);
                    }
                    
                    // Mark this file as processed
                    state.processed_files.insert(filename, true);
                    any_processed = true;
                },
                Err(e) => {
                    println!("Error processing backup file {}: {}", filename, e);
                }
            }
        }
        
        // Save state if any file was processed
        if any_processed {
            drop(state); // Release lock before saving
            self.save_state()?;
        }
        
        Ok(())
    }
    
    // Process a log file from the given timestamp, returning the last processed timestamp
    fn process_log_file(&self, file_path: &Path, start_timestamp: &Option<String>) -> io::Result<Option<String>> {
        // Open the file for reading
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut last_timestamp: Option<String> = None;
        let mut found_starting_point = start_timestamp.is_none();
        
        // Process each line
        for line in reader.lines() {
            let line = line?;
            
            // Extract timestamp from this line
            if let Some(timestamp) = self.extract_timestamp(&line) {
                // If we haven't found our starting point yet, check if this line is after our start timestamp
                if !found_starting_point {
                    if let Some(start_ts) = start_timestamp {
                        if timestamp > *start_ts {
                            found_starting_point = true;
                        } else {
                            // Skip this line as we've already processed it
                            continue;
                        }
                    }
                }
                
                // Process this line
                if found_starting_point && line.contains("|ID:") {
                    println!("{}", line);
                }
                
                // Update last seen timestamp
                last_timestamp = Some(timestamp);
            }
        }
        
        Ok(last_timestamp)
    }

    fn watch_file(&self) -> io::Result<()> {
        // Ensure log directory exists
        self.ensure_log_directory()?;
        
        println!("Starting to watch log file: {}", self.path.display());
        
        // Process any unprocessed backup files first
        if let Err(e) = self.process_backup_files() {
            println!("Error processing backup files: {}", e);
        }
        
        // Check if we need to rotate the log before watching
        self.check_and_rotate_log()?;
        
        // Try to open the file, create it if it doesn't exist
        let file = OpenOptions::new()
            .read(true)
            .write(true) // Add write permission to ensure proper file creation
            .create(true)
            .open(&self.path)?;
            
        let mut reader = BufReader::new(file);
        
        // Get last processed timestamp
        let start_timestamp = {
            self.state.lock().unwrap().last_processed_timestamp.clone()
        };
        
        // If we have a last processed timestamp, scan file to find that position
        let mut found_starting_point = start_timestamp.is_none();
        let mut current_pos = 0;
        
        if !found_starting_point {
            println!("Searching for last processed timestamp: {:?}", start_timestamp);
            
            // Read through file to find our position
            let mut line = String::new();
            while reader.read_line(&mut line)? > 0 {
                if let Some(timestamp) = self.extract_timestamp(&line) {
                    if let Some(start_ts) = &start_timestamp {
                        if &timestamp > start_ts {
                            // Found a line after our last processed timestamp
                            found_starting_point = true;
                            
                            // Process this line
                            if line.contains("|ID:") {
                                println!("{}", line.trim_end());
                            }
                            
                            // Update state with this timestamp
                            {
                                let mut state = self.state.lock().unwrap();
                                state.last_processed_timestamp = Some(timestamp);
                            }
                            
                            break;
                        }
                    }
                }
                line.clear();
                current_pos = reader.stream_position()?;
            }
            
            // If we didn't find our starting point, start from current position
            if !found_starting_point {
                println!("No newer logs found after last processed timestamp, continuing from end of file");
                current_pos = reader.stream_position()?;
            }
        } else {
            // No last timestamp, start from end of file
            current_pos = reader.seek(SeekFrom::End(0))?;
        }
        
        // Track when we last checked file size and saved state
        let mut last_size_check = Instant::now();
        let mut last_state_save = Instant::now();
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
            
            // Periodically save state
            if last_state_save.elapsed() >= STATE_SAVE_INTERVAL {
                if let Err(e) = self.save_state() {
                    println!("Failed to save log state: {}", e);
                }
                last_state_save = Instant::now();
            }
            
            // Try to read new content
            let mut buffer = String::new();
            
            // Check if our file position is still valid
            let new_pos = reader.stream_position()?;
            if new_pos < current_pos {
                // File was truncated, reset position
                println!("Log file was truncated, resetting position");
                current_pos = 0;
                reader.seek(SeekFrom::Start(0))?;
            } else {
                // Ensure we're at our current position
                if new_pos != current_pos {
                    reader.seek(SeekFrom::Start(current_pos))?;
                }
            }
            
            // Read next line
            let bytes_read = reader.read_line(&mut buffer)?;
            
            if bytes_read > 0 {
                // Update our position
                current_pos = reader.stream_position()?;
                
                // Remove the trailing newline
                let line = buffer.trim_end().to_string();
                
                // Extract timestamp from this line
                if let Some(timestamp) = self.extract_timestamp(&line) {
                    // Process this line
                    if line.contains("|ID:") {
                        println!("{}", line);
                    }
                    
                    // Update last processed timestamp
                    {
                        let mut state = self.state.lock().unwrap();
                        state.last_processed_timestamp = Some(timestamp);
                    }
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

            // Save state before retry
            if let Err(e) = watcher.save_state() {
                println!("Failed to save log state before retry: {}", e);
            }

            // Wait before trying again
            thread::sleep(RETRY_INTERVAL);
        }
    })
}