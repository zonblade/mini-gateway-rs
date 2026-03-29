pub mod models;

use crate::client::ApiClient;
use crate::common::{IdBody, MessageResponse};
use crate::error::CliError;
use crate::output::{self, Format};
use clap::Subcommand;
use models::GatewayNode;

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

pub async fn run(
    client: &mut ApiClient,
    action: &GwnodeAction,
    fmt: &Format,
) -> Result<(), CliError> {
    match action {
        GwnodeAction::List { proxy_id } => list(client, proxy_id.as_deref(), fmt).await,
        GwnodeAction::Get { id } => get(client, id, fmt).await,
        GwnodeAction::Create {
            proxy_id,
            title,
            target,
            priority,
        } => create(client, proxy_id, title, target, *priority, fmt).await,
        GwnodeAction::Delete { id, yes } => delete(client, id, *yes).await,
    }
}

async fn list(
    client: &mut ApiClient,
    proxy_id: Option<&str>,
    fmt: &Format,
) -> Result<(), CliError> {
    let path = match proxy_id {
        Some(pid) => format!("/api/v1/settings/gwnode/list/{pid}"),
        None => "/api/v1/settings/gwnode/list".to_string(),
    };

    let nodes: Vec<GatewayNode> = client.get(&path).await?;

    match fmt {
        Format::Json => output::print_json(&nodes),
        Format::Yaml => output::print_yaml(&nodes),
        Format::Table => {
            let headers = &["ID", "PROXY_ID", "TITLE", "TARGET", "PRIORITY"];
            let rows: Vec<Vec<String>> = nodes
                .iter()
                .map(|n| {
                    vec![
                        n.id.clone(),
                        n.proxy_id.clone(),
                        n.title.clone(),
                        n.alt_target.clone(),
                        n.priority.to_string(),
                    ]
                })
                .collect();
            output::print_table(headers, &rows);
            Ok(())
        }
    }
}

async fn get(client: &mut ApiClient, id: &str, fmt: &Format) -> Result<(), CliError> {
    let node: GatewayNode = client.get(&format!("/api/v1/settings/gwnode/{id}")).await?;

    match fmt {
        Format::Json => output::print_json(&node),
        Format::Yaml => output::print_yaml(&node),
        Format::Table => {
            println!("Gateway Node: {}", node.title);
            println!("  ID:       {}", node.id);
            println!("  Proxy ID: {}", node.proxy_id);
            println!("  Target:   {}", node.alt_target);
            println!("  Priority: {}", node.priority);
            if let Some(ref dn) = node.domain_name {
                println!("  Domain:   {dn}");
            }
            Ok(())
        }
    }
}

async fn create(
    client: &mut ApiClient,
    proxy_id: &str,
    title: &str,
    target: &str,
    priority: i32,
    fmt: &Format,
) -> Result<(), CliError> {
    let input = GatewayNode {
        id: String::new(),
        proxy_id: proxy_id.to_string(),
        title: title.to_string(),
        alt_target: target.to_string(),
        priority,
        domain_id: None,
        domain_name: None,
    };

    let result: GatewayNode = client
        .post_json("/api/v1/settings/gwnode/set", &input)
        .await?;

    match fmt {
        Format::Json => output::print_json(&result),
        Format::Yaml => output::print_yaml(&result),
        Format::Table => {
            println!("Gateway node created: {} ({})", result.title, result.id);
            Ok(())
        }
    }
}

async fn delete(client: &mut ApiClient, id: &str, yes: bool) -> Result<(), CliError> {
    if !yes {
        eprint!("Are you sure you want to delete gateway node '{id}'? [y/N]: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let resp: MessageResponse = client
        .post_json(
            "/api/v1/settings/gwnode/delete",
            &IdBody { id: id.to_string() },
        )
        .await?;
    println!("{}", resp.message);
    Ok(())
}
