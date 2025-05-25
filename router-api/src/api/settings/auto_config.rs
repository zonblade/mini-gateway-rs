//! # Auto-Config API Endpoints
//!
//! This module provides endpoints for importing and exporting gateway configuration in YAML format.
//! It allows for bulk operations through a single API call, making it easier to set up and manage
//! gateway configurations.

use actix_web::{post, get, web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::users::helper::{ClaimsFromRequest, is_staff_or_admin};
use super::{
    Proxy, ProxyDomain, GatewayNode, Gateway,
    proxy_queries, proxydomain_queries, gwnode_queries, gateway_queries
};
use crate::sync;

/// Structure representing a domain in the YAML configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlDomain {
    /// Domain name
    pub domain: String,
    /// Whether TLS is enabled
    #[serde(default)]
    pub tls: bool,
    /// TLS certificate content
    #[serde(default)]
    pub tls_cert: Option<String>,
    /// TLS private key content
    #[serde(default)]
    pub tls_key: Option<String>,
}

/// Structure representing a gateway path in the YAML configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlPath {
    /// Priority of the path (lower number = higher priority)
    pub priority: i32,
    /// Pattern for URL matching
    pub pattern: String,
    /// Target where matching requests should be routed
    pub target: String,
}

/// Structure representing a gateway in the YAML configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlGateway {
    /// Name of the gateway
    pub name: String,
    /// Domain associated with this gateway
    pub domain: String,
    /// Target address
    pub target: String,
    /// Paths configured for this gateway
    pub path: Vec<YamlPath>,
}

/// Structure representing highspeed configuration in the YAML
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlHighspeed {
    /// Whether highspeed mode is enabled
    pub enabled: bool,
    /// Target gateway name for highspeed mode
    pub target: String,
}

/// Structure representing a proxy in the YAML configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlProxy {
    /// Name of the proxy
    pub name: String,
    /// Listen address
    pub listen: String,
    /// Domains associated with this proxy
    pub domains: Vec<YamlDomain>,
    /// Highspeed configuration
    pub highspeed: Option<YamlHighspeed>,
    /// Gateways associated with this proxy
    pub gateway: Vec<YamlGateway>,
}

/// Root structure of the YAML configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConfig {
    /// List of proxies in the configuration
    pub proxy: Vec<YamlProxy>,
}

