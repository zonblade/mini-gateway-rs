use chrono::TimeZone;
use chrono::{DateTime, Duration, Utc};
use lazy_static::lazy_static;
use std::collections::{HashMap, VecDeque};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogStoreError {
    #[error("Failed to acquire read lock: {0}")]
    ReadLockError(String),

    #[error("Failed to acquire write lock: {0}")]
    WriteLockError(String),
}

// Enum for selecting bytes metric type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BytesMetric {
    BytesIn,
    BytesOut,
    BytesTotal,
}

pub struct TemporaryLog {
    pub date_time: chrono::DateTime<chrono::Utc>,
    pub status_code: i32,
    pub conn_id: String,
    pub conn_req: i8,   // 1 indicate connection in
    pub conn_res: i8,   // 1 indicate connection completed, 0 indicate connection dirupted
    pub bytes_in: i32,  // bytes in
    pub bytes_out: i32, // bytes out
}

pub struct LogCaptureTimeframe {
    pub date_time: chrono::DateTime<chrono::Utc>,
    pub value: i32,
    pub high: i32,
    pub low: i32,
}

impl Clone for TemporaryLog {
    fn clone(&self) -> Self {
        Self {
            date_time: self.date_time,
            status_code: self.status_code,
            conn_id: self.conn_id.clone(),
            conn_req: self.conn_req,
            conn_res: self.conn_res,
            bytes_in: self.bytes_in,
            bytes_out: self.bytes_out,
        }
    }
}

struct LogStore {
    logs: VecDeque<TemporaryLog>,
}

impl LogStore {
    fn new() -> Self {
        Self {
            logs: VecDeque::new(),
        }
    }

