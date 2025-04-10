//! # System Module
//! 
//! The system module provides core infrastructure components for the router's operation.
//! It contains server implementation, protocol handlers, termination controls, and
//! default error page handlers.
//! 
//! ## Module Structure
//! 
//! * `default_page`: Handlers for serving default content for error conditions and security monitoring
//! * `protocol`: Implementation of the custom protocol for inter-service communication
//! * `server`: Core server initialization and management functionality
//! * `terminator`: Signal handling and graceful shutdown mechanisms
//! * `listeners`: Module for managing network listeners
//! 
//! ## Responsibility
//! 
//! This module is responsible for the lowest-level components of the router system,
//! managing network connections, server lifecycle, and system-level protocols.

pub mod default_page;
pub mod protocol;
pub mod server;
pub mod terminator;
pub mod netlisten;
pub mod writer;
