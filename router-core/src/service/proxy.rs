use crate::app::proxy::ProxyApp;
use crate::app::proxy_fast;
use pingora::listeners::Listeners;
use pingora::services::listening::Service;
use pingora::upstreams::peer::BasicPeer;

/// Creates a proxy service that listens on a specified address and forwards traffic to a proxy address.
///
/// # Arguments
/// * `addr` - The address on which the service will listen for incoming connections.
/// * `proxy_addr` - The address of the upstream proxy to which traffic will be forwarded.
///
/// # Returns
/// A `Service<ProxyApp>` instance configured to forward traffic to the specified proxy address.
pub fn proxy_service(addr: &str) -> Service<ProxyApp> {

    Service::with_listeners(
        "Proxy Service".to_string(),
        Listeners::tcp(addr),
        ProxyApp::new(addr),
    )
}
pub fn proxy_service_fast(addr: &str, addr_to: &str) -> Service<proxy_fast::ProxyApp> {

    let peer = BasicPeer::new(addr_to);

    Service::with_listeners(
        "Proxy Service".to_string(),
        Listeners::tcp(addr),
        proxy_fast::ProxyApp::new(peer),
    )
}

/// Creates a TLS-enabled proxy service that listens on a specified address and forwards traffic to a proxy address.
///
/// # Arguments
/// * `addr` - The address on which the service will listen for incoming connections.
/// * `proxy_addr` - The address of the upstream proxy to which traffic will be forwarded.
/// * `proxy_sni` - The Server Name Indication (SNI) to use for the TLS connection to the upstream proxy.
/// * `cert_path` - The file path to the TLS certificate for the service.
/// * `key_path` - The file path to the private key corresponding to the TLS certificate.
///
/// # Returns
/// A `Service<ProxyApp>` instance configured to forward traffic to the specified proxy address with TLS enabled.
///
/// # Panics
/// This function will panic if the TLS listener cannot be created (e.g., due to invalid certificate or key paths).
pub fn proxy_service_tls(
    addr: &str,
    cert_path: &str,
    key_path: &str,
) -> Service<ProxyApp> {
    Service::with_listeners(
        "Proxy Service TLS".to_string(),
        Listeners::tls(addr, cert_path, key_path).unwrap(),
        ProxyApp::new(addr),
    )
}

pub fn proxy_service_tls_fast(
    addr: &str,
    addr_to: &str,
    addr_sni: &str,
    cert_path: &str,
    key_path: &str,
) -> Service<proxy_fast::ProxyApp> {

    let mut peer = BasicPeer::new(addr_to);
    peer.sni = addr_sni.into();
    
    // Check if certificate and key files exist
    if !std::path::Path::new("/home/wangsa/project/mini-router/localhost.pem").exists() {
        log::error!("TLS certificate file not found: {}", cert_path);
    }
    
    if !std::path::Path::new("/home/wangsa/project/mini-router/localhost-key.pem").exists() {
        log::error!("TLS key file not found: {}", key_path);
    }
    
    let listeners = match Listeners::tls(addr, cert_path, key_path) {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to create TLS listener: {}. Check that your certificate is valid and not expired.", e);
            log::error!("Certificate path: {}, Key path: {}", cert_path, key_path);
            panic!("TLS setup failed: {}", e);
        }
    };
    
    Service::with_listeners(
        "Proxy Service TLS".to_string(),
        listeners,
        proxy_fast::ProxyApp::new(peer),
    )
}
