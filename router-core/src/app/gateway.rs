//! # Gateway Application Module
//!
//! This module implements a configurable HTTP gateway/router that directs incoming
//! HTTP requests to appropriate backend services based on path patterns.
//!
//! ## Features
//!
//! * **Pattern-based routing**: Uses regular expressions to match request paths
//! * **Path transformation**: Rewrites URLs before forwarding to backends
//! * **Priority-based rules**: Higher priority rules are evaluated first
//! * **Query parameter preservation**: Maintains original query parameters during rewrites
//! * **Default fallback**: Routes unmatched requests to a configurable default service
//!
//! ## Architecture
//!
//! The gateway acts as a reverse proxy using the Pingora framework, examining incoming
//! HTTP requests and forwarding them to the appropriate backend services based on
//! configurable routing rules. Each rule specifies a pattern to match, how to transform
//! the path, and where to send the request.
//!
//! ## Example Flow
//!
//! 1. Request arrives: `GET /api/users?page=2`
//! 2. Gateway matches the path against the `/api/(.*)` pattern
//! 3. Path is transformed to `/v2/api/users` (while preserving `?page=2`)
//! 4. Request is forwarded to the configured backend service
//! 5. Response from the backend is returned to the client

use async_trait::async_trait;
use pingora::prelude::HttpPeer;
use pingora::proxy::{ProxyHttp, Session};
use pingora::upstreams::peer::BasicPeer;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};
use std::time::{Duration, Instant};

use crate::config::{self, GatewayNode, DEFAULT_PORT};

/// # Redirect Rule
///
/// Defines a single routing rule that determines how requests matching specific
/// path patterns should be handled and redirected to backend services.
///
/// ## Pattern Matching and Transformation
///
/// The rule uses regular expressions with capture groups to match and transform paths:
/// - The `pattern` field contains a regex that matches against request paths
/// - The `target` field defines how to transform the path, using `$n` syntax to refer to capture groups
///
/// ## Example
///
/// With a pattern of `^/api/(.*)$` and target of `/v2/api/$1`:
/// - A request to `/api/users` would be transformed to `/v2/api/users`
/// - The capture group `(.*)` captures `users` and `$1` in the target refers to this captured text
///
/// ## Fields
///
/// * `pattern` - Regular expression pattern to match against incoming request paths
/// * `target` - Template for path transformation, may include capture group references like `$1`
/// * `alt_listen` - The address:port string identifying which listener this rule belongs to
/// * `alt_target` - Destination backend service for matched requests
/// * `priority` - Rule evaluation priority (higher values are processed first)
#[derive(Clone, Debug)]
struct RedirectRule {
    pattern: Regex,
    target: String,
    alt_listen: String,
    alt_target: Option<BasicPeer>,
    priority: usize,
}

// Static map to hold redirect rules for different sources
static REDIRECT_RULES: LazyLock<RwLock<HashMap<String, Vec<RedirectRule>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

// Static to hold the saved ID for config versioning
static SAVED_CONFIG_ID: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new(String::new()));

/// # Gateway Application
///
/// The main application that implements HTTP proxy routing functionality.
///
/// The gateway holds a prioritized collection of redirect rules and uses them to
/// determine how to route incoming HTTP requests. When a request arrives, the gateway
/// evaluates it against all applicable rules (filtered by listener address) in priority
/// order and routes the request according to the first matching rule.
///
/// ## Rule Filtering and Priority
///
/// Rules are filtered by the listener address provided during initialization, ensuring
/// that each gateway instance only processes rules relevant to its listening socket.
/// Rules are sorted by priority, with higher priority rules evaluated first.
///
/// ## Rule Matching Process
///
/// For each request, the gateway:
/// 1. Examines the request path
/// 2. Tests it against each rule's pattern in priority order
/// 3. For the first matching rule:
///    - Transforms the path according to the rule's target pattern
///    - Preserves query parameters
///    - Routes to the specified backend service
/// 4. If no rules match, routes to a default fallback service
///
/// ## Fields
///
/// * `source` - The listener address this gateway instance is bound to
pub struct GatewayApp {
    source: String,
    last_check_time: RwLock<Instant>, // Use RwLock for thread-safe interior mutability
    check_interval: Duration,         // Interval between configuration checks
}

