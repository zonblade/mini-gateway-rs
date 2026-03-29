pub mod models;

use crate::client::ApiClient;
use crate::error::CliError;
use crate::output::{self, Format};
use clap::Subcommand;
use models::{Proxy, ProxyDetail, ProxyInput, ProxyWithDomains};

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

pub async fn run(
    client: &mut ApiClient,
    action: &ProxyAction,
    fmt: &Format,
) -> Result<(), CliError> {
    match action {
        ProxyAction::List => list(client, fmt).await,
        ProxyAction::Get { id } => get(client, id, fmt).await,
        ProxyAction::Create { .. } => create(client, action, fmt).await,
        ProxyAction::Delete { id, yes } => delete(client, id, *yes).await,
    }
}

async fn list(client: &mut ApiClient, fmt: &Format) -> Result<(), CliError> {
    let proxies: Vec<ProxyWithDomains> = client.get("/api/v1/settings/proxies").await?;

    match fmt {
        Format::Json => output::print_json(&proxies),
        Format::Yaml => output::print_yaml(&proxies),
        Format::Table => {
            let headers = &["ID", "TITLE", "LISTEN", "TARGET", "HIGH_SPEED", "DOMAINS"];
            let rows: Vec<Vec<String>> = proxies
                .iter()
                .map(|p| {
                    vec![
                        p.proxy.id.clone(),
                        p.proxy.title.clone(),
                        p.proxy.addr_listen.clone(),
                        p.proxy.addr_target.clone(),
                        p.proxy.high_speed.to_string(),
                        p.domains.len().to_string(),
                    ]
                })
                .collect();
            output::print_table(headers, &rows);
            Ok(())
        }
    }
}

async fn get(client: &mut ApiClient, id: &str, fmt: &Format) -> Result<(), CliError> {
    let proxy: ProxyDetail = client.get(&format!("/api/v1/settings/proxy/{id}")).await?;

    match fmt {
        Format::Json => output::print_json(&proxy),
        Format::Yaml => output::print_yaml(&proxy),
        Format::Table => {
            println!("Proxy: {}", proxy.proxy.title);
            println!("  ID:         {}", proxy.proxy.id);
            println!("  Listen:     {}", proxy.proxy.addr_listen);
            println!("  Target:     {}", proxy.proxy.addr_target);
            println!("  High Speed: {}", proxy.proxy.high_speed);
            if let Some(ref addr) = proxy.proxy.high_speed_addr {
                println!("  HS Addr:    {addr}");
            }
            if !proxy.domains.is_empty() {
                println!("  Domains:");
                for d in &proxy.domains {
                    let sni = d.sni.as_deref().unwrap_or("-");
                    println!("    - {} (tls: {}, sni: {})", d.id, d.tls, sni);
                }
            }
            Ok(())
        }
    }
}

async fn create(
    client: &mut ApiClient,
    action: &ProxyAction,
    fmt: &Format,
) -> Result<(), CliError> {
    let ProxyAction::Create {
        title,
        listen,
        target,
        high_speed,
        high_speed_addr,
        high_speed_gwid,
    } = action
    else {
        unreachable!()
    };

    let input = ProxyInput {
        proxy: Proxy {
            id: String::new(),
            title: title.to_string(),
            addr_listen: listen.to_string(),
            addr_target: target.to_string(),
            high_speed: *high_speed,
            high_speed_addr: high_speed_addr.clone(),
            high_speed_gwid: high_speed_gwid.clone(),
        },
        domains: Vec::new(),
    };

    let result: ProxyDetail = client.post_json("/api/v1/settings/proxy", &input).await?;

    match fmt {
        Format::Json => output::print_json(&result),
        Format::Yaml => output::print_yaml(&result),
        Format::Table => {
            println!(
                "Proxy created: {} ({})",
                result.proxy.title, result.proxy.id
            );
            Ok(())
        }
    }
}

async fn delete(client: &mut ApiClient, id: &str, yes: bool) -> Result<(), CliError> {
    if !yes {
        eprint!("Are you sure you want to delete proxy '{id}'? [y/N]: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let msg: String = client
        .delete_text(&format!("/api/v1/settings/proxy/{id}"))
        .await?;
    println!("{msg}");
    Ok(())
}
