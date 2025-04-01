mod handlers;
mod models;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(handlers::create_user::init)
            .service(handlers::get_user::init)
            .service(handlers::create_user::init)
            .service(handlers::update_user::init)
            .service(handlers::delete_user::init)
    );
}

// Initialize users table in the database
pub fn init_database() -> Result<(), crate::module::database::DatabaseError> {
    let db = crate::module::database::get_connection()?;
    
    // Create users table if not exists with role field
    db.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('admin', 'staff', 'user')),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    Ok(())
}