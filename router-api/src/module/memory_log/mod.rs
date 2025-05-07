// -- lib.rs --
// A raw implementation of shared memory in Rust using direct system calls
pub mod spawner;

use std::ffi::CString;
use std::io::{self, Error, ErrorKind};
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

// Constants for shared memory
pub const MAX_MEMORY_SIZE: usize = 50 * 1024 * 1024; // 50MB max memory (for larger buffer)
const ENTRY_MAX_SIZE: usize = 4096; // Maximum 4KB per entry
const SHM_METADATA_SIZE: usize = 2048; // Space for metadata at the beginning (2KB)
const PROXY_LOGGER_NAME: &str = "/gwrs-proxy";
const GATEWAY_LOGGER_NAME: &str = "/gwrs-gateway";

// Control structure at the beginning of shared memory
#[repr(C, align(64))]
pub struct QueueControl {
    // Mutex for synchronization
    lock: AtomicU32,
    // Queue state
    write_index: AtomicUsize,
    read_index: AtomicUsize,
    count: AtomicUsize,
    capacity: AtomicUsize,
    // Slots for future metadata
    _reserved: [u8; 2048], // Increased to 2KB for future expansion
}

// A simple mutex implementation using an atomic
impl QueueControl {
    pub fn new(capacity: usize) -> Self {
        Self {
            lock: AtomicU32::new(0),
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
            count: AtomicUsize::new(0),
            capacity: AtomicUsize::new(capacity),
            _reserved: [0; 2048],
        }
    }

    pub fn lock(&self) {
        // Simple spin lock
        while self.lock.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_err() {
            std::hint::spin_loop();
        }
    }

    pub fn unlock(&self) {
        self.lock.store(0, Ordering::Release);
    }
    pub fn dequeue_item(&self, read_idx: usize, capacity: usize) {
        // Update read index with Release ordering
        self.read_index.store((read_idx + 1) % capacity, Ordering::Release);
        // Update count with Release ordering
        self.count.fetch_sub(1, Ordering::Release);
    }
}

// Consumer side
pub struct SharedMemoryConsumer {
    ptr: *mut u8,
    size: usize,
    control: *mut QueueControl,
    data_start: *mut u8,
    shm_fd: i32,
    shm_name: CString,
}

// Implementing consumer
impl SharedMemoryConsumer {
    // Open existing shared memory
    pub fn open(name: &str, expected_size: usize) -> io::Result<Self> {
        // Create a C-style string for the name
        let c_name = CString::new(name)
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid name"))?;

        // Open shared memory object
        let fd = unsafe {
            libc::shm_open(
                c_name.as_ptr(),
                libc::O_RDWR,  // We need write access for the control structure
                0o600,
            )
        };

        if fd < 0 {
            return Err(Error::last_os_error());
        }

        // Map memory into our address space
        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                expected_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            let err = Error::last_os_error();
            unsafe { libc::close(fd) };
            return Err(err);
        }

        let control_ptr = ptr as *mut QueueControl;
        let data_start = unsafe { (ptr as *mut u8).add(SHM_METADATA_SIZE) };

