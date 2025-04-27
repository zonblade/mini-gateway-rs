use crate::module::database::{get_connection_log, DatabaseError, DatabaseResult};
use crate::module::udp_log_fetcher::{LogMessage, LogMessageFormatted};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, SystemTime};
use crossbeam_channel::{bounded, Receiver, Sender};
use log;

/// Structure to hold log data before saving to database
struct LogEntry {
    id: String,
    connection_type: String,
    packet_size: usize,
    status: String,
    comment: String,
    timestamp: SystemTime,
}

/// A database pooling system for log messages
pub struct UdpLogDb {
    log_pool: Arc<Mutex<Vec<LogEntry>>>,
    running: Arc<RwLock<bool>>,
    db_flush_interval: Duration,
    table_name: Arc<String>,
}

impl UdpLogDb {
    /// Create a new UDP log database pooler with default 5-second flush interval and default table name "logs"
    pub fn new() -> Self {
        Self::with_params(Duration::from_secs(5), "logs")
    }

    /// Create a new UDP log database pooler with a custom flush interval and default table name "logs"
    pub fn with_flush_interval(interval: Duration) -> Self {
        Self::with_params(interval, "logs")
    }

    /// Create a new UDP log database pooler with a custom table name and default 5-second flush interval
    pub fn with_table_name(table_name: &str) -> Self {
        Self::with_params(Duration::from_secs(5), table_name)
    }

    /// Create a new UDP log database pooler with custom flush interval and table name
    pub fn with_params(interval: Duration, table_name: &str) -> Self {
        UdpLogDb {
            log_pool: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            db_flush_interval: interval,
            table_name: Arc::new(table_name.to_string()),
        }
    }

