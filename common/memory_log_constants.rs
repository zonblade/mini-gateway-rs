//! Shared constants for memory log implementation
//! 
//! This file should be included by both router-core and router-api
//! to ensure consistency in memory structures and sizes.

/// Maximum size of the shared memory region
pub const MAX_MEMORY_SIZE: usize = 256 * 1024 * 1024; // 256MB - compromise between core and api

/// Maximum size of each entry in the log
pub const ENTRY_MAX_SIZE: usize = 256; // 256 bytes per entry - enough for most log messages

/// Size of metadata section at start of shared memory
pub const SHM_METADATA_SIZE: usize = 2048; // 2KB for metadata

/// Names for the shared memory regions
pub const PROXY_LOGGER_NAME: &str = "/gwrs-proxy";
pub const GATEWAY_LOGGER_NAME: &str = "/gwrs-gateway";

/// Log levels
pub const LEVEL_TRACE: u8 = 0;
pub const LEVEL_DEBUG: u8 = 1;
pub const LEVEL_INFO: u8 = 2;
pub const LEVEL_WARN: u8 = 3;
pub const LEVEL_ERROR: u8 = 4;
