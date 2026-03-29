use crate::error::CliError;
use log::info;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const CONFIG_TEMPLATE: &str = include_str!("../../templates/router-config.yaml");

pub fn run(location: &Path) -> Result<(), CliError> {
    info!("Initializing configuration file in: {}", location.display());

    let config_path = location.join("router-config.yaml");

    let mut file = File::create(&config_path)
        .map_err(|e| CliError::Config(format!("failed to create config file: {e}")))?;

    file.write_all(CONFIG_TEMPLATE.as_bytes())?;

    println!("Configuration file created at: {}", config_path.display());
    println!("\nTo use this configuration:");
    println!("1. Edit the file to match your setup");
    println!("2. Use 'gwrs config router-config.yaml' to upload it");
    println!("3. Add authentication with --user/--pass or --osenv");
    println!("\nSee the comments in the file for field descriptions.");

    Ok(())
}
