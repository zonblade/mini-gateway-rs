use log::{Level, LevelFilter, Metadata, Record};
use std::io::{BufWriter, Write};
use std::fs::{File, rename};
use std::path::Path;
use std::sync::RwLock;

const MAX_LOG_SIZE: u64 = 1 * 1024 * 1024 * 1024; // 1GB in bytes

// Define a custom logger that will direct logs to different files based on message content
pub struct TagBasedLogger {
    // Default writer for logs that don't match specific tags
    pub default_writer: RwLock<BufWriter<File>>,
    pub default_path: String,
    // Map of tag-based writers (tag -> writer)
    pub tag_writers: Vec<(&'static str, RwLock<BufWriter<File>>, String)>,
    // Level filter
    pub level_filter: LevelFilter,
}

impl TagBasedLogger {
    // Helper function to format and write the log entry
    fn write_log_entry(&self, writer: &mut BufWriter<File>, record: &Record) {
        writeln!(
            writer,
            "[{}] {} [{}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.module_path().unwrap_or("unknown"),
            record.args()
        ).ok();
    }

    // Helper function to handle flushing for logs
    fn flush_if_needed(&self, writer: &mut BufWriter<File>, _level: Level) {
        // Always flush logs to ensure they're written to disk immediately
        writer.flush().ok();
    }

    // Check if log rotation is needed and rotate if necessary
    fn check_rotation(&self, writer: &mut BufWriter<File>, log_path: &str) -> std::io::Result<()> {
        let file = writer.get_ref();
        let metadata = file.metadata()?;
        
        if metadata.len() >= MAX_LOG_SIZE {
            // Flush any pending data
            writer.flush()?;
            
            // Create backup filename
            let backup_path = format!("{}.1", log_path);
            
            // We need to replace the writer with a new one to release the file handle
            // First take ownership of the current file path
            let path_to_rotate = log_path.to_string();
            
            // Create a temporary file path for later cleanup
            let temp_path = format!("{}.new", log_path);
            
            // Create a temporary writer to swap with our current one
            let temp_file = File::create(&temp_path)?;
            
            // Replace the writer with a temporary one, effectively closing the original file
            std::mem::swap(writer, &mut BufWriter::new(temp_file));
            
            // Now we can safely rename the original file since we've released the handle
            if Path::new(&path_to_rotate).exists() {
                rename(&path_to_rotate, backup_path)?;
            }
            
            // Create our new log file and assign it to the writer
            let new_file = File::create(&path_to_rotate)?;
            *writer = BufWriter::new(new_file);
            
            // Remove the temporary file we created
            let _ = std::fs::remove_file(&temp_path);
        }
        
        Ok(())
    }
}

impl log::Log for TagBasedLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let message = record.args().to_string();
        let mut matched = false;

        // Try to write to tag-specific writers
        for (pattern, writer, path) in &self.tag_writers {
            if message.contains(pattern) {
                match writer.write() {
                    Ok(mut writer) => {
                        // Check and rotate if needed
                        if let Err(e) = self.check_rotation(&mut writer, path) {
                            // Log rotation error and continue
                            if let Ok(mut default_writer) = self.default_writer.write() {
                                writeln!(
                                    default_writer,
                                    "[{}] ERROR [logger] Failed to rotate log with pattern '{}': {}",
                                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                                    pattern,
                                    e
                                ).ok();
                                self.flush_if_needed(&mut default_writer, Level::Error);
                            }
                        }

                        self.write_log_entry(&mut writer, record);
                        self.flush_if_needed(&mut writer, record.level());
                        matched = true; // Only mark as matched if we actually wrote the log
                    }
                    Err(e) => {
                        // We failed to get the lock, try to log this to default writer
                        if let Ok(mut default_writer) = self.default_writer.write() {
                            writeln!(
                                default_writer,
                                "[{}] ERROR [logger] Failed to write log with pattern '{}': {}",
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                                pattern,
                                e
                            ).ok();
                            self.flush_if_needed(&mut default_writer, Level::Error);
                        }
                    }
                }
                // Continue checking other patterns for multiple file logging
            }
        }

        // Write to default writer if no specific tag matched
        if !matched {
            if let Ok(mut writer) = self.default_writer.write() {
                // Check and rotate if needed for default logger
                if let Err(e) = self.check_rotation(&mut writer, &self.default_path) {
                    // Just continue on rotation error for default logger
                    eprintln!("Failed to rotate default log: {}", e);
                }
                
                self.write_log_entry(&mut writer, record);
                self.flush_if_needed(&mut writer, record.level());
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut writer) = self.default_writer.write() {
            writer.flush().ok();
        }

        for (_, writer, _) in &self.tag_writers {
            if let Ok(mut writer) = writer.write() {
                writer.flush().ok();
            }
        }
    }
}
