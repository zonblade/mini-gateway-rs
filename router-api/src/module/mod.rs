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
//!
//! ## Usage
//!
//! These modules are typically used as dependencies by higher-level components in
//! the application, such as API endpoints and service implementations. They provide
//! foundational capabilities that other parts of the system can build upon.

pub mod database;