use crate::app::proxy_fast;
use dns_lookup::lookup_host;
use pingora::listeners::Listeners;
use pingora::services::listening::Service;
use pingora::upstreams::peer::BasicPeer;


pub fn proxy_service_fast(addr: &str, addr_to: &str) -> Service<proxy_fast::ProxyApp> {

    let mut addr_target = addr_to.to_string();
    let is_ip = addr_to.bytes().filter(|&b| b == b'.').count() == 4;
    if !is_ip {
        let ipx = lookup_host(&addr_to);
        if let Ok(ipx) = ipx {
            if let Some(ip) = ipx.first() {
                addr_target = ip.to_string();
                log::info!("Resolved {} to {}", addr_to, addr_target);
            } else {
                log::error!("Failed to resolve address: {}", addr_to);
            }
        }
    }
    let peer = BasicPeer::new(&addr_target);

    Service::with_listeners(
        "Proxy Service".to_string(),
        Listeners::tcp(addr),
        proxy_fast::ProxyApp::new(peer, String::from(addr)),
    )
}

pub fn proxy_service_tls_fast(
    addr: &str,
    addr_to: &str,
    _addr_sni: &str,
    cert_path: &str,
    key_path: &str,
) -> Service<proxy_fast::ProxyApp> {
    let mut addr_target = addr_to.to_string();
    let is_ip = addr_to.bytes().filter(|&b| b == b'.').count() == 4;
    if !is_ip {
        let ipx = lookup_host(&addr_to);
        if let Ok(ipx) = ipx {
            if let Some(ip) = ipx.first() {
                addr_target = ip.to_string();
                log::info!("Resolved {} to {}", addr_to, addr_target);
            } else {
                log::error!("Failed to resolve address: {}", addr_to);
            }
        }
    }

    let peer = BasicPeer::new(&addr_target);
    
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
    
    Service::with_listeners(
        "Proxy Service TLS".to_string(),
        listeners,
        proxy_fast::ProxyApp::new(peer,String::from(addr)),
    )
}
