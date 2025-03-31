use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info};

use super::service_protocol::ServiceProtocol;

/// # Service Handler
///
/// The ServiceHandler is the central registry for all protocol services.
/// It manages the collection of available services and their lifecycle,
/// providing a mechanism to look up appropriate services based on client requests.
///
/// ## Responsibilities
///
/// - Storing registered services in a thread-safe container
/// - Providing access to services by name
/// - Managing service registration and initialization
/// - Supporting concurrent access from multiple connections
///
/// ## Architecture
///
/// The handler uses a simple registry pattern with services stored in a HashMap,
/// keyed by their service name. It provides methods for:
/// - Adding individual services
/// - Adding multiple services at once
/// - Retrieving services by name
/// - Listing all available services
///
/// ## Thread Safety
///
/// The handler itself is not thread safe, but it is typically wrapped in an
/// Arc<RwLock<>> (see SharedServiceHandler) to allow safe concurrent access
/// from multiple connections across different threads.
pub struct ServiceHandler {
    /// Map of service names to their implementations
    services: HashMap<String, Box<dyn ServiceProtocol>>,
}

impl ServiceHandler {
    /// Create a new, empty service handler instance
    ///
    /// This constructor initializes an empty service registry that can
    /// be populated with services using the add_service methods.
    ///
    /// ## Returns
    ///
    /// A new ServiceHandler instance with no registered services.
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Register a single service with the handler
    ///
    /// This method adds a service to the registry with the specified name.
    /// If a service with this name already exists, it will be replaced.
    ///
    /// ## Parameters
    ///
    /// * `name` - Unique identifier for the service
    /// * `service` - The service implementation boxed as a trait object
    pub fn add_service(&mut self, name: String, service: Box<dyn ServiceProtocol>) {
        info!("Registering service: {}", name);
        self.services.insert(name, service);
    }

    /// Register multiple services at once
    ///
    /// This is a convenience method for registering several services in
    /// a single operation. It iterates through the provided list and
    /// calls add_service for each entry.
    ///
    /// ## Parameters
    ///
    /// * `services_with_names` - A vector of name-service pairs to register
    pub fn add_services(&mut self, services_with_names: Vec<(String, Box<dyn ServiceProtocol>)>) {
        for (name, service) in services_with_names {
            self.add_service(name, service);
        }
    }

    /// Retrieve a service by name
    ///
    /// This method looks up a service in the registry by its name.
    ///
    /// ## Parameters
    ///
    /// * `name` - The name of the service to retrieve
    ///
    /// ## Returns
    ///
    /// An Option containing either:
    /// - Some(&Box<dyn ServiceProtocol>) if the service exists
    /// - None if no service with the given name is registered
    pub fn get_service(&self, name: &str) -> Option<&Box<dyn ServiceProtocol>> {
        self.services.get(name)
    }

    /// Get all registered services
    ///
    /// This method provides access to the complete map of services.
    ///
    /// ## Returns
    ///
    /// A reference to the HashMap containing all registered services
    pub fn get_services(&self) -> &HashMap<String, Box<dyn ServiceProtocol>> {
        &self.services
    }

    /// Initialize and prepare all services for operation
    ///
    /// This method logs information about registered services and could
    /// be extended to perform additional initialization steps for each service.
    ///
    /// In a more complex implementation, this might:
    /// - Start background workers for each service
    /// - Initialize connection pools
    /// - Pre-warm caches
    /// - Set up health checks
    pub fn join(&self) {
        info!("Service handler started with {} services", self.services.len());
        
        // Log registered services
        for name in self.services.keys() {
            info!("Active service: {}", name);
        }
    }
}

/// # Shared Service Handler
///
/// Thread-safe wrapper around ServiceHandler that can be shared across multiple
/// threads or async tasks. It combines Arc (for shared ownership) with RwLock
/// (for safe concurrent access with reader-writer semantics).
///
/// The RwLock allows:
/// - Multiple readers to access the handler simultaneously
/// - Exclusive access for writers when modifying the handler
///
/// This type is typically created using the init() function and passed
/// to the connection handlers that need to access services.
pub type SharedServiceHandler = Arc<RwLock<ServiceHandler>>;

/// # Initialize a New Shared Service Handler
///
/// Creates and returns a new thread-safe service handler that can be
/// shared across multiple connections and threads.
///
/// ## Returns
///
/// A SharedServiceHandler (Arc<RwLock<ServiceHandler>>) ready to register services
pub fn init() -> SharedServiceHandler {
    info!("Initializing service handler");
    Arc::new(RwLock::new(ServiceHandler::new()))
}

/// # Register a Service with a Given Name
///
/// Helper function to simplify service registration by converting a concrete
/// service implementation into a trait object package ready for registration.
///
/// ## Type Parameters
///
/// * `T` - A type that implements ServiceProtocol and has a 'static lifetime
///
/// ## Parameters
///
/// * `name` - The name to register the service under
/// * `service` - The concrete service implementation
///
/// ## Returns
///
/// A tuple containing the service name and the boxed service trait object
pub fn register_service<T: ServiceProtocol + 'static>(name: &str, service: T) -> (String, Box<dyn ServiceProtocol>) {
    (name.to_string(), Box::new(service))
}