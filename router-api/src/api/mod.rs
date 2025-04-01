pub mod settings;
pub mod users;
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Initialize the users database
    if let Err(e) = users::init_database() {
        eprintln!("Error initializing users database: {}", e);
    }

    cfg.service(
        web::scope("/api/v1")
            .configure(settings::configure)
            .configure(users::configure)
    );
}