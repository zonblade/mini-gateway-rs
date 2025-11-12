use crate::config::{self, GatewayNode, GatewayNodeSNI};
use crate::system::prottp::app::tls_tools::AppTlsTools;
use crate::system::terminator;

fn extract_first_certificate(pem_chain: &str) -> String {
    const BEGIN_CERT: &str = "-----BEGIN CERTIFICATE-----";
    const END_CERT: &str = "-----END CERTIFICATE-----";
    
    // Early return for empty input
    if pem_chain.trim().is_empty() {
        eprintln!("[----] Empty PEM data provided");
        return String::new();
    }
    
    let cert_count = pem_chain.matches(BEGIN_CERT).count();
    
    match cert_count {
        0 => {
            eprintln!("[----] No valid certificates found in PEM data");
            return pem_chain.to_string();
        }
        1 => {
            eprintln!("[----] Single certificate found, no splitting needed");
        }
        _ => {
            eprintln!("[----] Certificate chain contains {} certificates, extracting first one only", cert_count);
        }
    }
    
    // Normalize line endings and trim whitespace
    let normalized = pem_chain.trim().replace("\r\n", "\n");
    
    // Find the first certificate using more efficient approach
    if let Some(start) = normalized.find(BEGIN_CERT) {
        // Search for END_CERT starting from the position after BEGIN_CERT
        let search_start = start + BEGIN_CERT.len();
        if let Some(relative_end) = normalized[search_start..].find(END_CERT) {
            let end_pos = search_start + relative_end + END_CERT.len();
            let first_cert = &normalized[start..end_pos];
            
            // Ensure proper PEM format with trailing newline
            let result = if first_cert.ends_with('\n') {
                first_cert.to_string()
            } else {
                format!("{}\n", first_cert)
            };
            
            eprintln!("[----] Extracted first certificate: {} bytes from total {} bytes", 
                      result.len(), pem_chain.len());
            return result;
        }
    }
    
    eprintln!("[----] Could not extract first certificate from PEM chain, using original");
    pem_chain.to_string()
}

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
                        let first_cert_only = extract_first_certificate(&tls.tls_pem.clone().unwrap_or_default());
                        
                        let (pem_path, key_path) = AppTlsTools::gateway(
                            tls.clone(),
                            first_cert_only,
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
