use crate::output::Format;
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

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "table")]
    pub format: Format,

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
    /// Manage proxies
    Proxy {
        #[command(subcommand)]
        action: ProxyAction,
    },
    /// Manage proxy domains
    Domain {
        #[command(subcommand)]
        action: DomainAction,
    },
    /// Manage gateway nodes
    Gwnode {
        #[command(subcommand)]
        action: GwnodeAction,
    },
    /// Manage gateway routing rules
    Gateway {
        #[command(subcommand)]
        action: GatewayAction,
    },
    /// Manage users
    User {
        #[command(subcommand)]
        action: UserAction,
    },
    /// Manage TLS certificates
    Cert {
        #[command(subcommand)]
        action: CertAction,
    },
    /// Sync configuration to core nodes
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },
}

// --- Proxy ---

#[derive(Subcommand)]
pub enum ProxyAction {
    /// List all proxies
    List,
    /// Get a proxy by ID
    Get { id: String },
    /// Create a new proxy
    Create {
        /// Proxy title
        #[arg(long)]
        title: String,
        /// Listen address (e.g. 0.0.0.0:443)
        #[arg(long)]
        listen: String,
        /// Target address (e.g. 127.0.0.1:8080)
        #[arg(long, default_value = "")]
        target: String,
        /// Enable high-speed mode
        #[arg(long)]
        high_speed: bool,
        /// High-speed target address
        #[arg(long)]
        high_speed_addr: Option<String>,
        /// High-speed gateway node ID
        #[arg(long)]
        high_speed_gwid: Option<String>,
    },
    /// Delete a proxy
    Delete {
        id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

// --- Domain ---

#[derive(Subcommand)]
pub enum DomainAction {
    /// List proxy domains
    List {
        /// Filter by proxy ID
        #[arg(long)]
        proxy_id: Option<String>,
        /// Filter by gateway node ID
        #[arg(long)]
        gwnode_id: Option<String>,
    },
    /// Get a proxy domain by ID
    Get { id: String },
    /// Create a new proxy domain
    Create {
        /// Proxy ID to associate with
        #[arg(long)]
        proxy_id: String,
        /// Gateway node ID to bind
        #[arg(long)]
        gwnode_id: Option<String>,
        /// Enable TLS
        #[arg(long)]
        tls: bool,
        /// SNI hostname
        #[arg(long)]
        sni: Option<String>,
    },
    /// Delete a proxy domain
    Delete {
        id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

// --- Gateway Node ---

#[derive(Subcommand)]
pub enum GwnodeAction {
    /// List gateway nodes
    List {
        /// Filter by proxy ID
        #[arg(long)]
        proxy_id: Option<String>,
    },
    /// Get a gateway node by ID
    Get { id: String },
    /// Create a new gateway node
    Create {
        /// Proxy ID to associate with
        #[arg(long)]
        proxy_id: String,
        /// Gateway node title
        #[arg(long)]
        title: String,
        /// Alternative target address (e.g. 127.0.0.1:3000)
        #[arg(long)]
        target: String,
        /// Priority (default: 100, higher = higher priority)
        #[arg(long, default_value = "100")]
        priority: i32,
    },
    /// Delete a gateway node
    Delete {
        id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

// --- Gateway ---

#[derive(Subcommand)]
pub enum GatewayAction {
    /// List gateways
    List {
        /// Filter by gateway node ID
        #[arg(long)]
        gwnode_id: Option<String>,
    },
    /// Get a gateway by ID
    Get { id: String },
    /// Create a new gateway routing rule
    Create {
        /// Gateway node ID to associate with
        #[arg(long)]
        gwnode_id: String,
        /// URL pattern (regex)
        #[arg(long)]
        pattern: String,
        /// Target URL pattern
        #[arg(long)]
        target: String,
        /// Priority (lower = higher priority)
        #[arg(long)]
        priority: i32,
    },
    /// Delete a gateway
    Delete {
        id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

// --- User ---

#[derive(Subcommand)]
pub enum UserAction {
    /// List all users (admin only)
    List,
    /// Get a user by ID
    Get { id: String },
    /// Create a new user (admin only)
    Create {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
        /// Role: admin, staff, or user
        #[arg(long, default_value = "user")]
        role: String,
    },
    /// Update a user
    Update {
        id: String,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        role: Option<String>,
    },
    /// Delete a user
    Delete {
        id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

// --- Certificate ---

#[derive(Subcommand)]
pub enum CertAction {
    /// Generate a TLS certificate via Let's Encrypt
    Generate {
        /// Domain name
        #[arg(long)]
        domain: String,
        /// Proxy ID to bind the certificate to
        #[arg(long)]
        proxy_id: String,
        /// Email for Let's Encrypt
        #[arg(long)]
        email: Option<String>,
        /// Use Let's Encrypt staging environment
        #[arg(long)]
        staging: bool,
    },
    /// Renew all certificates due for renewal
    RenewAll,
    /// List certificates due for renewal
    DueForRenewal,
}

// --- Sync ---

#[derive(Subcommand)]
pub enum SyncAction {
    /// Sync proxy nodes to core
    Proxy,
    /// Sync gateway nodes to core
    Gateway,
}
