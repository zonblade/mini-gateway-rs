use crate::client::ApiClient;
use crate::common::SyncResponse;
use crate::error::CliError;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum SyncAction {
    /// Sync proxy nodes to core
    Proxy,
    /// Sync gateway nodes to core
    Gateway,
}

pub async fn run(client: &mut ApiClient, action: &SyncAction) -> Result<(), CliError> {
    match action {
        SyncAction::Proxy => {
            let resp: SyncResponse = client
                .post_json("/api/v1/sync/proxy", &serde_json::Value::Null)
                .await?;
            println!("{}: {}", resp.status, resp.message);
            Ok(())
        }
        SyncAction::Gateway => {
            let resp: SyncResponse = client
                .post_json("/api/v1/sync/gateway", &serde_json::Value::Null)
                .await?;
            println!("{}: {}", resp.status, resp.message);
            Ok(())
        }
    }
}
