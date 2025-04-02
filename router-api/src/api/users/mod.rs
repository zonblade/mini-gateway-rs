mod handlers;
mod models;
pub mod helper;

use actix_web::web;
// Re-export auth helpers for use in other modules
pub use helper::{RoleAuth, UserSelfCheck, ClaimsFromRequest, Claims};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            // Public endpoint (no auth required)
            .service(handlers::login::init)
            
            // Admin-only endpoints
            .service(
                web::scope("/admin")
                    .wrap(RoleAuth::admin())
                    .service(handlers::get_users::init)
                    .service(handlers::create_user::init)
            )
            
            // User-specific endpoints with self-check or admin override
            .service(handlers::get_user::init)
            .service(
                web::resource("/{user_id}")
                    .wrap(UserSelfCheck::self_and_admin())
                    .route(web::put().to(handlers::update_user::init))
                    .route(web::delete().to(handlers::delete_user::init))
            )
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
    
    // Create a default admin user if no users exist
    let user_count: i64 = db.query_one(
        "SELECT COUNT(*) FROM users",
        [],
        |row| row.get::<_, i64>(0)
    )?.unwrap_or(0);
    
    if user_count == 0 {
        // Create a default admin user
        db.execute(
            "INSERT INTO users (id, username, email, password_hash, role) VALUES (?, ?, ?, ?, ?)",
            [
                &uuid::Uuid::new_v4().to_string(),
                "admin",
                "admin@example.com",
                "hashed_adminpassword", // In a real app, use proper password hashing
                "admin"
            ],
        )?;
        
        println!("Created default admin user (username: admin, password: adminpassword)");
    }
    
    Ok(())
}