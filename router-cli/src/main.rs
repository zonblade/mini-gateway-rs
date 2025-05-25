use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::{Read, Write}, path::PathBuf};

/// Mini-Gateway Router CLI Tool
#[derive(Parser)]
#[command(name = "gwrs")]
#[command(about = "CLI tool for Mini-Gateway Router API", long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(long)]
    config: Option<PathBuf>,

    /// Use credentials from OS environment variables (GWRS_USER, GWRS_PASS)
    #[arg(long, global = true)]
    osenv: bool,

    /// Username for API authentication
    #[arg(short, long, global = true)]
    user: Option<String>,

    /// Password for API authentication
    #[arg(short, long, global = true)]
    pass: Option<String>,

    /// API base URL (default: http://localhost:24042)
    #[arg(long, global = true, default_value = "http://localhost:24042")]
    url: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new configuration file
    Init {
        /// Location to create the configuration file (default: current directory)
        #[arg(value_name = "LOCATION")]
        location: Option<PathBuf>,
    },
    /// Upload configuration to the router
    Config {
        /// Path to the configuration file
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

    match cli.command {
        Some(Commands::Init { location }) => {
            init_config(&location.unwrap_or_else(|| PathBuf::from(".")))?;
        }
        Some(Commands::Config { config }) => {
            // Get credentials
            let (username, password) = get_credentials(&Credentials { 
                osenv: cli.osenv, 
                user: cli.user, 
                pass: cli.pass 
            })?;

            debug!("Using API URL: {}", cli.url);
            debug!("Using username: {}", username);

            // Authenticate and get token
            let token = authenticate(&cli.url, &username, &password)?;
            debug!("Authentication successful, token received");

            // Upload config
            upload_config(&cli.url, &token, &config)?;
        }
        None => {
            if let Some(config) = cli.config {
                // Get credentials
                let (username, password) = get_credentials(&Credentials { 
                    osenv: cli.osenv, 
                    user: cli.user, 
                    pass: cli.pass 
                })?;

                debug!("Using API URL: {}", cli.url);
                debug!("Using username: {}", username);

                // Authenticate and get token
                let token = authenticate(&cli.url, &username, &password)?;
                debug!("Authentication successful, token received");

                // Upload config
                upload_config(&cli.url, &token, &config)?;
            } else {
                error!("No configuration file specified. Use --config or the config subcommand");
                anyhow::bail!("No configuration file specified. Use --config or the config subcommand");
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Credentials {
    osenv: bool,
    user: Option<String>,
    pass: Option<String>,
}

fn get_credentials(cli: &Credentials) -> Result<(String, String)> {
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

fn init_config(location: &PathBuf) -> Result<()> {
    info!("Initializing configuration file in: {}", location.display());

    let config_path = location.join("router-config.yaml");
    let config_content = r#"# Mini-Gateway Router Configuration
# This file contains the configuration for your router setup

proxy:
  - name: "proxy1"
    listen: "127.0.0.1:8080"
    domains:
      - domain: "example.com"
        tls: false
        tls_cert: |
          -----BEGIN CERTIFICATE-----
          cert
          -----END CERTIFICATE-----
        tls_key: |
          -----BEGIN PRIVATE KEY-----
          key
          -----END PRIVATE KEY-----
    highspeed:
      enabled: true
      target: "gateway1"
    gateway:
      - name: "gateway1"
        domain: "example.com"
        target: "127.0.0.1:8080"
        path:
          - priority: 1
            pattern: "^(.*)$"
            target: "/$1"
"#;

    let mut file = File::create(&config_path)
        .context("Failed to create configuration file")?;
    
    file.write_all(config_content.as_bytes())
        .context("Failed to write configuration file")?;

    info!("Configuration file created at: {}", config_path.display());
    println!("Configuration file created at: {}", config_path.display());
    println!("\nConfiguration file structure explanation:");
    println!("1. proxy: Define your proxy servers with their settings");
    println!("   - name: Unique identifier for the proxy");
    println!("   - listen: Address and port to listen on");
    println!("   - domains: List of domains this proxy handles");
    println!("     - domain: Domain name");
    println!("     - tls: Enable/disable TLS");
    println!("     - tls_cert: TLS certificate (if tls is true)");
    println!("     - tls_key: TLS private key (if tls is true)");
    println!("   - highspeed: High-speed routing settings");
    println!("     - enabled: Enable/disable high-speed routing");
    println!("     - target: Target gateway for high-speed routing");
    println!("   - gateway: List of gateways for this proxy");
    println!("     - name: Gateway name");
    println!("     - domain: Domain for this gateway");
    println!("     - target: Target address and port");
    println!("     - path: URL path routing rules");
    println!("       - priority: Rule priority (lower numbers = higher priority)");
    println!("       - pattern: Regex pattern to match");
    println!("       - target: Target path pattern");
    println!("\nTo use this configuration:");
    println!("1. Edit the file to match your setup");
    println!("2. Use 'gwrs config router-config.yaml' to upload it");
    println!("3. Add authentication with --user/--pass or --osenv");

    Ok(())
}

fn authenticate(base_url: &str, username: &str, password: &str) -> Result<String> {
    info!("Authenticating with username: {}", username);

    let login_url = format!("{}/api/v1/users/login", base_url);
    let login_request = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };

    let response = ureq::post(&login_url)
        .send_json(ureq::json!(login_request))
        .context("Failed to send login request")?;

    let login_response = response
        .into_json::<LoginResponse>()
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

fn upload_config(
    base_url: &str,
    token: &str,
    config_path: &PathBuf,
) -> Result<()> {
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
    let upload_url = format!("{}/api/v1/settings/auto-config", base_url);

    // Send request
    let response = ureq::post(&upload_url)
        .set("Authorization", &format!("Bearer {}", token))
        .set("Content-Type", "application/yaml")
        .send_string(&contents)
        .context("Failed to send configuration upload request")?;

    // Check status
    let status = response.status();
    if status >= 400 {
        let error_text = response
            .into_string()
            .unwrap_or_else(|_| "Unknown error".to_string());
        error!("Upload failed with status {}: {}", status, error_text);
        anyhow::bail!("Upload failed with status {}: {}", status, error_text);
    }

    let upload_response = response
        .into_json::<ConfigUploadResponse>()
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
