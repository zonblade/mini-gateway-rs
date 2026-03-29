pub mod models;

use crate::client::ApiClient;
use crate::common::{IdBody, MessageResponse};
use crate::error::CliError;
use crate::output::{self, Format};
use clap::Subcommand;
use models::Gateway;

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

pub async fn run(
    client: &mut ApiClient,
    action: &GatewayAction,
    fmt: &Format,
) -> Result<(), CliError> {
    match action {
        GatewayAction::List { gwnode_id } => list(client, gwnode_id.as_deref(), fmt).await,
        GatewayAction::Get { id } => get(client, id, fmt).await,
        GatewayAction::Create {
            gwnode_id,
            pattern,
            target,
            priority,
        } => create(client, gwnode_id, pattern, target, *priority, fmt).await,
        GatewayAction::Delete { id, yes } => delete(client, id, *yes).await,
    }
}

async fn list(
    client: &mut ApiClient,
    gwnode_id: Option<&str>,
    fmt: &Format,
) -> Result<(), CliError> {
    let path = match gwnode_id {
        Some(gid) => format!("/api/v1/settings/gateway/list/{gid}"),
        None => "/api/v1/settings/gateway/list".to_string(),
    };

    let gateways: Vec<Gateway> = client.get(&path).await?;

    match fmt {
        Format::Json => output::print_json(&gateways),
        Format::Yaml => output::print_yaml(&gateways),
        Format::Table => {
            let headers = &["ID", "GWNODE_ID", "PATTERN", "TARGET", "PRIORITY"];
            let rows: Vec<Vec<String>> = gateways
                .iter()
                .map(|g| {
                    vec![
                        g.id.clone(),
                        g.gwnode_id.clone(),
                        g.pattern.clone(),
                        g.target.clone(),
                        g.priority.to_string(),
                    ]
                })
                .collect();
            output::print_table(headers, &rows);
            Ok(())
        }
    }
}

async fn get(client: &mut ApiClient, id: &str, fmt: &Format) -> Result<(), CliError> {
    let gw: Gateway = client
        .get(&format!("/api/v1/settings/gateway/{id}"))
        .await?;

    match fmt {
        Format::Json => output::print_json(&gw),
        Format::Yaml => output::print_yaml(&gw),
        Format::Table => {
            println!("Gateway: {}", gw.id);
            println!("  GWNode ID: {}", gw.gwnode_id);
            println!("  Pattern:   {}", gw.pattern);
            println!("  Target:    {}", gw.target);
            println!("  Priority:  {}", gw.priority);
            Ok(())
        }
    }
}

async fn create(
    client: &mut ApiClient,
    gwnode_id: &str,
    pattern: &str,
    target: &str,
    priority: i32,
    fmt: &Format,
) -> Result<(), CliError> {
    let input = Gateway {
        id: String::new(),
        gwnode_id: gwnode_id.to_string(),
        pattern: pattern.to_string(),
        target: target.to_string(),
        priority,
    };

    let result: Gateway = client
        .post_json("/api/v1/settings/gateway/set", &input)
        .await?;

    match fmt {
        Format::Json => output::print_json(&result),
        Format::Yaml => output::print_yaml(&result),
        Format::Table => {
            println!(
                "Gateway created: {} (pattern: {})",
                result.id, result.pattern
            );
            Ok(())
        }
    }
}

async fn delete(client: &mut ApiClient, id: &str, yes: bool) -> Result<(), CliError> {
    if !yes {
        eprint!("Are you sure you want to delete gateway '{id}'? [y/N]: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let resp: MessageResponse = client
        .post_json(
            "/api/v1/settings/gateway/delete",
            &IdBody { id: id.to_string() },
        )
        .await?;
    println!("{}", resp.message);
    Ok(())
}