impl GatewayApp {
    /// # Create a new Gateway Application
    ///
    /// Initializes a new gateway instance with predefined redirect rules, filtered
    /// by the specified listener address.
    ///
    /// ## Rule Configuration
    ///
    /// The rules are filtered by the `alt_source` parameter, so only rules matching
    /// the specified listener address will be active for this gateway instance.
    ///
    /// ## Connection Management
    ///
    /// This method also initializes connection handlers for all target backends that
    /// might be needed by the active rules.
    ///
    /// ## Arguments
    ///
    /// * `alt_source` - The listener address:port string for this gateway instance
    ///
    /// ## Returns
    ///
    /// A new `GatewayApp` instance with configured and prioritized redirect rules
    pub fn new(alt_source: &str) -> Self {
        log::debug!("Creating GatewayApp with source: {}", alt_source);
        let app = GatewayApp {
            source: alt_source.to_string(),
            last_check_time: RwLock::new(Instant::now()),
            check_interval: Duration::from_secs(5), // Check config every 5 seconds
        };
        app.populate();
        app
    }

    fn populate(&self) {
        log::debug!("Populating redirect rules for source: {}", self.source);
        // Get read access to check if we need to update
        let config_id = config::RoutingData::GatewayID.get();

        log::debug!("Current config ID: {}", config_id);

        {
            // First use a read lock to check if update is needed
            let saved_id_guard = SAVED_CONFIG_ID.read().unwrap();
            if *saved_id_guard == config_id {
                log::debug!("No changes in routing rules, skipping population");
                return;
            }
        }

        log::debug!("Updating redirect rules for source: {}", self.source);

        let node = config::RoutingData::GatewayRouting.xget::<Vec<GatewayNode>>();
        let mut redirects: Vec<RedirectRule> = vec![];

        // Process gateway rules if they exist
        if let Some(rules) = node {
            for rule in rules {
                let pattern = Regex::new(&rule.path_listen).unwrap();
                let target = rule.path_target.clone();
                let alt_listen = rule.addr_listen.clone();
                let alt_target = rule.addr_target.clone();
                let priority = rule.priority as usize;
                redirects.push(RedirectRule {
                    pattern,
                    target,
                    alt_listen,
                    alt_target: Some(BasicPeer::new(&alt_target)),
                    priority,
                });
            }
        }

        log::debug!("Redirect rules loaded: {}", redirects.len());

        if redirects.is_empty() {
            log::debug!("No redirect rules found");
            return;
        }

        // Process and store rules for each source
        let mut source_rules: Vec<RedirectRule> = redirects
            .into_iter()
            .filter(|rule| rule.alt_listen == self.source)
            .collect();

        // Sort by priority
        source_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Now acquire write locks to update the data
        {
            // Lock the REDIRECT_RULES for writing
            let mut rules_map = REDIRECT_RULES.write().unwrap();
            // Store rules for this source
            rules_map.insert(self.source.clone(), source_rules);
        }

        {
            // Lock the SAVED_CONFIG_ID for writing
            let mut saved_id_guard = SAVED_CONFIG_ID.write().unwrap();
            // Update saved ID to indicate we've processed this configuration
            *saved_id_guard = config_id;
        }
    }

    // Helper to get the rules for this instance's source
    fn get_rules(&self) -> Vec<RedirectRule> {
        // Only need read access here
        let rules_map = REDIRECT_RULES.read().unwrap();
        rules_map.get(&self.source).cloned().unwrap_or_default()
    }

    // Check if we should refresh configuration based on time interval and update the timestamp
    fn should_check_config(&self) -> bool {
        let now = Instant::now();
        let should_update = {
            let last_check = self.last_check_time.read().unwrap();
            now.duration_since(*last_check) >= self.check_interval
        };

        if should_update {
            // Update the timestamp using RwLock interior mutability
            let mut last_check = self.last_check_time.write().unwrap();
            *last_check = now;
        }

        should_update
    }
}

#[async_trait]
impl ProxyHttp for GatewayApp {
    /// # Proxy Context Type
    ///
    /// Defines the type of context data used during proxy operations.
    /// Currently empty as no context data is needed.
    type CTX = ();

    /// # Create Proxy Context
    ///
    /// Initializes a new context for a proxy session. The context can be used to
    /// store state or data that needs to be passed between different proxy handler methods.
    ///
    /// ## Returns
    ///
    /// An empty context value, as this implementation doesn't require context data.
    fn new_ctx(&self) -> Self::CTX {}

