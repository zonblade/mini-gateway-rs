use crate::error::CliError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenCache {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub base_url: String,
}

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

pub fn read_cached_token(base_url: &str) -> Option<String> {
    let path = cache_path()?;
    let data = std::fs::read_to_string(&path).ok()?;
    let cache: TokenCache = serde_json::from_str(&data).ok()?;

    if cache.base_url != base_url {
        return None;
    }

    let margin = chrono::Duration::minutes(5);
    if Utc::now() + margin >= cache.expires_at {
        return None;
    }

    Some(cache.token)
}

pub fn write_cached_token(base_url: &str, token: &str) -> Result<(), CliError> {
    let Some(path) = cache_path() else {
        return Ok(());
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let cache = TokenCache {
        token: token.to_string(),
        expires_at: Utc::now() + chrono::Duration::minutes(55),
        base_url: base_url.to_string(),
    };

    let data = serde_json::to_string_pretty(&cache)?;
    std::fs::write(&path, data)?;
    Ok(())
}

pub fn clear_cached_token() {
    if let Some(path) = cache_path() {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_credentials_from_args() {
        let (user, pass) = resolve_credentials(false, Some("admin"), Some("secret")).unwrap();
        assert_eq!(user, "admin");
        assert_eq!(pass, "secret");
    }

    #[test]
    fn resolve_credentials_missing_user() {
        let result = resolve_credentials(false, None, Some("secret"));
        assert!(result.is_err());
    }

    #[test]
    fn resolve_credentials_missing_pass() {
        let result = resolve_credentials(false, Some("admin"), None);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_credentials_missing_both() {
        let result = resolve_credentials(false, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_credentials_from_env() {
        std::env::set_var("GWRS_USER", "envuser");
        std::env::set_var("GWRS_PASS", "envpass");
        let (user, pass) = resolve_credentials(true, None, None).unwrap();
        assert_eq!(user, "envuser");
        assert_eq!(pass, "envpass");
        std::env::remove_var("GWRS_USER");
        std::env::remove_var("GWRS_PASS");
    }

    #[test]
    fn token_cache_serialization() {
        let cache = TokenCache {
            token: "test-token".to_string(),
            expires_at: Utc::now() + chrono::Duration::minutes(55),
            base_url: "http://localhost:24042".to_string(),
        };
        let json = serde_json::to_string(&cache).unwrap();
        let parsed: TokenCache = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.token, "test-token");
        assert_eq!(parsed.base_url, "http://localhost:24042");
    }
}
