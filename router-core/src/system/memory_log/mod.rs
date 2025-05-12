// A raw implementation of shared memory in Rust using direct system calls
pub mod sender;

use std::ffi::CString;
use std::io::{self, Error, ErrorKind};
use std::mem;
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

// Constants for shared memory - adjusting for more efficient memory usage
pub const MAX_MEMORY_SIZE: usize = 50 * 1024 * 1024; // 50MB max memory
const ENTRY_MAX_SIZE: usize = 4096; // Reduce from 4KB to 2048 bytes since most logs are small
const SHM_METADATA_SIZE: usize = 2048; // Space for metadata at the beginning (2KB)
pub const LEVEL_TRACE: u8 = 0; // Most verbose, finest-grained information
pub const LEVEL_DEBUG: u8 = 1; // Detailed debugging information
pub const LEVEL_INFO: u8 = 2; // General informational messages
pub const LEVEL_WARN: u8 = 3; // Warning messages, potential issues
pub const LEVEL_ERROR: u8 = 4; // Error conditions, but application can continue

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
    // Overflow tracking
    overflow_count: AtomicUsize,
    // Slots for future metadata
    _reserved: [u8; 2048], // Increased to 2KB for future expansion
}

// Overflow handling policy
#[derive(Debug, Clone, Copy)]
pub enum OverflowPolicy {
    Block,     // Return error when queue is full (default)
    Overwrite, // Overwrite oldest entries when queue is full
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
            overflow_count: AtomicUsize::new(0),
            _reserved: [0; 2048],
        }
    }
    pub fn lock(&self) -> Result<(), io::Error> {
        // Add a timeout to prevent indefinite spinning
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(500); // 500ms max wait

        while self
            .lock
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // Check for timeout
            if start.elapsed() > timeout {
                return Err(Error::new(
                    ErrorKind::TimedOut,
                    "Failed to acquire lock within timeout",
                ));
            }
            std::hint::spin_loop();
        }

        Ok(()) // Successfully acquired lock
    }

    pub fn unlock(&self) {
        // Only unlock if currently locked
        let was_locked = self
            .lock
            .compare_exchange(1, 0, Ordering::Release, Ordering::Relaxed)
            .is_ok();
        if !was_locked {
            // Optionally log this situation for debugging
            // eprintln!("[-LO-] Attempted to unlock an already unlocked mutex");
        }
    }

    // Enhanced enqueue_item with bounds checking and corruption prevention
    pub fn enqueue_item(&self, write_idx: usize, capacity: usize) {
        // Update write index with Release ordering to make changes visible
        self.write_index
            .store((write_idx + 1) % capacity, Ordering::Release);

        // Check current count before updating to prevent overflow
        let current_count = self.count.load(Ordering::Acquire);

        // If count is suspiciously high, reset it to prevent overflow
        if current_count > capacity * 2 || current_count == usize::MAX - 1 {
            self.count.store(capacity, Ordering::Release);
        } else {
            // Update count with Release ordering - use saturating_add to prevent overflow
            let _ = self
                .count
                .fetch_update(Ordering::Release, Ordering::Relaxed, |c| {
                    // Only increment if not suspiciously large
                    if c < capacity * 2 {
                        Some(c + 1)
                    } else {
                        // Reset to capacity (full) if suspiciously large
                        Some(capacity)
                    }
                });
        }
    }

    // Add a method to safely decrement count (for consumer implementations)
    #[allow(dead_code)]
    pub fn dequeue_item(&self) {
        let current = self.count.load(Ordering::Acquire);
        if current > 0 {
            self.count.fetch_sub(1, Ordering::Release);
        }
    }

    // Enhanced validation with more diagnostics
    pub fn validate_and_fix(&self, capacity: usize) -> bool {
        let count = self.count.load(Ordering::Acquire);
        let current_capacity = self.capacity.load(Ordering::Acquire);
        let write_idx = self.write_index.load(Ordering::Acquire);
        let read_idx = self.read_index.load(Ordering::Acquire);

        // Perform more thorough validation
        let corrupted = count > capacity * 2
            || write_idx >= capacity
            || read_idx >= capacity
            || current_capacity != capacity
            || count == usize::MAX;

        if corrupted {
            self.force_reset(capacity);
            return true;
        }

        false
    }

    // Enhanced reset with better diagnostics
    pub fn force_reset(&self, capacity: usize) {
        // Capture original values for debugging
        let _old_count = self.count.load(Ordering::Acquire);
        let _old_write_idx = self.write_index.load(Ordering::Acquire);
        let _old_read_idx = self.read_index.load(Ordering::Acquire);

        // Reset all fields
        self.write_index.store(0, Ordering::Release);
        self.read_index.store(0, Ordering::Release);
        self.count.store(0, Ordering::Release);
        self.capacity.store(capacity, Ordering::Release);
        self.overflow_count.store(0, Ordering::Release);
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
    overflow_policy: OverflowPolicy,
}

