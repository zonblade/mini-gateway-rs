use crate::config::{self, ProxyNode};
use crate::system::prottp::app::tls_tools::AppTlsTools;
use crate::system::terminator;

/// now proxy data always accept high speed.
pub fn init(payload: String) -> Result<(), serde_json::Error> {
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
                    let (pem_path, key_path) = AppTlsTools::proxy(
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
