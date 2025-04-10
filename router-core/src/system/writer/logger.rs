use log::{Level, LevelFilter, Metadata, Record};
use regex::Regex;
use std::io::{BufWriter, Write};
use std::sync::RwLock;

// Define a custom logger that will direct logs to different files based on message content
pub struct TagBasedLogger {
    // Default writer for logs that don't match specific tags
    pub default_writer: RwLock<BufWriter<std::fs::File>>,
    // Map of tag-based writers (tag -> writer)
    pub tag_writers: Vec<(Regex, RwLock<BufWriter<std::fs::File>>)>,
    // Level filter
    pub level_filter: LevelFilter,
}

impl log::Log for TagBasedLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = record.args().to_string();

            // Check if message matches any of our tag patterns
            let mut matched = false;

            for (pattern, writer) in &self.tag_writers {
                if pattern.is_match(&message) {
                    if let Ok(mut writer) = writer.write() {
                        writeln!(
                            writer,
                            "[{}] {} [{}] {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            record.level(),
                            record.module_path().unwrap_or("unknown"),
                            record.args()
                        )
                        .ok();

                        // Flush high-priority logs immediately
                        if record.level() <= Level::Warn {
                            writer.flush().ok();
                        }
                    }
                    matched = true;
                    // We want logs to potentially go to multiple files if they match multiple patterns
                }
            }

            // If no specific tag matched or we want to log to the default file anyway
            if !matched {
                if let Ok(mut writer) = self.default_writer.write() {
                    writeln!(
                        writer,
                        "[{}] {} [{}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        record.module_path().unwrap_or("unknown"),
                        record.args()
                    )
                    .ok();

                    // Flush high-priority logs immediately
                    if record.level() <= Level::Warn {
                        writer.flush().ok();
                    }
                }
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
