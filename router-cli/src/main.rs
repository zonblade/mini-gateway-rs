use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use reqwest::{blocking::Client, header};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::File,
    io::Read,
    path::PathBuf,
};

/// Mini-Gateway Router CLI Tool
#[derive(Parser)]
#[command(name = "gwrs")]
#[command(about = "CLI tool for Mini-Gateway Router API", long_about = None)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    /// Use credentials from OS environment variables (GWRS_USER, GWRS_PASS)
    #[arg(long, global = true)]
    osenv: bool,

    /// Username for API authentication
    #[arg(short, long, global = true)]
    user: Option<String>,

    /// Password for API authentication
    #[arg(short, long, global = true)]
    pass: Option<String>,

    /// API base URL (default: http://localhost:3000)
    #[arg(long, global = true, default_value = "http://localhost:24042")]
    api_url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Upload a configuration file to the router
    Config {
        /// Path to the configuration file
        #[arg(long, required = true)]
        config: PathBuf,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginResponse {
    success: bool,
    token: Option<String>,
    user_id: Option<String>,
    username: Option<String>,
    role: Option<String>,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigUploadResponse {
    success: bool,
    created: Option<ConfigCreated>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigCreated {
    proxies: usize,
    domains: usize,
    gwnodes: usize,
    gateways: usize,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    // Get credentials
    let (username, password) = get_credentials(&cli)?;

    debug!("Using API URL: {}", cli.api_url);
    debug!("Using username: {}", username);

    // Create HTTP client
    let client = Client::new();

    // Authenticate and get token
    let token = authenticate(&client, &cli.api_url, &username, &password)?;
    debug!("Authentication successful, token received");

    // Process command
    match &cli.command {
        Commands::Config { config } => {
            upload_config(&client, &cli.api_url, &token, config)?;
        }
    }

    Ok(())
}

fn get_credentials(cli: &Cli) -> Result<(String, String)> {
    if cli.osenv {
        debug!("Getting credentials from environment variables");
        let username = env::var("GWRS_USER").context("GWRS_USER environment variable not set")?;
        let password = env::var("GWRS_PASS").context("GWRS_PASS environment variable not set")?;
        Ok((username, password))
    } else if let (Some(user), Some(pass)) = (&cli.user, &cli.pass) {
        debug!("Using credentials from command line arguments");
        Ok((user.clone(), pass.clone()))
    } else {
        error!("No credentials provided. Use --osenv or provide --user and --pass");
        anyhow::bail!("No credentials provided. Use --osenv or provide --user and --pass");
    }
}

fn authenticate(client: &Client, base_url: &str, username: &str, password: &str) -> Result<String> {
    info!("Authenticating with username: {}", username);

    let login_url = format!("{}/api/v1/users/login", base_url);
    let login_request = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };

    let response = client
        .post(&login_url)
        .json(&login_request)
        .send()
        .context("Failed to send login request")?;

    let login_response = response
        .json::<LoginResponse>()
        .context("Failed to parse login response")?;

    if !login_response.success {
        error!("Authentication failed: {}", login_response.message);
        anyhow::bail!("Authentication failed: {}", login_response.message);
    }

    match login_response.token {
        Some(token) => Ok(token),
        None => {
            error!("No token received despite successful login");
            anyhow::bail!("No token received despite successful login");
        }
    }
}

fn upload_config(client: &Client, base_url: &str, token: &str, config_path: &PathBuf) -> Result<()> {
    info!("Uploading configuration from: {}", config_path.display());

    // Read the configuration file
    let mut file = File::open(config_path).context("Failed to open configuration file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .context("Failed to read configuration file")?;

    // Validate YAML format
    if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(&contents) {
        error!("Invalid YAML format: {}", e);
        anyhow::bail!("Invalid YAML format: {}", e);
    }

    // Prepare request
    let upload_url = format!("{}/api/v1/auto-config", base_url);
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/yaml"),
    );

    // Send request
    let response = client
        .post(&upload_url)
        .headers(headers)
        .body(contents)
        .send()
        .context("Failed to send configuration upload request")?;

    // Check status
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
        error!("Upload failed with status {}: {}", status, error_text);
        anyhow::bail!("Upload failed with status {}: {}", status, error_text);
    }

    let upload_response = response
        .json::<ConfigUploadResponse>()
        .context("Failed to parse upload response")?;

    if let Some(error) = upload_response.error {
        error!("Upload failed: {}", error);
        anyhow::bail!("Upload failed: {}", error);
    }

    if let Some(created) = upload_response.created {
        info!(
            "Configuration uploaded successfully! Created: {} proxies, {} domains, {} gateway nodes, {} gateways",
            created.proxies, created.domains, created.gwnodes, created.gateways
        );
        println!(
            "Configuration uploaded successfully! Created: {} proxies, {} domains, {} gateway nodes, {} gateways",
            created.proxies, created.domains, created.gwnodes, created.gateways
        );
    } else {
        info!("Configuration uploaded successfully!");
        println!("Configuration uploaded successfully!");
    }

    Ok(())
}