// Implementing producer
impl SharedMemoryProducer {
    // Create a new shared memory region
    #[allow(dead_code)]
    pub fn create(name: &str, total_size: usize) -> io::Result<Self> {
        Self::create_with_options(name, total_size, false, OverflowPolicy::Block)
    }

    // Create with options for fresh start and overflow policy
    pub fn create_with_options(
        name: &str,
        total_size: usize,
        fresh_start: bool,
        overflow_policy: OverflowPolicy,
    ) -> io::Result<Self> {
        // Calculate capacity based on total size and entry size
        let data_size = total_size.saturating_sub(SHM_METADATA_SIZE);
        let capacity = data_size / ENTRY_MAX_SIZE;

        if capacity == 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Memory size too small"));
        }

        // Create a C-style string for the name
        let c_name = match CString::new(name) {
            Ok(name) => name,
            Err(_) => {
                return Err(Error::new(ErrorKind::InvalidInput, "Invalid name"));
            }
        };

        // If fresh start is requested, try to unlink existing memory first
        if fresh_start {
            unsafe {
                // Attempt to unlink with more visibility into errors
                let unlink_result = libc::shm_unlink(c_name.as_ptr());
                if unlink_result != 0 {
                    let err = io::Error::last_os_error();
                    // Only fail if it's not a "not found" error
                    // We shouldn't fail if the memory wasn't there to begin with
                    if err.kind() != ErrorKind::NotFound {
                        // Try one fallback approach - sometimes permission issues are transient
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        let retry_result = libc::shm_unlink(c_name.as_ptr());
                        if retry_result != 0 {
                            let retry_err = io::Error::last_os_error();
                            if retry_err.kind() != ErrorKind::NotFound {
                                // Only return error if it's serious and persistent
                                return Err(retry_err);
                            }
                        }
                    }
                }
            }
        }

        // Open shared memory object with shm_open
        let fd = unsafe {
            // Allow retries for shm_open in case of temporary failures
            let mut attempts = 0;
            let max_attempts = 3;
            let mut _last_err = Error::new(ErrorKind::Other, "Unknown error");

            loop {
                let result = libc::shm_open(
                    c_name.as_ptr(),
                    libc::O_CREAT | libc::O_RDWR,
                    0o600, // Read/write for owner only
                );

                if result >= 0 {
                    break result;
                }

                _last_err = io::Error::last_os_error();
                attempts += 1;

                if attempts >= max_attempts {
                    break -1; // Give up after max attempts
                }

                // Small delay between attempts
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        };

        if fd < 0 {
            let err = Error::last_os_error();
            return Err(err);
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

        // ENHANCED INITIALIZATION: Check if this is an existing shared memory region
        // If so, validate and possibly reset the control structure
        let control_ptr = ptr as *mut QueueControl;
        let mut control_initialized = false;

        unsafe {
            // Check if we can read the capacity field to determine if memory was already initialized
            let existing_capacity = (*control_ptr).capacity.load(Ordering::Acquire);

            // If capacity seems to exist and has a reasonable value
            if existing_capacity > 0 && existing_capacity <= capacity * 2 {
                control_initialized = true;

                // Validate structure and fix if needed
                let was_corrupted = (*control_ptr).validate_and_fix(capacity);

                if was_corrupted {
                } else if fresh_start {
                    // Even if not corrupted, if fresh_start was requested, reset the structure
                    (*control_ptr).force_reset(capacity);
                }
            } else if existing_capacity > 0 {
                control_initialized = true;
                // Reset control structure since capacity is suspicious
                (*control_ptr).force_reset(capacity);
            }
        }

        // If control structure wasn't initialized or was reset, initialize it now
        if !control_initialized {
            unsafe {
                ptr::write(control_ptr, QueueControl::new(capacity));
            }
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
            overflow_policy,
        })
    }

