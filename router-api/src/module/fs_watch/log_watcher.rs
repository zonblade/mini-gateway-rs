use std::fs::{OpenOptions, File, read_dir};
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::{PathBuf, Path};
use std::thread;
use std::time::{Duration, Instant};
use std::env;
use std::collections::HashSet;
use std::sync::Arc;
use chrono::Local;

use super::constants::{RETRY_INTERVAL, POLL_INTERVAL, SCAN_INTERVAL};
use super::file_id::{FileId, get_file_id};
use super::utils::get_default_log_dir;
use super::db_pool::{LogDbPool, LogEntry, get_log_db_pool};

pub struct LogWatcher {
    path: PathBuf,
    processed_lines: usize,
    processed_files: HashSet<FileId>, // Track already processed files by ID
    db_pool: Arc<LogDbPool>, // Database pool for batching log entries
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
            processed_lines: 0,
            processed_files: HashSet::new(),
            db_pool: Arc::new(get_log_db_pool()), // Initialize database pool
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

    // Find all log files related to our main log file
    fn find_related_log_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut result = Vec::new();
        
        // Add the main file if it exists
        if self.path.exists() {
            result.push(self.path.clone());
        }
        
        // Get directory and base filename
        if let Some(parent) = self.path.parent() {
            if let Some(filename) = self.path.file_name() {
                let filename_str = filename.to_string_lossy().into_owned();
                
                // Scan directory for files with similar names (backup files)
                if let Ok(entries) = read_dir(parent) {
                    for entry in entries.filter_map(Result::ok) {
                        let path = entry.path();
                        if path == self.path {
                            continue; // Skip main file, already added
                        }
                        
                        if let Some(name) = path.file_name() {
                            let name_str = name.to_string_lossy();
                            // Match backup patterns like filename.1, filename.123456789.log, etc.
                            if name_str.starts_with(&filename_str) || 
                               (name_str.contains(&filename_str) && name_str.contains(".")) {
                                result.push(path);
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by modification time, newest first
        result.sort_by(|a, b| {
            let time_a = a.metadata().and_then(|m| m.modified()).unwrap_or_else(|_| std::time::SystemTime::now());
            let time_b = b.metadata().and_then(|m| m.modified()).unwrap_or_else(|_| std::time::SystemTime::now());
            time_b.cmp(&time_a)  // Newest first
        });
        
        Ok(result)
    }

    // Open the log file for reading
    fn open_file(&self, path: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .open(path)
    }

    // Main tailing function that follows log file changes
    pub fn tail_file(&mut self) -> io::Result<()> {
        // Ensure log directory exists
        self.ensure_log_directory()?;
        
        // Initialize with no current file
        let mut current_file: Option<File> = None;
        let mut current_path: Option<PathBuf> = None;
        let mut current_id: Option<FileId> = None;
        let mut reader: Option<BufReader<File>> = None;
        
        // Stats for diagnostics
        let mut last_stats_time = Instant::now();
        let stats_interval = Duration::from_secs(60);
        
        // Time of last scan for rotated files
        let mut last_scan_time = Instant::now();
        
        println!("Starting to watch log file: {}", self.path.display());
        
        loop {
            // Check if we need to scan for rotated files
            if last_scan_time.elapsed() >= SCAN_INTERVAL {
                match self.find_related_log_files() {
                    Ok(files) => {
                        if !files.is_empty() {
                            println!("Found {} log files during scan", files.len());
                            
                            // Check if our current file is gone or has changed
                            let current_exists = current_path.as_ref().map_or(false, |p| p.exists());
                            
                            // If our current file is gone or this is initial startup with no file
                            if !current_exists || current_file.is_none() {
                                // Try each file, prioritizing newer files
                                for path in &files {
                                    match self.open_file(path) {
                                        Ok(file) => {
                                            match get_file_id(&file) {
                                                Ok(id) => {
                                                    // Skip if we've already processed this file
                                                    if self.processed_files.contains(&id) {
                                                        continue;
                                                    }
                                                    
                                                    // We found a new file to process
                                                    println!("Switching to log file: {}", path.display());
                                                    current_file = Some(file);
                                                    current_path = Some(path.clone());
                                                    current_id = Some(id);
                                                    
                                                    // Create new reader starting at beginning
                                                    let f = current_file.as_ref().unwrap();
                                                    reader = Some(BufReader::new(f.try_clone()?));
                                                    break;
                                                },
                                                Err(e) => println!("Failed to get file ID: {}", e)
                                            }
                                        },
                                        Err(e) => println!("Failed to open file {}: {}", path.display(), e)
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => println!("Failed to scan for log files: {}", e)
                }
                
                last_scan_time = Instant::now();
            }
            
            // Check if the main file exists
            let main_file_exists = self.path.exists();
            
            // If we don't have a file open yet, try to open the main file
            if current_file.is_none() && main_file_exists {
                match self.open_file(&self.path) {
                    Ok(file) => {
                        match get_file_id(&file) {
                            Ok(id) => {
                                // Skip if we've already processed this file
                                if !self.processed_files.contains(&id) {
                                    println!("Opened main log file for tailing");
                                    current_file = Some(file);
                                    current_path = Some(self.path.clone());
                                    current_id = Some(id);
                                    
                                    // Create a new reader
                                    let f = current_file.as_ref().unwrap();
                                    let mut buf_reader = BufReader::new(f.try_clone()?);
                                    
                                    // Determine if we should start from beginning or end
                                    if self.processed_lines == 0 {
                                        // First time opening, go to the end
                                        buf_reader.seek(SeekFrom::End(0))?;
                                    }
                                    
                                    reader = Some(buf_reader);
                                } else {
                                    println!("Skipping already processed main log file");
                                }
                            },
                            Err(e) => {
                                println!("Failed to get file ID: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        if e.kind() != io::ErrorKind::NotFound {
                            println!("Error opening log file: {}", e);
                        }
                        thread::sleep(RETRY_INTERVAL);
                        continue;
                    }
                }
            }
            
            // If we have a file open
            if let Some(_f) = current_file.as_ref() {
                // Periodically check for file rotation by comparing file IDs
                if let Some(ref path) = current_path {
                    if path.exists() {
                        match self.open_file(path) {
                            Ok(new_file) => {
                                match get_file_id(&new_file) {
                                    Ok(new_id) => {
                                        // If file ID changed, the file has been rotated
                                        if current_id.is_some() && current_id.unwrap() != new_id {
                                            // Mark current file as processed
                                            if let Some(id) = current_id {
                                                self.processed_files.insert(id);
                                            }
                                            
                                            // Check if we already processed this new file
                                            if self.processed_files.contains(&new_id) {
                                                // Try to find another file during next scan
                                                current_file = None;
                                                current_path = None;
                                                current_id = None;
                                                reader = None;
                                                println!("New file already processed, will search for more files");
                                                continue;
                                            }
                                            
                                            println!("Log file has been rotated, switching to new file");
                                            current_file = Some(new_file);
                                            current_id = Some(new_id);
                                            reader = Some(BufReader::new(current_file.as_ref().unwrap().try_clone()?));
                                            continue;
                                        }
                                    },
                                    Err(e) => println!("Failed to get new file ID: {}", e)
                                }
                            },
                            Err(e) => println!("Failed to open file for ID check: {}", e)
                        }
                    } else {
                        // Our current file no longer exists
                        // Mark it as processed before moving on
                        if let Some(id) = current_id {
                            self.processed_files.insert(id);
                        }
                        
                        println!("Current log file no longer exists, will search for another file");
                        current_file = None;
                        current_path = None;
                        current_id = None;
                        reader = None;
                        
                        // Immediately trigger a scan to find backup files
                        last_scan_time = Instant::now() - SCAN_INTERVAL;
                        thread::sleep(POLL_INTERVAL);
                        continue;
                    }
                }
                
                // Process new lines from the file
                if let Some(r) = reader.as_mut() {
                    let mut line = String::new();
                    match r.read_line(&mut line) {
                        Ok(0) => {
                            // No new data, sleep briefly
                            thread::sleep(POLL_INTERVAL);
                        },
                        Ok(_) => {
                            // Process the line - remove trailing newline
                            if line.ends_with('\n') {
                                line.pop();
                                if line.ends_with('\r') {
                                    line.pop();
                                }
                            }
                            
                            // Only process log lines containing "|ID:"
                            if line.contains("|ID:") {
                                // Print line to console
                                println!("{}", line);
                                
                                // Parse the log line into components
                                // Expected format: [RFC3339_TIME_FORMAT] INDICATOR [path] [PXY] |ID:<STRING> ,STATUS:<STRING> ,SIZE:<NUM> ,COMMENT:<STRING> |
                                let mut timestamp = String::new();
                                let mut indicator = String::from("[system]"); // Default indicator
                                let mut path = String::from("log"); // Default path
                                let mut id = String::new();
                                let mut status = String::new();
                                let mut size = 0;
                                let mut comment = String::new();
                                
                                // Extract timestamp, indicator, and path from the part before the pipe
                                let prefix = &line[0..pipe_idx].trim();
                                
                                // Attempt to parse parts before the pipe section
                                if let Some(bracket_idx) = prefix.find('[') {
                                    if bracket_idx > 0 {
                                        // Extract timestamp
                                        timestamp = prefix[0..bracket_idx].trim().to_string();
                                        
                                        // Parse the remaining parts in brackets
                                        let parts: Vec<&str> = prefix[bracket_idx..].split(']').collect();
                                        if parts.len() > 1 {
                                            // Extract indicator - this could be ERROR, WARN, LOG, etc.
                                            indicator = parts[0][1..].trim().to_string();
                                            
                                            // Try to extract path from the next bracket
                                            if let Some(next_bracket) = parts[1].find('[') {
                                                if next_bracket < parts[1].len() - 1 {
                                                    path = parts[1][next_bracket+1..].trim().to_string();
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // No bracket found, check if there's a common indicator without brackets
                                    let common_indicators = ["ERROR", "WARN", "INFO", "DEBUG", "LOG", "TRACE"];
                                    for &ind in &common_indicators {
                                        if let Some(pos) = prefix.find(ind) {
                                            // Extract timestamp before the indicator
                                            if pos > 0 {
                                                timestamp = prefix[0..pos].trim().to_string();
                                            }
                                            // Set the indicator
                                            indicator = ind.to_string();
                                            break;
                                        }
                                    }
                                }
                                
                                // Extract the content after the pipe
                                let content = &line[pipe_idx+1..];
                                
                                // Parse ID
                                if let Some(id_start) = content.find("ID:") {
                                    let id_content = &content[id_start+3..];
                                    if let Some(comma_idx) = id_content.find(',') {
                                        id = id_content[0..comma_idx].trim().to_string();
                                    }
                                }
                                
                                // Parse STATUS
                                if let Some(status_start) = content.find("STATUS:") {
                                    let status_content = &content[status_start+7..];
                                    if let Some(comma_idx) = status_content.find(',') {
                                        status = status_content[0..comma_idx].trim().to_string();
                                    }
                                }
                                
                                // Parse SIZE
                                if let Some(size_start) = content.find("SIZE:") {
                                    let size_content = &content[size_start+5..];
                                    if let Some(comma_idx) = size_content.find(',') {
                                        if let Ok(s) = size_content[0..comma_idx].trim().parse::<usize>() {
                                            size = s;
                                        }
                                    }
                                }
                                
                                // Parse COMMENT
                                if let Some(comment_start) = content.find("COMMENT:") {
                                    let comment_content = &content[comment_start+8..];
                                    if let Some(pipe_idx) = comment_content.find('|') {
                                        comment = comment_content[0..pipe_idx].trim().to_string();
                                    } else {
                                        comment = comment_content.trim().to_string();
                                    }
                                }
                                
                                // If timestamp is empty, use current time
                                if timestamp.is_empty() {
                                    timestamp = chrono::Local::now().to_rfc3339();
                                }
                                
                                // Create log entry with the parsed components
                                let log_entry = LogEntry {
                                    timestamp,
                                    indicator,
                                    path,
                                    id,
                                    status,
                                    size,
                                    comment,
                                };
                                
                                // Add to pool - will be flushed every 5 seconds
                                self.db_pool.add_log(log_entry);
                            }
                            
                            self.processed_lines += 1;
                        },
                        Err(e) => {
                            println!("Error reading line: {}, reopening file", e);
                            // Don't mark as processed on read error
                            current_file = None;
                            current_path = None;
                            current_id = None;
                            reader = None;
                            thread::sleep(RETRY_INTERVAL);
                            continue;
                        }
                    }
                }
                
                // Log stats periodically
                if last_stats_time.elapsed() >= stats_interval {
                    println!("Log watcher stats: processed {} lines, {} files", 
                             self.processed_lines, self.processed_files.len());
                    last_stats_time = Instant::now();
                }
            } else if !main_file_exists {
                // Trigger a scan to find backup files if main file doesn't exist
                last_scan_time = Instant::now() - SCAN_INTERVAL;
                // File doesn't exist and we don't have one open, wait and retry
                thread::sleep(RETRY_INTERVAL);
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
    let mut watcher = LogWatcher::new();
    
    // Spawn a dedicated thread for the log watcher
    thread::spawn(move || {
        loop {
            match watcher.tail_file() {
                Ok(_) => println!("Log watcher stopped unexpectedly"),
                Err(e) => println!("Error in log watcher: {}", e)
            }
            
            // Wait before retrying
            thread::sleep(RETRY_INTERVAL);
        }
    })
}