    /// # Determine Upstream Target
    ///
    /// This method is called for each incoming request to determine where to route it.
    /// It's the core of the gateway's routing logic and implements path matching,
    /// transformation, and forwarding.
    ///
    /// ## Processing Steps
    ///
    /// 1. Extract the request path from the session
    /// 2. Iterate through redirect rules in priority order
    /// 3. For each rule, check if the path matches the rule's pattern
    /// 4. If a match is found and has a target:
    ///    - Transform the path using capture groups
    ///    - Preserve query parameters from the original URL
    ///    - Create a new URL with the transformed path and original query
    ///    - Update the request URI
    ///    - Create a peer representing the target backend
    /// 5. If no rules match, route to the default fallback service
    ///
    /// ## Capture Group Handling
    ///
    /// The method supports regex capture groups in patterns and can substitute them
    /// into the target path using `$n` syntax, where `n` is the capture group index.
    ///
    /// ## Arguments
    ///
    /// * `session` - The HTTP session containing the request to be routed
    /// * `_ctx` - Unused context parameter
    ///
    /// ## Returns
    ///
    /// A `Result` containing a boxed `HttpPeer` that represents the upstream server
    /// that will handle the request.
    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();

        // Only check for config changes periodically using our interior mutability pattern
        if self.should_check_config() {
            self.populate();
        }

        // Get the rules for this source
        let rules = self.get_rules();

        // Try to match path against our redirect rules
        for rule in &rules {
            if let Some(captures) = rule.pattern.captures(path) {
                if let Some(alt_target) = &rule.alt_target {
                    // Start with target pattern - avoid clone by using a reference when possible
                    let target_ref = &rule.target;

                    // Check if we need replacements at all
                    let needs_replacement = target_ref.contains('$');

                    // Only allocate and process if replacements are needed
                    let final_path = if needs_replacement {
                        let mut new_path = target_ref.to_string();
                        // Pre-compute capture group references to avoid multiple format! calls
                        let mut replacements = Vec::with_capacity(captures.len() - 1);

                        for i in 1..captures.len() {
                            if let Some(capture) = captures.get(i) {
                                replacements.push((format!("${}", i), capture.as_str()));
                            }
                        }

                        // Do all replacements in one pass
                        for (pattern, replacement) in replacements {
                            new_path = new_path.replace(&pattern, replacement);
                        }

                        new_path
                    } else {
                        target_ref.to_string()
                    };

                    // Avoid cloning the URI if possible, work directly with the session's URI
                    let uri_ref = &mut session.req_header_mut().uri;

                    // Get the original query without allocating when possible
                    let query_opt = uri_ref.query();

                    // Construct the final path and query
                    let new_path_and_query = if let Some(query) = query_opt {
                        format!("{}?{}", final_path, query)
                    } else {
                        final_path
                    };

                    // Rebuild the URI with the new path and query
                    let mut parts = uri_ref.clone().into_parts();
                    parts.path_and_query = Some(
                        http::uri::PathAndQuery::from_maybe_shared(new_path_and_query.into_bytes())
                            .expect("Valid URI"),
                    );
                    *uri_ref = http::Uri::from_parts(parts).expect("Valid URI");

                    // Create the peer directly with the correct String type
                    let addr_str = alt_target._address.to_string();
                    let new_peer = HttpPeer::new(addr_str, false, String::new());
                    return Ok(Box::new(new_peer));
                }
            }
        }

        // Default fallback if no rules match or if matched rule has no alt_target
        let port_str = DEFAULT_PORT.p404;
        let parts: Vec<&str> = port_str.split(':').collect();
        let addr = (parts[0], parts[1].parse::<u16>().unwrap_or(80));
        log::debug!("No matching rules, connecting to default {addr:?}");
        let peer = Box::new(HttpPeer::new(addr, false, String::new()));
        Ok(peer)
    }

    /// # Log Request Metrics
    ///
    /// Records metrics and logs information about completed requests. This method
    /// is called after a request has been processed and the response has been sent.
    ///
    /// ## Logging Features
    ///
    /// - Extracts and logs the request path
    /// - Extracts and logs the HTTP response status code
    /// - Extracts and logs the response body size in bytes
    /// - Provides a placeholder for additional metric collection
    ///   (e.g., Prometheus counters, latency metrics)
    ///
    /// ## Arguments
    ///
    /// * `session` - The HTTP session containing the processed request and response
    /// * `_e` - Optional error that occurred during request processing
    /// * `_ctx` - Unused context parameter
    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        _ctx: &mut Self::CTX,
    ) {
        let response_code = session
            .response_written()
            .map_or(0, |resp| resp.status.as_u16());

        // Extract the request path
        let path = session.req_header().uri.path().to_string();
        // remove query parameters from the path
        let path = path.split('?').next().unwrap_or(path.as_str()).to_string();
        let body_size = session.body_bytes_sent();
        
        log::info!(
            "[GWX] | ID:{}, CONN:TLS/TCP, STATUS:{}, SIZE:{}, COMMENT:|",
            path,
            response_code,
            body_size,
        );
    }
}