        Ok(SharedMemoryConsumer {
            ptr: ptr as *mut u8,
            size: expected_size,
            control: control_ptr,
            data_start,
            shm_fd: fd,
            shm_name: c_name,
        })
    }

    // Dequeue data from shared memory
    pub fn dequeue(&self) -> io::Result<Option<Vec<u8>>> {
        unsafe {
            // Lock the queue
            (*self.control).lock();
            
            // Use explicit Acquire ordering for cross-process visibility
            let count = (*self.control).count.load(Ordering::Acquire);
            if count == 0 {
                (*self.control).unlock();
                return Ok(None);
            }
            
            // Get current read position with explicit Acquire ordering
            let read_idx = (*self.control).read_index.load(Ordering::Acquire);
            let capacity = (*self.control).capacity.load(Ordering::Acquire);

            // Calculate offset in buffer
            let offset = read_idx * ENTRY_MAX_SIZE;
            
            // Get pointer to position
            let entry_ptr = self.data_start.add(offset);
            
            // Read entry size first
            let entry_size = *(entry_ptr as *const usize);
            
            // Read the actual data
            let mut data = vec![0u8; entry_size];
            ptr::copy_nonoverlapping(
                entry_ptr.add(mem::size_of::<usize>()),
                data.as_mut_ptr(),
                entry_size,
            );
            
            // Update read index
            (*self.control).read_index.store(
                (read_idx + 1) % capacity,
                Ordering::Relaxed,
            );
            
            // Update count
            (*self.control).count.fetch_sub(1, Ordering::Relaxed);
            
            // Use the new helper method to update indices
            (*self.control).dequeue_item(read_idx, capacity);
            
            // Unlock
            (*self.control).unlock();
            
            Ok(Some(data))
        }
    }

    // Dequeue with timeout - for controlled consumption
    pub fn dequeue_with_timeout(&self, timeout_ms: u64) -> io::Result<Option<Vec<u8>>> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        
        while start.elapsed() < timeout {
            match self.dequeue()? {
                Some(data) => return Ok(Some(data)),
                None => std::thread::sleep(std::time::Duration::from_millis(10)),
            }
        }
        
        Ok(None)
    }

    // Get number of items in queue
    pub fn queue_size(&self) -> usize {
        unsafe { (*self.control).count.load(Ordering::Relaxed) }
    }
    
    // Get maximum capacity of queue
    pub fn capacity(&self) -> usize {
        unsafe { (*self.control).capacity.load(Ordering::Relaxed) }
    }
    
    // Clean up shared memory (call this when done with it)
    pub fn cleanup(&self) -> io::Result<()> {
        let result = unsafe { libc::shm_unlink(self.shm_name.as_ptr()) };
        if result < 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
}

impl Drop for SharedMemoryConsumer {
    fn drop(&mut self) {
        unsafe {
            // Unmap memory
            libc::munmap(self.ptr as *mut libc::c_void, self.size);
            // Close file descriptor
            libc::close(self.shm_fd);
            // Note: We don't unlink here unless explicitly requested
        }
    }
}

// -- For Logger Implementation --

#[repr(C)]
pub struct LogEntry {
    timestamp: u64,
    level: u8,
    message_len: u32,
    // Message follows immediately after header
}

// Log consumer implementation
pub struct LogConsumer {
    shm: SharedMemoryConsumer,
}

impl LogConsumer {
    pub fn new(name: &str, size: usize) -> io::Result<Self> {
        let shm = SharedMemoryConsumer::open(name, size)?;
        Ok(LogConsumer { shm })
    }
    
    pub fn get_next_log(&self) -> io::Result<Option<(u64, u8, String)>> {
        match self.shm.dequeue()? {
            Some(buffer) => {
                // Parse the header
                if buffer.len() < mem::size_of::<LogEntry>() {
                    return Err(Error::new(ErrorKind::InvalidData, "Invalid log entry"));
                }
                
                unsafe {
                    let entry = ptr::read(buffer.as_ptr() as *const LogEntry);
                    
                    // Make sure message length is valid
                    let expected_len = mem::size_of::<LogEntry>() + entry.message_len as usize;
                    if buffer.len() != expected_len {
                        return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
                    }
                    
                    // Extract message
                    let message_start = mem::size_of::<LogEntry>();
                    let message_bytes = &buffer[message_start..];
                    
                    // Convert to string
                    let message = String::from_utf8_lossy(message_bytes).to_string();
                    
                    Ok(Some((entry.timestamp, entry.level, message)))
                }
            },
            None => Ok(None),
        }
    }
    
