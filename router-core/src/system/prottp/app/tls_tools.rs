use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

use crate::config::{GatewayNodeSNI, ProxyNode};

pub struct AppTlsTools;

impl AppTlsTools {
    // path for (pem, key)
    pub fn proxy(data: ProxyNode, pem: String, key: String) -> (String, String) {
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

    pub fn gateway(data: GatewayNodeSNI, pem: String, key: String) -> (String, String) {
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

}
