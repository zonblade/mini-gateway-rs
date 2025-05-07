// -- lib.rs --
// A raw implementation of shared memory in Rust using direct system calls

use std::ffi::CString;
use std::io::{self, Error, ErrorKind};
use std::mem;
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

// Constants for shared memory
pub const MAX_MEMORY_SIZE: usize = 1024 * 1024 * 1024; // 1GB max memory
const ENTRY_MAX_SIZE: usize = 64 * 1024; // 64KB per entry
const SHM_METADATA_SIZE: usize = 4096; // Space for metadata at the beginning

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
    _reserved: [u8; 4000], // Padding to 4096 bytes
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
            _reserved: [0; 4000],
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
}

// Producer side
pub struct SharedMemoryProducer {
    ptr: *mut u8,
    size: usize,
    control: *mut QueueControl,
    data_start: *mut u8,
    shm_fd: i32,
    shm_name: CString,
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

// Implementing producer
impl SharedMemoryProducer {
    // Create a new shared memory region
    pub fn create(name: &str, total_size: usize) -> io::Result<Self> {
        // Calculate capacity based on total size and entry size
        let data_size = total_size.saturating_sub(SHM_METADATA_SIZE);
        let capacity = data_size / ENTRY_MAX_SIZE;
        
        if capacity == 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Memory size too small"));
        }

        // Create a C-style string for the name
        let c_name = CString::new(name)
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid name"))?;

        // Open shared memory object with shm_open
        let fd = unsafe {
            libc::shm_open(
                c_name.as_ptr(),
                libc::O_CREAT | libc::O_RDWR,
                0o600, // Read/write for owner only
            )
        };

        if fd < 0 {
            return Err(Error::last_os_error());
        }

        // Set size of shared memory
        if unsafe { libc::ftruncate(fd, total_size as libc::off_t) } < 0 {
            let err = Error::last_os_error();
            unsafe { libc::close(fd) };
            return Err(err);
        }

        // Map memory into our address space
        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                total_size,
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

        // Initialize control structure
        let control_ptr = ptr as *mut QueueControl;
        unsafe {
            ptr::write(control_ptr, QueueControl::new(capacity));
        }

        // Calculate data start pointer
        let data_start = unsafe { (ptr as *mut u8).add(SHM_METADATA_SIZE) };

        Ok(SharedMemoryProducer {
            ptr: ptr as *mut u8,
            size: total_size,
            control: control_ptr,
            data_start,
            shm_fd: fd,
            shm_name: c_name,
        })
    }

    // Enqueue data to the shared memory
    pub fn enqueue(&self, data: &[u8]) -> io::Result<()> {
        if data.len() > ENTRY_MAX_SIZE - mem::size_of::<usize>() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Data too large: {} bytes (max {})", 
                        data.len(), ENTRY_MAX_SIZE - mem::size_of::<usize>()),
            ));
        }

        unsafe {
            // Lock the queue
            (*self.control).lock();

            // Check if queue is full
            let count = (*self.control).count.load(Ordering::Relaxed);
            let capacity = (*self.control).capacity.load(Ordering::Relaxed);
            
            if count >= capacity {
                (*self.control).unlock();
                return Err(Error::new(ErrorKind::Other, "Queue is full"));
            }

            // Get current write position
            let write_idx = (*self.control).write_index.load(Ordering::Relaxed);
            
            // Calculate offset in buffer
            let offset = write_idx * ENTRY_MAX_SIZE;
            
            // Get pointer to position
            let entry_ptr = self.data_start.add(offset);
            
            // Write entry size first
            *(entry_ptr as *mut usize) = data.len();
            
            // Then write the actual data
            ptr::copy_nonoverlapping(
                data.as_ptr(),
                entry_ptr.add(mem::size_of::<usize>()),
                data.len(),
            );
            
            // Update write index
            (*self.control).write_index.store(
                (write_idx + 1) % capacity, 
                Ordering::Relaxed,
            );
            
            // Update count
            (*self.control).count.fetch_add(1, Ordering::Relaxed);
            
            // Unlock
            (*self.control).unlock();
        }

        Ok(())
    }

    // Get current number of items in queue
    pub fn queue_size(&self) -> usize {
        unsafe { (*self.control).count.load(Ordering::Relaxed) }
    }
    
    // Get maximum capacity of the queue
    pub fn capacity(&self) -> usize {
        unsafe { (*self.control).capacity.load(Ordering::Relaxed) }
    }
}

