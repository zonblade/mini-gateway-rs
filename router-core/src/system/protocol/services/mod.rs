mod service_protocol;
mod handler;
mod example_service;

pub use service_protocol::ServiceProtocol;
pub use handler::{ServiceHandler, SharedServiceHandler, init, register_service};
pub use example_service::ExampleService;