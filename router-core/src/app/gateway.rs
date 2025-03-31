use async_trait::async_trait;
use log::info;
use pingora::connectors::TransportConnector;
use pingora::prelude::HttpPeer;
use pingora::proxy::{ProxyHttp, Session};
use pingora::upstreams::peer::BasicPeer;
use regex::Regex;

pub struct RedirectRule {
    pattern: Regex,
    target: String,
    alt_listen: String,
    alt_target: Option<BasicPeer>,
    priority: usize,
}

pub struct GatewayApp {
    redirects: Vec<RedirectRule>,
}

impl GatewayApp {
    pub fn new(alt_source: &str) -> Self {
        let mut redirects = vec![
            RedirectRule {
                pattern: Regex::new("^/favicon\\.ico$").unwrap(),
                target: "/favicon.ico".to_string(),
                alt_target: Some(BasicPeer::new("127.0.0.1:3000")),
                alt_listen: "127.0.0.1:30001".to_string(),
                priority: 0,
            },
            RedirectRule {
                pattern: Regex::new("^/api/(.*)$").unwrap(),
                target: "/v2/api/$1".to_string(),
                alt_target: Some(BasicPeer::new("127.0.0.1:3002")),
                alt_listen: "127.0.0.1:30003".to_string(),
                priority: 1,
            },
            RedirectRule {
                pattern: Regex::new(r"^/ws(.*)$").unwrap(),
                target: "/$1".to_string(),
                alt_target: Some(BasicPeer::new("127.0.0.1:3004")),
                alt_listen: "127.0.0.1:30003".to_string(),
                priority: 1,
            },
            // RedirectRule {
            //     pattern: Regex::new(r"^/(.*)$").unwrap(),
            //     target: "/$1".to_string(),
            //     alt_target: Some(BasicPeer::new("127.0.0.1:3002")),
            //     alt_listen: "127.0.0.1:9010".to_string(),
            //     priority: 0,
            // },
        ];
        redirects.retain(|rule| rule.alt_listen == alt_source);
        redirects.sort_by(|a, b| b.priority.cmp(&a.priority));
        let mut client_connectors = std::collections::HashMap::new();
        for rule in &redirects {
            if let Some(target) = &rule.alt_target {
                let addr = format!("{}", target);
                if !client_connectors.contains_key(&addr) {
                    client_connectors.insert(addr, TransportConnector::new(None));
                }
            }
        }
        GatewayApp {
            redirects,
        }
    }
}

#[async_trait]
impl ProxyHttp for GatewayApp {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}
    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();

        // Try to match path against our redirect rules
        for rule in &self.redirects {
            if let Some(captures) = rule.pattern.captures(path) {
                if let Some(alt_target) = &rule.alt_target {
                    // Transform the path based on the rule's target pattern
                    let mut new_path = rule.target.clone();

                    // Replace capture groups like $1, $2, etc. in the target pattern
                    for i in 1..captures.len() {
                        if let Some(capture) = captures.get(i) {
                            new_path = new_path.replace(&format!("${}", i), capture.as_str());
                        }
                    }

                    // Update the request path
                    let uri = session.req_header_mut().uri.clone();
                    let mut parts = uri.into_parts();

                    // Get the original path and query
                    let path_and_query = parts
                        .path_and_query
                        .unwrap_or_else(|| http::uri::PathAndQuery::from_static("/"));

                    // Preserve the query string if there is one
                    let query = path_and_query
                        .query()
                        .map(|q| format!("?{}", q))
                        .unwrap_or_default();

                    // Create the new path with the transformed path and original query
                    let new_path_and_query = format!("{}{}", new_path, query);
                    parts.path_and_query = Some(
                        http::uri::PathAndQuery::from_maybe_shared(new_path_and_query.into_bytes())
                            .expect("Valid URI"),
                    );

                    // Update the URI in the request header
                    session.req_header_mut().uri = http::Uri::from_parts(parts).expect("Valid URI");

                    let addr = alt_target._address.to_string();
                    let new_peer = HttpPeer::new(addr, false, "".to_string());
                    let peer = Box::new(new_peer);
                    return Ok(peer);
                }
            }
        }

        // Default fallback if no rules match or if matched rule has no alt_target
        let addr = ("127.0.0.1", 12871);
        info!("No matching rules, connecting to default {addr:?}");
        let peer = Box::new(HttpPeer::new(addr, false, "".to_string()));
        Ok(peer)
    }
    // Log request and response metrics.
    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        _ctx: &mut Self::CTX,
    ) {
        let response_code = session
            .response_written()
            .map_or(0, |resp| resp.status.as_u16());
        info!("Response code: {}", response_code);
        // Insert any additional metric logging here (e.g., Prometheus counters)
    }
}
