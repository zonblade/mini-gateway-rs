use crate::api::users::models::{Role, User};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// JWT claims structure for our tokens
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
pub struct AuthConfig {
    /// Secret key for signing and verifying tokens
    secret_key: String,
    /// Token validity duration in minutes
    token_validity: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            // In production, this would be loaded from environment or secure storage
            secret_key: "your_jwt_secret_key_should_be_long_and_secure".to_string(),
            // Default token validity: 60 minutes (1 hour)
            token_validity: 60,
        }
    }
}

impl AuthConfig {
    /// Create a new AuthConfig with custom settings
    pub fn new(secret_key: String, token_validity_minutes: u64) -> Self {
        Self {
            secret_key,
            token_validity: token_validity_minutes,
        }
    }
}

/// Generates a JWT token for a user
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
pub fn is_admin(role: &str) -> bool {
    role == Role::Admin.to_string()
}

/// Convenience function to check if a user has staff role or above
pub fn is_staff_or_admin(role: &str) -> bool {
    role == Role::Admin.to_string() || role == Role::Staff.to_string()
}

/// Convenience function to check if a user has user role or above
pub fn is_user_or_above(role: &str) -> bool {
    true // Everyone has at least user level privileges if they have a valid token
}

/// Checks if user with given ID and role can modify another user with target_id
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