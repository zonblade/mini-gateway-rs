use std::fs::{OpenOptions, File, read_dir};
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::{PathBuf, Path};
use std::thread;
use std::time::{Duration, Instant};
use std::env;
use std::collections::HashSet;

#[cfg(target_os = "macos")]
fn get_default_log_dir() -> String {
    String::from("/tmp/gwrs/log/core.proxy.log")
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
const SCAN_INTERVAL: Duration = Duration::from_secs(5); // How often to scan for rotated files

// Structure to hold file stats for detecting file changes
#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FileId {
    dev: u64, // device ID
    ino: u64, // inode number
}

#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FileId {
    volume_serial_number: u32,
    file_index: u64,
}

// Get a unique file identifier that survives across renames
#[cfg(unix)]
fn get_file_id(file: &File) -> io::Result<FileId> {
    use std::os::unix::fs::MetadataExt;
    let metadata = file.metadata()?;
    Ok(FileId {
        dev: metadata.dev(),
        ino: metadata.ino(),
    })
}

#[cfg(windows)]
fn get_file_id(file: &File) -> io::Result<FileId> {
    use std::os::windows::fs::MetadataExt;
    let metadata = file.metadata()?;
    Ok(FileId {
        volume_serial_number: metadata.volume_serial_number(),
        file_index: metadata.file_index(),
    })
}

pub struct LogWatcher {
    path: PathBuf,
    processed_lines: usize,
    processed_files: HashSet<FileId>, // Track already processed files by ID
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
    fn tail_file(&mut self) -> io::Result<()> {
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
                            
                            // Only print log lines containing "|ID:"
                            if line.contains("|ID:") {
                                println!("{}", line);
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