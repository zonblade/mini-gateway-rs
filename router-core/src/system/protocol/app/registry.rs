use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::config::{self, GatewayNode, ProxyNode};
use crate::system::protocol::types::ConnectionParams;
use crate::system::protocol::ServiceProtocol;
use crate::system::terminator;

pub struct DataRegistry {
    name: String,
}

impl DataRegistry {
    // path for (pem, key)
    fn save_tls(data: ProxyNode, pem: String, key: String) -> (String, String) {
        let data_str = match serde_json::to_string(&data) {
            Ok(data) => data,
            Err(e) => {
                log::error!("Failed to serialize proxy data: {}", e);
                return (String::new(), String::new());
            }
        };
        let checksum = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(data_str.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        // save to folder /tmp/gwrs/cert
        // if folder not exist, create it
        // if file not exist, create it
        let path = format!("/tmp/gwrs/cert/{}", checksum);
        let pem_path = format!("{}/{}.pem", path, checksum);
        let key_path = format!("{}/{}.key", path, checksum);

        // save pem and key to file
        match std::fs::create_dir_all(&path) {
            Ok(_) => {
                match std::fs::write(&pem_path, pem) {
                    Ok(_) => log::debug!("PEM file saved to {}", pem_path),
                    Err(e) => log::error!("Failed to save PEM file: {}", e),
                }
                match std::fs::write(&key_path, key) {
                    Ok(_) => log::debug!("Key file saved to {}", key_path),
                    Err(e) => log::error!("Failed to save Key file: {}", e),
                }
            }
            Err(e) => log::error!("Failed to create directory {}: {}", path, e),
        }

        (pem_path, key_path)
    }

    fn proxy_data(payload: String) -> Result<(), serde_json::Error> {
        let checksum = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(payload.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        let proxy_data = serde_json::from_str::<Vec<ProxyNode>>(&payload);
        let proxy_data = match proxy_data {
            Ok(data) => {
                let mut data_node = Vec::new();
                for node in data {
                    log::debug!("Parsed proxy data: {:#?}", node.clone());
                    let mut tls_key = None;
                    let mut tls_pem = None;
                    if node.tls {
                        let (pem_path, key_path) = Self::save_tls(
                            node.clone(),
                            node.tls_pem.unwrap_or_default(),
                            node.tls_key.unwrap_or_default(),
                        );
                        tls_key = Some(key_path);
                        tls_pem = Some(pem_path);
                    }
                    data_node.push(ProxyNode {
                        tls_pem,
                        tls_key,
                        ..node
                    });
                }
                data_node
            }
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
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(payload.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        let gateway_data = serde_json::from_str::<Vec<GatewayNode>>(&payload);
        let gateway_data = match gateway_data {
            Ok(data) => {
                log::warn!("Parsed gateway data: {:?}", data);
                data
            }
            Err(e) => {
                log::error!("Failed to parse gateway data: {}", e);
                return Err(e);
            }
        };
        log::debug!("Parsed gateway data: {:#?}", gateway_data);
        config::RoutingData::GatewayID.set(&checksum);
        config::RoutingData::GatewayRouting.xset(&gateway_data);
        sleep(Duration::from_millis(500));
        terminator::service::init();
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
       log::debug!("Received request: {}", request_str);
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

       log::debug!(
            "Request [{}]: service={}, action={}, status={}, metrics=[{}]",
            self.name, service, action, status_str, metrics_info
        );
    }
}
