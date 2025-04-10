use log::{Level, LevelFilter, Metadata, Record};
use std::io::{BufWriter, Write, Seek, SeekFrom};
use std::fs::{File, rename};
use std::path::Path;
use std::sync::RwLock;

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10MB in bytes

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
    fn flush_if_needed(&self, writer: &mut BufWriter<File>, level: Level) {
        // Always flush logs to ensure they're written to disk immediately
        writer.flush().ok();
    }

    // Check if log rotation is needed and rotate if necessary
    fn check_rotation(&self, writer: &mut BufWriter<File>, log_path: &str) -> std::io::Result<()> {
        let file = writer.get_ref();
        let metadata = file.metadata()?;
        
        if metadata.len() >= MAX_LOG_SIZE {
            // Flush before rotating
            writer.flush()?;
            
            // Create backup filename
            let backup_path = format!("{}.1", log_path);
            
            // Close and reopen the file to release file handle
            drop(&mut *writer);
            
            // Rename current log to backup
            if Path::new(log_path).exists() {
                rename(log_path, backup_path)?;
            }
            
            // Open new log file
            let new_file = File::create(log_path)?;
            *writer = BufWriter::new(new_file);
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
