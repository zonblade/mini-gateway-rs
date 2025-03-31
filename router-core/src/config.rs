use mini_config::Configure;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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
    GatewayRouting,
}

#[derive(Debug, Clone, Configure)]
pub enum GeneralConfig {
    RedisURI
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyNode {
    pub tls: bool,
    pub sni: Option<String>,
    pub tls_pem: Option<String>,
    pub tls_key: Option<String>,
    pub addr_listen: String,
    pub addr_target: String,
    pub priority: i8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayNode {
    pub priority: i8,
    pub addr_listen: String,
    pub addr_target: String,
    pub path_listen: String,
    pub path_target: String,
}

pub fn str_to_json<T>(json_str: &str) -> T
where
    T: DeserializeOwned,
{
    serde_json::from_str(json_str).unwrap()
}

pub fn init(){
    // initiate the routing id
    RoutingData::ProxyID.set("-");
    RoutingData::GatewayID.set("-");

    // initiate the routing data
    RoutingData::GatewayRouting.xset::<Vec<GatewayNode>>(vec![]);
    RoutingData::ProxyRouting.xset::<Vec<ProxyNode>>(vec![]);
}