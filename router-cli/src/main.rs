mod auth;
mod cli;
mod client;
mod commands;
mod error;
mod models;
mod output;

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
            let mut client = authed_client(&cli).await?;
            commands::config::run(&mut client, config).await
        }
        Some(Commands::Export { output }) => {
            let output_path = output
                .clone()
                .unwrap_or_else(|| PathBuf::from("gateway-config.yaml"));
            let mut client = authed_client(&cli).await?;
            commands::export::run(&mut client, &output_path).await
        }
        Some(Commands::Proxy { action }) => {
            let mut client = authed_client(&cli).await?;
            commands::proxy::run(&mut client, action, &cli.format).await
        }
        Some(Commands::Domain { action }) => {
            let mut client = authed_client(&cli).await?;
            commands::domain::run(&mut client, action, &cli.format).await
        }
        Some(Commands::Gwnode { action }) => {
            let mut client = authed_client(&cli).await?;
            commands::gwnode::run(&mut client, action, &cli.format).await
        }
        Some(Commands::Gateway { action }) => {
            let mut client = authed_client(&cli).await?;
            commands::gateway::run(&mut client, action, &cli.format).await
        }
        Some(Commands::User { action }) => {
            let mut client = authed_client(&cli).await?;
            commands::user::run(&mut client, action, &cli.format).await
        }
        Some(Commands::Cert { action }) => {
            let mut client = authed_client(&cli).await?;
            commands::cert::run(&mut client, action, &cli.format).await
        }
        Some(Commands::Sync { action }) => {
            let mut client = authed_client(&cli).await?;
            commands::sync::run(&mut client, action).await
        }
        None => {
            if let Some(config) = &cli.config {
                let mut client = authed_client(&cli).await?;
                commands::config::run(&mut client, config).await
            } else {
                Err(CliError::Config(
                    "no command specified. Use --help for usage information".into(),
                ))
            }
        }
    }
}

async fn authed_client(cli: &Cli) -> Result<ApiClient, CliError> {
    debug!("Using API URL: {}", cli.url);
    let mut client = ApiClient::new(&cli.url, cli.no_cache);
    let (user, pass) =
        auth::resolve_credentials(cli.osenv, cli.user.as_deref(), cli.pass.as_deref())?;
    client.authenticate(&user, &pass).await?;
    Ok(client)
}
