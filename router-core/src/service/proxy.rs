use crate::app::proxy::ProxyApp;
use pingora::listeners::Listeners;
use pingora::services::listening::Service;

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