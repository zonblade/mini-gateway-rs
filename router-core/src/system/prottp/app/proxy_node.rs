use crate::config::{self, ProxyNode};
use crate::system::prottp::app::tls_tools::AppTlsTools;
use crate::system::terminator;

// Alternative approach with certificate counting and better logging
fn extract_first_certificate_with_logging(pem_chain: &str) -> String {
    let begin_cert = "-----BEGIN CERTIFICATE-----";
    let end_cert = "-----END CERTIFICATE-----";
    
    // Count how many certificates are in the chain
    let cert_count = pem_chain.matches(begin_cert).count();
    
    if cert_count > 1 {
        eprintln!("[----] Certificate chain contains {} certificates, extracting first one only", cert_count);
    } else if cert_count == 1 {
        eprintln!("[----] Single certificate found, no splitting needed");
    } else {
        eprintln!("[----] No valid certificates found in PEM data");
        return pem_chain.to_string();
    }
    
    // Trim whitespace and normalize line endings
    let normalized = pem_chain.trim().replace("\r\n", "\n");
    
    if let Some(start) = normalized.find(begin_cert) {
        if let Some(end) = normalized[start..].find(end_cert) {
            let end_pos = start + end + end_cert.len();
            let first_cert = &normalized[start..end_pos];
            
            // Ensure the certificate ends with a newline for proper PEM format
            let result = if first_cert.ends_with('\n') {
                first_cert.to_string()
            } else {
                format!("{}\n", first_cert)
            };
            
            eprintln!("[----] Extracted first certificate: {} bytes", result.len());
            return result;
        }
    }
    
    eprintln!("[----] Could not extract first certificate from PEM chain, using original");
    pem_chain.to_string()
}


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

                    let first_cert_only = extract_first_certificate_with_logging(&node.clone().tls_pem.unwrap_or_default());

                    let (pem_path, key_path) = AppTlsTools::proxy(
                        node.clone(),
                        first_cert_only,
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
