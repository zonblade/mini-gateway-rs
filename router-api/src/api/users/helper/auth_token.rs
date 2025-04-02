//! # Authentication Token Module
//!
//! This module provides JWT (JSON Web Token) based authentication for the API.
//! It implements token generation, validation, and role-based permission checking.
//!
//! ## Features
//!
//! - Generation of JWT tokens with configurable expiration
//! - Validation of tokens and extraction of claims
//! - Role-based permission checking functions
//! - Secure random key generation for token signing
//!
//! ## Security Characteristics
//!
//! - Tokens contain user identity and role information
//! - Each service restart generates a new random signing key (forcing re-login)
//! - Tokens have a configurable expiration time (default: 60 minutes)
//! - Role checks provide granular access control throughout the application
//!
//! ## Usage Example
//!
//! ```rust
//! use crate::api::users::helper::auth_token::{AuthConfig, generate_token, validate_token};
//!
//! // Create default config (random key, 60 min expiry)
//! let config = AuthConfig::default();
//!
//! // Generate a token for a user
//! let token = generate_token(&user, &config)?;
//!
//! // Later, validate the token and extract claims
//! let claims = validate_token(&token, &config)?;
//! ```

use crate::api::users::models::{Role, User};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rand::{distributions::Alphanumeric, Rng};
use std::sync::LazyLock;

static GLOBAL_SECRET: LazyLock<String> = LazyLock::new(|| {
    // Generate a 64-character random secret key when first accessed
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
});

/// JWT claims structure for our tokens
///
/// This structure defines the payload contained within JWT tokens.
/// It includes user identification, role information, and expiration metadata.
///
/// The claims follow standard JWT conventions:
/// - `sub` (subject): Contains the user ID
/// - `exp` (expiration time): Unix timestamp when the token expires
/// - `iat` (issued at): Unix timestamp when the token was created
///
/// Additionally, it includes custom claims:
/// - `username`: For display and identification purposes
/// - `role`: For authorization checks
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (the user ID)
    pub sub: String,
    
    /// Username for information purposes
    pub username: String,
    
    /// User role for authorization
    pub role: String,
    
    /// Expiration time (Unix timestamp)
    pub exp: u64,
    
    /// Issued at time (Unix timestamp)
    pub iat: u64,
}

/// Config for token generation and validation
///
/// This structure holds the configuration needed for JWT token operations:
/// - A secret key used for signing and verifying tokens
/// - The token validity duration (in minutes)
///
/// By default, it generates a random secret key on instantiation,
/// which means tokens will be invalidated when the service restarts.
pub struct AuthConfig {
    /// Secret key for signing and verifying tokens
    secret_key: String,
    
    /// Token validity duration in minutes
    token_validity: u64,
}

impl Default for AuthConfig {
    /// Creates a default configuration with a random key and 60-minute validity
    ///
    /// This default configuration:
    /// - Generates a cryptographically secure random key (64 characters)
    /// - Sets token validity to 60 minutes (1 hour)
    ///
    /// Because the key is randomly generated on each initialization,
    /// users will need to re-login after a service restart.
    fn default() -> Self {
        Self {
            // Generate a random secret key on each service startup
            // This ensures users must relogin after a service restart for security
            secret_key: GLOBAL_SECRET.clone(),
            // Default token validity: 60 minutes (1 hour)
            token_validity: 60,
        }
    }
}

impl AuthConfig {
    /// Create a new AuthConfig with custom settings
    ///
    /// # Parameters
    ///
    /// * `secret_key` - The secret key to use for signing tokens
    /// * `token_validity_minutes` - How long tokens should be valid (in minutes)
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = AuthConfig::new(
    ///     "my-super-secret-key-that-is-long-and-secure".to_string(),
    ///     120, // 2 hours
    /// );
    /// ```
    pub fn new(secret_key: String, token_validity_minutes: u64) -> Self {
        Self {
            secret_key,
            token_validity: token_validity_minutes,
        }
    }
}

/// Generates a JWT token for a user
///
/// Creates a JWT token containing the user's ID, username, and role,
/// signed with the provided configuration's secret key.
///
/// # Parameters
///
/// * `user` - The user for whom to generate a token
/// * `config` - The authentication configuration to use
///
/// # Returns
///
/// A `Result` containing either the JWT token string or a JWT error
///
/// # Errors
///
/// Returns an error if token creation fails, which could happen if:
/// - The algorithm is unsupported
/// - The claims cannot be serialized
/// - The token cannot be signed
pub fn generate_token(user: &User, config: &AuthConfig) -> Result<String, JwtError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    
    let expiration = now + (config.token_validity * 60); // Convert minutes to seconds
    
    let claims = Claims {
        sub: user.id.clone(),
        username: user.username.clone(),
        role: user.role.to_string(),
        exp: expiration,
        iat: now,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret_key.as_bytes())
    )
}

/// Validates a JWT token and returns the claims if valid
///
/// Verifies the token signature and expiration time, then returns
/// the decoded claims if the token is valid.
///
/// # Parameters
///
/// * `token` - The JWT token string to validate
/// * `config` - The authentication configuration to use
///
/// # Returns
///
/// A `Result` containing either the decoded claims or a JWT error
///
/// # Errors
///
/// Returns an error if validation fails, which could happen if:
/// - The token is malformed
/// - The token signature is invalid
/// - The token has expired
/// - Required claims are missing
pub fn validate_token(token: &str, config: &AuthConfig) -> Result<Claims, JwtError> {
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret_key.as_bytes()),
        &validation
    )?;
    
    Ok(token_data.claims)
}

/// Convenience function to check if a user has admin role
///
/// # Parameters
///
/// * `role` - The role string to check
///
/// # Returns
///
/// `true` if the role is "admin", `false` otherwise
pub fn is_admin(role: &str) -> bool {
    role == Role::Admin.to_string()
}

/// Convenience function to check if a user has staff role or above
///
/// This checks if a user has either staff or admin role, making it
/// useful for endpoints that should be accessible to both roles.
///
/// # Parameters
///
/// * `role` - The role string to check
///
/// # Returns
///
/// `true` if the role is "admin" or "staff", `false` otherwise
pub fn is_staff_or_admin(role: &str) -> bool {
    role == Role::Admin.to_string() || role == Role::Staff.to_string()
}

/// Convenience function to check if a user has user role or above
///
/// This function always returns true as all authenticated users have
/// at least basic user privileges. It's included for API consistency
/// with the other role-checking functions.
///
/// # Parameters
///
/// * `_role` - The role string (unused but included for API consistency)
///
/// # Returns
///
/// Always returns `true` for any authenticated user
pub fn is_user_or_above(_role: &str) -> bool {
    true // Everyone has at least user level privileges if they have a valid token
}

/// Checks if user with given ID and role can modify another user with target_id
///
/// This implements the permission logic for user modification:
/// - Admins can modify any user
/// - Staff can modify any user (except admins, checked at controller level)
/// - Regular users can only modify themselves
///
/// # Parameters
///
/// * `user_id` - The ID of the user attempting the modification
/// * `user_role` - The role of the user attempting the modification
/// * `target_id` - The ID of the user being modified
///
/// # Returns
///
/// `true` if the modification is allowed, `false` otherwise
pub fn can_modify_user(user_id: &str, user_role: &str, target_id: &str) -> bool {
    // Admins can modify any user
    if is_admin(user_role) {
        return true;
    }
    
    // Staff can modify any user except admins (determined at the controller level)
    if is_staff_or_admin(user_role) {
        return true;
    }
    
    // Regular users can only modify themselves
    user_id == target_id
}