    // Add a method to create with specific capacity requirement
    pub fn create_with_capacity(
        name: &str,
        min_capacity: usize,
        fresh_start: bool,
        overflow_policy: OverflowPolicy,
    ) -> io::Result<Self> {
        // Calculate required size based on capacity
        let required_data_size = min_capacity * ENTRY_MAX_SIZE;
        let required_total_size = required_data_size + SHM_METADATA_SIZE;

        // Ensure we don't exceed reasonable limits (2GB for 32-bit compatibility)
        let max_reasonable_size = 2 * 1024 * 1024 * 1024;
        let total_size = if required_total_size > max_reasonable_size {
            max_reasonable_size
        } else {
            required_total_size
        };

        Self::create_with_options(name, total_size, fresh_start, overflow_policy)
    }

    // Add a method to verify entry length and ensure it fits
    fn verify_entry_size(data_len: usize) -> io::Result<()> {
        let max_allowed = ENTRY_MAX_SIZE - mem::size_of::<usize>();

        if data_len > max_allowed {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Data too large: {} bytes (max {})", data_len, max_allowed),
            ));
        }
        Ok(())
    }

    // Add more robust corruption detection
    pub fn check_and_reset_if_corrupted(&self) -> bool {
        unsafe {
            let capacity = (*self.control).capacity.load(Ordering::Acquire);
            let count = (*self.control).count.load(Ordering::Acquire);
            let write_idx = (*self.control).write_index.load(Ordering::Acquire);
            let read_idx = (*self.control).read_index.load(Ordering::Acquire);

            // Use more robust corruption checks
            let is_corrupted = count > capacity * 2
                || count == usize::MAX
                || count > 10_000_000
                || write_idx >= capacity
                || read_idx >= capacity;

            if is_corrupted {
                // Acquire lock before resetting
                let _ = (*self.control).lock();

                // Force reset the control structure
                (*self.control).force_reset(capacity);

                // Release lock
                (*self.control).unlock();

                return true;
            }
            false
        }
    }

    // Enhanced enqueue with entry size verification and more robust corruption handling
    pub fn enqueue(&self, data: &[u8]) -> io::Result<()> {
        // First verify that the data will fit in our reduced entry size
        Self::verify_entry_size(data.len())?;

        // Check for corruption before we even try to enqueue
        if self.check_and_reset_if_corrupted() {
            // Continue with the operation after reset
        }

        unsafe {
            match (*self.control).lock() {
                Ok(()) => {
                    // We got the lock, now use a defer-like pattern to ensure unlock
                    struct LockGuard<'a> {
                        control: &'a QueueControl,
                    }
                    
                    impl<'a> Drop for LockGuard<'a> {
                        fn drop(&mut self) {
                            self.control.unlock();
                        }
                    }
                    
                    // Create a guard that will automatically unlock when it goes out of scope
                    let _guard = LockGuard { control: &*self.control };
                    
                    // After getting the lock, run a full validation of the control structure
                    let count = (*self.control).count.load(Ordering::Acquire);
                    let capacity = (*self.control).capacity.load(Ordering::Acquire);
                    // let write_idx = (*self.control).write_index.load(Ordering::Acquire);

                    // Double-check for corruption after acquiring the lock
                    if count > capacity * 2 || count == usize::MAX {
                        // Reset the control structure
                        (*self.control).force_reset(capacity);
                    }

                    // Handle queue full situation based on policy
                    if count >= capacity {
                        match self.overflow_policy {
                            OverflowPolicy::Block => {
                                // Record overflow event - directly access the atomic field
                                (*self.control)
                                    .overflow_count
                                    .fetch_add(1, Ordering::Relaxed);

                                // Unlock and return error
                                (*self.control).unlock();
                                return Err(Error::new(ErrorKind::Other, "Queue is full"));
                            }
                            OverflowPolicy::Overwrite => {
                                // Record overflow event - directly access the atomic field
                                (*self.control)
                                    .overflow_count
                                    .fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }

                    // Get current write position with explicit Acquire ordering
                    let write_idx = (*self.control).write_index.load(Ordering::Acquire);

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

                    // Verify count is reasonable before updating
                    if count < capacity * 2 {
                        (*self.control).enqueue_item(write_idx, capacity);
                    } else {
                        // Force reset counters if they appear corrupted
                        (*self.control).force_reset(capacity);
                        // Try again with reset counters
                        let new_write_idx = (*self.control).write_index.load(Ordering::Acquire);
                        (*self.control).enqueue_item(new_write_idx, capacity);
                    }

                    // Unlock
                    (*self.control).unlock();
                },

                Err(e) => {
                    return Err(e);
                }
            }

            Ok(())
        }
    }

    // Get current number of items in queue
    #[allow(dead_code)]
    pub fn queue_size(&self) -> usize {
        unsafe {
            let count = (*self.control).count.load(Ordering::Acquire);
            count
        }
    }

    // Get maximum capacity of the queue
    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        unsafe { (*self.control).capacity.load(Ordering::Acquire) }
    }

    // Get overflow count - directly access the atomic field
    #[allow(dead_code)]
    pub fn overflow_count(&self) -> usize {
        unsafe { (*self.control).overflow_count.load(Ordering::Relaxed) }
    }

    // Clean up shared memory (call this when done with it)
    pub fn cleanup(&self) -> io::Result<()> {
        eprintln!("[-LO-] Cleaning up log producer... x");
        let result = unsafe { libc::shm_unlink(self.shm_name.as_ptr()) };
        if result < 0 {
            let err = Error::last_os_error();
            return Err(err);
        }
        Ok(())
    }
}