    /// Initialize the database, creating tables if they don't exist
    pub fn init_database(&self) -> DatabaseResult<()> {
        let db = get_connection_log()?;
        let table = self.table_name.as_str();

        // Create logs table if it doesn't exist
        // Using parameterized table name with format! since SQLite doesn't support binding for table names
        let create_table_sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id TEXT NOT NULL,
                connection_type TEXT NOT NULL,
                packet_size INTEGER,
                status TEXT,
                comment TEXT,
                timestamp TEXT,
                PRIMARY KEY (id, connection_type)
            )",
            table
        );

        db.execute(&create_table_sql, [])?;

        Ok(())
    }

    /// Add a log message to the pool
    pub fn add_log(&self, log_message: &LogMessage, formatted: &LogMessageFormatted) {
        let entry = LogEntry {
            id: formatted.id.clone(),
            connection_type: formatted.connection_type.clone(),
            packet_size: formatted.packet_size,
            status: formatted.status.clone(),
            comment: formatted.comment.clone(),
            timestamp: log_message.timestamp,
        };

        let mut pool = self.log_pool.lock().unwrap();
        pool.push(entry);
    }

    /// Process a raw log message into a formatted one
    pub fn process_message(&self, log_message: &LogMessage) -> Option<LogMessageFormatted> {
        // Parse the message format
        let parts: HashMap<String, String> = log_message.message
            .split(',')
            .filter_map(|part| {
                let kv: Vec<&str> = part.trim().splitn(2, ':').collect();
                if kv.len() == 2 {
                    Some((kv[0].trim().to_string(), kv[1].trim().to_string()))
                } else {
                    None
                }
            })
            .collect();

        // Extract the required fields
        let id = parts.get("ID").cloned().unwrap_or_default();
        let connection_type = parts.get("CONN").cloned().unwrap_or_default();
        
        // Parse packet size, default to 0 if not present or not parseable
        let packet_size = parts.get("SIZE")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
            
        // Get status and comment if available
        let status = parts.get("STATUS").cloned().unwrap_or_default();
        let comment = parts.get("COMMENT").cloned().unwrap_or_default();

        if !id.is_empty() {
            Some(LogMessageFormatted {
                id,
                connection_type,
                packet_size,
                status,
                comment,
            })
        } else {
            None
        }
    }

    /// Flush the log pool to the database
    fn flush_to_db(&self) -> Result<usize, DatabaseError> {
        let mut pool = self.log_pool.lock().unwrap();
        if pool.is_empty() {
            return Ok(0);
        }

        let db = get_connection_log()?;
        let mut count = 0;
        let table = self.table_name.as_str();

        // Use a transaction for better performance
        db.transaction(|conn| {
            for entry in pool.iter() {
                // Format timestamp as ISO 8601 string
                let timestamp = match entry.timestamp.duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(duration) => {
                        let secs = duration.as_secs();
                        let millis = duration.subsec_millis();
                        let dt = chrono::NaiveDateTime::from_timestamp_opt(secs as i64, millis * 1_000_000).unwrap();
                        dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
                    },
                    Err(_) => String::from("1970-01-01 00:00:00.000"),
                };
                
                // Use INSERT OR REPLACE to update existing entries or insert new ones
                // Using parameterized table name with format! since SQLite doesn't support binding for table names
                let insert_sql = format!(
                    "INSERT OR REPLACE INTO {} (id, connection_type, packet_size, status, comment, timestamp) 
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    table
                );
                
                conn.execute(
                    &insert_sql,
                    &[
                        &entry.id, 
                        &entry.connection_type, 
                        &entry.packet_size.to_string(), 
                        &entry.status, 
                        &entry.comment, 
                        &timestamp
                    ],
                )?;
                
                count += 1;
            }
            
            Ok(())
        })?;

        // Clear the pool after successful flush
        pool.clear();
        
        Ok(count)
    }

    /// Start the database pooling thread
    pub fn start(&self) -> Result<(), String> {
        if *self.running.read().unwrap() {
            return Err("Already running".to_string());
        }
        
        // Initialize database tables
        if let Err(e) = self.init_database() {
            return Err(format!("Failed to initialize database: {}", e));
        }
        
        // Set running flag
        let mut running = self.running.write().unwrap();
        *running = true;
        drop(running);
        
        // Clone Arc references for the thread
        let log_pool = Arc::clone(&self.log_pool);
        let running = Arc::clone(&self.running);
        let interval = self.db_flush_interval;
        let table_name = Arc::clone(&self.table_name);
        
        // Spawn the database flushing thread
        thread::spawn(move || {
            log::info!(
                "Starting database pooling thread with {} second flush interval for table '{}'", 
                interval.as_secs(),
                table_name
            );
            
            while *running.read().unwrap() {
                // Sleep for the flush interval
                thread::sleep(interval);
                
                // Check if we're still running after the sleep
                if !*running.read().unwrap() {
                    break;
                }
                
                // Create a reference to self for flush_to_db
                let db_pool = UdpLogDb {
                    log_pool: Arc::clone(&log_pool),
                    running: Arc::clone(&running),
                    db_flush_interval: interval,
                    table_name: Arc::clone(&table_name),
                };
                
                // Flush logs to database
                match db_pool.flush_to_db() {
                    Ok(count) => {
                        if count > 0 {
                            log::debug!("Flushed {} log entries to table '{}'", count, table_name);
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to flush logs to table '{}': {}", table_name, e);
                    }
                }
            }
            
            log::info!("Database pooling thread for table '{}' stopping", table_name);
        });
        
        Ok(())
    }
    
    /// Stop the database pooling thread
    pub fn stop(&self) -> Result<(), String> {
        let mut running = self.running.write().unwrap();
        if !*running {
            return Err("Not running".to_string());
        }
        
        *running = false;
        
        // Flush remaining logs before stopping
        match self.flush_to_db() {
            Ok(count) => {
                log::info!("Final flush: {} log entries written to table '{}'", count, self.table_name);
            },
            Err(e) => {
                log::error!("Failed to perform final flush to table '{}': {}", self.table_name, e);
            }
        }
        
        Ok(())
    }
}

/// Initialize a new UDP log database pooler with default settings and start it
pub fn init() -> UdpLogDb {
    let db_pool = UdpLogDb::new();
    
    match db_pool.start() {
        Ok(_) => log::info!("UDP log database pooling started successfully"),
        Err(e) => log::error!("Failed to start UDP log database pooling: {}", e),
    }
    
    db_pool
}

/// Initialize a new UDP log database pooler with a custom table name and start it
pub fn init_with_table(table_name: &str) -> UdpLogDb {
    let db_pool = UdpLogDb::with_table_name(table_name);
    
    match db_pool.start() {
        Ok(_) => log::info!("UDP log database pooling for table '{}' started successfully", table_name),
        Err(e) => log::error!("Failed to start UDP log database pooling for table '{}': {}", table_name, e),
    }
    
    db_pool
}