/// Uploads a configuration file and applies it to the system
///
/// This endpoint processes an uploaded YAML configuration file and creates
/// the corresponding proxy, domain, gateway node, and gateway configurations.
///
/// # Endpoint
///
/// `POST /api/v1/auto-config`
///
/// # Request Body
///
/// The request body should be a YAML document conforming to the configuration schema.
///
/// # Response
///
/// ## Success (200 OK)
/// Returns a summary of the created resources.
///
/// ## Bad Request (400)
/// Returned when the YAML is invalid or configuration conflicts with existing resources.
///
/// ## Forbidden (403)
/// Returned when the user doesn't have admin or staff privileges.
#[post("/auto-config")]
pub async fn upload_config(
    req: HttpRequest,
    body: web::Bytes,
) -> impl Responder {
    // Extract authenticated user's claims
    let claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Failed to get user authentication"})
            )
        }
    };
    
    // Verify user has admin or staff role
    if !is_staff_or_admin(&claims.role) {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"error": "Only administrators and staff can upload configurations"})
        );
    }
    
    // Parse YAML configuration
    let config: YamlConfig = match serde_yaml::from_slice(&body) {
        Ok(config) => config,
        Err(e) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": format!("Invalid YAML configuration: {}", e)})
            )
        }
    };

    // Delete all existing configurations
    // First delete all gateways
    if let Err(e) = gateway_queries::delete_all_gateways() {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete existing gateways: {}", e)
        }));
    }

    // Then delete all gateway nodes
    if let Err(e) = gwnode_queries::delete_all_gateway_nodes() {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete existing gateway nodes: {}", e)
        }));
    }

    // Then delete all proxy domains
    if let Err(e) = proxydomain_queries::delete_all_proxy_domains() {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete existing proxy domains: {}", e)
        }));
    }

    // Finally delete all proxies
    if let Err(e) = proxy_queries::delete_all_proxies() {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete existing proxies: {}", e)
        }));
    }
    
    // Process each proxy in the configuration
    let mut created_proxies = Vec::new();
    let mut created_domains = Vec::new();
    let mut created_gwnodes = Vec::new();
    let mut created_gateways = Vec::new();
    
    for yaml_proxy in config.proxy {
        // Create proxy
        let proxy_id = Uuid::new_v4().to_string();
        let mut proxy = Proxy {
            id: proxy_id.clone(),
            title: yaml_proxy.name.clone(),
            addr_listen: yaml_proxy.listen.clone(),
            addr_target: match proxy_queries::generate_target_address() {
                Ok(addr) => addr,
                Err(e) => {
                    return HttpResponse::BadRequest().json(
                        serde_json::json!({"error": format!("Failed to generate target address: {}", e)})
                    )
                }
            },
            high_speed: yaml_proxy.highspeed.as_ref().map_or(false, |hs| hs.enabled),
            high_speed_addr: None,
            high_speed_gwid: None,
        };
        
        // Save proxy
        if let Err(e) = proxy_queries::save_proxy(&proxy) {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": format!("Failed to create proxy '{}': {}", yaml_proxy.name, e)})
            )
        }
        created_proxies.push(proxy.clone());
        
        // Process domains
        let mut domain_map = std::collections::HashMap::new();
        for yaml_domain in &yaml_proxy.domains {
            let domain_id = Uuid::new_v4().to_string();
            let domain = ProxyDomain {
                id: domain_id.clone(),
                proxy_id: Some(proxy_id.clone()),
                tls: yaml_domain.tls,
                tls_pem: yaml_domain.tls_cert.clone(),
                tls_key: yaml_domain.tls_key.clone(),
                sni: Some(yaml_domain.domain.clone()),
            };
            
            // Save domain
            if let Err(e) = proxydomain_queries::save_proxy_domain(&domain) {
                return HttpResponse::BadRequest().json(
                    serde_json::json!({"error": format!("Failed to create domain '{}': {}", yaml_domain.domain, e)})
                )
            }
            domain_map.insert(yaml_domain.domain.clone(), domain_id.clone());
            created_domains.push(domain);
        }
        
        // Process gateways
        let mut gwnode_map = std::collections::HashMap::new();
        for yaml_gateway in &yaml_proxy.gateway {
            let gwnode_id = Uuid::new_v4().to_string();
            let domain_id = domain_map.get(&yaml_gateway.domain).cloned();
            
            let gwnode = GatewayNode {
                id: gwnode_id.clone(),
                proxy_id: proxy_id.clone(),
                title: yaml_gateway.name.clone(),
                alt_target: yaml_gateway.target.clone(),
                priority: 100, // Default priority
                domain_id,
                domain_name: Some(yaml_gateway.domain.clone()),
            };
            
            // Save gateway node
            if let Err(e) = gwnode_queries::save_gateway_node(&gwnode) {
                return HttpResponse::BadRequest().json(
                    serde_json::json!({"error": format!("Failed to create gateway node '{}': {}", yaml_gateway.name, e)})
                )
            }
            gwnode_map.insert(yaml_gateway.name.clone(), gwnode_id.clone());
            created_gwnodes.push(gwnode);
            
            // Process paths
            for yaml_path in &yaml_gateway.path {
                let gateway_id = Uuid::new_v4().to_string();
                let gateway = Gateway {
                    id: gateway_id,
                    gwnode_id: gwnode_id.clone(),
                    pattern: yaml_path.pattern.clone(),
                    target: yaml_path.target.clone(),
                    priority: yaml_path.priority,
                };
                
                // Save gateway
                if let Err(e) = gateway_queries::save_gateway(&gateway) {
                    return HttpResponse::BadRequest().json(
                        serde_json::json!({"error": format!("Failed to create gateway path for '{}': {}", yaml_gateway.name, e)})
                    )
                }
                created_gateways.push(gateway);
            }
        }
        
        // Handle highspeed if enabled
        if let Some(highspeed) = &yaml_proxy.highspeed {
            if highspeed.enabled {
                if let Some(gwnode_id) = gwnode_map.get(&highspeed.target) {
                    // Update the proxy with highspeed information
                    proxy.high_speed = true;
                    proxy.high_speed_gwid = Some(gwnode_id.clone());
                    
                    // Retrieve the gwnode to get its alt_target for high_speed_addr
                    match gwnode_queries::get_gateway_node_by_id(gwnode_id) {
                        Ok(Some(gwnode)) => {
                            proxy.high_speed_addr = Some(gwnode.alt_target.clone());
                            // Update the proxy
                            if let Err(e) = proxy_queries::save_proxy(&proxy) {
                                return HttpResponse::BadRequest().json(
                                    serde_json::json!({"error": format!("Failed to update proxy with highspeed settings: {}", e)})
                                )
                            }
                        },
                        Ok(None) => {
                            return HttpResponse::BadRequest().json(
                                serde_json::json!({"error": format!("Gateway node '{}' not found for highspeed target", highspeed.target)})
                            )
                        },
                        Err(e) => {
                            return HttpResponse::BadRequest().json(
                                serde_json::json!({"error": format!("Failed to retrieve gateway node for highspeed: {}", e)})
                            )
                        }
                    }
                }
            }
        }
    }
    
    // Add sync calls after successful configuration
    match sync::proxy_node_tcp::sync_proxy_nodes_to_registry().await {
        Ok(_) => log::info!("Successfully synced proxy nodes to registry"),
        Err(e) => log::warn!("Failed to sync proxy nodes to registry: {}. Continuing anyway.", e),
    }

    match sync::gateway_node_tcp::sync_gateway_nodes_to_registry().await {
        Ok(_) => log::info!("Successfully synced gateway nodes to registry"),
        Err(e) => log::warn!("Failed to sync gateway nodes to registry: {}. Continuing anyway.", e),
    }

    match sync::gateway_node_tcp::sync_gateway_paths_to_registry().await {
        Ok(_) => log::info!("Successfully synced gateway paths to registry"),
        Err(e) => log::warn!("Failed to sync gateway paths to registry: {}. Continuing anyway.", e),
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "created": {
            "proxies": created_proxies.len(),
            "domains": created_domains.len(),
            "gwnodes": created_gwnodes.len(),
            "gateways": created_gateways.len()
        }
    }))
}