// Drop implementation to clean up resources
impl Drop for SharedMemoryProducer {
    fn drop(&mut self) {
        unsafe {
            // Unmap memory
            let unmap_result = libc::munmap(self.ptr as *mut libc::c_void, self.size);
            if unmap_result != 0 {}

            // Close file descriptor
            let close_result = libc::close(self.shm_fd);
            if close_result != 0 {}
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
    // Standard constructor - creates with default options
    #[allow(dead_code)]
    pub fn new(name: &str, size: usize) -> io::Result<Self> {
        let shm = SharedMemoryProducer::create(name, size)?;
        Ok(LogProducer { shm })
    }

    // Constructor with fresh start option - removes existing memory
    #[allow(dead_code)]
    pub fn new_with_fresh_start(name: &str, size: usize) -> io::Result<Self> {
        let shm = SharedMemoryProducer::create_with_options(
            name,
            size,
            true, // Fresh start
            OverflowPolicy::Block,
        )?;
        Ok(LogProducer { shm })
    }

    // Constructor with all options - enhanced with better error handling
    pub fn new_with_options(
        name: &str,
        size: usize,
        fresh_start: bool,
        overflow_policy: OverflowPolicy,
    ) -> io::Result<Self> {
        let actual_size = if size > 1024 * 1024 * 1024 {
            1024 * 1024 * 1024 // 1GB max
        } else {
            size
        };

        let shm = match SharedMemoryProducer::create_with_options(
            name,
            actual_size,
            fresh_start,
            overflow_policy,
        ) {
            Ok(producer) => producer,
            Err(e) => {
                if size > 100 * 1024 * 1024
                    && (e.kind() == ErrorKind::PermissionDenied
                        || e.kind() == ErrorKind::AddrNotAvailable
                        || e.kind() == ErrorKind::OutOfMemory)
                {
                    return Self::new_with_options(
                        name,
                        100 * 1024 * 1024,
                        fresh_start,
                        overflow_policy,
                    );
                }

                return Err(e);
            }
        };

        Ok(LogProducer { shm })
    }

    // New constructor with capacity requirement
    pub fn new_with_capacity(
        name: &str,
        min_capacity: usize,
        fresh_start: bool,
        overflow_policy: OverflowPolicy,
    ) -> io::Result<Self> {
        let actual_capacity = if min_capacity < 1_000 {
            1_000
        } else if min_capacity > 10_000_000 {
            10_000_000
        } else {
            min_capacity
        };

        let shm = match SharedMemoryProducer::create_with_capacity(
            name,
            actual_capacity,
            fresh_start,
            overflow_policy,
        ) {
            Ok(producer) => producer,
            Err(e) => {
                return Err(e);
            }
        };

        Ok(LogProducer { shm })
    }

    pub fn log(&self, level: u8, message: &str) -> io::Result<()> {
        let header_size = mem::size_of::<LogEntry>();
        let total_size_of_payload = header_size + message.len(); // Total size of LogEntry struct + message bytes

        if message.is_empty() {
            println!("Empty message received");
            return Ok(());
        }
        // Maximum size an entry can hold for its payload (LogEntry + message)
        // This is distinct from ENTRY_MAX_SIZE which includes the initial 'data_len' field written by SharedMemoryProducer.
        let max_payload_in_shm_entry = ENTRY_MAX_SIZE - mem::size_of::<usize>();

        if total_size_of_payload > max_payload_in_shm_entry {
            let suffix = "...";
            let suffix_len = suffix.len();

            // Calculate available space for the message text itself, considering header and suffix
            let available_for_message_text = max_payload_in_shm_entry
                .saturating_sub(header_size)
                .saturating_sub(suffix_len);

            let final_message_to_log: String;

            if available_for_message_text == 0 {
                // Not enough space for even one char + suffix. Try to log just the suffix if it fits.
                if header_size + suffix_len <= max_payload_in_shm_entry {
                    final_message_to_log = suffix.to_string();
                } else {
                    // Cannot even fit the suffix. This implies ENTRY_MAX_SIZE is extremely small or header is huge.
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        "Log entry too small for header and truncation suffix.",
                    ));
                }
            } else {
                // Ensure cut_at does not exceed message length and respects char boundaries.
                let mut cut_at = std::cmp::min(message.len(), available_for_message_text);
                while !message.is_char_boundary(cut_at) && cut_at > 0 {
                    cut_at -= 1;
                }

                if cut_at == 0 && !message.is_empty() {
                    // Not even a single character of the original message fits with the suffix.
                    // Log only "..." if it fits by itself.
                    if header_size + suffix_len <= max_payload_in_shm_entry {
                        final_message_to_log = suffix.to_string();
                    } else {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "Log entry too small for even one char and truncation suffix.",
                        ));
                    }
                } else {
                    final_message_to_log = format!("{}{}", &message[..cut_at], suffix);
                }
            }