// Drop implementation to clean up resources
impl Drop for SharedMemoryProducer {
    fn drop(&mut self) {
        unsafe {
            // Unmap memory
            libc::munmap(self.ptr as *mut libc::c_void, self.size);
            // Close file descriptor
            libc::close(self.shm_fd);
            // Note: We don't unlink the memory here - consumer needs it
        }
    }
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

            // Check if queue is empty
            let count = (*self.control).count.load(Ordering::Relaxed);
            if count == 0 {
                (*self.control).unlock();
                return Ok(None);
            }

            // Get current read position
            let read_idx = (*self.control).read_index.load(Ordering::Relaxed);
            let capacity = (*self.control).capacity.load(Ordering::Relaxed);
            
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

// Log producer implementation
pub struct LogProducer {
    shm: SharedMemoryProducer,
}

impl LogProducer {
    pub fn new(name: &str, size: usize) -> io::Result<Self> {
        let shm = SharedMemoryProducer::create(name, size)?;
        Ok(LogProducer { shm })
    }
    
    pub fn log(&self, level: u8, message: &str) -> io::Result<()> {
        // Calculate total size needed
        let header_size = mem::size_of::<LogEntry>();
        let total_size = header_size + message.len();
        
        // Prepare the buffer
        let mut buffer = Vec::with_capacity(total_size);
        
        // Create and serialize the log entry header
        let entry = LogEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            level,
            message_len: message.len() as u32,
        };
        
        // Append header to buffer
        let entry_ptr = &entry as *const LogEntry as *const u8;
        let entry_bytes = unsafe { slice::from_raw_parts(entry_ptr, header_size) };
        buffer.extend_from_slice(entry_bytes);
        
        // Append message
        buffer.extend_from_slice(message.as_bytes());
        
        // Send to shared memory
        self.shm.enqueue(&buffer)
    }
    
    pub fn queue_size(&self) -> usize {
        self.shm.queue_size()
    }
    
    pub fn capacity(&self) -> usize {
        self.shm.capacity()
    }
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



// -- Examples --
#[cfg(test)]
mod tests {
    use super::*;
    
    // Example proxy.rs
    pub fn proxy_example() {
        // Create shared memory for logs
        let log_producer = LogProducer::new("/my-app-logs", 100 * 1024 * 1024)
            .expect("Failed to create shared memory");
        
        // Log some messages
        for i in 0..10 {
            log_producer.log(1, &format!("Request #{} processed", i))
                .expect("Failed to log message");
        }
        
        println!("Logged 10 messages, queue size: {}", log_producer.queue_size());
    }

    // Example logger.rs
    pub fn logger_example() {
        // Open shared memory
        let log_consumer = LogConsumer::new("/my-app-logs", 100 * 1024 * 1024)
            .expect("Failed to open shared memory");
        
        // Process logs in batches with a controlled rate
        let mut batch = Vec::new();
        const BATCH_SIZE: usize = 100;
        
        println!("Starting log processing, queue size: {}", log_consumer.queue_size());
        
        loop {
            // Try to get a log entry with a timeout of 100ms
            match log_consumer.get_log_with_timeout(100) {
                Ok(Some((timestamp, level, message))) => {
                    // Process log entry
                    let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
                        .unwrap_or(chrono::DateTime::UNIX_EPOCH);
                    
                    batch.push((datetime, level, message));
                    
                    // If batch is full, commit to database
                    if batch.len() >= BATCH_SIZE {
                        println!("Processing batch of {} logs", batch.len());
                        // Here you would save to database
                        batch.clear();
                    }
                },
                Ok(None) => {
                    // Timeout occurred, process any remaining logs
                    if !batch.is_empty() {
                        println!("Processing partial batch of {} logs", batch.len());
                        // Here you would save to database
                        batch.clear();
                    }
                    
                    // Sleep a bit to avoid busy waiting
                    std::thread::sleep(std::time::Duration::from_millis(50));
                },
                Err(e) => {
                    eprintln!("Error getting log: {}", e);
                    break;
                },
            }
            
            // Exit if requested (you'd implement a proper signal handler in a real application)
            if std::path::Path::new("/tmp/stop-logger").exists() {
                println!("Stop signal received");
                break;
            }
        }
        
        // Clean up shared memory when done
        log_consumer.cleanup().expect("Failed to clean up shared memory");
    }
}