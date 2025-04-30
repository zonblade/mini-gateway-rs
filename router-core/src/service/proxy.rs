use crate::app::proxy_fast;
use pingora::listeners::Listeners;
use pingora::services::listening::Service;
use pingora::upstreams::peer::BasicPeer;


pub fn proxy_service_fast(addr: &str, addr_to: &str) -> Service<proxy_fast::ProxyApp> {

    let peer = BasicPeer::new(addr_to);

    Service::with_listeners(
        addr_to.to_string(),
        Listeners::tcp(addr),
        proxy_fast::ProxyApp::new(peer, None),
    )
}

pub fn proxy_service_tls_fast(
    addr: &str,
    addr_to: &str,
    addr_sni: &str,
    cert_path: &str,
    key_path: &str,
) -> Service<proxy_fast::ProxyApp> {

    let peer = BasicPeer::new(addr_to);
    
    // Check if certificate and key files exist
    if !std::path::Path::new(cert_path).exists() {
        log::error!("TLS certificate file not found: {}", cert_path);
    }
    
    if !std::path::Path::new(key_path).exists() {
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

    log::info!("TLS listener created");
    
    Service::with_listeners(
        addr_to.to_string(),
        listeners,
        proxy_fast::ProxyApp::new(peer, Some(addr_sni.to_string())),
    )
}
