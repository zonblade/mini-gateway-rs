//! # Certificate Automation Module
//!
//! This module integrates the certbot certificate generation system with the proxy domain database.
//! It handles automatic certificate generation, storage, and renewal tracking.

use crate::api::settings::{ProxyDomain, proxydomain_queries};
use crate::module::certbot_runner::{CertbotManager, CertbotConfig, CertificateInfo};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use log::{info, warn, error, debug};
use std::process::Command;

/// Result type for certificate automation operations
pub type Result<T> = std::result::Result<T, CertificateAutomationError>;

/// Errors that can occur during certificate automation
#[derive(Debug, thiserror::Error)]
pub enum CertificateAutomationError {
    #[error("Certbot error: {0}")]
    CertbotError(#[from] crate::module::certbot_runner::CertbotError),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] crate::module::database::DatabaseError),
    
    #[error("Domain validation error: {0}")]
    DomainValidation(String),
    
    #[error("Certificate parsing error: {0}")]
    CertificateParsing(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Certificate automation manager that integrates certbot with the database
pub struct CertificateAutomationManager {
    certbot_manager: CertbotManager,
}

impl CertificateAutomationManager {
    /// Create a new certificate automation manager with default staging configuration
    pub fn new_staging() -> Self {
        let config = CertbotConfig {
            staging: true,  // Safe staging environment
            email: Some("email@domain.com".to_string()),
            non_interactive: true,
            agree_tos: true,
            no_eff_email: true,
            register_unsafely_without_email: false,
            // Use custom directory instead of /etc/letsencrypt
            config_dir: Some("/data/certbot/config".to_string()),
            work_dir: Some("/data/certbot/work".to_string()),
            logs_dir: Some("/data/certbot/logs".to_string()),
            use_sudo: false,
            ..Default::default()
        };

        Self {
            certbot_manager: CertbotManager::new(config),
        }
    }
    
    /// Create a new certificate automation manager with production configuration
    pub fn new_production(email: Option<String>) -> Self {
        let config = CertbotConfig {
            staging: false,  // Production environment
            email: email.clone(),
            non_interactive: true,
            agree_tos: true,
            no_eff_email: true,
            register_unsafely_without_email: email.is_none(),
            // Use custom directory instead of /etc/letsencrypt
            config_dir: Some("/data/certbot/config".to_string()),
            work_dir: Some("/data/certbot/work".to_string()),
            logs_dir: Some("/data/certbot/logs".to_string()),
            use_sudo: false,
            ..Default::default()
        };
        
        Self {
            certbot_manager: CertbotManager::new(config),
        }
    }
    
    /// Create a new certificate automation manager with custom configuration
    pub fn new(config: CertbotConfig) -> Self {
        Self {
            certbot_manager: CertbotManager::new(config),
        }
    }
    
    /// Generate or renew a certificate for a domain and save it to the database
    ///
    /// This function:
    /// 1. Checks if domain has tls_autron enabled, if false, uses existing cert from database
    /// 2. Checks for existing certbot certificates and validates expiry
    /// 3. Uses certbot to generate/renew the certificate only if needed
    /// 4. Reads the certificate and key files
    /// 5. Updates the proxy domain record with the certificate data
    /// 6. Sets the expected renewal date
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain name to generate certificate for
    /// * `proxy_id` - The proxy ID this domain belongs to
    ///
    /// # Returns
    ///
    /// Returns the updated ProxyDomain with certificate data
    pub async fn ensure_certificate_and_save(&self, domain: &str, proxy_id: &str) -> Result<ProxyDomain> {
        info!("Starting certificate automation for domain: {} (proxy: {})", domain, proxy_id);
        
        // Step 1: Validate domain name
        if domain.is_empty() {
            return Err(CertificateAutomationError::DomainValidation(
                "Domain name cannot be empty".to_string()
            ));
        }
        
        // Step 2: Find existing domain record to check tls_autron flag
        let domains = proxydomain_queries::get_proxy_domains_by_proxy_id(proxy_id)?;
        let existing_domain = domains.into_iter()
            .find(|d| d.sni.as_ref().map(|s| s.as_str()) == Some(domain));
        
        // Step 3: Check if tls_autron is enabled, if not, use existing cert from database
        if let Some(domain_record) = &existing_domain {
            if !domain_record.tls_autron {
                info!("Domain {} has tls_autron=false, checking certificate origin", domain);
                
                // Check if there are certificates in database
                if let (Some(ref cert_pem), Some(ref key_pem)) = (&domain_record.tls_pem, &domain_record.tls_key) {
                    if !cert_pem.is_empty() && !key_pem.is_empty() {
                        // Check if these certificates came from certbot generation
                        match self.certbot_manager.detect_certificate_origin(domain) {
                            Some(origin) if origin == "generated" => {
                                warn!("Domain {} has tls_autron=false but certificate appears to be auto-generated. Checking expiry.", domain);
                                
                                // For generated certificates, check if they need renewal based on expiry
                                let cert_info = CertificateInfo::new_with_config_dir(domain, &self.certbot_manager.get_config_dir());
                                match self.check_certificate_expiry(&cert_info.path_cert) {
                                    Ok(expires_at) => {
                                        let now = Utc::now();
                                        let days_until_expiry = (expires_at - now).num_days();
                                        
                                        if days_until_expiry <= 30 {
                                            info!("Auto-generated certificate for {} expires in {} days but tls_autron=false. Manual intervention needed.", domain, days_until_expiry);
                                        }
                                    },
                                    Err(_) => {
                                        warn!("Could not check expiry for auto-generated certificate on domain {} with tls_autron=false", domain);
                                    }
                                }
                                
                                info!("Using existing auto-generated certificate from database for domain: {}", domain);
                                return Ok(domain_record.clone());
                            },
                            Some(_) | None => {
                                info!("Using existing manual certificate from database for domain: {}", domain);
                                return Ok(domain_record.clone());
                            }
                        }
                    }
                } else {
                    // No certificate in database but tls_autron=false, check if there are certificates in folder
                    match self.certbot_manager.detect_certificate_origin(domain) {
                        Some(origin) => {
                            warn!("Domain {} has tls_autron=false but {} certificate found in folder - not updating database", domain, origin);
                        },
                        None => {
                            info!("Domain {} has tls_autron=false and no certificates found", domain);
                        }
                    }
                }
                
                warn!("Domain {} has tls_autron=false but no certificate data in database", domain);
                return Err(CertificateAutomationError::Configuration(
                    format!("Domain {} has automatic certificates disabled but no certificate data available", domain)
                ));
            }
            
            // Check if we need to use a different manager based on tls_mode
            let default_tls_mode = "staging".to_string();
            let tls_mode = domain_record.tls_mode.as_ref().unwrap_or(&default_tls_mode);
            
            info!("Domain {} has tls_mode: {} (from DB: {:?})", domain, tls_mode, domain_record.tls_mode);
            
            // If the current manager doesn't match the required mode, we need to use a different approach
            let current_is_production = self.certbot_manager.is_production();
            info!("Domain {} current manager is_production: {}, required mode: {}", domain, current_is_production, tls_mode);
            
            if (tls_mode == "prod" && !current_is_production) ||
               (tls_mode != "prod" && current_is_production) {
                info!("Domain {} requires different TLS mode ({}), creating appropriate manager", domain, tls_mode);
                
                // Create the appropriate manager and proceed with certificate generation
                let appropriate_manager = if tls_mode == "prod" {
                    info!("Creating production manager for domain {}", domain);
                    CertificateAutomationManager::new_production(Some("email@domain.com".to_string()))
                } else {
                    info!("Creating staging manager for domain {}", domain);
                    CertificateAutomationManager::new_staging()
                };
                
                // Generate certificate using the appropriate manager and return immediately
                return appropriate_manager.ensure_certificate_and_save_internal(domain, proxy_id).await;
            } else {
                info!("Domain {} will use current manager (production: {})", domain, current_is_production);
            }
        }
        
        // Step 4: Check if certificate already exists and is valid
        debug!("Checking if certificate already exists for domain: {}", domain);
        match self.certbot_manager.certificate_exists(domain) {
            Ok(true) => {
                info!("Certificate files found for domain: {}, checking expiry", domain);
                // Certificate exists, check if it's still valid (not expired or expiring soon)
                let cert_info = CertificateInfo::new_with_config_dir(domain, &self.certbot_manager.get_config_dir());
                match self.check_certificate_expiry(&cert_info.path_cert) {
                    Ok(expires_at) => {
                        let now = Utc::now();
                        let days_until_expiry = (expires_at - now).num_days();
                        
                        if days_until_expiry > 30 {
                            info!("Certificate for domain {} is valid for {} more days, validating domain match", domain, days_until_expiry);
                            
                            // Validate certificate matches the expected domain
                            match self.validate_certificate_domain(&cert_info.path_cert, domain) {
                                Ok(true) => {
                                    info!("Certificate domain validation passed for {}, using existing certificate", domain);
                                    // Use existing certificate without calling certbot
                                    let cert_info = CertificateInfo {
                                        expired_at: Some(expires_at),
                                        ..cert_info
                                    };
                                    return self.process_existing_certificate(cert_info, domain, proxy_id).await;
                                },
                                Ok(false) => {
                                    warn!("Certificate domain validation failed for {}, will renew", domain);
                                },
                                Err(e) => {
                                    warn!("Could not validate certificate domain for {}: {}, will renew", domain, e);
                                }
                            }
                        } else {
                            info!("Certificate for domain {} expires in {} days, will renew", domain, days_until_expiry);
                        }
                    },
                    Err(e) => {
                        warn!("Could not check certificate expiry for domain {}: {}, will renew", domain, e);
                    }
                }
            },
            Ok(false) => {
                info!("No existing certificate found for domain: {}, will create new", domain);
            },
            Err(e) => {
                warn!("Error checking certificate existence for domain {}: {}, will attempt to create", domain, e);
            }
        }
        
        // Step 5: Generate or renew certificate using certbot
        debug!("Calling certbot to ensure certificate for domain: {}", domain);
        let cert_info = self.certbot_manager.ensure_certificate(domain)
            .map_err(CertificateAutomationError::CertbotError)?;
        
        info!("Certificate operation completed for domain: {}", domain);
        debug!("Certificate paths - cert: {}, key: {}", cert_info.path_cert, cert_info.path_key);
        
        // Step 3: Read certificate and key files
        let cert_pem = std::fs::read_to_string(&cert_info.path_cert)
            .map_err(|e| CertificateAutomationError::CertificateParsing(
                format!("Failed to read certificate file {}: {}", cert_info.path_cert, e)
            ))?;
            
        let key_pem = std::fs::read_to_string(&cert_info.path_key)
            .map_err(|e| CertificateAutomationError::CertificateParsing(
                format!("Failed to read key file {}: {}", cert_info.path_key, e)
            ))?;
        
        debug!("Successfully read certificate and key files");
        
        // Step 4: Calculate expected renewal date (30 days before expiration, or 60 days from now if no expiration)
        let expected_renew = if let Some(expiration) = cert_info.expired_at {
            // Calculate renewal date as 30 days before expiration
            let renewal_date = expiration - Duration::days(30);
            renewal_date.to_rfc3339()
        } else {
            // If no expiration date, set renewal for 60 days from now (typical Let's Encrypt duration - 30 days)
            let renewal_date = Utc::now() + Duration::days(60);
            renewal_date.to_rfc3339()
        };
        
        debug!("Calculated expected renewal date: {}", expected_renew);
        
        // Step 5: Find existing domain record or create new one
        let domains = proxydomain_queries::get_proxy_domains_by_proxy_id(proxy_id)?;
        let mut proxy_domain = domains.into_iter()
            .find(|d| d.sni.as_ref().map(|s| s.as_str()) == Some(domain))
            .unwrap_or_else(|| {
                debug!("Creating new proxy domain record for domain: {}", domain);
                ProxyDomain {
                    id: Uuid::new_v4().to_string(),
                    proxy_id: Some(proxy_id.to_string()),
                    tls: true,
                    tls_pem: None,
                    tls_key: None,
                    sni: Some(domain.to_string()),
                    tls_autron: true,
                    tls_mode: Some("staging".to_string()), // Default to staging
                    expected_renew: None,
                }
            });
        
        // Step 6: Update domain record with certificate data
        proxy_domain.tls = true;
        proxy_domain.tls_autron = true;
        proxy_domain.tls_pem = Some(cert_pem);
        proxy_domain.tls_key = Some(key_pem);
        proxy_domain.expected_renew = Some(expected_renew);
        
        // Step 7: Save to database
        debug!("Saving updated proxy domain to database");
        proxydomain_queries::save_proxy_domain(&proxy_domain)?;
        
        info!("Certificate automation completed successfully for domain: {}", domain);
        
        Ok(proxy_domain)
    }

    /// Internal method to generate certificate without checking tls_mode recursion
    /// This method performs the actual certificate generation logic
    async fn ensure_certificate_and_save_internal(&self, domain: &str, proxy_id: &str) -> Result<ProxyDomain> {
        info!("Starting certificate generation for domain: {} (proxy: {})", domain, proxy_id);
        
        // Validate domain name
        if domain.is_empty() {
            return Err(CertificateAutomationError::DomainValidation(
                "Domain name cannot be empty".to_string()
            ));
        }
        
        // Check if certificate already exists and is valid
        debug!("Checking if certificate already exists for domain: {}", domain);
        match self.certbot_manager.certificate_exists(domain) {
            Ok(true) => {
                info!("Certificate files found for domain: {}, checking expiry", domain);
                // Certificate exists, check if it's still valid (not expired or expiring soon)
                let cert_info = CertificateInfo::new_with_config_dir(domain, &self.certbot_manager.get_config_dir());
                match self.check_certificate_expiry(&cert_info.path_cert) {
                    Ok(expires_at) => {
                        let now = Utc::now();
                        let days_until_expiry = (expires_at - now).num_days();
                        
                        if days_until_expiry > 30 {
                            info!("Certificate for domain {} is valid for {} more days, validating domain match", domain, days_until_expiry);
                            
                            // Validate certificate matches the expected domain
                            match self.validate_certificate_domain(&cert_info.path_cert, domain) {
                                Ok(true) => {
                                    info!("Certificate domain validation passed for {}, using existing certificate", domain);
                                    // Use existing certificate without calling certbot
                                    let cert_info = CertificateInfo {
                                        expired_at: Some(expires_at),
                                        ..cert_info
                                    };
                                    return self.process_existing_certificate(cert_info, domain, proxy_id).await;
                                },
                                Ok(false) => {
                                    warn!("Certificate domain validation failed for {}, will renew", domain);
                                },
                                Err(e) => {
                                    warn!("Could not validate certificate domain for {}: {}, will renew", domain, e);
                                }
                            }
                        } else {
                            info!("Certificate for domain {} expires in {} days, will renew", domain, days_until_expiry);
                        }
                    },
                    Err(e) => {
                        warn!("Could not check certificate expiry for domain {}: {}, will renew", domain, e);
                    }
                }
            },
            Ok(false) => {
                info!("No existing certificate found for domain: {}, will create new", domain);
            },
            Err(e) => {
                warn!("Error checking certificate existence for domain {}: {}, will attempt to create", domain, e);
            }
        }
        
        // Generate or renew certificate using certbot
        debug!("Calling certbot to ensure certificate for domain: {}", domain);
        let cert_info = self.certbot_manager.ensure_certificate(domain)
            .map_err(CertificateAutomationError::CertbotError)?;
        
        info!("Certificate operation completed for domain: {}", domain);
        debug!("Certificate paths - cert: {}, key: {}", cert_info.path_cert, cert_info.path_key);
        
        // Read certificate and key files
        let cert_pem = std::fs::read_to_string(&cert_info.path_cert)
            .map_err(|e| CertificateAutomationError::CertificateParsing(
                format!("Failed to read certificate file {}: {}", cert_info.path_cert, e)
            ))?;
            
        let key_pem = std::fs::read_to_string(&cert_info.path_key)
            .map_err(|e| CertificateAutomationError::CertificateParsing(
                format!("Failed to read key file {}: {}", cert_info.path_key, e)
            ))?;
        
        debug!("Successfully read certificate and key files");
        
        // Calculate expected renewal date (30 days before expiration, or 60 days from now if no expiration)
        let expected_renew = if let Some(expiration) = cert_info.expired_at {
            // Calculate renewal date as 30 days before expiration
            let renewal_date = expiration - Duration::days(30);
            renewal_date.to_rfc3339()
        } else {
            // If no expiration date, set renewal for 60 days from now (typical Let's Encrypt duration - 30 days)
            let renewal_date = Utc::now() + Duration::days(60);
            renewal_date.to_rfc3339()
        };
        
        debug!("Calculated expected renewal date: {}", expected_renew);
        
        // Find existing domain record or create new one
        let domains = proxydomain_queries::get_proxy_domains_by_proxy_id(proxy_id)?;
        let mut proxy_domain = domains.into_iter()
            .find(|d| d.sni.as_ref().map(|s| s.as_str()) == Some(domain))
            .unwrap_or_else(|| {
                debug!("Creating new proxy domain record for domain: {}", domain);
                ProxyDomain {
                    id: Uuid::new_v4().to_string(),
                    proxy_id: Some(proxy_id.to_string()),
                    tls: true,
                    tls_pem: None,
                    tls_key: None,
                    sni: Some(domain.to_string()),
                    tls_autron: true,
                    tls_mode: Some("staging".to_string()), // Default to staging
                    expected_renew: None,
                }
            });
        
        // Update domain record with certificate data
        proxy_domain.tls = true;
        proxy_domain.tls_autron = true;
        proxy_domain.tls_pem = Some(cert_pem);
        proxy_domain.tls_key = Some(key_pem);
        proxy_domain.expected_renew = Some(expected_renew);
        
        // Save to database
        debug!("Saving updated proxy domain to database");
        proxydomain_queries::save_proxy_domain(&proxy_domain)?;
        
        info!("Certificate automation completed successfully for domain: {}", domain);
        
        Ok(proxy_domain)
    }
    
    /// Generate certificates for all domains with tls_autron=true that need renewal
    ///
    /// This function scans all proxy domains and renews certificates that are:
    /// - Marked for automatic certificate management (tls_autron=true)
    /// - Due for renewal (expected_renew date has passed)
    /// - Missing certificate data (tls_pem or tls_key is empty)
    ///
    /// # Returns
    ///
    /// Returns a tuple of (successful_renewals, failed_renewals)
    pub async fn renew_due_certificates(&self) -> Result<(usize, usize)> {
        info!("Starting automatic certificate renewal check");
        
        // Get all proxy domains
        let all_domains = proxydomain_queries::get_all_proxy_domains()?;
        
        let mut successful = 0;
        let mut failed = 0;
        let now = Utc::now();
        
        for domain in all_domains {
            // Skip domains that don't have automatic certificates enabled
            if !domain.tls_autron {
                continue;
            }
            
            // Skip domains without a domain name
            let domain_name = match &domain.sni {
                Some(name) if !name.is_empty() => name,
                _ => {
                    warn!("Skipping domain {} - no domain name (sni) specified", domain.id);
                    continue;
                }
            };
            
            // Skip domains without a proxy_id
            let proxy_id = match &domain.proxy_id {
                Some(id) if !id.is_empty() => id,
                _ => {
                    warn!("Skipping domain {} - no proxy_id specified", domain.id);
                    continue;
                }
            };
            
            // Check if renewal is needed
            let needs_renewal = if let Some(expected_renew_str) = &domain.expected_renew {
                match DateTime::parse_from_rfc3339(expected_renew_str) {
                    Ok(expected_renew) => {
                        let renewal_due = expected_renew.with_timezone(&Utc) <= now;
                        debug!("Domain {} renewal due: {} (expected: {}, now: {})", 
                               domain_name, renewal_due, expected_renew, now);
                        renewal_due
                    },
                    Err(e) => {
                        warn!("Invalid expected_renew date for domain {}: {} - treating as needing renewal", 
                              domain_name, e);
                        true
                    }
                }
            } else {
                // No expected renewal date - treat as needing certificate generation
                debug!("Domain {} has no expected_renew date - treating as needing certificate", domain_name);
                true
            };
            
            // Also check if certificate data is missing in database
            let missing_cert_data = domain.tls_pem.is_none() || domain.tls_key.is_none() ||
                domain.tls_pem.as_ref().map_or(true, |s| s.is_empty()) ||
                domain.tls_key.as_ref().map_or(true, |s| s.is_empty());
            
            if missing_cert_data {
                debug!("Domain {} is missing certificate data in database, checking folder", domain_name);
                
                // Check if certificates exist in certbot folder even if not in database
                if let Some(manager) = self.get_appropriate_manager_for_domain(&domain) {
                    match manager.certbot_manager.detect_certificate_origin(domain_name) {
                        Some(origin) => {
                            info!("Domain {} has {} certificate in folder but missing from database", domain_name, origin);
                            // We'll let the renewal process handle loading it into the database
                        },
                        None => {
                            debug!("Domain {} has no certificate in folder either", domain_name);
                        }
                    }
                }
            }
            
            if needs_renewal || missing_cert_data {
                info!("Renewing certificate for domain: {}", domain_name);
                
                match self.ensure_certificate_and_save(domain_name, proxy_id).await {
                    Ok(_) => {
                        successful += 1;
                        info!("Successfully renewed certificate for domain: {}", domain_name);
                    },
                    Err(e) => {
                        failed += 1;
                        error!("Failed to renew certificate for domain {}: {}", domain_name, e);
                    }
                }
            } else {
                debug!("Domain {} does not need renewal yet", domain_name);
            }
        }
        
        info!("Certificate renewal check completed. Successful: {}, Failed: {}", successful, failed);
        
        Ok((successful, failed))
    }
    
    /// Get domains that are due for renewal
    ///
    /// Returns a list of domains that need certificate renewal
    pub fn get_domains_due_for_renewal(&self) -> Result<Vec<ProxyDomain>> {
        let all_domains = proxydomain_queries::get_all_proxy_domains()?;
        let now = Utc::now();
        let mut due_domains = Vec::new();
        
        for domain in all_domains {
            if !domain.tls_autron {
                continue;
            }
            
            let needs_renewal = if let Some(expected_renew_str) = &domain.expected_renew {
                match DateTime::parse_from_rfc3339(expected_renew_str) {
                    Ok(expected_renew) => expected_renew.with_timezone(&Utc) <= now,
                    Err(_) => true,
                }
            } else {
                true
            };
            
            let missing_cert_data = domain.tls_pem.is_none() || domain.tls_key.is_none() ||
                domain.tls_pem.as_ref().map_or(true, |s| s.is_empty()) ||
                domain.tls_key.as_ref().map_or(true, |s| s.is_empty());
            
            if needs_renewal || missing_cert_data {
                due_domains.push(domain);
            }
        }
        
        Ok(due_domains)
    }
    
    /// Check certificate expiry using openssl command
    fn check_certificate_expiry(&self, cert_path: &str) -> std::result::Result<DateTime<Utc>, Box<dyn std::error::Error>> {
        let output = Command::new("openssl")
            .args(["x509", "-in", cert_path, "-noout", "-enddate"])
            .output()?;

        if output.status.success() {
            let date_str = String::from_utf8_lossy(&output.stdout);
            // Output format: "notAfter=Jan 1 00:00:00 2024 GMT"
            if let Some(date_part) = date_str.strip_prefix("notAfter=") {
                let date_part = date_part.trim();
                // Parse the date string
                match DateTime::parse_from_str(date_part, "%b %d %H:%M:%S %Y GMT") {
                    Ok(parsed_date) => Ok(parsed_date.with_timezone(&Utc)),
                    Err(_) => {
                        // Try alternative format without GMT
                        match DateTime::parse_from_str(&date_part.replace(" GMT", " +0000"), "%b %d %H:%M:%S %Y %z") {
                            Ok(parsed_date) => Ok(parsed_date.with_timezone(&Utc)),
                            Err(e) => Err(Box::new(e))
                        }
                    }
                }
            } else {
                Err("Invalid openssl output format".into())
            }
        } else {
            Err(String::from_utf8_lossy(&output.stderr).into())
        }
    }

    /// Validate that certificate matches the expected domain name
    fn validate_certificate_domain(&self, cert_path: &str, expected_domain: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        // Get subject and SAN from certificate
        let subject_output = Command::new("openssl")
            .args(["x509", "-in", cert_path, "-noout", "-subject"])
            .output()?;

        let san_output = Command::new("openssl")
            .args(["x509", "-in", cert_path, "-noout", "-ext", "subjectAltName"])
            .output()?;

        let mut domains_found = Vec::new();

        // Parse subject CN
        if subject_output.status.success() {
            let subject_str = String::from_utf8_lossy(&subject_output.stdout);
            // Extract CN from subject line like "subject=CN=example.com"
            if let Some(cn_part) = subject_str.split("CN=").nth(1) {
                if let Some(cn_value) = cn_part.split(',').next() {
                    domains_found.push(cn_value.trim().to_string());
                }
            }
        }

        // Parse SAN extensions
        if san_output.status.success() {
            let san_str = String::from_utf8_lossy(&san_output.stdout);
            // Parse SAN like "DNS:example.com, DNS:www.example.com"
            for line in san_str.lines() {
                if line.contains("DNS:") {
                    for part in line.split(',') {
                        if let Some(dns_part) = part.trim().strip_prefix("DNS:") {
                            domains_found.push(dns_part.trim().to_string());
                        }
                    }
                }
            }
        }

        // Check if expected domain matches any found domain
        let domain_matches = domains_found.iter().any(|d| d == expected_domain);
        
        if domain_matches {
            debug!("Certificate domain validation passed for {}: found domains {:?}", expected_domain, domains_found);
        } else {
            warn!("Certificate domain validation failed for {}: found domains {:?}", expected_domain, domains_found);
        }

        Ok(domain_matches)
    }
    
    /// Get appropriate certificate manager based on domain's tls_mode
    fn get_appropriate_manager_for_domain(&self, domain: &ProxyDomain) -> Option<CertificateAutomationManager> {
        let default_tls_mode = "staging".to_string();
        let tls_mode = domain.tls_mode.as_ref().unwrap_or(&default_tls_mode);

        if tls_mode == "prod" {
            Some(CertificateAutomationManager::new_production(Some("email@domain.com".to_string())))
        } else {
            Some(CertificateAutomationManager::new_staging())
        }
    }
    
    /// Process existing certificate and save to database
    async fn process_existing_certificate(&self, cert_info: CertificateInfo, domain: &str, proxy_id: &str) -> Result<ProxyDomain> {
        debug!("Processing existing certificate for domain: {}", domain);
        
        // Read certificate and key files
        let cert_pem = std::fs::read_to_string(&cert_info.path_cert)
            .map_err(|e| CertificateAutomationError::CertificateParsing(
                format!("Failed to read certificate file {}: {}", cert_info.path_cert, e)
            ))?;
            
        let key_pem = std::fs::read_to_string(&cert_info.path_key)
            .map_err(|e| CertificateAutomationError::CertificateParsing(
                format!("Failed to read key file {}: {}", cert_info.path_key, e)
            ))?;
        
        debug!("Successfully read existing certificate and key files");
        
        // Calculate expected renewal date (30 days before expiration, or 60 days from now if no expiration)
        let expected_renew = if let Some(expiration) = cert_info.expired_at {
            // Calculate renewal date as 30 days before expiration
            let renewal_date = expiration - Duration::days(30);
            renewal_date.to_rfc3339()
        } else {
            // If no expiration date, set renewal for 60 days from now
            let renewal_date = Utc::now() + Duration::days(60);
            renewal_date.to_rfc3339()
        };
        
        debug!("Calculated expected renewal date: {}", expected_renew);
        
        // Find existing domain record or create new one
        let domains = proxydomain_queries::get_proxy_domains_by_proxy_id(proxy_id)?;
        let mut proxy_domain = domains.into_iter()
            .find(|d| d.sni.as_ref().map(|s| s.as_str()) == Some(domain))
            .unwrap_or_else(|| {
                debug!("Creating new proxy domain record for domain: {}", domain);
                ProxyDomain {
                    id: Uuid::new_v4().to_string(),
                    proxy_id: Some(proxy_id.to_string()),
                    tls: true,
                    tls_pem: None,
                    tls_key: None,
                    sni: Some(domain.to_string()),
                    tls_autron: true,
                    tls_mode: Some("staging".to_string()), // Default to staging
                    expected_renew: None,
                }
            });
        
        // Update domain record with certificate data
        proxy_domain.tls = true;
        proxy_domain.tls_autron = true;
        proxy_domain.tls_pem = Some(cert_pem);
        proxy_domain.tls_key = Some(key_pem);
        proxy_domain.expected_renew = Some(expected_renew);
        
        // Save to database
        debug!("Saving updated proxy domain to database");
        proxydomain_queries::save_proxy_domain(&proxy_domain)?;
        
        info!("Successfully processed existing certificate for domain: {}", domain);
        
        Ok(proxy_domain)
    }
}

/// Convenience function to generate a certificate using default staging configuration
pub async fn generate_certificate_staging(domain: &str, proxy_id: &str) -> Result<ProxyDomain> {
    let manager = CertificateAutomationManager::new_staging();
    manager.ensure_certificate_and_save(domain, proxy_id).await
}

/// Convenience function to generate a certificate using production configuration
pub async fn generate_certificate_production(domain: &str, proxy_id: &str, email: Option<String>) -> Result<ProxyDomain> {
    let manager = CertificateAutomationManager::new_production(email);
    manager.ensure_certificate_and_save(domain, proxy_id).await
}

/// Convenience function to renew all due certificates
/// This function is deprecated - use CertificateAutomationManager directly to respect tls_mode
pub async fn renew_all_due_certificates(_email: Option<String>) -> Result<(usize, usize)> {
    // Create a manager that will check each domain's tls_mode individually
    let manager = CertificateAutomationManager::new_staging();
    manager.renew_due_certificates().await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_certificate_automation_manager_creation() {
        let staging_manager = CertificateAutomationManager::new_staging();
        // Just verify we can create the manager without panicking
        assert_eq!(std::mem::size_of_val(&staging_manager), std::mem::size_of::<CertificateAutomationManager>());
        
        let production_manager = CertificateAutomationManager::new_production(Some("test@example.com".to_string()));
        assert_eq!(std::mem::size_of_val(&production_manager), std::mem::size_of::<CertificateAutomationManager>());
    }
}
