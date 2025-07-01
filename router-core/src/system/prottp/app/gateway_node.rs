use crate::config::{self, GatewayNode, GatewayNodeSNI};
use crate::system::prottp::app::tls_tools::AppTlsTools;
use crate::system::terminator;

pub fn init(payload: String) -> Result<(), serde_json::Error> {
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
                        let (pem_path, key_path) = AppTlsTools::gateway(
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