            // Call self recursively with the truncated message.
            // This recursive call needs to be careful; the new total_size_of_payload must be <= max_payload_in_shm_entry.
            // The logic above should ensure final_message_to_log is short enough.
            return self.log(level, &final_message_to_log);
        }

        // Prepare the buffer
        let mut buffer = Vec::with_capacity(total_size_of_payload);

        // Create and serialize the log entry header
        let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(_) => {
                0 // Use 0 as fallback timestamp
            }
        };

        let entry = LogEntry {
            timestamp,
            level,
            message_len: message.len() as u32,
        };

        // Append header to buffer
        let entry_ptr = &entry as *const LogEntry as *const u8;
        let entry_bytes = unsafe { slice::from_raw_parts(entry_ptr, header_size) };
        buffer.extend_from_slice(entry_bytes);

        // Append message
        buffer.extend_from_slice(message.as_bytes());

        // Send to shared memory with better error reporting
        match self.shm.enqueue(&buffer) {
            Ok(_) => {
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    #[allow(dead_code)]
    pub fn queue_size(&self) -> usize {
        self.shm.queue_size()
    }

    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        self.shm.capacity()
    }

    #[allow(dead_code)]
    pub fn overflow_count(&self) -> usize {
        self.shm.overflow_count()
    }

    pub fn cleanup(&self) -> io::Result<()> {
        eprintln!("[-LO-] Cleaning up log producer...");
        match self.shm.cleanup() {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

// Default configuration for global logger
const PROXY_LOGGER_NAME: &str = "/gwrs-proxy";
const GATEWAY_LOGGER_NAME: &str = "/gwrs-gateway";

// Global logger instances
static mut GLOBAL_LOG_PROXY: Option<LogProducer> = None;
static mut GLOBAL_LOG_GATEWAY: Option<LogProducer> = None;

/// Gets or initializes the proxy logger instance
///
/// # Safety
/// Not thread-safe. Should only be called from a single thread.
#[allow(static_mut_refs)]
pub unsafe fn proxy_logger() -> io::Result<&'static LogProducer> {
    if GLOBAL_LOG_PROXY.is_none() {
        eprintln!("[-LO-] Initializing proxy logger...");
        // Request 10 million entries with smaller size
        let desired_capacity = 10_000_000; // 10 million entries

        // Create with capacity-based approach
        match LogProducer::new_with_capacity(
            PROXY_LOGGER_NAME,
            desired_capacity,
            true,                      // Force fresh start to clear memory
            OverflowPolicy::Overwrite, // Overwrite when full
        ) {
            Ok(logger) => {
                GLOBAL_LOG_PROXY = Some(logger);
            }
            Err(_) => {
                // Fall back to using default approach if capacity-based approach fails

                // Use a larger size for the shared memory (1GB instead of 100MB)
                let logger_size = MAX_MEMORY_SIZE; // Use 1GB for more capacity

                match LogProducer::new_with_options(
                    PROXY_LOGGER_NAME,
                    logger_size,
                    true,                      // Force fresh start to clear memory
                    OverflowPolicy::Overwrite, // Overwrite when full
                ) {
                    Ok(logger) => {
                        GLOBAL_LOG_PROXY = Some(logger);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
    }

    match &GLOBAL_LOG_PROXY {
        Some(logger) => Ok(logger),
        None => Err(Error::new(
            ErrorKind::Other,
            "Failed to initialize proxy logger",
        )),
    }
}

/// Gets or initializes the gateway logger instance
///
/// # Safety
/// Not thread-safe. Should only be called from a single thread.
#[allow(static_mut_refs)]
pub unsafe fn gateway_logger() -> io::Result<&'static LogProducer> {
    if GLOBAL_LOG_GATEWAY.is_none() {
        eprintln!("[-LO-] Initializing gateway logger...");
        // Request 10 million entries with smaller size
        let desired_capacity = 10_000_000; // 10 million entries

        // Create with capacity-based approach
        match LogProducer::new_with_capacity(
            GATEWAY_LOGGER_NAME,
            desired_capacity,
            true,                      // Force fresh start to clear memory
            OverflowPolicy::Overwrite, // Overwrite when full
        ) {
            Ok(logger) => {
                GLOBAL_LOG_GATEWAY = Some(logger);
            }
            Err(_) => {
                // Fall back to using default approach if capacity-based approach fails

                let logger_size = MAX_MEMORY_SIZE; // Use 1GB for more capacity

                match LogProducer::new_with_options(
                    GATEWAY_LOGGER_NAME,
                    logger_size,
                    true,                      // Force fresh start to clear memory
                    OverflowPolicy::Overwrite, // Overwrite when full
                ) {
                    Ok(logger) => {
                        GLOBAL_LOG_GATEWAY = Some(logger);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
    }

    match &GLOBAL_LOG_GATEWAY {
        Some(logger) => Ok(logger),
        None => Err(Error::new(
            ErrorKind::Other,
            "Failed to initialize gateway logger",
        )),
    }
}

/// Log a message using the proxy logger
///
/// # Safety
/// Not thread-safe. Should only be called from a single thread.
pub unsafe fn log_proxy(level: u8, message: &str) -> io::Result<()> {
    match proxy_logger() {
        Ok(logger) => match logger.log(level, message) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

/// Log a message using the gateway logger
///
/// # Safety
/// Not thread-safe. Should only be called from a single thread.
pub unsafe fn log_gateway(level: u8, message: &str) -> io::Result<()> {
    match gateway_logger() {
        Ok(logger) => match logger.log(level, message) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

#[allow(static_mut_refs)]
pub fn log_cleanup() -> io::Result<()> {
    let mut result = Ok(());

    unsafe {
        if let Some(logger) = GLOBAL_LOG_PROXY.take() {
            if let Err(e) = logger.cleanup() {
                result = Err(e);
            }
        }

        if let Some(logger) = GLOBAL_LOG_GATEWAY.take() {
            if let Err(e) = logger.cleanup() {
                if result.is_ok() {
                    result = Err(e);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example proxy.rs
    #[allow(dead_code)]
    pub fn proxy_example() {
        // Create shared memory for logs
        let log_producer = match LogProducer::new_with_options(
            "/my-app-logs",
            100 * 1024 * 1024,
            true,                      // Fresh start
            OverflowPolicy::Overwrite, // Overwrite when full
        ) {
            Ok(producer) => producer,
            Err(_) => {
                return;
            }
        };

        // Log some messages
        for i in 0..10 {
            if let Err(_) = log_producer.log(1, &format!("Request #{} processed", i)) {}
        }
    }

    #[test]
    pub fn test_producer() {
        // Create with fresh start and overwrite policy using capacity-based approach
        let log_producer = match LogProducer::new_with_capacity(
            PROXY_LOGGER_NAME,
            500_000,                   // Request 500,000 entries
            true,                      // Fresh start
            OverflowPolicy::Overwrite, // Overwrite when full
        ) {
            Ok(producer) => producer,
            Err(_) => {
                return;
            }
        };

        let mut counter = 0;
        loop {
            // Create a message with timestamp and counter
            let message = format!("[INFO] [PXY] Test message #{}", counter);

            // Log the message
            match log_producer.log(2, &message) {
                Ok(_) => {}
                Err(_) => {}
            }

            counter += 1;

            // Faster testing
            if counter % 100 == 0 {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }

            // Optional: exit after some number of messages for testing
            if counter >= 100000 {
                break;
            }
        }
    }
}
