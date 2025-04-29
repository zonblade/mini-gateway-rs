/// Provides logging functionality based on tags, forwarding messages via UDP.
// filepath: /Users/zonblade/Project/runegram/mini-gateway-rs/router-core/src/system/writer/logger.rs
use log::{LevelFilter, Metadata, Record};

use crate::system::udp_sender;

/// A custom logger implementation that filters messages based on tags and forwards them
/// to specific UDP endpoints determined by those tags.
///
/// This logger allows routing log messages to different destinations based on patterns
/// associated with `tag_writers`. It uses a specified `level_filter` to control
/// the verbosity of the logs being processed.
pub struct TagBasedLogger {
    /// A list of string patterns. Log messages matching any of these patterns
    /// will be forwarded by the corresponding UDP writer.
    pub tag_writers: Vec<&'static str>,
    /// The minimum log level required for a message to be processed by this logger.
    pub level_filter: LevelFilter,
}

impl log::Log for TagBasedLogger {
    /// Determines if a log record with the given metadata should be logged.
    ///
    /// This method checks if the record's level is at least as severe as the
    /// logger's configured `level_filter`.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The metadata associated with the log record.
    ///
    /// # Returns
    ///
    /// `true` if the log record should be processed, `false` otherwise.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    /// Processes a log record if it is enabled.
    ///
    /// If the record meets the level criteria set by `enabled`, this method
    /// converts the log arguments to a string message and iterates through the
    /// configured `tag_writers`. For each pattern, it attempts to send the
    /// message via the `udp_sender::switch_log` function.
    ///
    /// # Arguments
    ///
    /// * `record` - The log record to process.
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let message = record.args().to_string();
        let mut found = false;

        // Iterate through each tag pattern and send the log message accordingly.
        for pattern in &self.tag_writers {
            if message.contains(pattern) {
                // Send the log message to the corresponding UDP endpoint.
                udp_sender::switch_log(pattern, &message);
                found = true;
            }
        }

        // If no tag matched, log a warning about the unrecognized message.
        if !found {
            udp_sender::switch_log("-", &message);
        }
    }

    /// Flushes any buffered log records.
    ///
    /// This implementation is a no-op as the logging is done synchronously
    /// via UDP sending in the `log` method.
    fn flush(&self) {}
}
