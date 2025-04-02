//! # User Data Models
//!
//! This module contains the data structures and type definitions for user management.
//! It defines the core entities such as User, Role, and request/response DTOs used
//! throughout the user management API.
//!
//! ## Core Entities
//!
//! - `Role`: An enum representing the user's authorization level
//! - `User`: The primary user entity with authentication and profile information
//! - `UserResponse`: A public-facing view of user data (excludes sensitive fields)
//!
//! ## Request and Response DTOs
//!
//! - `CreateUserRequest`: Data transfer object for user creation
//! - `UpdateUserRequest`: Data transfer object for user updates
//! - `UserResponse`: Data transfer object for sending user data to clients
//!
//! ## Security Considerations
//!
//! The module implements several security patterns:
//! - The `password_hash` field is marked to skip serialization to prevent leaking credentials
//! - `UserResponse` omits sensitive fields when sending user data to clients
//! - Role conversions support flexible string representations while maintaining type safety

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User roles for authorization
///
/// This enum represents the hierarchical role system within the application:
///
/// * `Admin`: Highest privilege level with complete system access
/// * `Staff`: Elevated privileges for user management and some system settings
/// * `User`: Standard user with basic permissions
///
/// The enum provides serialization/deserialization for use with APIs,
/// and conversion methods to/from string for database storage.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Role {
    /// Administrator role with full system access
    #[serde(rename = "admin")]
    Admin,
    
    /// Staff role with elevated privileges
    #[serde(rename = "staff")]
    Staff,
    
    /// Regular user role with basic permissions
    #[serde(rename = "user")]
    User,
}

impl ToString for Role {
    /// Converts a Role enum to its string representation
    ///
    /// This method is used when storing roles in the database or
    /// serializing for API responses.
    ///
    /// # Returns
    ///
    /// A lowercase string representation of the role: "admin", "staff", or "user"
    fn to_string(&self) -> String {
        match self {
            Role::Admin => "admin".to_string(),
            Role::Staff => "staff".to_string(),
            Role::User => "user".to_string(),
        }
    }
}

impl From<String> for Role {
    /// Creates a Role from a string
    ///
    /// This method is used when loading roles from the database or
    /// deserializing from API requests. It implements a fallback strategy
    /// where unrecognized roles default to User for safety.
    ///
    /// # Parameters
    ///
    /// * `value`: A string representation of the role
    ///
    /// # Returns
    ///
    /// The corresponding Role enum value, defaulting to User if the string
    /// doesn't match any known role
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "admin" => Role::Admin,
            "staff" => Role::Staff,
            _ => Role::User, // Default to user for any other value
        }
    }
}

/// Core user entity
///
/// This struct represents a user in the system, containing both profile information
/// and authentication details. It's used for database operations and internal
/// system logic.
///
/// Some fields like `password_hash` are marked to skip serialization to ensure
/// they're never accidentally exposed in API responses.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier (UUID)
    pub id: String,
    
    /// Unique username for login and display
    pub username: String,
    
    /// User's email address
    pub email: String,
    
    /// Hashed password (never exposed in API responses)
    #[serde(skip_serializing)]
    pub password_hash: String,
    
    /// User's authorization role
    pub role: Role,
    
    /// Timestamp when the user was created
    pub created_at: Option<String>,
    
    /// Timestamp when the user was last updated
    pub updated_at: Option<String>,
}

impl User {
    /// Creates a new user with the given attributes
    ///
    /// This method handles generating a unique ID and converting the password
    /// to a hashed format. In a production application, this should use a proper
    /// password hashing algorithm like bcrypt or Argon2.
    ///
    /// # Parameters
    ///
    /// * `username`: The user's unique username
    /// * `email`: The user's email address
    /// * `password`: The plaintext password (will be hashed)
    /// * `role`: The user's authorization role
    ///
    /// # Returns
    ///
    /// A new User instance ready to be stored in the database
    pub fn new(username: String, email: String, password: String, role: Role) -> Self {
        // In a real app, you would use a proper password hashing library like bcrypt
        // For this example, we'll just simulate a hash with a simple function
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            email,
            password_hash: format!("hashed_{}", password), // Simulated hash
            role,
            created_at: None,
            updated_at: None,
        }
    }
}

/// Request DTO for user creation
///
/// This data transfer object is used for creating new users through the API.
/// It captures all required fields and makes the role optional (defaulting to
/// regular user if not specified).
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    /// Unique username for login and display
    pub username: String,
    
    /// User's email address
    pub email: String,
    
    /// Password in plaintext (will be hashed during processing)
    pub password: String,
    
    /// Optional role (defaults to User if not specified)
    pub role: Option<Role>,
}

/// Request DTO for user updates
///
/// This data transfer object is used for updating existing users. All fields
/// are optional, allowing partial updates where only specified fields are changed.
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    /// New username (if changing)
    pub username: Option<String>,
    
    /// New email address (if changing)
    pub email: Option<String>,
    
    /// New password (if changing)
    pub password: Option<String>,
    
    /// New role (if changing)
    pub role: Option<Role>,
}

/// Response DTO for user data
///
/// This data transfer object is the public-facing representation of user data,
/// omitting sensitive information like password hashes. It's used for all API
/// responses that return user information.
#[derive(Debug, Serialize)]
pub struct UserResponse {
    /// Unique identifier (UUID)
    pub id: String,
    
    /// Username for login and display
    pub username: String,
    
    /// User's email address
    pub email: String,
    
    /// User's authorization role
    pub role: Role,
    
    /// Timestamp when the user was created
    pub created_at: Option<String>,
    
    /// Timestamp when the user was last updated
    pub updated_at: Option<String>,
}

impl From<User> for UserResponse {
    /// Converts a User entity to a UserResponse DTO
    ///
    /// This provides a clean way to transform internal user entities to
    /// public-facing representations, ensuring sensitive data is never leaked.
    ///
    /// # Parameters
    ///
    /// * `user`: The internal User entity to convert
    ///
    /// # Returns
    ///
    /// A UserResponse instance suitable for API responses
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}