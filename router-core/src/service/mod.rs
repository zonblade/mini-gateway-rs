//! # Service Module
//! 
//! The service module provides the core service implementations for the mini-gateway router.
//! It contains the implementations for proxy services, gateway services, and the service
//! registry that enables communication between components.
//! 
//! ## Module Structure
//! 
//! * `proxy`: Implements proxy service functionality, allowing TCP/TLS traffic forwarding
//! * `gateway`: Implements HTTP gateway services with path-based routing and transformations
//! * `registry`: Provides service discovery and registration for inter-service communication
//! 
//! ## Architecture
//! 
//! The service implementations follow a layered approach:
//! 
//! 1. The registry layer manages service discovery and metadata
//! 2. The proxy/gateway services implement core networking functionality
//! 3. Each service integrates with the Pingora framework for high-performance networking
//! 
//! ## Service Lifecycle
//! 
//! Services in this module follow a consistent lifecycle:
//! 
//! 1. Registration with the service registry during initialization
//! 2. Configuration via dynamic updates through Redis/DragonflyDB
//! 3. Request processing based on current configuration
//! 4. Graceful shutdown when termination signals are received

pub mod proxy;
pub mod registry;
pub mod gateway;