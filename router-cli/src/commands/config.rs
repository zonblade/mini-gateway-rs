use crate::client::ApiClient;
use crate::error::CliError;
use log::info;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ConfigUploadResponse {
    created: Option<ConfigCreated>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConfigCreated {
    proxies: usize,
    domains: usize,
    gwnodes: usize,
    gateways: usize,
}

pub async fn run(client: &mut ApiClient, config_path: &Path) -> Result<(), CliError> {
    info!("Uploading configuration from: {}", config_path.display());

    let contents = std::fs::read_to_string(config_path)
        .map_err(|e| CliError::Config(format!("failed to read config file: {e}")))?;

    serde_yaml::from_str::<serde_yaml::Value>(&contents)?;

    let resp: ConfigUploadResponse = client
        .post_string(
            "/api/v1/settings/auto-config",
            &contents,
            "application/yaml",
        )
        .await?;

    if let Some(error) = resp.error {
        return Err(CliError::Api {
            status: 400,
            message: error,
        });
    }

    if let Some(created) = resp.created {
        println!(
            "Configuration uploaded successfully! Created: {} proxies, {} domains, {} gateway nodes, {} gateways",
            created.proxies, created.domains, created.gwnodes, created.gateways
        );
    } else {
        println!("Configuration uploaded successfully!");
    }

    Ok(())
}
