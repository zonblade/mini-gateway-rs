use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::fs::{self, File, OpenOptions};
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, BufReader};
use tokio::sync::Mutex;
use tokio::time::{interval, sleep, timeout};

use crate::module::database::get_connection_log;

use super::constants::{POLL_INTERVAL, RETRY_INTERVAL, SCAN_INTERVAL};
use super::db_pool::{get_log_db_pool, LogDbPool, LogEntry};
use super::file_id::{get_file_id, FileId};
use super::utils::get_default_log_dir;

pub struct LogWatcher {
    path: PathBuf,
    processed_lines: usize,
    processed_files: HashSet<FileId>, // Track already processed files by ID
    db_pool: Arc<LogDbPool>,          // Database pool for batching log entries
    // File reading state
    current_file: Option<File>,
    current_path: Option<PathBuf>,
    current_id: Option<FileId>,
    reader: Option<BufReader<File>>,
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

        let db = get_connection_log().expect("msg: Failed to connect to database");
        let db = Arc::new(db);

        Self {
            path: PathBuf::from(expanded_path),
            processed_lines: 0,
            processed_files: HashSet::new(),
            db_pool: get_log_db_pool(db), // Pass the database client to the pool
            current_file: None,
            current_path: None,
            current_id: None,
            reader: None,
        }
    }

    // Create log directory if it doesn't exist
    async fn ensure_log_directory(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            if !Path::new(parent).exists() {
                println!("Creating log directory: {}", parent.display());
                fs::create_dir_all(parent).await?;
            }
        }
        Ok(())
    }

    // Find all log files related to our main log file
    async fn find_related_log_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut result = Vec::new();

        // Add the main file if it exists
        if Path::new(&self.path).exists() {
            result.push(self.path.clone());
        }

        // Get directory and base filename
        if let Some(parent) = self.path.parent() {
            if let Some(filename) = self.path.file_name() {
                let filename_str = filename.to_string_lossy().into_owned();

                // Scan directory for files with similar names (backup files)
                if let Ok(mut entries) = fs::read_dir(parent).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path == self.path {
                            continue; // Skip main file, already added
                        }

                        if let Some(name) = path.file_name() {
                            let name_str = name.to_string_lossy();
                            // Match backup patterns like filename.1, filename.123456789.log, etc.
                            if name_str.starts_with(&filename_str)
                                || (name_str.contains(&filename_str) && name_str.contains("."))
                            {
                                result.push(path);
                            }
                        }
                    }
                }
            }
        }

        // Sort by modification time, newest first
        // Note: Doing this sequentially since it's relatively infrequent
        result.sort_by(|a, b| {
            let time_a = a
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| std::time::SystemTime::now());
            let time_b = b
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| std::time::SystemTime::now());
            time_b.cmp(&time_a) // Newest first
        });

        Ok(result)
    }

    // Open the log file for reading
    async fn open_file(&self, path: &Path) -> io::Result<File> {
        OpenOptions::new().read(true).open(path).await
    }

    // Process a single line from the log file
    fn process_log_line(&mut self, line: &str) -> bool {
        // Only process log lines containing "|ID:"
        if !line.contains("|ID:") {
            return false;
        }

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

        // Find the pipe character that separates header from content
        let pipe_idx = match line.find('|') {
            Some(idx) => idx,
            None => {
                println!("Malformed log line, missing pipe separator: {}", line);
                return false; // Skip this line if it doesn't have a pipe
            }
        };

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
                            path = parts[1][next_bracket + 1..].trim().to_string();
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
        let content = &line[pipe_idx + 1..];

        // Parse ID
        if let Some(id_start) = content.find("ID:") {
            let id_content = &content[id_start + 3..];
            if let Some(comma_idx) = id_content.find(',') {
                id = id_content[0..comma_idx].trim().to_string();
            }
        }

        // Parse STATUS
        if let Some(status_start) = content.find("STATUS:") {
            let status_content = &content[status_start + 7..];
            if let Some(comma_idx) = status_content.find(',') {
                status = status_content[0..comma_idx].trim().to_string();
            }
        }

        // Parse SIZE
        if let Some(size_start) = content.find("SIZE:") {
            let size_content = &content[size_start + 5..];
            if let Some(comma_idx) = size_content.find(',') {
                if let Ok(s) = size_content[0..comma_idx].trim().parse::<usize>() {
                    size = s;
                }
            }
        }

        // Parse COMMENT
        if let Some(comment_start) = content.find("COMMENT:") {
            let comment_content = &content[comment_start + 8..];
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
        
        true
    }

    // Create an async task to report stats periodically
    async fn start_stats_reporter(stats_mutex: Arc<Mutex<(usize, usize)>>) {
        let stats_interval = Duration::from_secs(60);
        let mut interval = interval(stats_interval);

        loop {
            interval.tick().await;
            
            let stats = {
                // Lock only briefly to read the stats
                let stats = stats_mutex.lock().await;
                *stats
            };
            
            println!(
                "Log watcher stats: processed {} lines, {} files",
                stats.0, stats.1
            );
        }
    }

    // After opening a file, we need to get its ID, which requires converting from tokio::fs::File to std::fs::File
    async fn get_file_id_async(&self, file: &File) -> io::Result<FileId> {
        // Use into_std safely - first clone the file so we don't lose the original
        let file_clone = file.try_clone().await?;
        
        // Convert to a standard File
        let std_file = file_clone.into_std().await;
        
        // Now get the file ID using the existing function
        super::file_id::get_file_id(&std_file)
    }

    // Main tailing function that follows log file changes
    pub async fn tail_file(&mut self) -> io::Result<()> {
        // Ensure log directory exists
        self.ensure_log_directory().await?;

        // Stats for diagnostics - using mutex so they can be accessed from other tasks
        let stats_mutex = Arc::new(Mutex::new((self.processed_lines, self.processed_files.len())));
        let stats_mutex_clone = stats_mutex.clone();
        
        // Spawn a task to report stats periodically without capturing self
        tokio::spawn(async move {
            // Use the static method instead of the instance method
            Self::start_stats_reporter(stats_mutex_clone).await;
        });

        // Time of last scan for rotated files
        let mut last_scan_time = Instant::now();

        println!("Starting to watch log file: {}", self.path.display());

        loop {
            // Check if we need to scan for rotated files
            if last_scan_time.elapsed() >= SCAN_INTERVAL {
                match self.find_related_log_files().await {
                    Ok(files) => {
                        if !files.is_empty() {
                            println!("Found {} log files during scan", files.len());

                            // Check if our current file is gone or has changed
                            let current_exists = match &self.current_path {
                                Some(p) => Path::new(p).exists(),
                                None => false,
                            };

                            // If our current file is gone or this is initial startup with no file
                            if !current_exists || self.current_file.is_none() {
                                // Try each file, prioritizing newer files
                                for path in &files {
                                    match self.open_file(path).await {
                                        Ok(file) => {
                                            match self.get_file_id_async(&file).await {
                                                Ok(id) => {
                                                    // Skip if we've already processed this file
                                                    if self.processed_files.contains(&id) {
                                                        continue;
                                                    }

                                                    // We found a new file to process
                                                    println!(
                                                        "Switching to log file: {}",
                                                        path.display()
                                                    );
                                                    
                                                    // Create new reader starting at beginning
                                                    let file_clone = file.try_clone().await?;
                                                    let buf_reader = BufReader::new(file_clone);
                                                    
                                                    self.current_file = Some(file);
                                                    self.current_path = Some(path.clone());
                                                    self.current_id = Some(id);
                                                    self.reader = Some(buf_reader);
                                                    
                                                    break;
                                                }
                                                Err(e) => println!("Failed to get file ID: {}", e),
                                            }
                                        }
                                        Err(e) => println!(
                                            "Failed to open file {}: {}",
                                            path.display(),
                                            e
                                        ),
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => println!("Failed to scan for log files: {}", e),
                }

                last_scan_time = Instant::now();
            }

            // Check if the main file exists
            let main_file_exists = Path::new(&self.path).exists();

            // If we don't have a file open yet, try to open the main file
            if self.current_file.is_none() && main_file_exists {
                match self.open_file(&self.path).await {
                    Ok(file) => {
                        match self.get_file_id_async(&file).await {
                            Ok(id) => {
                                // Skip if we've already processed this file
                                if !self.processed_files.contains(&id) {
                                    println!("Opened main log file for tailing");
                                    
                                    // Create a new reader
                                    let file_clone = file.try_clone().await?;
                                    let mut buf_reader = BufReader::new(file_clone);

                                    // Determine if we should start from beginning or end
                                    if self.processed_lines == 0 {
                                        // First time opening, go to the end
                                        buf_reader.seek(io::SeekFrom::End(0)).await?;
                                    }

                                    self.current_file = Some(file);
                                    self.current_path = Some(self.path.clone());
                                    self.current_id = Some(id);
                                    self.reader = Some(buf_reader);
                                } else {
                                    println!("Skipping already processed main log file");
                                }
                            }
                            Err(e) => {
                                println!("Failed to get file ID: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        if e.kind() != io::ErrorKind::NotFound {
                            println!("Error opening log file: {}", e);
                        }
                        sleep(RETRY_INTERVAL).await;
                        continue;
                    }
                }
            }

            // If we have a file open
            if self.current_file.is_some() {
                // Periodically check for file rotation by comparing file IDs
                if let Some(ref path) = self.current_path {
                    if Path::new(path).exists() {
                        match self.open_file(path).await {
                            Ok(new_file) => {
                                match self.get_file_id_async(&new_file).await {
                                    Ok(new_id) => {
                                        // If file ID changed, the file has been rotated
                                        if let Some(current_id) = self.current_id {
                                            if current_id != new_id {
                                                // Mark current file as processed
                                                self.processed_files.insert(current_id);

                                                // Check if we already processed this new file
                                                if self.processed_files.contains(&new_id) {
                                                    // Try to find another file during next scan
                                                    self.current_file = None;
                                                    self.current_path = None;
                                                    self.current_id = None;
                                                    self.reader = None;
                                                    println!("New file already processed, will search for more files");
                                                    continue;
                                                }

                                                println!("Log file has been rotated, switching to new file");
                                                
                                                // Create new reader
                                                let file_clone = new_file.try_clone().await?;
                                                let buf_reader = BufReader::new(file_clone);
                                                
                                                self.current_file = Some(new_file);
                                                self.current_id = Some(new_id);
                                                self.reader = Some(buf_reader);
                                                continue;
                                            }
                                        }
                                    }
                                    Err(e) => println!("Failed to get new file ID: {}", e),
                                }
                            }
                            Err(e) => println!("Failed to open file for ID check: {}", e),
                        }
                    } else {
                        // Our current file no longer exists
                        // Mark it as processed before moving on
                        if let Some(id) = self.current_id {
                            self.processed_files.insert(id);
                        }

                        println!("Current log file no longer exists, will search for another file");
                        self.current_file = None;
                        self.current_path = None;
                        self.current_id = None;
                        self.reader = None;

                        // Immediately trigger a scan to find backup files
                        last_scan_time = Instant::now() - SCAN_INTERVAL;
                        sleep(POLL_INTERVAL).await;
                        continue;
                    }
                }

                // Process new lines from the file
                if let Some(reader) = &mut self.reader {
                    let mut line = String::new();
                    
                    // Use timeout to avoid blocking indefinitely on read_line
                    match timeout(POLL_INTERVAL, reader.read_line(&mut line)).await {
                        Ok(result) => match result {
                            Ok(0) => {
                                // No new data, sleep briefly
                                sleep(POLL_INTERVAL).await;
                            }
                            Ok(_) => {
                                // Process the line - remove trailing newline
                                if line.ends_with('\n') {
                                    line.pop();
                                    if line.ends_with('\r') {
                                        line.pop();
                                    }
                                }

                                if self.process_log_line(&line) {
                                    self.processed_lines += 1;
                                    
                                    // Update stats
                                    let mut stats = stats_mutex.lock().await;
                                    stats.0 = self.processed_lines;
                                }
                            }
                            Err(e) => {
                                println!("Error reading line: {}, reopening file", e);
                                // Don't mark as processed on read error
                                self.current_file = None;
                                self.current_path = None;
                                self.current_id = None;
                                self.reader = None;
                                sleep(RETRY_INTERVAL).await;
                                continue;
                            }
                        },
                        Err(_) => {
                            // Timeout reached, just continue to the next iteration
                            continue;
                        }
                    }
                }
            } else if !main_file_exists {
                // Trigger a scan to find backup files if main file doesn't exist
                last_scan_time = Instant::now() - SCAN_INTERVAL;
                // File doesn't exist and we don't have one open, wait and retry
                sleep(RETRY_INTERVAL).await;
            }
            
            // Update the file count in stats
            {
                let mut stats = stats_mutex.lock().await;
                stats.1 = self.processed_files.len();
            }
        }
    }
}
