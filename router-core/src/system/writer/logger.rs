use log::{Level, LevelFilter, Metadata, Record};
use std::io::{BufWriter, Write};
use std::sync::RwLock;

// Define a custom logger that will direct logs to different files based on message content
pub struct TagBasedLogger {
    // Default writer for logs that don't match specific tags
    pub default_writer: RwLock<BufWriter<std::fs::File>>,
    // Map of tag-based writers (tag -> writer)
    pub tag_writers: Vec<(&'static str, RwLock<BufWriter<std::fs::File>>)>,
    // Level filter
    pub level_filter: LevelFilter,
}

impl TagBasedLogger {
    // Helper function to format and write the log entry
    fn write_log_entry(&self, writer: &mut BufWriter<std::fs::File>, record: &Record) {
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
    fn flush_if_needed(&self, writer: &mut BufWriter<std::fs::File>, level: Level) {
        // Always flush logs to ensure they're written to disk immediately
        writer.flush().ok();
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
        for (pattern, writer) in &self.tag_writers {
            // println!("Message: {}, Pattern: {}", message, pattern);

            if message.contains(pattern) {
                // println!("Matched pattern:");
                match writer.write() {
                    Ok(mut writer) => {
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
                self.write_log_entry(&mut writer, record);
                self.flush_if_needed(&mut writer, record.level());
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
