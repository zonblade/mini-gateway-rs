use async_trait::async_trait;
use log::{debug, info};
use serde_json::json;
use std::collections::HashMap;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::app::gateway;
use crate::config::{self, GatewayNode, ProxyNode};
use crate::system::protocol::types::ConnectionParams;
use crate::system::protocol::ServiceProtocol;
use crate::system::terminator;

pub struct DataRegistry {
    name: String,
}

impl DataRegistry {
    fn proxy_data(payload: String) -> Result<(), serde_json::Error> {
        let checksum = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(payload.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        let proxy_data = serde_json::from_str::<Vec<ProxyNode>>(&payload);
        let proxy_data = match proxy_data {
            Ok(data) => data,
            Err(e) => {
                log::error!("Failed to parse proxy data: {}", e);
                return Err(e);
            }
        };
        config::RoutingData::ProxyID.xset(checksum);
        config::RoutingData::ProxyRouting.xset(proxy_data);
        // restart services
        terminator::service::init();
        Ok(())
    }

    fn gateway_data(payload: String) -> Result<(), serde_json::Error> {
        let checksum = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(payload.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        let gateway_data = serde_json::from_str::<Vec<GatewayNode>>(&payload);
        let gateway_data = match gateway_data {
            Ok(data) => {
                log::warn!("Parsed gateway data: {:?}", data);
                data
            },
            Err(e) => {
                log::error!("Failed to parse gateway data: {}", e);
                return Err(e);
            }
        };
        config::RoutingData::GatewayID.xset(checksum);
        config::RoutingData::GatewayRouting.xset(gateway_data);
        Ok(())
    }
}

#[async_trait]
impl ServiceProtocol for DataRegistry {
    fn new() -> Self {
        Self {
            name: "registry".to_string(),
        }
    }

    async fn upstream_peer(
        &self,
        socket: &mut TcpStream,
        buffer: &[u8],
        _buffer_size: usize,
        params: &ConnectionParams,
    ) -> io::Result<()> {
        let request_str = String::from_utf8_lossy(buffer);
        info!("Received request: {}", request_str);
        let action = &params.action;

        let response = match (action.as_str(), request_str.as_ref()) {
            ("proxy", payload) => match Self::proxy_data(payload.to_string()) {
                Ok(()) => json!({
                    "status": "success",
                    "message": "Proxy data updated successfully"
                }),
                Err(e) => json!({
                    "status": "error",
                    "message": format!("Failed to parse proxy data: {}", e)
                }),
            },
            ("gateway", payload) => match Self::gateway_data(payload.to_string()) {
                Ok(()) => json!({
                    "status": "success",
                    "message": "Gateway data updated successfully"
                }),
                Err(e) => json!({
                    "status": "error",
                    "message": format!("Failed to parse gateway data: {}", e)
                }),
            },
            _ => {
                json!({
                    "status": "error",
                    "message": "Unknown action"
                })
            }
        };
        let response_str = response.to_string();
        socket.write_all(response_str.as_bytes()).await?;
        socket.flush().await?;

        Ok(())
    }

    async fn logging(
        &self,
        params: &ConnectionParams,
        status: Option<&str>,
        metrics: Option<HashMap<String, String>>,
    ) {
        let service = &params.service;
        let action = &params.action;

        let status_str = status.unwrap_or("unknown");

        let metrics_info = match metrics {
            Some(m) => {
                let mut info = String::new();
                for (k, v) in m {
                    info.push_str(&format!("{}={}, ", k, v));
                }
                info
            }
            None => "no metrics".to_string(),
        };

        info!(
            "Request [{}]: service={}, action={}, status={}, metrics=[{}]",
            self.name, service, action, status_str, metrics_info
        );
    }
}
