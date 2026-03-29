#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("config error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
