use crate::client::ApiClient;
use crate::error::CliError;
use log::info;
use std::path::Path;

pub async fn run(client: &mut ApiClient, output_path: &Path) -> Result<(), CliError> {
    info!("Downloading configuration to: {}", output_path.display());

    let contents: String = client.get_string("/api/v1/settings/auto-config").await?;

    // Validate YAML format
    serde_yaml::from_str::<serde_yaml::Value>(&contents)?;

    std::fs::write(output_path, &contents)?;

    println!(
        "Configuration downloaded successfully to {}",
        output_path.display()
    );

    Ok(())
}
