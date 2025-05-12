use chrono::TimeZone;
use chrono::{DateTime, Duration, Utc};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
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
#[allow(dead_code)]
pub enum BytesMetric {
    BytesIn,
    BytesOut,
    BytesTotal,
}

pub struct TemporaryLog {
    pub date_time: chrono::DateTime<chrono::Utc>,
    pub status_code: i32,
    pub peer: (String, String),
    pub conn_id: String,
    pub conn_type: String,
    pub conn_req: i8,   // 1 indicate connection in
    pub conn_res: i8,   // 1 indicate connection completed, 0 indicate connection dirupted
    pub bytes_in: i32,  // bytes in
    pub bytes_out: i32, // bytes out
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
            peer: self.peer.clone(),
            conn_id: self.conn_id.clone(),
            conn_type: self.conn_type.clone(),
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
            log::debug!("DEBUG: LogStore is empty, no logs to process");
            
            // Even with empty logs, create entries for every 15-second interval
            let start_ts = start.timestamp() / 15;
            let end_ts = end.timestamp() / 15;
            
            for interval_ts in start_ts..=end_ts {
                let interval_datetime = Utc
                    .timestamp_opt(interval_ts * 15, 0)
                    .single()
                    .unwrap_or(Utc::now());
                
                result.push(LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                });
            }
            
            return result;
        }
    
        // Group logs by 15-second intervals
        let mut time_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();
    
        for log in self
            .logs
            .iter()
            .filter(|log| log.date_time >= start && log.date_time <= end)
        {
            // Use 15-second timestamp as key for grouping
            let interval_ts = log.date_time.timestamp() / 15;
            time_groups.entry(interval_ts).or_default().push(log);
        }
    
        // Create a HashMap to store the computed results for each interval
        let mut interval_results = HashMap::new();
        
        // Process each time interval
        for (interval_ts, logs) in time_groups {
            let interval_datetime = Utc
                .timestamp_opt(interval_ts * 15, 0)
                .single()
                .unwrap_or(Utc::now());
    
            // Group logs by connection ID
            let mut connections: HashMap<String, (bool, bool)> = HashMap::new(); // (has_req, has_res)
            
            for log in &logs {
                let entry = connections.entry(log.conn_id.clone()).or_insert((false, false));
                
                match log.conn_type.as_str() {
                    "REQ" => entry.0 = true,
                    "RES" => entry.1 = true,
                    "DOWNSTREAM" => entry.0 = true,
                    "UPSTREAM"   => entry.1 = true,
                    _ => {}
                }
            }
            
            // Calculate success metrics
            let total_connections = connections.len() as i32;
            let successful_connections = connections
                .values()
                .filter(|(has_req, has_res)| *has_req && *has_res)
                .count() as i32;
                
            // Calculate success rate as percentage
            let success_rate = if total_connections > 0 {
                (successful_connections * 100) / total_connections
            } else {
                0
            };
    
            interval_results.insert(interval_ts, LogCaptureTimeframe {
                date_time: interval_datetime,
                value: success_rate,          // Success rate percentage
                high: total_connections,      // Total connections
                low: successful_connections,  // Successful connections
            });
        }
    
        // Generate entries for every 15-second interval in the requested range
        let start_ts = start.timestamp() / 15;
        let end_ts = end.timestamp() / 15;
        
        for interval_ts in start_ts..=end_ts {
            let timeframe = if let Some(existing) = interval_results.get(&interval_ts) {
                existing.clone() // Use existing data if available
            } else {
                // Create an entry with zeros if no data for this interval
                let interval_datetime = Utc
                    .timestamp_opt(interval_ts * 15, 0)
                    .single()
                    .unwrap_or(Utc::now());
                
                LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                }
            };
            
            result.push(timeframe);
        }
        
        log::debug!("DEBUG: Complete result with filled gaps contains {} entries", result.len());
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
            log::debug!("DEBUG: LogStore is empty, no logs to process");
            
            // Generate entries for every 15-second interval in the requested range
            let start_ts = start.timestamp() / 15;
            let end_ts = end.timestamp() / 15;
            
            for interval_ts in start_ts..=end_ts {
                let interval_datetime = Utc
                    .timestamp_opt(interval_ts * 15, 0)
                    .single()
                    .unwrap_or(Utc::now());
                
                result.push(LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                });
            }
            
            return result;
        }

        log::debug!("DEBUG: Status code analysis for code {} from {} to {}", 
                  status_filter, start, end);
        log::debug!("DEBUG: Total logs in store: {}", self.logs.len());
        
        // Count logs with matching status code
        let status_matching_logs = self.logs.iter()
            .filter(|log| log.status_code == status_filter && log.date_time >= start && log.date_time <= end)
            .count();
        
        log::debug!("DEBUG: Found {} logs with status code {}", status_matching_logs, status_filter);
        
        // Group all logs by connection ID first to find request-response pairs
        let mut conn_logs: HashMap<String, Vec<&TemporaryLog>> = HashMap::new();
        
        // Collect all logs in the time range
        for log in self.logs.iter().filter(|log| log.date_time >= start && log.date_time <= end) {
            conn_logs.entry(log.conn_id.clone()).or_default().push(log);
        }
        
        log::debug!("DEBUG: Found {} unique connections in time range", conn_logs.len());
        
        // Filter connections that have status code we're looking for
        let mut interval_groups: HashMap<i64, Vec<i64>> = HashMap::new();
        
        // Calculate response times for each connection
        for (conn_id, logs) in &conn_logs {
            // Sort logs by timestamp
            let mut logs_sorted = logs.clone();
            logs_sorted.sort_by_key(|log| log.date_time);
            
            // Find requests and their matching responses
            let mut req_time: Option<DateTime<Utc>> = None;
            let mut found_response = false;
            
            for log in logs_sorted {
                // For debugging, trace the connection's logs
                log::trace!("DEBUG: Conn {}: {:?} {} at {}", 
                          conn_id, log.conn_type, log.status_code, log.date_time);
                
                if log.conn_type == "REQ" || log.conn_type == "DOWNSTREAM" {
                    // Store request time
                    req_time = Some(log.date_time);
                    log::trace!("DEBUG: Found request for conn {}", conn_id);
                } else if (log.conn_type == "RES" || log.conn_type == "UPSTREAM") 
                          && log.status_code == status_filter 
                          && req_time.is_some() {
                    // Calculate response time in milliseconds
                    let response_time = log.date_time
                        .signed_duration_since(req_time.unwrap())
                        .num_milliseconds();
                    
                    log::trace!("DEBUG: Found matching response for conn {}, time: {}ms", 
                              conn_id, response_time);
                    found_response = true;
                    
                    // Group by 15-second interval instead of minute
                    let interval_ts = log.date_time.timestamp() / 15;
                    interval_groups.entry(interval_ts).or_default().push(response_time);
                    
                    // Reset for next request-response pair
                    req_time = None;
                }
            }
            
            if !found_response && logs.len() > 0 {
                // If we have logs but no matching response, log this fact
                log::trace!("DEBUG: Conn {} has no matching response with status {}", 
                          conn_id, status_filter);
            }
        }
        
        log::debug!("DEBUG: Found {} 15-second interval groups with response times", interval_groups.len());
        
        // Create a HashMap to store the computed results
        let mut interval_results = HashMap::new();
        
        // If we didn't find any response time data, look for ANY logs with the status code
        if interval_groups.is_empty() && status_matching_logs > 0 {
            log::debug!("DEBUG: No request-response pairs found, using simpler status code matching");
            
            // Fallback: Just group by 15-second interval any logs with matching status code
            let mut fallback_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();
            
            for log in self.logs.iter().filter(|log| {
                log.date_time >= start && log.date_time <= end && log.status_code == status_filter
            }) {
                let interval_ts = log.date_time.timestamp() / 15;
                fallback_groups.entry(interval_ts).or_default().push(log);
            }
            
            // Create timeframe entries from this simpler grouping
            for (interval_ts, logs) in fallback_groups {
                let interval_datetime = Utc
                    .timestamp_opt(interval_ts * 15, 0)
                    .single()
                    .unwrap_or(Utc::now());
                
                let count = logs.len() as i32;
                
                interval_results.insert(interval_ts, LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: count,            // Number of logs with this status code
                    high: 0,                 // No response time info available
                    low: 0,                  // No response time info available
                });
            }
        } else {
            // Create timeframe entries for each 15-second interval
            for (interval_ts, response_times) in interval_groups {
                let interval_datetime = Utc
                    .timestamp_opt(interval_ts * 15, 0)
                    .single()
                    .unwrap_or(Utc::now());

                let count = response_times.len() as i32;
                
                // Find max and min response times
                let max_response_time = response_times.iter().max().map(|&t| t as i32).unwrap_or(0);
                let min_response_time = response_times.iter().min().map(|&t| t as i32).unwrap_or(0);

                interval_results.insert(interval_ts, LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: count,            // Number of responses with this status code
                    high: max_response_time, // Highest response time in ms
                    low: min_response_time,  // Lowest response time in ms
                });
            }
        }

        // Generate entries for every 15-second interval in the requested range
        let start_ts = start.timestamp() / 15;
        let end_ts = end.timestamp() / 15;
        
        for interval_ts in start_ts..=end_ts {
            let timeframe = if let Some(existing) = interval_results.get(&interval_ts) {
                existing.clone() // Use existing data if available
            } else {
                // Create an entry with zeros if no data for this interval
                let interval_datetime = Utc
                    .timestamp_opt(interval_ts * 15, 0)
                    .single()
                    .unwrap_or(Utc::now());
                
                LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                }
            };
            
            result.push(timeframe);
        }

        log::debug!("DEBUG: Final result contains {} entries", result.len());
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
            log::debug!("DEBUG: LogStore is empty, no logs to process");
            
            // Generate entries for every 15-second interval in the requested range
            let start_ts = start.timestamp() / 15;
            let end_ts = end.timestamp() / 15;
            
            for interval_ts in start_ts..=end_ts {
                let interval_datetime = Utc
                    .timestamp_opt(interval_ts * 15, 0)
                    .single()
                    .unwrap_or(Utc::now());
                
                result.push(LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                });
            }
            
            return result;
        }

        // Filter logs by time range and stalled connections, then group by 15-second intervals
        let mut interval_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();

        for log in self.logs.iter().filter(|log| {
            log.date_time >= start && log.date_time <= end && log.conn_req == 1 && log.conn_res == 0
        }) {
            // Use 15-second timestamp as key for grouping
            let interval_ts = log.date_time.timestamp() / 15;
            interval_groups.entry(interval_ts).or_default().push(log);
        }

        // Create a HashMap to store the computed results
        let mut interval_results = HashMap::new();
        
        // Create timeframe entries for each 15-second interval
        for (interval_ts, logs) in interval_groups {
            let interval_datetime = Utc
                .timestamp_opt(interval_ts * 15, 0)
                .single()
                .unwrap_or(Utc::now());

            let stall_count = logs.len() as i32;

            // Find max time of any stalled connection in this interval
            let max_stall_time = if logs.len() > 1 {
                logs.iter()
                    .map(|log| log.date_time.timestamp())
                    .max()
                    .unwrap_or(0) as i32
            } else {
                0
            };

            // Find min time of any stalled connection in this interval
            let min_stall_time = if logs.len() > 1 {
                logs.iter()
                    .map(|log| log.date_time.timestamp())
                    .min()
                    .unwrap_or(0) as i32
            } else {
                0
            };

            interval_results.insert(interval_ts, LogCaptureTimeframe {
                date_time: interval_datetime,
                value: stall_count,   // Number of stalled connections
                high: max_stall_time, // Latest stall in this interval
                low: min_stall_time,  // Earliest stall in this interval
            });
        }

        // Generate entries for every 15-second interval in the requested range
        let start_ts = start.timestamp() / 15;
        let end_ts = end.timestamp() / 15;
        
        for interval_ts in start_ts..=end_ts {
            let timeframe = if let Some(existing) = interval_results.get(&interval_ts) {
                existing.clone() // Use existing data if available
            } else {
                // Create an entry with zeros if no data for this interval
                let interval_datetime = Utc
                    .timestamp_opt(interval_ts * 15, 0)
                    .single()
                    .unwrap_or(Utc::now());
                
                LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                }
            };
            
            result.push(timeframe);
        }

        log::debug!("DEBUG: Complete result with filled gaps contains {} entries", result.len());
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
            log::debug!("DEBUG: LogStore is empty, no logs to process");
            
            // Even with empty logs, we'll create entries for the entire time range
            // Generate entries for every 15-second interval in the requested range
            let start_ts = start.timestamp() / 15;
            let end_ts = end.timestamp() / 15;
            
            for interval_ts in start_ts..=end_ts {
                let interval_datetime = Utc
                    .timestamp_opt(interval_ts * 15, 0)
                    .single()
                    .unwrap_or(Utc::now());
                
                result.push(LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                });
            }
            
            return result;
        }

        // Group logs by 15-second intervals for aggregation
        let mut interval_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();

        for log in self
            .logs
            .iter()
            .filter(|log| log.date_time >= start && log.date_time <= end)
        {
            // Use 15-second timestamp as key for grouping
            let interval_ts = log.date_time.timestamp() / 15;
            interval_groups.entry(interval_ts).or_default().push(log);
        }

        log::debug!("DEBUG: Number of 15-second interval groups formed: {}", interval_groups.len());

        // Create a HashMap to store the computed results for each interval
        let mut interval_results = HashMap::new();

        // Process each 15-second interval that has logs
        for (interval_ts, logs) in interval_groups {
            let interval_datetime = Utc
                .timestamp_opt(interval_ts * 15, 0)
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

            // Calculate the overall 15-second average
            let total_bytes: i32 = logs.iter().map(get_bytes).sum();
            let avg_bytes_15sec = if !logs.is_empty() {
                total_bytes / logs.len() as i32
            } else {
                0
            };

            // Now break down into 1-second intervals to find high/low
            let mut second_groups: HashMap<i64, Vec<&TemporaryLog>> = HashMap::new();
            
            // Group logs by second within this 15-second interval
            for log in &logs {
                // Use exact timestamp as key (seconds precision)
                let second_ts = log.date_time.timestamp();
                second_groups.entry(second_ts).or_default().push(log);
            }
            
            // Calculate average for each 1-second interval
            let mut one_sec_averages: Vec<i32> = Vec::new();
            
            for (_second_ts, second_logs) in second_groups {
                if !second_logs.is_empty() {
                    let second_total: i32 = second_logs.iter().map(get_bytes).sum();
                    let second_avg = second_total / second_logs.len() as i32;
                    one_sec_averages.push(second_avg);
                }
            }
            
            // Find highest and lowest 1-second averages
            let highest_1sec_avg = one_sec_averages.iter().max().copied().unwrap_or(0);
            let lowest_1sec_avg = if one_sec_averages.is_empty() {
                0
            } else {
                *one_sec_averages.iter().min().unwrap_or(&0)
            };
            
            log::debug!(
                "DEBUG: Interval {}: 15-sec avg={}, highest 1-sec={}, lowest 1-sec={}, metric={:?}",
                interval_datetime, avg_bytes_15sec, highest_1sec_avg, lowest_1sec_avg, metric
            );

            // Store the result for this interval
            interval_results.insert(interval_ts, LogCaptureTimeframe {
                date_time: interval_datetime,
                value: avg_bytes_15sec,   // Average over the entire 15-second interval
                high: highest_1sec_avg,   // Highest 1-second average within this interval
                low: lowest_1sec_avg,     // Lowest 1-second average within this interval
            });
        }

        // Generate entries for every 15-second interval in the requested range
        let start_ts = start.timestamp() / 15;
        let end_ts = end.timestamp() / 15;
        
        for interval_ts in start_ts..=end_ts {
            let timeframe = if let Some(existing) = interval_results.get(&interval_ts) {
                existing.clone() // Use existing data if available
            } else {
                // Create an entry with zeros if no data for this interval
                let interval_datetime = Utc
                    .timestamp_opt(interval_ts * 15, 0)
                    .single()
                    .unwrap_or(Utc::now());
                
                LogCaptureTimeframe {
                    date_time: interval_datetime,
                    value: 0,
                    high: 0,
                    low: 0,
                }
            };
            
            result.push(timeframe);
        }

        log::debug!("DEBUG: Complete result with filled gaps contains {} entries", result.len());
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