    fn append_data(&mut self, log: TemporaryLog) {
        // Check if adding this log would make the timespan exceed 35 minutes
        if !self.logs.is_empty() {
            if let Some(oldest_log) = self.logs.front() {
                let oldest_log_time = oldest_log.date_time;
                let time_diff = log
                    .date_time
                    .signed_duration_since(oldest_log_time)
                    .num_minutes();

                if time_diff > 35 {
                    // Remove the oldest 5 minutes worth of logs to relax storage
                    let cutoff = oldest_log_time + Duration::minutes(5);
                    while !self.logs.is_empty() {
                        if let Some(front_log) = self.logs.front() {
                            if front_log.date_time < cutoff {
                                self.logs.pop_front();
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        // Add the new log
        self.logs.push_back(log);
    }

    fn get_data_time_frame(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<LogCaptureTimeframe> {
        let mut result = Vec::new();

        if self.logs.is_empty() {
            return result;
        }

        // Group logs by minute for aggregation
        let mut minute_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();

        for log in self
            .logs
            .iter()
            .filter(|log| log.date_time >= start && log.date_time <= end)
        {
            // Use minute timestamp as key for grouping
            let minute_ts = log.date_time.timestamp() / 60;
            minute_groups.entry(minute_ts).or_default().push(log);
        }

        // Create timeframe entries for each minute
        for (minute_ts, logs) in minute_groups {
            let minute_datetime = Utc
                .timestamp_opt(minute_ts * 60, 0)
                .single()
                .unwrap_or(Utc::now());

            let count = logs.len() as i32;
            let max_bytes = logs
                .iter()
                .map(|log| log.bytes_in + log.bytes_out)
                .max()
                .unwrap_or(0);
            let min_bytes = logs
                .iter()
                .map(|log| log.bytes_in + log.bytes_out)
                .min()
                .unwrap_or(0);

            result.push(LogCaptureTimeframe {
                date_time: minute_datetime,
                value: count,    // Number of logs in this minute
                high: max_bytes, // Highest byte count in this minute
                low: min_bytes,  // Lowest byte count in this minute
            });
        }

        result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
        result
    }

    fn get_data_time_frame_by_status_code(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        status_filter: i32,
    ) -> Vec<LogCaptureTimeframe> {
        let mut result = Vec::new();

        if self.logs.is_empty() {
            return result;
        }

        // Filter logs by time range and status code, then group by minute
        let mut minute_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();

        for log in self.logs.iter().filter(|log| {
            log.date_time >= start && log.date_time <= end && log.status_code == status_filter
        }) {
            // Use minute timestamp as key for grouping
            let minute_ts = log.date_time.timestamp() / 60;
            minute_groups.entry(minute_ts).or_default().push(log);
        }

        // Create timeframe entries for each minute
        for (minute_ts, logs) in minute_groups {
            let minute_datetime = Utc
                .timestamp_opt(minute_ts * 60, 0)
                .single()
                .unwrap_or(Utc::now());

            let count = logs.len() as i32;
            let max_response_time = logs
                .iter()
                .map(|log| {
                    // Approximate response time from bytes (just as an example metric)
                    log.bytes_in + log.bytes_out
                })
                .max()
                .unwrap_or(0);

            let min_response_time = logs
                .iter()
                .map(|log| log.bytes_in + log.bytes_out)
                .min()
                .unwrap_or(0);

            result.push(LogCaptureTimeframe {
                date_time: minute_datetime,
                value: count,            // Number of logs with this status code
                high: max_response_time, // Highest response size
                low: min_response_time,  // Lowest response size
            });
        }

        result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
        result
    }

    fn get_data_time_frame_by_conn_stall(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<LogCaptureTimeframe> {
        let mut result = Vec::new();

        if self.logs.is_empty() {
            return result;
        }

        // Filter logs by time range and stalled connections, then group by minute
        let mut minute_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();

        for log in self.logs.iter().filter(|log| {
            log.date_time >= start && log.date_time <= end && log.conn_req == 1 && log.conn_res == 0
        }) {
            // Use minute timestamp as key for grouping
            let minute_ts = log.date_time.timestamp() / 60;
            minute_groups.entry(minute_ts).or_default().push(log);
        }

        // Create timeframe entries for each minute
        for (minute_ts, logs) in minute_groups {
            let minute_datetime = Utc
                .timestamp_opt(minute_ts * 60, 0)
                .single()
                .unwrap_or(Utc::now());

            let stall_count = logs.len() as i32;

            // Find max time of any stalled connection in this minute (as proxy for worst stall)
            let max_stall_time = if logs.len() > 1 {
                logs.iter()
                    .map(|log| log.date_time.timestamp())
                    .max()
                    .unwrap_or(0) as i32
            } else {
                0
            };

            // Find min time of any stalled connection in this minute
            let min_stall_time = if logs.len() > 1 {
                logs.iter()
                    .map(|log| log.date_time.timestamp())
                    .min()
                    .unwrap_or(0) as i32
            } else {
                0
            };

            result.push(LogCaptureTimeframe {
                date_time: minute_datetime,
                value: stall_count,   // Number of stalled connections
                high: max_stall_time, // Latest stall in this minute
                low: min_stall_time,  // Earliest stall in this minute
            });
        }

        result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
        result
    }

    fn get_bytes_io_frame(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        metric: BytesMetric,
    ) -> Vec<LogCaptureTimeframe> {
        let mut result = Vec::new();

        if self.logs.is_empty() {
            return result;
        }

        // Group logs by minute for aggregation
        let mut minute_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();

        for log in self
            .logs
            .iter()
            .filter(|log| log.date_time >= start && log.date_time <= end)
        {
            // Use minute timestamp as key for grouping
            let minute_ts = log.date_time.timestamp() / 60;
            minute_groups.entry(minute_ts).or_default().push(log);
        }

        // Create timeframe entries for each minute
        for (minute_ts, logs) in minute_groups {
            let minute_datetime = Utc
                .timestamp_opt(minute_ts * 60, 0)
                .single()
                .unwrap_or(Utc::now());

            // Function to get the appropriate bytes value based on the metric
            let get_bytes = |log: &&TemporaryLog| -> i32 {
                match metric {
                    BytesMetric::BytesIn => log.bytes_in,
                    BytesMetric::BytesOut => log.bytes_out,
                    BytesMetric::BytesTotal => log.bytes_in + log.bytes_out,
                }
            };

            // Calculate metrics based on the selected byte type
            let total_bytes: i32 = logs.iter().map(get_bytes).sum();
            let avg_bytes = if !logs.is_empty() {
                total_bytes / logs.len() as i32
            } else {
                0
            };

            // Find max and min bytes
            let max_bytes = logs.iter().map(get_bytes).max().unwrap_or(0);
            let min_bytes = logs.iter().map(get_bytes).min().unwrap_or(0);

            result.push(LogCaptureTimeframe {
                date_time: minute_datetime,
                value: avg_bytes, // Average bytes per request in this minute
                high: max_bytes,  // Highest byte count in this minute
                low: min_bytes,   // Lowest byte count in this minute
            });
        }

        result.sort_by(|a, b| a.date_time.cmp(&b.date_time));
        result
    }
}

// Create a global instance of the LogStore
lazy_static! {
    static ref LOG_STORE_PROXY: RwLock<LogStore> = RwLock::new(LogStore::new());
    static ref LOG_STORE_GATEWAY: RwLock<LogStore> = RwLock::new(LogStore::new());
}

pub mod tlog_proxy {

    use super::*;

    // Helper functions to get read/write locks with proper error handling
    fn get_read_lock_proxy<'a>() -> Result<RwLockReadGuard<'a, LogStore>, LogStoreError> {
        LOG_STORE_PROXY
            .read()
            .map_err(|e| LogStoreError::ReadLockError(format!("{}", e)))
    }

    fn get_write_lock_proxy<'a>() -> Result<RwLockWriteGuard<'a, LogStore>, LogStoreError> {
        LOG_STORE_PROXY
            .write()
            .map_err(|e| LogStoreError::WriteLockError(format!("{}", e)))
    }

    // Public functions to interact with the global LogStore
    pub fn append_data(log: TemporaryLog) -> Result<(), LogStoreError> {
        let mut store = get_write_lock_proxy()?;
        store.append_data(log);
        Ok(())
    }

    pub fn get_data_time_frame(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let store = get_read_lock_proxy()?;
        Ok(store.get_data_time_frame(start, end))
    }

    pub fn get_data_time_frame_by_status_code(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        status_filter: i32,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let store = get_read_lock_proxy()?;
        Ok(store.get_data_time_frame_by_status_code(start, end, status_filter))
    }

    pub fn get_data_time_frame_by_conn_stall(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let store = get_read_lock_proxy()?;
        Ok(store.get_data_time_frame_by_conn_stall(start, end))
    }

    pub fn get_bytes_io_frame(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        metric: BytesMetric,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let store = get_read_lock_proxy()?;
        Ok(store.get_bytes_io_frame(start, end, metric))
    }
}

pub mod tlog_gateway {

    use super::*;

    // Helper functions to get read/write locks with proper error handling
    fn get_read_lock_gateway<'a>() -> Result<RwLockReadGuard<'a, LogStore>, LogStoreError> {
        LOG_STORE_GATEWAY
            .read()
            .map_err(|e| LogStoreError::ReadLockError(format!("{}", e)))
    }

    fn get_write_lock_gateway<'a>() -> Result<RwLockWriteGuard<'a, LogStore>, LogStoreError> {
        LOG_STORE_GATEWAY
            .write()
            .map_err(|e| LogStoreError::WriteLockError(format!("{}", e)))
    }

    // Public functions to interact with the global LogStore
    pub fn append_data(log: TemporaryLog) -> Result<(), LogStoreError> {
        let mut store = get_write_lock_gateway()?;
        store.append_data(log);
        Ok(())
    }

    pub fn get_data_time_frame(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let store = get_read_lock_gateway()?;
        Ok(store.get_data_time_frame(start, end))
    }

    pub fn get_data_time_frame_by_status_code(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        status_filter: i32,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let store = get_read_lock_gateway()?;
        Ok(store.get_data_time_frame_by_status_code(start, end, status_filter))
    }

    pub fn get_data_time_frame_by_conn_stall(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let store = get_read_lock_gateway()?;
        Ok(store.get_data_time_frame_by_conn_stall(start, end))
    }

    pub fn get_bytes_io_frame(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        metric: BytesMetric,
    ) -> Result<Vec<LogCaptureTimeframe>, LogStoreError> {
        let store = get_read_lock_gateway()?;
        Ok(store.get_bytes_io_frame(start, end, metric))
    }
}
