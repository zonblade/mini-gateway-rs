use std::ffi::CString;
use std::io::{self, Error, ErrorKind};
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

// Architecture detection
#[cfg(target_arch = "x86_64")]
const ARCH_NAME: &str = "x86_64";
#[cfg(target_arch = "aarch64")]
const ARCH_NAME: &str = "aarch64";

// Constants for shared memory
pub const MAX_MEMORY_SIZE: usize = 50 * 1024 * 1024; // 50MB max memory (for larger buffer)
pub const ENTRY_MAX_SIZE: usize = 4096; // Maximum 4KB per entry
pub const SHM_METADATA_SIZE: usize = 2048; // Space for metadata at the beginning (2KB)
pub const PROXY_LOGGER_NAME: &str = "/gwrs-proxy";
pub const GATEWAY_LOGGER_NAME: &str = "/gwrs-gateway";

// Architecture-specific memory ordering helpers
#[inline(always)]
fn acquire_ordering() -> Ordering {
    // Both architectures support Acquire ordering efficiently
    Ordering::Acquire
}

#[inline(always)]
fn release_ordering() -> Ordering {
    // Both architectures support Release ordering efficiently
    Ordering::Release
}

#[inline(always)]
fn memory_fence_release() {
    // On ARM64, this compiles to DMB ISH instruction
    // On x86_64, this is often a no-op due to strong memory model
    std::sync::atomic::fence(Ordering::Release);
}

#[inline(always)]
fn memory_fence_acquire() {
    // On ARM64, this compiles to DMB ISH instruction
    // On x86_64, this is often a no-op due to strong memory model
    std::sync::atomic::fence(Ordering::Acquire);
}

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

// A simple mutex implementation using an atomic
impl QueueControl {
    #[allow(dead_code)]
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

        // Use Acquire ordering for the lock to ensure all subsequent reads
        // see values written before the lock was released
        while self
            .lock
            .compare_exchange_weak(0, 1, acquire_ordering(), Ordering::Relaxed)
            .is_err()
        {
            // Check for timeout
            if start.elapsed() > timeout {
                return Err(Error::new(
                    ErrorKind::TimedOut,
                    "Failed to acquire lock within timeout",
                ));
            }
            // This is architecture-aware and will use appropriate pause instruction
            std::hint::spin_loop();
        }

        // Memory fence to ensure lock acquisition is visible
        memory_fence_acquire();
        
        Ok(()) // Successfully acquired lock
    }

    pub fn unlock(&self) {
        // Memory fence before unlock to ensure all writes are visible
        memory_fence_release();
        
        // Use Release ordering to ensure all previous writes are visible
        // to the next thread that acquires the lock
        self.lock.store(0, release_ordering());
    }

    pub fn dequeue_item(&self, read_idx: usize, capacity: usize) {
        // Update read index with Release ordering
        self.read_index
            .store((read_idx + 1) % capacity, release_ordering());
        
        // Memory fence to ensure index update is visible before count update
        memory_fence_release();
        
        // Update count with Release ordering
        self.count.fetch_sub(1, release_ordering());
    }
}

// Consumer side
pub struct SharedMemoryConsumer {
    ptr: *mut u8,
    size: usize,
    control: *mut QueueControl,
    data_start: *mut u8,
    shm_fd: i32,
    _shm_name: CString,
}

// Safety: SharedMemoryConsumer operations are not thread-safe by default
// Users must ensure proper synchronization when sharing between threads
unsafe impl Send for SharedMemoryConsumer {}

// Implementing consumer
impl SharedMemoryConsumer {
    // Open existing shared memory
    pub fn open(name: &str, expected_size: usize) -> io::Result<Self> {
        // Log architecture for debugging
        eprintln!("[-LO-] Opening shared memory consumer on {} architecture", ARCH_NAME);
        
        // Create a C-style string for the name
        let c_name =
            CString::new(name).map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid name"))?;

        // Open shared memory object
        let fd = unsafe {
            let mut attempts = 0;
            let max_attempts = 3;
            
            loop {
                let result = libc::shm_open(
                    c_name.as_ptr(),
                    libc::O_RDWR, // We need write access for the control structure
                    0o600,
                );
                
                if result >= 0 {
                    break result;
                }
                
                attempts += 1;
                if attempts >= max_attempts {
                    break -1;
                }
                
                // Small delay between attempts
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
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

        // Verify the control structure looks valid
        unsafe {
            // Memory fence to ensure we see the latest values
            memory_fence_acquire();
            
            let capacity = (*control_ptr).capacity.load(acquire_ordering());
            if capacity == 0 {
                libc::munmap(ptr, expected_size);
                libc::close(fd);
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Shared memory appears uninitialized (capacity is 0)",
                ));
            }
        }

        Ok(SharedMemoryConsumer {
            ptr: ptr as *mut u8,
            size: expected_size,
            control: control_ptr,
            data_start,
            shm_fd: fd,
            _shm_name: c_name,
        })
    }

