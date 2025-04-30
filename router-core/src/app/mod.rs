//! # Application Module
//! 
//! The application module provides the core application logic for the mini-gateway router system.
//! It contains the implementation of proxy and gateway components that handle the actual
//! request routing, transformations, and forwarding.
//! 
//! ## Module Structure
//! 
//! * `proxy`: Implements proxying functionality for TCP/TLS connections
//! * `gateway`: Implements HTTP gateway functionality with path-based routing
//! 
//! ## Responsibility
//! 
//! This module is responsible for the business logic of traffic routing:
//! - Determining where to route incoming requests based on configuration
//! - Transforming requests (headers, paths, etc.) as needed
//! - Handling load balancing and failover
//! - Implementing security policies and access controls
//! 
//! The implementations in this module build on the lower-level system components
//! to provide the actual gateway and proxy behavior defined by user configuration.
pub mod proxy_fast;