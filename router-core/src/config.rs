use mini_config::Configure;

pub struct DefaultPort {
    pub p404: &'static str,
    pub p500: &'static str,
    pub tls_honeypot: &'static str,
}

pub(crate) const DEFAULT_PORT: DefaultPort = DefaultPort {
    p404: "127.0.0.1:60404",
    p500: "127.0.0.1:60500",
    tls_honeypot: "127.0.0.1:60443",
};

#[derive(Debug, Clone, Configure)]
pub enum RoutingData {
    ProxyID,
    ProxyRouting,
    GatewayID,
    GatewayRouting
}

