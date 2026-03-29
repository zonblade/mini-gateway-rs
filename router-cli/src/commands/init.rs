use crate::error::CliError;
use log::info;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn run(location: &Path) -> Result<(), CliError> {
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
        tls_autron: false  # Enable automatic TLS certificate generation via Let's Encrypt
        tls_mode: "staging"  # TLS mode when tls_autron is true: "staging" or "prod"
        tls_email: "admin@example.com"  # Email for Let's Encrypt (required when tls_autron is true)
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
          - priority: 2
            pattern: "^/api/debug/(.*)$"
            target: "/debug/$1"
          - priority: 3
            pattern: "^/health$"
            target: "/health"
"#;

    let mut file = File::create(&config_path)
        .map_err(|e| CliError::Config(format!("failed to create config file: {e}")))?;

    file.write_all(config_content.as_bytes())?;

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
    println!("     - tls_autron: Enable automatic TLS via Let's Encrypt");
    println!("     - tls_mode: Certificate mode when tls_autron is true (\"staging\" or \"prod\")");
    println!("     - tls_email: Email for Let's Encrypt (required when tls_autron is true)");
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