    pub fn get_log_with_timeout(&self, timeout_ms: u64) -> io::Result<Option<(u64, u8, String)>> {
        match self.shm.dequeue_with_timeout(timeout_ms)? {
            Some(buffer) => {
                // Parse the header
                if buffer.len() < mem::size_of::<LogEntry>() {
                    return Err(Error::new(ErrorKind::InvalidData, "Invalid log entry"));
                }
                
                unsafe {
                    let entry = ptr::read(buffer.as_ptr() as *const LogEntry);
                    
                    // Make sure message length is valid
                    let expected_len = mem::size_of::<LogEntry>() + entry.message_len as usize;
                    if buffer.len() != expected_len {
                        return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
                    }
                    
                    // Extract message
                    let message_start = mem::size_of::<LogEntry>();
                    let message_bytes = &buffer[message_start..];
                    
                    // Convert to string
                    let message = String::from_utf8_lossy(message_bytes).to_string();
                    
                    Ok(Some((entry.timestamp, entry.level, message)))
                }
            },
            None => Ok(None),
        }
    }
    
    pub fn queue_size(&self) -> usize {
        self.shm.queue_size()
    }
    
    pub fn capacity(&self) -> usize {
        self.shm.capacity()
    }
    
    pub fn cleanup(&self) -> io::Result<()> {
        self.shm.cleanup()
    }
}


// Example logger.rs
pub fn listen_proxy() {
    log::info!("Starting log consumer...");
    // Open shared memory
    let log_consumer = LogConsumer::new(PROXY_LOGGER_NAME, MAX_MEMORY_SIZE)
        .expect("Failed to open shared memory");
    
    // Process logs in batches with a controlled rate
    let mut batch = Vec::new();
    const BATCH_SIZE: usize = 100;
    
    log::info!("Starting log processing, queue size: {}", log_consumer.queue_size());
    
    let mut consecutive_empty = 0;
    let mut message_counter = 0;
    
    loop {
        // Add detailed queue monitoring
        let queue_size = log_consumer.queue_size();
                
        // Print milestone message every 100,000 records
            eprint!("Processed {} messages, Error {} message, queue size: {}\r", 
            message_counter, consecutive_empty, queue_size);
        // println!("Queue status: size={}, capacity={}", 
        //         queue_size, log_consumer.capacity());
        
        // Try to get a log entry with a timeout of 100ms
        match log_consumer.get_log_with_timeout(10) {
            Ok(Some((timestamp, level, message))) => {
                consecutive_empty = 0;
                message_counter += 1;
                
                // Process log entry with more detailed output
                let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
                    .unwrap_or(chrono::DateTime::UNIX_EPOCH);

                batch.push((datetime, level, message));
                
                // If batch is full, commit to database
                if batch.len() >= BATCH_SIZE {
                    // log::info!("Processing batch of {} logs", batch.len());
                    batch.clear();
                }
                
                // Don't sleep after successful reads to catch up quickly
            },
            Ok(None) => {
                consecutive_empty += 1;
                
                // Timeout occurred, process any remaining logs
                if !batch.is_empty() {
                    // log::info!("Processing partial batch of {} logs", batch.len());
                    for (datetime, level, message) in &batch {
                        // eprintln!("Batch item: {} - {}: {}", datetime, level, message);
                    }
                    batch.clear();
                }
                
                // Apply adaptive waiting strategy
                let wait_time = if consecutive_empty < 5 {
                    // Quick checks for first few empty iterations
                    10
                } else if consecutive_empty < 20 {
                    // Medium wait time after several empty checks
                    50
                } else {
                    // Longer wait time for prolonged empty periods
                    200
                };
                
                std::thread::sleep(std::time::Duration::from_millis(wait_time));
            },
            Err(e) => {
                // log::warn!("No log at the record: {}", e);
                consecutive_empty += 1;
                // println!("Error getting log: {}", e);
                // break;
                // sleep for a bit to avoid busy loop
                std::thread::sleep(std::time::Duration::from_millis(10));
            },
        }
    }
}
