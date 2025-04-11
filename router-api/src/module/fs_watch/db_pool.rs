use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;
use std::collections::{VecDeque, HashMap};
use chrono::{DateTime, Local};

use super::constants::DB_FLUSH_INTERVAL;

/// A log entry to be saved to the database
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub indicator: String,
    pub path: String,
    pub id: String,
    pub status: String,
    pub size: usize,
    pub comment: String,
}

/// A record storing the aggregated information for a specific ID
#[derive(Debug, Clone)]
pub struct IdSizeRecord {
    pub id: String,
    pub total_size: usize,
    pub last_timestamp: String,
    pub last_status: String,
    pub last_path: String,
}

/// Database connection pool for batching log entries
pub struct LogDbPool {
    buffer: Arc<RwLock<VecDeque<LogEntry>>>,
    id_records: Arc<RwLock<HashMap<String, IdSizeRecord>>>,
    last_flush_time: Arc<RwLock<Instant>>,
}

impl LogDbPool {
    /// Create a new database pool for log entries
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(RwLock::new(VecDeque::new())),
            id_records: Arc::new(RwLock::new(HashMap::new())),
            last_flush_time: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Add a log entry to the buffer
    pub fn add_log(&self, entry: LogEntry) {
        // First, update the id_records with this new entry
        {
            let mut records = self.id_records.write().unwrap();
            if let Some(record) = records.get_mut(&entry.id) {
                // Update existing record
                record.total_size += entry.size;
                record.last_timestamp = entry.timestamp.clone();
                record.last_status = entry.status.clone();
                record.last_path = entry.path.clone();
            } else {
                // Create new record
                records.insert(
                    entry.id.clone(),
                    IdSizeRecord {
                        id: entry.id.clone(),
                        total_size: entry.size,
                        last_timestamp: entry.timestamp.clone(),
                        last_status: entry.status.clone(),
                        last_path: entry.path.clone(),
                    },
                );
            }
        }
        
        // Write lock only when adding to buffer
        {
            let mut buffer = self.buffer.write().unwrap();
            buffer.push_back(entry);
        }
        
        // First check with a read lock if we need to flush
        let should_flush = {
            let last_flush = self.last_flush_time.read().unwrap();
            last_flush.elapsed() >= DB_FLUSH_INTERVAL
        };
        
        if should_flush {
            // Only take write lock when actually flushing
            let buffer_clone = self.buffer.clone();
            let id_records_clone = self.id_records.clone();
            
            // Update flush time with write lock
            {
                let mut last_flush = self.last_flush_time.write().unwrap();
                *last_flush = Instant::now();
            }
            
            // Spawn background thread to handle the flush
            thread::spawn(move || {
                // Take entries from buffer
                let entries = {
                    let mut buffer = buffer_clone.write().unwrap();
                    buffer.drain(..).collect::<Vec<LogEntry>>()
                };
                
                // Get a snapshot of the current ID records for saving
                let records_to_save = {
                    let records = id_records_clone.read().unwrap();
                    records.values().cloned().collect::<Vec<IdSizeRecord>>()
                };
                
                if !entries.is_empty() {
                    println!("Processed {} log entries, tracking {} unique IDs", 
                             entries.len(), records_to_save.len());
                    Self::save_id_records_to_database(&records_to_save);
                }
            });
        }
    }
    
    /// Start the periodic flush timer in a background thread
    pub fn start_periodic_flush(&self) -> thread::JoinHandle<()> {
        let buffer = self.buffer.clone();
        let id_records = self.id_records.clone();
        let last_flush_time = self.last_flush_time.clone();
        
        thread::spawn(move || {
            loop {
                thread::sleep(DB_FLUSH_INTERVAL);
                
                // First check with read lock if we need to flush
                let should_flush = {
                    let last_flush = last_flush_time.read().unwrap();
                    last_flush.elapsed() >= DB_FLUSH_INTERVAL
                };
                
                if should_flush {
                    // Only take write locks when needed
                    
                    // Take entries from buffer
                    let entries = {
                        let mut buffer_lock = buffer.write().unwrap();
                        buffer_lock.drain(..).collect::<Vec<LogEntry>>()
                    };
                    
                    // Get a snapshot of the current ID records for saving
                    let records_to_save = {
                        let records = id_records.read().unwrap();
                        records.values().cloned().collect::<Vec<IdSizeRecord>>()
                    };
                    
                    // Update last flush time
                    {
                        let mut last_flush = last_flush_time.write().unwrap();
                        *last_flush = Instant::now();
                    }
                    
                    if !records_to_save.is_empty() {
                        println!("Periodic flush: tracking {} unique IDs, processed {} new entries", 
                                 records_to_save.len(), entries.len());
                        LogDbPool::save_id_records_to_database(&records_to_save);
                    }
                }
            }
        })
    }
    
    /// Save ID records to database (placeholder implementation)
    fn save_id_records_to_database(records: &[IdSizeRecord]) {
        // TODO: Implement actual database saving logic
        // For now, just format and print the records
        for record in records {
            // Format using a simplified pattern focused on ID stats
            let formatted_record = format!(
                "ID Stats: {} | Total Size: {} bytes | Last update: {} | Status: {} | Path: {}", 
                record.id, 
                record.total_size,
                record.last_timestamp,
                record.last_status,
                record.last_path
            );
            
            println!("Saving to DB: {}", formatted_record);
            
            // Here you would upsert into your database
            // Example SQL-like operation:
            // db_connection.execute(
            //    "INSERT INTO id_stats (id, total_size, last_timestamp, last_status, last_path) 
            //     VALUES (?, ?, ?, ?, ?) 
            //     ON CONFLICT (id) DO UPDATE SET 
            //       total_size = ?, 
            //       last_timestamp = ?, 
            //       last_status = ?, 
            //       last_path = ?", 
            //    &[&record.id, &record.total_size, &record.last_timestamp, &record.last_status, &record.last_path,
            //      &record.total_size, &record.last_timestamp, &record.last_status, &record.last_path]);
        }
    }
    
    /// Legacy method - kept for backward compatibility but now routes to the ID-based implementation
    fn save_to_database(entries: &[LogEntry]) {
        println!("Using legacy save_to_database method - will be processed via ID tracking");
        
        // Convert entries to a map of ID records
        let mut id_records: HashMap<String, IdSizeRecord> = HashMap::new();
        
        for entry in entries {
            if let Some(record) = id_records.get_mut(&entry.id) {
                // Update existing record
                record.total_size += entry.size;
                record.last_timestamp = entry.timestamp.clone();
                record.last_status = entry.status.clone();
                record.last_path = entry.path.clone();
            } else {
                // Create new record
                id_records.insert(
                    entry.id.clone(),
                    IdSizeRecord {
                        id: entry.id.clone(),
                        total_size: entry.size,
                        last_timestamp: entry.timestamp.clone(),
                        last_status: entry.status.clone(),
                        last_path: entry.path.clone(),
                    },
                );
            }
        }
        
        // Save the aggregated ID records
        Self::save_id_records_to_database(&id_records.values().cloned().collect::<Vec<_>>());
    }
}

/// Global instance of the log database pool
pub fn get_log_db_pool() -> Arc<LogDbPool> {
    // This would typically use lazy_static! or once_cell to create a singleton
    // For simplicity, we'll create a new instance each time (not ideal for production)
    let pool = Arc::new(LogDbPool::new());
    
    // Start the periodic flush thread
    pool.start_periodic_flush();
    
    pool
}