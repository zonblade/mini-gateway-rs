pub mod models;

use crate::client::ApiClient;
use crate::common::SyncResponse;
use crate::error::CliError;
use crate::output::{self, Format};
use clap::Subcommand;
use models::{CertGenerateRequest, CertGenerateResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct DueForRenewalItem {
    domain: Option<String>,
    expected_renew: Option<String>,
}

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

pub async fn run(
    client: &mut ApiClient,
    action: &CertAction,
    fmt: &Format,
) -> Result<(), CliError> {
    match action {
        CertAction::Generate {
            domain,
            proxy_id,
            email,
            staging,
        } => generate(client, domain, proxy_id, email.as_deref(), *staging, fmt).await,
        CertAction::RenewAll => renew_all(client, fmt).await,
        CertAction::DueForRenewal => due_for_renewal(client, fmt).await,
    }
}

async fn generate(
    client: &mut ApiClient,
    domain: &str,
    proxy_id: &str,
    email: Option<&str>,
    staging: bool,
    fmt: &Format,
) -> Result<(), CliError> {
    let input = CertGenerateRequest {
        domain: domain.to_string(),
        proxy_id: proxy_id.to_string(),
        email: email.map(String::from),
        staging: if staging { Some(true) } else { None },
    };

    let result: CertGenerateResponse = client
        .post_json("/api/v1/settings/certificates/generate", &input)
        .await?;

    match fmt {
        Format::Json => output::print_json(&result),
        Format::Yaml => output::print_yaml(&result),
        Format::Table => {
            println!("Status: {}", result.status);
            println!("Message: {}", result.message);
            if let Some(ref d) = result.domain {
                println!("Domain: {d}");
            }
            if let Some(ref r) = result.expected_renew {
                println!("Expected Renewal: {r}");
            }
            Ok(())
        }
    }
}

async fn renew_all(client: &mut ApiClient, fmt: &Format) -> Result<(), CliError> {
    let result: SyncResponse = client
        .post_json(
            "/api/v1/settings/certificates/renew-all",
            &serde_json::Value::Null,
        )
        .await?;

    match fmt {
        Format::Json => output::print_json(&result),
        Format::Yaml => output::print_yaml(&result),
        Format::Table => {
            println!("{}: {}", result.status, result.message);
            Ok(())
        }
    }
}

async fn due_for_renewal(client: &mut ApiClient, fmt: &Format) -> Result<(), CliError> {
    let items: Vec<DueForRenewalItem> = client
        .get("/api/v1/settings/certificates/due-for-renewal")
        .await?;

    match fmt {
        Format::Json => output::print_json(&items),
        Format::Yaml => output::print_yaml(&items),
        Format::Table => {
            let headers = &["DOMAIN", "EXPECTED_RENEW"];
            let rows: Vec<Vec<String>> = items
                .iter()
                .map(|i| {
                    vec![
                        i.domain.clone().unwrap_or_default(),
                        i.expected_renew.clone().unwrap_or_default(),
                    ]
                })
                .collect();
            output::print_table(headers, &rows);
            Ok(())
        }
    }
}
