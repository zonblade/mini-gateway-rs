use crate::error::CliError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Cached token stored at ~/.config/gwrs/token.json
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenCache {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub base_url: String,
}

/// Resolve credentials from CLI args or environment variables.
pub fn resolve_credentials(
    osenv: bool,
    user: Option<&str>,
    pass: Option<&str>,
) -> Result<(String, String), CliError> {
    if osenv {
        let username = std::env::var("GWRS_USER")
            .map_err(|_| CliError::Auth("GWRS_USER environment variable not set".into()))?;
        let password = std::env::var("GWRS_PASS")
            .map_err(|_| CliError::Auth("GWRS_PASS environment variable not set".into()))?;
        Ok((username, password))
    } else if let (Some(user), Some(pass)) = (user, pass) {
        Ok((user.to_string(), pass.to_string()))
    } else {
        Err(CliError::Auth(
            "no credentials provided. Use --osenv or provide --user and --pass".into(),
        ))
    }
}

fn cache_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("gwrs").join("token.json"))
}

/// Read cached token if it exists, is not expired, and matches the base URL.
pub fn read_cached_token(base_url: &str) -> Option<String> {
    let path = cache_path()?;
    let data = std::fs::read_to_string(&path).ok()?;
    let cache: TokenCache = serde_json::from_str(&data).ok()?;

    if cache.base_url != base_url {
        return None;
    }

    // 5-minute safety margin before expiry
    let margin = chrono::Duration::minutes(5);
    if Utc::now() + margin >= cache.expires_at {
        return None;
    }

    Some(cache.token)
}

/// Write token to cache file.
pub fn write_cached_token(base_url: &str, token: &str) -> Result<(), CliError> {
    let Some(path) = cache_path() else {
        return Ok(());
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // JWT tokens from the API expire in 60 minutes
    let cache = TokenCache {
        token: token.to_string(),
        expires_at: Utc::now() + chrono::Duration::minutes(55),
        base_url: base_url.to_string(),
    };

    let data = serde_json::to_string_pretty(&cache)?;
    std::fs::write(&path, data)?;
    Ok(())
}

/// Clear the cached token.
pub fn clear_cached_token() {
    if let Some(path) = cache_path() {
        let _ = std::fs::remove_file(path);
    }
}
