use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Mini-Gateway Router CLI Tool
#[derive(Parser)]
#[command(name = "gwrs")]
#[command(about = "CLI tool for Mini-Gateway Router API", long_about = None)]
pub struct Cli {
    /// Path to the configuration file (shorthand for `gwrs config <path>`)
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Use credentials from OS environment variables (GWRS_USER, GWRS_PASS)
    #[arg(long, global = true)]
    pub osenv: bool,

    /// Username for API authentication
    #[arg(short, long, global = true)]
    pub user: Option<String>,

    /// Password for API authentication
    #[arg(short, long, global = true)]
    pub pass: Option<String>,

    /// API base URL
    #[arg(long, global = true, default_value = "http://localhost:24042")]
    pub url: String,

    /// Skip token cache, force fresh authentication
    #[arg(long, global = true)]
    pub no_cache: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// Export configuration from the router
    Export {
        /// Output file location (default: ./gateway-config.yaml)
        #[arg(value_name = "OUTPUT")]
        output: Option<PathBuf>,
    },
}
