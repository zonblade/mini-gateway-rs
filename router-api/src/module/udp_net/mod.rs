//! # Module System
//!
//! This module provides core functionality and shared components that can be used
//! by multiple parts of the router-api application. It serves as a collection of
//! reusable infrastructure components that support the application's business logic.
//!
//! ## Structure
//!
//! The module system is organized into the following submodules:
//!
//! - `database`: Provides database connection and management functionality, including
//!   connection pooling, migrations, and query utilities.
//! - `udp_log_fetcher`: Provides UDP socket-based logging infrastructure.
//! - `udp_logger`: Implements multi-port UDP logging with proper port isolation.
//! - `udp_log_db`: Provides a database pooling system for log messages with periodic flushing.
//! - `udp_log_processor`: Processes log messages and sends them to the database pool.
//!
//! ## Usage
//!
//! These modules are typically used as dependencies by higher-level components in
//! the application, such as API endpoints and service implementations. They provide
//! foundational capabilities that other parts of the system can build upon.

pub mod udp_log;
pub mod udp_log_fetcher;
pub mod udp_log_processor;
pub mod udp_logger;
