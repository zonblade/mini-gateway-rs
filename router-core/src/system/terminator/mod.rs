//! # Terminator Module
//! 
//! The terminator module provides functionality for gracefully shutting down the router system.
//! It includes mechanisms for termination initiated both through command line input and through
//! service requests from external components.
//! 
//! ## Module Structure
//! 
//! * `cli`: Implements command-line based termination through keyboard shortcuts (Ctrl+X)
//! * `service`: Implements service-based termination through external requests
//! 
//! ## Termination Process
//! 
//! The termination process is designed to ensure that all components of the router system
//! shut down gracefully:
//! 
//! 1. A termination signal is received (keyboard shortcut or service message)
//! 2. The terminator sets appropriate shutdown flags
//! 3. Active connections are allowed to complete (or time out)
//! 4. Resources are freed and servers are shut down in the correct order
//! 5. The application exits with an appropriate status code
//! 
//! ## Usage
//! 
//! The terminator module provides two primary mechanisms for shutdown:
//! 
//! 1. CLI-based termination: `terminator::cli::init()` checks for keyboard shortcuts
//! 2. Service-based termination: `terminator::service::init()` initiates programmatic shutdown

pub mod cli;
pub mod service;
