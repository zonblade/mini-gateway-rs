use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info};

use super::service_protocol::ServiceProtocol;

/// ServiceHandler manages the collection of services and their lifecycle
pub struct ServiceHandler {
    services: HashMap<String, Box<dyn ServiceProtocol>>,
}

impl ServiceHandler {
    /// Create a new service handler instance
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Add a single service to the handler
    pub fn add_service(&mut self, name: String, service: Box<dyn ServiceProtocol>) {
        info!("Registering service: {}", name);
        self.services.insert(name, service);
    }

    /// Add multiple services at once
    pub fn add_services(&mut self, services_with_names: Vec<(String, Box<dyn ServiceProtocol>)>) {
        for (name, service) in services_with_names {
            self.add_service(name, service);
        }
    }

    /// Get a service by name
    pub fn get_service(&self, name: &str) -> Option<&Box<dyn ServiceProtocol>> {
        self.services.get(name)
    }

    /// Get all registered services
    pub fn get_services(&self) -> &HashMap<String, Box<dyn ServiceProtocol>> {
        &self.services
    }

    /// Run the service handler - in a real implementation this might start a background worker
    /// or set up other resources needed for the services to operate
    pub fn join(&self) {
        info!("Service handler started with {} services", self.services.len());
        
        // Log registered services
        for name in self.services.keys() {
            info!("Active service: {}", name);
        }
    }
}

/// Shared handler that can be used across threads
pub type SharedServiceHandler = Arc<RwLock<ServiceHandler>>;

/// Initialize a new shared service handler
pub fn init() -> SharedServiceHandler {
    info!("Initializing service handler");
    Arc::new(RwLock::new(ServiceHandler::new()))
}

/// Register a service with a given name
pub fn register_service<T: ServiceProtocol + 'static>(name: &str, service: T) -> (String, Box<dyn ServiceProtocol>) {
    (name.to_string(), Box::new(service))
}