    // Dequeue data from shared memory
    pub fn dequeue(&self) -> io::Result<Option<Vec<u8>>> {
        unsafe {
            // Lock the queue
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

                    // Use explicit Acquire ordering for cross-process visibility
                    let count = (*self.control).count.load(acquire_ordering());
                    if count == 0 {
                        return Ok(None);
                    }

                    // Get current read position with explicit Acquire ordering
                    let read_idx = (*self.control).read_index.load(acquire_ordering());
                    let capacity = (*self.control).capacity.load(acquire_ordering());

                    // Safety check - if read index is out of bounds, something is wrong
                    if read_idx >= capacity {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Invalid read index: {} (capacity: {})", read_idx, capacity),
                        ));
                    }

                    // Calculate offset in buffer
                    let offset = read_idx * ENTRY_MAX_SIZE;

                    // Verify that offset is within bounds of allocated memory
                    if offset >= self.size - SHM_METADATA_SIZE {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "Buffer offset out of bounds: {} (max: {})",
                                offset,
                                self.size - SHM_METADATA_SIZE
                            ),
                        ));
                    }

                    // Get pointer to position
                    let entry_ptr = self.data_start.add(offset);

                    // Memory fence to ensure we see the latest data
                    memory_fence_acquire();

                    // Read entry size first
                    let entry_size = ptr::read(entry_ptr as *const usize);

                    // Check entry size is sensible
                    if entry_size == 0 || entry_size > ENTRY_MAX_SIZE - mem::size_of::<usize>() {
                        // Skip this entry by advancing read index
                        (*self.control).dequeue_item(read_idx, capacity);

                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "Invalid entry size: {} (max: {}), skipping entry",
                                entry_size,
                                ENTRY_MAX_SIZE - mem::size_of::<usize>()
                            ),
                        ));
                    }

                    // Memory fence before reading data to ensure size is read before data
                    memory_fence_acquire();

                    // Read the actual data
                    let mut data = vec![0u8; entry_size];
                    ptr::copy_nonoverlapping(
                        entry_ptr.add(mem::size_of::<usize>()),
                        data.as_mut_ptr(),
                        entry_size,
                    );

                    // Update read index and count
                    (*self.control).dequeue_item(read_idx, capacity);

                    // Note: Unlock happens automatically via LockGuard drop

                    Ok(Some(data))
                },
                Err(e) => Err(e),
            }
        }
    }

    // Dequeue with timeout - for controlled consumption
    pub fn dequeue_with_timeout(&self, timeout_ms: u64) -> io::Result<Option<Vec<u8>>> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            match self.dequeue() {
                Ok(Some(data)) => return Ok(Some(data)),
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),
                Err(e) => {
                    // Log the error if it's not a timeout
                    if e.kind() != ErrorKind::TimedOut {
                        eprintln!("[-LO-] Error in dequeue on {}: {}", ARCH_NAME, e);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    // Continue trying unless it's a fatal error
                    if e.kind() == ErrorKind::InvalidData {
                        return Err(e);
                    }
                }
            }
        }

        Ok(None)
    }

    // Get number of items in queue
    pub fn queue_size(&self) -> usize {
        unsafe { (*self.control).count.load(acquire_ordering()) }
    }

    // Get maximum capacity of queue
    pub fn capacity(&self) -> usize {
        unsafe { (*self.control).capacity.load(acquire_ordering()) }
    }

    // Get overflow count (if available)
    pub fn overflow_count(&self) -> usize {
        unsafe { (*self.control).overflow_count.load(Ordering::Relaxed) }
    }

    // Clean up shared memory resources but don't unlink (the producer/router-core owns the shared memory)
    pub fn cleanup(&self) -> io::Result<()> {
        eprintln!("[-LO-] Cleaning up consumer on {}...", ARCH_NAME);
        // Consumer should not call shm_unlink as it would remove the shared memory
        // that may still be in use by other processes.
        // Only unmap memory and close file descriptor in Drop implementation.
        Ok(())
    }
}

impl Drop for SharedMemoryConsumer {
    fn drop(&mut self) {
        unsafe {
            // Unmap memory
            let unmap_result = libc::munmap(self.ptr as *mut libc::c_void, self.size);
            if unmap_result != 0 {
                eprintln!("[-LO-] Failed to unmap memory: {}", Error::last_os_error());
            }
            
            // Close file descriptor
            let close_result = libc::close(self.shm_fd);
            if close_result != 0 {
                eprintln!("[-LO-] Failed to close file descriptor: {}", Error::last_os_error());
            }
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

// Safety: LogConsumer operations are not thread-safe by default
// Users must ensure proper synchronization when sharing between threads
unsafe impl Send for LogConsumer {}

impl LogConsumer {
    pub fn new(name: &str, size: usize) -> io::Result<Self> {
        eprintln!("[-LO-] Creating log consumer for {} on {}", name, ARCH_NAME);
        let shm = SharedMemoryConsumer::open(name, size)?;
        Ok(LogConsumer { shm })
    }

    #[allow(dead_code)]
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
            }
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
            }
            None => Ok(None),
        }
    }

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

    #[allow(dead_code)]
    pub fn cleanup(&self) -> io::Result<()> {
        self.shm.cleanup()
    }
}
