//! # Certificate Auto-Renewal Spawner
//!
//! This module provides background thread spawning for automatic certificate renewal.
//! It follows the same pattern as the memory_log spawner, creating a dedicated thread
//! that periodically checks for certificates due for renewal and renews them automatically.
//!
//! ## Features
//!
//! - Configurable renewal intervals via environment variables
//! - Randomized execution times to avoid load spikes
//! - Retry logic with exponential backoff for failed renewals
//! - Comprehensive logging and error handling
//! - Graceful handling of individual domain failures
//!
//! ## Configuration
//!
//! Environment variables:
//! - `CERT_RENEWAL_INTERVAL_HOURS`: Renewal check interval in hours (default: 6)
//! - `CERT_RENEWAL_JITTER_MINUTES`: Maximum random delay in minutes (default: 30)
//! - `CERT_RENEWAL_RETRY_ATTEMPTS`: Maximum retry attempts per domain (default: 5)
//! - `CERT_RENEWAL_RETRY_DELAY_SECONDS`: Base retry delay in seconds (default: 300)
//! - `CERT_RENEWAL_BUFFER_DAYS`: Days before expiry to trigger renewal (default: 3)

use crate::module::certificate_automation;
use crate::api::settings::{proxydomain_queries, ProxyDomain};
use std::time::Duration;
use rand::Rng;
use chrono::{DateTime, Utc};

/// Configuration for the auto-renewal daemon
#[derive(Debug, Clone)]
pub struct AutoRenewalConfig {
    /// Interval between renewal checks in hours
    pub interval_hours: u64,
    /// Maximum random jitter in minutes to add to each check
    pub jitter_minutes: u64,
    /// Maximum number of retry attempts for failed renewals
    pub retry_attempts: u32,
    /// Base delay between retries in seconds (exponential backoff)
    pub retry_delay_seconds: u64,
    /// Days before certificate expiry to trigger renewal
    pub buffer_days: i64,
}

impl Default for AutoRenewalConfig {
    fn default() -> Self {
        Self {
            interval_hours: 6,       // Check every 6 hours (more frequent monitoring)
            jitter_minutes: 30,      // Up to 30 minutes random delay
            retry_attempts: 5,       // Retry failed renewals 5 times
            retry_delay_seconds: 300, // Start with 5 minute delay
            buffer_days: 3,          // Renew 3 days before expiry
        }
    }
}

impl AutoRenewalConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            interval_hours: std::env::var("CERT_RENEWAL_INTERVAL_HOURS")
                .unwrap_or_else(|_| "6".to_string())
                .parse()
                .unwrap_or(6),
            jitter_minutes: std::env::var("CERT_RENEWAL_JITTER_MINUTES")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            retry_attempts: std::env::var("CERT_RENEWAL_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            retry_delay_seconds: std::env::var("CERT_RENEWAL_RETRY_DELAY_SECONDS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            buffer_days: std::env::var("CERT_RENEWAL_BUFFER_DAYS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
        }
    }
}

/// Spawn the auto-renewal background thread
pub fn spawn_auto_renewal() {
    log::info!("Starting certificate auto-renewal spawner...");
    
    let config = AutoRenewalConfig::from_env();
    log::info!("Auto-renewal configuration: {:?}", config);
    
    std::thread::spawn(move || {
        log::info!("Certificate auto-renewal thread started");
        log::info!("Renewal checks will run every {} hours with up to {} minutes jitter, renewing {} days before expiry", 
                  config.interval_hours, config.jitter_minutes, config.buffer_days);
        
        tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime for auto-renewal")
            .block_on(auto_renewal_daemon(config));
    });
    
    log::info!("Certificate auto-renewal spawner started and detached");
}

/// Main auto-renewal daemon loop
async fn auto_renewal_daemon(config: AutoRenewalConfig) {
    let base_interval = Duration::from_secs(config.interval_hours * 3600);
    let max_jitter = Duration::from_secs(config.jitter_minutes * 60);
    
    loop {
        // Wait for the base interval
        tokio::time::sleep(base_interval).await;
        
        // Add randomized jitter to avoid load spikes
        let jitter = Duration::from_secs(
            rand::thread_rng().gen_range(0..max_jitter.as_secs())
        );
        tokio::time::sleep(jitter).await;
        
        log::info!("Starting certificate expiry check (buffer: {} days)", config.buffer_days);
        
        // Check database for certificates that need renewal based on expiry dates
        match check_and_renew_expiring_certificates(&config).await {
            Ok((checked, renewed, failed)) => {
                log::info!("Certificate check completed: {} domains checked, {} renewed, {} failed", 
                          checked, renewed, failed);
                if renewed > 0 {
                    log::info!("Successfully renewed {} certificate(s)", renewed);
                }
                if failed > 0 {
                    log::warn!("Failed to renew {} certificate(s)", failed);
                }
            },
            Err(e) => {
                log::error!("Certificate expiry check failed: {}", e);
            }
        }
    }
}

