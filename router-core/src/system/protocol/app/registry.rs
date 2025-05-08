use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::config::{self, GatewayNode, GatewayNodeSNI, GatewayPath, ProxyNode};
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
        let key_path = format!("{}/{}-key.pem", path, checksum);

        // save pem and key to file
        match std::fs::create_dir_all(&path) {
            Ok(_) => {
                // Set directory permissions to 700
                if let Err(e) = std::fs::set_permissions(&path, Permissions::from_mode(0o700)) {
                    log::error!("Failed to set directory permissions: {}", e);
                }

                match std::fs::write(&pem_path, pem) {
                    Ok(_) => {
                        log::debug!("PEM file saved to {}", pem_path);
                        // Set certificate permissions to 644
                        if let Err(e) =
                            std::fs::set_permissions(&pem_path, Permissions::from_mode(0o644))
                        {
                            log::error!("Failed to set PEM file permissions: {}", e);
                        }
                    }
                    Err(e) => log::error!("Failed to save PEM file: {}", e),
                }

                match std::fs::write(&key_path, key) {
                    Ok(_) => {
                        log::debug!("Key file saved to {}", key_path);
                        // Set key file permissions to 600
                        if let Err(e) =
                            std::fs::set_permissions(&key_path, Permissions::from_mode(0o600))
                        {
                            log::error!("Failed to set Key file permissions: {}", e);
                        }
                    }
                    Err(e) => log::error!("Failed to save Key file: {}", e),
                }
            }
            Err(e) => log::error!("Failed to create directory {}: {}", path, e),
        }

        (pem_path, key_path)
    }

    fn save_tls_gwnode(data: GatewayNodeSNI, pem: String, key: String) -> (String, String) {
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
        let key_path = format!("{}/{}-key.pem", path, checksum);

        // save pem and key to file
        match std::fs::create_dir_all(&path) {
            Ok(_) => {
                // Set directory permissions to 700
                if let Err(e) = std::fs::set_permissions(&path, Permissions::from_mode(0o700)) {
                    log::error!("Failed to set directory permissions: {}", e);
                }

                match std::fs::write(&pem_path, pem) {
                    Ok(_) => {
                        log::debug!("PEM file saved to {}", pem_path);
                        // Set certificate permissions to 644
                        if let Err(e) =
                            std::fs::set_permissions(&pem_path, Permissions::from_mode(0o644))
                        {
                            log::error!("Failed to set PEM file permissions: {}", e);
                        }
                    }
                    Err(e) => log::error!("Failed to save PEM file: {}", e),
                }

                match std::fs::write(&key_path, key) {
                    Ok(_) => {
                        log::debug!("Key file saved to {}", key_path);
                        // Set key file permissions to 600
                        if let Err(e) =
                            std::fs::set_permissions(&key_path, Permissions::from_mode(0o600))
                        {
                            log::error!("Failed to set Key file permissions: {}", e);
                        }
                    }
                    Err(e) => log::error!("Failed to save Key file: {}", e),
                }
            }
            Err(e) => log::error!("Failed to create directory {}: {}", path, e),
        }

        (pem_path, key_path)
    }

    /// now proxy data always accept high speed.
    fn proxy_data(payload: String) -> Result<(), serde_json::Error> {
        let checksum = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(payload.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        let checksum_old = config::RoutingData::ProxyID.val().clone();
        if checksum == checksum_old {
            log::info!("Gateway node id : {}", checksum);
            return Ok(());
        }
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

    // path stay as it is
    fn gateway_data(payload: String) -> Result<(), serde_json::Error> {
        let checksum = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(payload.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        eprintln!("[-TC-]   Gateway id : {}", checksum);

        let gateway_data = serde_json::from_str::<Vec<GatewayPath>>(&payload);
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

        let gateway_existing = match config::RoutingData::GatewayRouting.xget::<Vec<GatewayPath>>()
        {
            Some(data) => data,
            None => {
                vec![]
            }
        };

        eprintln!(
            "[-TC-]   Count of existing gateway addresses: {}",
            gateway_existing.len()
        );

        // Get existing gateway addresses
        let gateway_existing: Vec<String> = gateway_existing
            .iter()
            .map(|x| x.addr_bind.clone())
            .collect();

        // Get incoming gateway addresses
        let gateway_incoming: Vec<String> =
            gateway_data.iter().map(|x| x.addr_bind.clone()).collect();

        // Find addresses that are in existing but not in incoming (to be removed)
        let addresses_to_remove: Vec<String> = gateway_existing
            .clone()
            .into_iter()
            .filter(|x| !gateway_incoming.contains(x))
            .collect();

        // Find addresses that are in incoming but not in existing (to be added)
        let addresses_to_add: Vec<String> = gateway_incoming
            .iter()
            .filter(|x| !gateway_existing.contains(x))
            .cloned()
            .collect();

        eprintln!(
            "[-TC-]   Addresses to remove: {:?}",
            addresses_to_remove.len()
        );
        eprintln!("[-TC-]   Addresses to add: {:?}", addresses_to_add.len());

        config::RoutingData::GatewayID.set(&checksum);
        config::RoutingData::GatewayRouting.xset(&gateway_data);

        Ok(())
    }

    /// for non high speed proxy the data will come to gateway node
    fn gwnode_data(payload: String) -> Result<(), serde_json::Error> {
        let checksum = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(payload.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        let checksum_old = config::RoutingData::GatewayNodeID.val().clone();

        if checksum == checksum_old {
            log::info!("Gateway node id : {}", checksum);
            return Ok(());
        }
        let gwnode_data = serde_json::from_str::<Vec<GatewayNode>>(&payload);
        let gwnode_data = match gwnode_data {
            Ok(data) => {
                let mut data_node = Vec::new();
                for node in data {
                    let mut tls_data = Vec::new();
                    for tls in node.tls {
                        let mut tls_key = None;
                        let mut tls_pem = None;
                        if tls.tls {
                            let (pem_path, key_path) = Self::save_tls_gwnode(
                                tls.clone(),
                                tls.tls_pem.unwrap_or_default(),
                                tls.tls_key.unwrap_or_default(),
                            );
                            tls_key = Some(key_path);
                            tls_pem = Some(pem_path);
                        }
                        tls_data.push(GatewayNodeSNI {
                            tls_pem,
                            tls_key,
                            ..tls
                        });
                    }

                    data_node.push(GatewayNode {
                        tls: tls_data,
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

        config::RoutingData::GatewayNodeID.set(&checksum);
        config::RoutingData::GatewayNodeListen.xset(&gwnode_data);

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
            ("gwnode", payload) => match Self::gwnode_data(payload.to_string()) {
                Ok(()) => json!({
                    "status": "success",
                    "message": "Gateway node data updated successfully"
                }),
                Err(e) => json!({
                    "status": "error",
                    "message": format!("Failed to parse gateway node data: {}", e)
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

        eprintln!(
            "[----]   $ Sending response: {}",
            response_str
        );

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
            self.name,
            service,
            action,
            status_str,
            metrics_info
        );
    }
}
