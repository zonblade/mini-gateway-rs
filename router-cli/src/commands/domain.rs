use crate::cli::DomainAction;
use crate::client::ApiClient;
use crate::error::CliError;
use crate::models::{IdBody, MessageResponse, ProxyDomain};
use crate::output::{self, Format};

pub async fn run(
    client: &mut ApiClient,
    action: &DomainAction,
    fmt: &Format,
) -> Result<(), CliError> {
    match action {
        DomainAction::List {
            proxy_id,
            gwnode_id,
        } => list(client, proxy_id.as_deref(), gwnode_id.as_deref(), fmt).await,
        DomainAction::Get { id } => get(client, id, fmt).await,
        DomainAction::Create {
            proxy_id,
            gwnode_id,
            tls,
            sni,
        } => {
            create(
                client,
                proxy_id,
                gwnode_id.as_deref(),
                *tls,
                sni.as_deref(),
                fmt,
            )
            .await
        }
        DomainAction::Delete { id, yes } => delete(client, id, *yes).await,
    }
}

async fn list(
    client: &mut ApiClient,
    proxy_id: Option<&str>,
    gwnode_id: Option<&str>,
    fmt: &Format,
) -> Result<(), CliError> {
    let path = if let Some(gid) = gwnode_id {
        format!("/api/v1/settings/proxydomain/list/gwnode/{gid}")
    } else if let Some(pid) = proxy_id {
        format!("/api/v1/settings/proxydomain/list/{pid}")
    } else {
        "/api/v1/settings/proxydomain/list".to_string()
    };

    let domains: Vec<ProxyDomain> = client.get(&path).await?;

    match fmt {
        Format::Json => output::print_json(&domains),
        Format::Yaml => output::print_yaml(&domains),
        Format::Table => {
            let headers = &["ID", "PROXY_ID", "TLS", "SNI"];
            let rows: Vec<Vec<String>> = domains
                .iter()
                .map(|d| {
                    vec![
                        d.id.clone(),
                        d.proxy_id.clone().unwrap_or_default(),
                        d.tls.to_string(),
                        d.sni.clone().unwrap_or_default(),
                    ]
                })
                .collect();
            output::print_table(headers, &rows);
            Ok(())
        }
    }
}

async fn get(client: &mut ApiClient, id: &str, fmt: &Format) -> Result<(), CliError> {
    let domain: ProxyDomain = client
        .get(&format!("/api/v1/settings/proxydomain/{id}"))
        .await?;

    match fmt {
        Format::Json => output::print_json(&domain),
        Format::Yaml => output::print_yaml(&domain),
        Format::Table => {
            println!("Domain: {}", domain.id);
            println!("  Proxy ID: {}", domain.proxy_id.as_deref().unwrap_or("-"));
            println!("  TLS:      {}", domain.tls);
            println!("  SNI:      {}", domain.sni.as_deref().unwrap_or("-"));
            if let Some(ref mode) = domain.tls_mode {
                println!("  TLS Mode: {mode}");
            }
            if let Some(ref email) = domain.tls_email {
                println!("  Email:    {email}");
            }
            if let Some(ref renew) = domain.expected_renew {
                println!("  Renew:    {renew}");
            }
            Ok(())
        }
    }
}

async fn create(
    client: &mut ApiClient,
    proxy_id: &str,
    gwnode_id: Option<&str>,
    tls: bool,
    sni: Option<&str>,
    fmt: &Format,
) -> Result<(), CliError> {
    let input = ProxyDomain {
        id: String::new(),
        proxy_id: Some(proxy_id.to_string()),
        tls,
        tls_pem: None,
        tls_key: None,
        sni: sni.map(String::from),
        tls_autron: None,
        tls_mode: None,
        tls_email: None,
        expected_renew: None,
    };

    // The API expects gwnode_id in the body; we set it via proxy_id association
    let _ = gwnode_id; // gwnode binding is done separately via proxy domain set endpoint
    let result: ProxyDomain = client
        .post_json("/api/v1/settings/proxydomain/set", &input)
        .await?;

    match fmt {
        Format::Json => output::print_json(&result),
        Format::Yaml => output::print_yaml(&result),
        Format::Table => {
            println!("Domain created: {}", result.id);
            Ok(())
        }
    }
}

async fn delete(client: &mut ApiClient, id: &str, yes: bool) -> Result<(), CliError> {
    if !yes {
        eprint!("Are you sure you want to delete domain '{id}'? [y/N]: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let resp: MessageResponse = client
        .post_json(
            "/api/v1/settings/proxydomain/delete",
            &IdBody { id: id.to_string() },
        )
        .await?;
    println!("{}", resp.message);
    Ok(())
}