/// Downloads the current configuration as a YAML file
///
/// This endpoint exports all proxy, domain, gateway node, and gateway configurations
/// into a YAML file format that can be uploaded back through the upload endpoint.
///
/// # Endpoint
///
/// `GET /api/v1/auto-config`
///
/// # Response
///
/// ## Success (200 OK)
/// Returns a YAML document containing the full configuration.
///
/// ## Forbidden (403)
/// Returned when the user doesn't have admin or staff privileges.
#[get("/auto-config")]
pub async fn download_config(req: HttpRequest) -> impl Responder {
    // Extract authenticated user's claims
    let claims = match req.get_claims() {
        Some(claims) => claims,
        None => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Failed to get user authentication"})
            )
        }
    };
    
    // Verify user has admin or staff role
    if !is_staff_or_admin(&claims.role) {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"error": "Only administrators and staff can download configurations"})
        );
    }
    
    // Retrieve all proxies
    let proxies = match proxy_queries::get_all_proxies() {
        Ok(proxies) => proxies,
        Err(e) => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to retrieve proxies: {}", e)})
            )
        }
    };
    
    // Build YAML configuration
    let mut yaml_proxies = Vec::new();
    
    for proxy in proxies {
        // Get domains for this proxy
        let domains = match proxydomain_queries::get_proxy_domains_by_proxy_id(&proxy.id) {
            Ok(domains) => domains,
            Err(e) => {
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": format!("Failed to retrieve domains for proxy '{}': {}", proxy.id, e)})
                )
            }
        };
        
        // Convert domains to YAML format
        let yaml_domains = domains.iter().map(|domain| YamlDomain {
            domain: domain.sni.clone().unwrap_or_default(),
            tls: domain.tls,
            tls_cert: domain.tls_pem.clone(),
            tls_key: domain.tls_key.clone(),
        }).collect::<Vec<_>>();
        
        // Get gateway nodes for this proxy
        let gwnodes = match gwnode_queries::get_gateway_nodes_by_proxy_id(&proxy.id) {
            Ok(gwnodes) => gwnodes,
            Err(e) => {
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": format!("Failed to retrieve gateway nodes for proxy '{}': {}", proxy.id, e)})
                )
            }
        };
        
        // Build gateways with their paths
        let mut yaml_gateways = Vec::new();
        for gwnode in &gwnodes {
            // Get gateways for this gwnode
            let gateways = match gateway_queries::get_gateways_by_gwnode_id(&gwnode.id) {
                Ok(gateways) => gateways,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": format!("Failed to retrieve gateways for gwnode '{}': {}", gwnode.id, e)})
                    )
                }
            };
            
            // Convert paths to YAML format
            let yaml_paths = gateways.iter().map(|gateway| YamlPath {
                priority: gateway.priority,
                pattern: gateway.pattern.clone(),
                target: gateway.target.clone(),
            }).collect::<Vec<_>>();
            
            // Add gateway to list
            if !yaml_paths.is_empty() {
                yaml_gateways.push(YamlGateway {
                    name: gwnode.title.clone(),
                    domain: gwnode.domain_name.clone().unwrap_or_default(),
                    target: gwnode.alt_target.clone(),
                    path: yaml_paths,
                });
            }
        }
        
        // Create highspeed configuration if enabled
        let yaml_highspeed = if proxy.high_speed {
            if let Some(gwid) = &proxy.high_speed_gwid {
                // Find the gwnode name from the ID
                let target_name = gwnodes.iter()
                    .find(|gw| &gw.id == gwid)
                    .map(|gw| gw.title.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                
                Some(YamlHighspeed {
                    enabled: true,
                    target: target_name,
                })
            } else {
                None
            }
        } else {
            None
        };
        
        // Add proxy to list
        yaml_proxies.push(YamlProxy {
            name: proxy.title,
            listen: proxy.addr_listen,
            domains: yaml_domains,
            highspeed: yaml_highspeed,
            gateway: yaml_gateways,
        });
    }
    
    // Create final YAML config
    let yaml_config = YamlConfig {
        proxy: yaml_proxies,
    };
    
    // Convert to YAML string
    let yaml_str = match serde_yaml::to_string(&yaml_config) {
        Ok(yaml) => yaml,
        Err(e) => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to serialize configuration to YAML: {}", e)})
            )
        }
    };
    
    HttpResponse::Ok()
        .content_type("application/yaml")
        .append_header(("Content-Disposition", "attachment; filename=\"gateway-config.yaml\""))
        .body(yaml_str)
} 