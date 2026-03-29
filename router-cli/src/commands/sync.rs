use crate::cli::SyncAction;
use crate::client::ApiClient;
use crate::error::CliError;
use crate::models::SyncResponse;

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
