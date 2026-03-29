mod auth;
mod cli;
mod client;
mod commands;
mod error;

use clap::Parser;
use cli::{Cli, Commands};
use client::ApiClient;
use error::CliError;
use log::debug;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    env_logger::init();
    if let Err(e) = run().await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { location }) => {
            let location = location.clone().unwrap_or_else(|| PathBuf::from("."));
            commands::init::run(&location)
        }
        Some(Commands::Config { config }) => {
            let mut client = build_client(&cli)?;
            let (user, pass) = client_creds(&cli)?;
            client.authenticate(&user, &pass).await?;
            commands::config::run(&mut client, config).await
        }
        Some(Commands::Export { output }) => {
            let output_path = output
                .clone()
                .unwrap_or_else(|| PathBuf::from("gateway-config.yaml"));
            let mut client = build_client(&cli)?;
            let (user, pass) = client_creds(&cli)?;
            client.authenticate(&user, &pass).await?;
            commands::export::run(&mut client, &output_path).await
        }
        None => {
            if let Some(config) = &cli.config {
                let mut client = build_client(&cli)?;
                let (user, pass) = client_creds(&cli)?;
                client.authenticate(&user, &pass).await?;
                commands::config::run(&mut client, config).await
            } else {
                Err(CliError::Config(
                    "no command specified. Use --help for usage information".into(),
                ))
            }
        }
    }
}

fn build_client(cli: &Cli) -> Result<ApiClient, CliError> {
    debug!("Using API URL: {}", cli.url);
    Ok(ApiClient::new(&cli.url, cli.no_cache))
}

fn client_creds(cli: &Cli) -> Result<(String, String), CliError> {
    auth::resolve_credentials(cli.osenv, cli.user.as_deref(), cli.pass.as_deref())
}
