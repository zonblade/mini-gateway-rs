pub mod settings;
pub mod users;
pub mod statistics;
pub mod sync;

use actix_web::web;
use users::{JwtAuth, init_database};

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Initialize the users database
    if let Err(e) = init_database() {
        eprintln!("Error initializing users database: {}", e);
    }
    
    cfg.service(
        web::scope("/api/v1")
            // Apply JWT authentication to all API routes
            // This middleware only verifies that the token is valid
            // Specific endpoints can enforce additional role requirements
            .wrap(JwtAuth::new())
            .configure(settings::configure)
            .configure(users::configure)
            // Statistics and sync modules are empty now, but will be protected when implemented
            // .configure(statistics::configure)
            // .configure(sync::configure)
    );
}