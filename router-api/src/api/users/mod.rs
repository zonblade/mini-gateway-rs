//! # User Management API Module
//!
//! This module provides a comprehensive set of endpoints and utilities for user management, including:
//!
//! - User authentication via JWT (JSON Web Tokens)
//! - Role-based access control (admin, staff, user)
//! - User profile management (create, read, update, delete)
//! - Self-service user management with appropriate permissions
//!
//! ## Authentication Flow
//!
//! 1. Users authenticate via the `/login` endpoint with username/password
//! 2. The system validates credentials and issues a JWT token
//! 3. Subsequent requests include this token in the `Authorization` header
//! 4. Middleware validates the token and extracts user information
//!
//! ## Authorization System
//!
//! The module implements a hierarchical role system:
//!
//! - **Admin**: Full system access, can manage all users and settings
//! - **Staff**: Extended privileges for managing regular users and some settings
//! - **User**: Basic access to own profile and public resources
//!
//! ## Security Features
//!
//! - Token-based authentication with configurable expiration
//! - Password hashing (simulated in this version)
//! - Role-based middleware for securing endpoints
//! - Self-check middleware to ensure users can only modify their own data
//!
//! ## Default Administrator Account
//!
//! The module automatically creates a default administrator account if no users
//! exist in the database. This ensures that there's always an admin user for
//! initial system setup.

mod handlers;
pub mod helper;
mod models;

use actix_web::web;
// Re-export auth helpers for use in other modules
pub use helper::{JwtAuth, RoleAuth, UserSelfCheck};

/// Configures user management routes and middleware
///
/// This function sets up the endpoints and middleware for user management:
///
/// - `/login` - Public endpoint for authentication
/// - `/admin/*` - Admin-only endpoints protected by role middleware
/// - `/{user_id}` - User-specific endpoints with self-check middleware
///
/// The routing structure enforces proper authorization:
/// - Only admins can list all users or create new users
/// - Users can view their own profiles
/// - Users can only update/delete their own profiles, with admin override
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            // Public endpoint (no auth required)
            .service(handlers::login::init)
            // Admin-only endpoints
            .service(
                web::scope("/admin")
                    .wrap(JwtAuth::new())
                    .wrap(RoleAuth::admin())
                    .service(handlers::get_users::init)
                    .service(handlers::create_user::init),
            )
            // User-specific endpoints with self-check or admin override
            .service(handlers::get_user::init)
            .service(
                web::resource("/{user_id}")
                    .wrap(JwtAuth::new())
                    .wrap(UserSelfCheck::self_and_admin())
                    .route(web::put().to(handlers::update_user::init))
                    .route(web::delete().to(handlers::delete_user::init)),
            ),
    );
}

/// Initializes the users database table and creates a default admin user if needed
///
/// This function:
/// 1. Creates the users table if it doesn't exist
/// 2. Checks if any users exist in the database
/// 3. If no users exist, creates a default administrator account
///
/// The default admin account has these credentials:
/// - Username: admin
/// - Password: adminpassword
/// - Email: admin@example.com
/// - Role: admin
///
/// # Returns
///
/// A result indicating success or a database error
///
/// # Errors
///
/// Returns an error if database connection fails or any database operations fail
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
    let user_count: i64 = db
        .query_one("SELECT COUNT(*) FROM users", [], |row| row.get::<_, i64>(0))?
        .unwrap_or(0);

    if user_count == 0 {
        // Create a default admin user
        db.execute(
            "INSERT INTO users (id, username, email, password_hash, role) VALUES (?, ?, ?, ?, ?)",
            [
                &uuid::Uuid::new_v4().to_string(),
                "admin",
                "admin@example.com",
                "hashed_adminpassword", // In a real app, use proper password hashing
                "admin",
            ],
        )?;

        log::debug!("Created default admin user (username: admin, password: adminpassword)");
    }

    Ok(())
}
