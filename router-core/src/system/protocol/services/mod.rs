/// # Protocol Services Module
///
/// This module implements a trait-based service system for the protocol server.
/// It allows dynamic registration and routing of client requests to the appropriate
/// service implementations based on the service name in the connection parameters.
///
/// ## Overview
///
/// The services module provides:
///
/// - A trait interface (`ServiceProtocol`) that defines the contract for service implementations
/// - A service handler system for registering and managing services
/// - Helper functions for service creation and registration
/// - An example service implementation for reference
///
/// ## Architecture
///
/// The service system follows a plugin-like architecture where:
///
/// 1. Each service implements the `ServiceProtocol` trait
/// 2. Services are registered with the service handler at startup
/// 3. The protocol server routes requests to the appropriate service based on the service name
/// 4. Each service handles its own request processing and logging
///
/// ## Thread Safety
///
/// The service system is designed to be thread-safe, using atomic references and
/// read-write locks to ensure safe concurrent access to services from multiple connections.

mod service_protocol;
mod handler;
mod example_service;

pub use service_protocol::ServiceProtocol;
pub use handler::{ServiceHandler, SharedServiceHandler, init, register_service};
pub use example_service::ExampleService;