/// Check database for expiring certificates and renew them
async fn check_and_renew_expiring_certificates(config: &AutoRenewalConfig) -> Result<(usize, usize, usize), Box<dyn std::error::Error + Send + Sync>> {
    // Get all domains with automatic certificates enabled
    let all_domains = proxydomain_queries::get_all_proxy_domains()
        .map_err(|e| format!("Failed to fetch domains from database: {}", e))?;
    
    let now = Utc::now();
    let mut checked = 0;
    let mut renewed = 0;
    let mut failed = 0;
    
    // Filter domains that need automatic certificates
    let auto_cert_domains: Vec<ProxyDomain> = all_domains.into_iter()
        .filter(|domain| {
            domain.tls_autron && 
            domain.sni.is_some() && 
            domain.proxy_id.is_some()
        })
        .collect();
    
    log::debug!("Found {} domains with automatic certificates enabled", auto_cert_domains.len());
    
    for domain in auto_cert_domains {
        checked += 1;
        
        let domain_name = domain.sni.as_ref().unwrap();
        let proxy_id = domain.proxy_id.as_ref().unwrap();
        
        // Check if certificate needs renewal based on expected_renew date
        let needs_renewal = match &domain.expected_renew {
            Some(expected_renew_str) => {
                match DateTime::parse_from_rfc3339(expected_renew_str) {
                    Ok(expected_renew) => {
                        let renewal_threshold = now + chrono::Duration::days(config.buffer_days);
                        let needs_renewal = expected_renew.with_timezone(&Utc) <= renewal_threshold;
                        
                        if needs_renewal {
                            log::info!("Domain {} needs renewal: expires {}, threshold {} (buffer: {} days)", 
                                     domain_name, expected_renew, renewal_threshold, config.buffer_days);
                        } else {
                            log::debug!("Domain {} OK: expires {}, threshold {} (buffer: {} days)", 
                                      domain_name, expected_renew, renewal_threshold, config.buffer_days);
                        }
                        
                        needs_renewal
                    },
                    Err(e) => {
                        log::warn!("Invalid expected_renew date for domain {}: {} - treating as needing renewal", 
                                 domain_name, e);
                        true
                    }
                }
            },
            None => {
                log::debug!("Domain {} has no expected_renew date - treating as needing certificate", domain_name);
                true
            }
        };
        
        if needs_renewal {
            log::info!("Renewing certificate for domain: {}", domain_name);
            
            match renew_single_domain_with_retry(&domain, config).await {
                Ok(_) => {
                    renewed += 1;
                    log::info!("Successfully renewed certificate for domain: {}", domain_name);
                },
                Err(e) => {
                    failed += 1;
                    log::error!("Failed to renew certificate for domain {}: {}", domain_name, e);
                }
            }
        }
    }
    
    Ok((checked, renewed, failed))
}

/// Renew a single domain certificate with retry logic using per-domain configuration
async fn renew_single_domain_with_retry(domain: &ProxyDomain, config: &AutoRenewalConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let domain_name = domain.sni.as_ref().ok_or("Domain has no SNI")?;
    let proxy_id = domain.proxy_id.as_ref().ok_or("Domain has no proxy_id")?;

    // Email should always exist due to API validation, but double-check
    let email = domain.tls_email.clone();
    if email.is_none() || email.as_ref().map(|e| e.trim().is_empty()).unwrap_or(true) {
        log::error!("Domain {} missing email - this should not happen due to validation", domain_name);
        return Err(format!("Domain {} has no email configured for Let's Encrypt", domain_name).into());
    }

    let mut attempt = 1;
    let mut last_error = None;

    while attempt <= config.retry_attempts {
        log::debug!("Certificate renewal attempt {} of {} for domain {}", attempt, config.retry_attempts, domain_name);

        // Create the appropriate manager based on domain's tls_mode and email
        let tls_mode = domain.tls_mode.as_deref().unwrap_or("staging");
        let manager = if tls_mode == "prod" {
            certificate_automation::CertificateAutomationManager::new_production(email.clone())
        } else {
            certificate_automation::CertificateAutomationManager::new_staging_with_email(email.clone())
        };

        match manager.ensure_certificate_and_save(domain_name, proxy_id).await {
            Ok(_) => {
                if attempt > 1 {
                    log::info!("Certificate renewal for {} succeeded on attempt {}", domain_name, attempt);
                }
                return Ok(());
            },
            Err(e) => {
                last_error = Some(e);

                if attempt < config.retry_attempts {
                    let delay = Duration::from_secs(
                        config.retry_delay_seconds * 2_u64.pow(attempt - 1)
                    );
                    log::warn!("Certificate renewal attempt {} failed for {}, retrying in {:?}: {}",
                              attempt, domain_name, delay, last_error.as_ref().unwrap());
                    tokio::time::sleep(delay).await;
                } else {
                    log::error!("Certificate renewal for {} failed after {} attempts: {}",
                               domain_name, config.retry_attempts, last_error.as_ref().unwrap());
                }

                attempt += 1;
            }
        }
    }

    Err(format!("Certificate renewal for {} failed after {} attempts: {}",
                domain_name, config.retry_attempts,
                last_error.unwrap()).into())
}

