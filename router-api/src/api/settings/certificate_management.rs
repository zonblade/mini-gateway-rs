//! # Certificate Management API Endpoints
//!
//! This module provides HTTP endpoints for managing SSL/TLS certificates using the
//! integrated certbot automation system.

use actix_web::{post, get, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::api::users::helper::{is_staff_or_admin, ClaimsFromRequest};
use crate::module::certificate_automation::{CertificateAutomationManager, CertificateAutomationError};

/// Request structure for manual certificate generation
#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateGenerationRequest {
    /// Domain name to generate certificate for
    pub domain: String,
    /// Proxy ID that this domain belongs to
    pub proxy_id: String,
    /// Email address for Let's Encrypt registration (optional)
    pub email: Option<String>,
    /// Whether to use staging environment (default: false for production)
    #[serde(default)]
    pub staging: bool,
}

/// Response structure for certificate operations
#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateResponse {
    /// Operation status
    pub status: String,
    /// Descriptive message
    pub message: String,
    /// Domain name that was processed
    pub domain: String,
    /// Expected renewal date (ISO 8601 format)
    pub expected_renew: Option<String>,
}

/// Manually generate or renew a certificate for a domain
///
/// This endpoint allows administrators to manually trigger certificate generation
/// or renewal for a specific domain using the integrated certbot system.
///
/// # Endpoint
///
/// `POST /settings/certificates/generate`
///
/// # Request Body
///
/// ```json
/// {
///   "domain": "example.com",
///   "proxy_id": "550e8400-e29b-41d4-a716-446655440000",
///   "email": "admin@example.com",
///   "staging": false
/// }
/// ```
///
/// # Response
///
/// ## Success (200 OK)
/// Returns certificate operation details including expected renewal date.
///
/// ## Bad Request (400)
/// Returned when the request is invalid or certificate generation fails.
///
/// ## Forbidden (403)
/// Returned when the user doesn't have admin or staff privileges.
#[post("/certificates/generate")]
pub async fn generate_certificate(
    req: HttpRequest,
    request: web::Json<CertificateGenerationRequest>,
) -> impl Responder {
    // Extract authenticated user's claims
    let claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Failed to get user authentication"
            }))
        }
    };

    // Verify user has admin or staff role
    if !is_staff_or_admin(&claims.role) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only administrators and staff can generate certificates"
        }));
    }

    let req_data = request.into_inner();

    // Validate input
    if req_data.domain.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Domain name cannot be empty"
        }));
    }

    if req_data.proxy_id.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Proxy ID cannot be empty"
        }));
    }

    // Create certificate automation manager based on staging parameter
    let manager = if req_data.staging {
        CertificateAutomationManager::new_staging()
    } else {
        CertificateAutomationManager::new_production(req_data.email.clone())
    };

    // Generate certificate
    match manager.ensure_certificate_and_save(&req_data.domain, &req_data.proxy_id).await {
        Ok(domain) => {
            let response = CertificateResponse {
                status: "success".to_string(),
                message: format!("Certificate successfully generated for domain {}", req_data.domain),
                domain: req_data.domain,
                expected_renew: domain.expected_renew,
            };
            
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            log::error!("Certificate generation failed for domain {}: {}", req_data.domain, e);
            
            let error_message = match e {
                CertificateAutomationError::CertbotError(certbot_err) => {
                    format!("Certbot error: {}", certbot_err)
                },
                CertificateAutomationError::DatabaseError(db_err) => {
                    format!("Database error: {}", db_err)
                },
                CertificateAutomationError::DomainValidation(msg) => {
                    format!("Domain validation error: {}", msg)
                },
                CertificateAutomationError::CertificateParsing(msg) => {
                    format!("Certificate parsing error: {}", msg)
                },
                CertificateAutomationError::Configuration(msg) => {
                    format!("Configuration error: {}", msg)
                },
            };
            
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to generate certificate: {}", error_message)
            }))
        }
    }
}

/// Renew all certificates that are due for renewal
///
/// This endpoint scans all domains with automatic certificate management enabled
/// and renews certificates that are due for renewal.
///
/// # Endpoint
///
/// `POST /settings/certificates/renew-all`
///
/// # Response
///
/// ## Success (200 OK)
/// Returns renewal statistics including successful and failed renewals.
///
/// ## Forbidden (403)
/// Returned when the user doesn't have admin or staff privileges.
#[post("/certificates/renew-all")]
pub async fn renew_all_certificates(req: HttpRequest) -> impl Responder {
    // Extract authenticated user's claims
    let claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Failed to get user authentication"
            }))
        }
    };

    // Verify user has admin or staff role
    if !is_staff_or_admin(&claims.role) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only administrators and staff can renew certificates"
        }));
    }

    // Create certificate automation manager that will respect each domain's tls_mode
    let manager = CertificateAutomationManager::new_staging();

    // Renew all due certificates - the ensure_certificate_and_save method will check each domain's tls_mode
    match manager.renew_due_certificates().await {
        Ok((successful, failed)) => {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": format!("Certificate renewal completed. {} successful, {} failed", successful, failed),
                "successful_renewals": successful,
                "failed_renewals": failed
            }))
        },
        Err(e) => {
            log::error!("Certificate renewal process failed: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to renew certificates: {}", e)
            }))
        }
    }
}

/// Get list of domains that are due for certificate renewal
///
/// This endpoint returns information about domains that need certificate renewal.
///
/// # Endpoint
///
/// `GET /settings/certificates/due-for-renewal`
///
/// # Response
///
/// ## Success (200 OK)
/// Returns a list of domains due for renewal.
///
/// ## Forbidden (403)
/// Returned when the user doesn't have admin or staff privileges.
#[get("/certificates/due-for-renewal")]
pub async fn get_due_for_renewal(req: HttpRequest) -> impl Responder {
    // Extract authenticated user's claims
    let claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Failed to get user authentication"
            }))
        }
    };

    // Verify user has admin or staff role
    if !is_staff_or_admin(&claims.role) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only administrators and staff can view certificate information"
        }));
    }

    // Create certificate automation manager
    let manager = CertificateAutomationManager::new_staging();

    // Get domains due for renewal
    match manager.get_domains_due_for_renewal() {
        Ok(domains) => {
            let domain_info: Vec<_> = domains.iter().map(|domain| {
                serde_json::json!({
                    "id": domain.id,
                    "domain": domain.sni,
                    "proxy_id": domain.proxy_id,
                    "expected_renew": domain.expected_renew,
                    "tls_autron": domain.tls_autron,
                    "has_certificate": domain.tls_pem.is_some() && domain.tls_key.is_some()
                })
            }).collect();

            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "domains": domain_info,
                "count": domains.len()
            }))
        },
        Err(e) => {
            log::error!("Failed to get domains due for renewal: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to get renewal information: {}", e)
            }))
        }
    }
}
