//! # Gateway Application Module (Optimized)
//!
//! This module implements a configurable HTTP gateway/router that directs incoming
//! HTTP requests to appropriate backend services based on path patterns. This version
//! includes several performance and robustness optimizations.
//!
//! ## Features
//!
//! * **Pattern-based routing**: Uses regular expressions to match request paths
//! * **Path transformation**: Efficiently rewrites URLs before forwarding using `Captures::expand`
//! * **Priority-based rules**: Higher priority rules are evaluated first
//! * **Query parameter preservation**: Maintains original query parameters during rewrites
//! * **Default fallback**: Routes unmatched requests to a precomputed default service
//! * **Sharded LRU Caching**: High-performance, contention-reduced caching using the `lru` crate.
//! * **Dynamic Configuration Reloading**: Refreshes routing rules based on configuration changes.
//!
//! ## Architecture
//!
//! The gateway acts as a reverse proxy using the Pingora framework, examining incoming
//! HTTP requests and forwarding them to the appropriate backend services based on
//! configurable routing rules. Each rule specifies a pattern to match, how to transform
//! the path, and where to send the request. Caching is heavily used to speed up routing decisions.
//!
//! ## Example Flow
//!
//! 1. Request arrives: `GET /api/users?page=2`
//! 2. Cache Check: Gateway checks the sharded cache using the full path + query (`/api/users?page=2`).
//! 3. Cache Miss:
//!    a. Gateway checks if configuration needs reloading.
//!    b. Matches the path `/api/users` against the `/api/(.*)` pattern.
//!    c. Path is transformed to `/v2/api/users` using `Captures::expand`.
//!    d. Query `?page=2` is appended.
//!    e. The result (`/v2/api/users?page=2`, target_peer) is stored in the cache.
//! 4. Cache Hit (or after miss processing):
//!    a. Request URI is updated to the target path + query.
//!    b. Request is forwarded to the configured backend service.
//! 5. Response from the backend is returned to the client.

use async_trait::async_trait;
use bytes::Bytes;
use http::StatusCode;
use log::{debug, error, info, warn}; use pingora::http::ResponseHeader;
// Use log macros consistently
use pingora::prelude::*; // Import commonly used items
use pingora::proxy::{ProxyHttp, Session};
use pingora::upstreams::peer::BasicPeer;
use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, LazyLock, RwLock};
use std::time::{Duration, Instant};
// lazy_static is not used anymore
use lru::LruCache; // Use the standard LRU crate

// Assuming these are correctly defined in your project structure
use crate::config::{self, GatewayPath, DEFAULT_PORT};

// Number of cache shards to reduce lock contention
const CACHE_SHARDS: usize = 16;
// Default capacity per shard if not otherwise specified
const DEFAULT_PER_SHARD_CAPACITY: usize = 250; // ~4000 total routes

// --- Sharded LRU Cache Implementation ---
// Uses the `lru` crate for efficient O(1) operations.

struct ShardedLruCache<K, V> {
    shards: Vec<RwLock<LruCache<K, V>>>,
}

impl<K: Hash + Eq + Clone, V: Clone> ShardedLruCache<K, V> {
    fn new(per_shard_capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(per_shard_capacity).unwrap_or_else(|| {
            warn!(
                "Invalid per_shard_capacity (0), using default: {}",
                DEFAULT_PER_SHARD_CAPACITY
            );
            // Use expect here as DEFAULT_PER_SHARD_CAPACITY is a known constant
            NonZeroUsize::new(DEFAULT_PER_SHARD_CAPACITY)
                .expect("Default shard capacity must be non-zero")
        });
        let mut shards = Vec::with_capacity(CACHE_SHARDS);
        for _ in 0..CACHE_SHARDS {
            shards.push(RwLock::new(LruCache::new(capacity)));
        }
        Self { shards }
    }

    #[inline] // Inline for potentially faster access
    fn get_shard_index(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % CACHE_SHARDS
    }

    /// Gets a value from the cache without updating the LRU order.
    fn get(&self, key: &K) -> Option<V> {
        let shard_index = self.get_shard_index(key);
        match self.shards[shard_index].read() {
            Ok(shard) => shard.peek(key).cloned(),
            Err(e) => {
                error!(
                    "Failed to acquire read lock on cache shard {}: {}",
                    shard_index, e
                );
                None // Handle poisoned lock
            }
        }
    }

    /// Inserts a value into the cache, potentially evicting the least recently used item.
    fn insert(&self, key: K, value: V) {
        let shard_index = self.get_shard_index(&key);
        match self.shards[shard_index].write() {
            Ok(mut shard) => {
                shard.put(key, value); // Discard return value
            }
            Err(e) => {
                error!(
                    "Failed to acquire write lock on cache shard {}: {}",
                    shard_index, e
                );
                // Handle poisoned lock - cannot insert
            }
        }
    }

    /// Clears all entries from all cache shards.
    fn clear(&self) {
        for (i, shard_lock) in self.shards.iter().enumerate() {
            match shard_lock.write() {
                Ok(mut shard) => shard.clear(),
                Err(e) => {
                    error!(
                        "Failed to acquire write lock on cache shard {} for clearing: {}",
                        i, e
                    );
                    // Handle poisoned lock - cannot clear this shard
                }
            }
        }
        debug!("Cleared entries from route cache (potentially skipping poisoned shards)");
    }
}

// --- Redirect Rule Definition ---

/// # Redirect Rule
/// Defines a single routing rule.
#[derive(Clone, Debug)]
struct RedirectRule {
    pattern: Regex,             // Compiled regex for matching
    target_template: String,    // Template string for path transformation (e.g., "/v2/api/$1")
    _alt_listen: String,        // Listener address this rule applies to
    alt_target: Arc<BasicPeer>, // Target backend service (Arc for cheap cloning)
    priority: usize,            // Rule evaluation priority (lower value = higher priority)
}

// --- Static Global State ---

// Holds compiled and sorted rules for each listener source. Arc<Vec> allows cheap cloning for reads.
static REDIRECT_RULES: LazyLock<RwLock<HashMap<String, Arc<Vec<RedirectRule>>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

// Holds the ID of the currently loaded configuration to detect changes.
static SAVED_CONFIG_ID: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new(String::new()));

// Precompute the default fallback peer.
static DEFAULT_FALLBACK_PEER: LazyLock<Box<HttpPeer>> = LazyLock::new(|| {
    let addr_str = DEFAULT_PORT.p404; // e.g., "127.0.0.1:4040"
                                      // Create HttpPeer directly - it doesn't return a Result like the code assumed
    let peer = HttpPeer::new(addr_str, false, String::new());
    info!("Precomputed default fallback peer: {}", addr_str);
    Box::new(peer)
    // Emergency fallback would be handled in upstream_peer if needed
});

// --- Gateway Application ---

/// # Gateway Application
/// The main application implementing HTTP proxy routing.
pub struct GatewayApp {
    source: String,                   // Listener address (e.g., "0.0.0.0:8080")
    last_check_time: RwLock<Instant>, // Last time config was checked
    check_interval: Duration,         // How often to check for config changes
    route_cache: Arc<ShardedLruCache<String, (String, Arc<BasicPeer>)>>, // Cache: key=path+query, value=(rewritten_path+query, target_peer)
}

impl GatewayApp {
    /// Creates a new GatewayApp instance for a specific listener source.
    pub fn new(alt_source: &str) -> Self {
        debug!("Creating GatewayApp for source: {}", alt_source);
        let app = GatewayApp {
            source: alt_source.to_string(),
            last_check_time: RwLock::new(Instant::now()),
            check_interval: Duration::from_secs(5), // Check config every 5 seconds
            // Use NonZeroUsize for cache capacity
            route_cache: Arc::new(ShardedLruCache::new(DEFAULT_PER_SHARD_CAPACITY)),
        };
        // Initial population of rules
        app.populate_rules(true);
        app
    }

    /// Populates or refreshes the routing rules from the configuration source.
    /// This is the main function responsible for loading and processing rules.
    fn populate_rules(&self, init: bool) {
        let current_config_id = config::RoutingData::GatewayID.get();
        debug!(
            "Checking configuration. Current ID: '{}'",
            current_config_id
        );
        // Fast path: Check if config ID has changed using a read lock first.
        let config_changed = {
            match SAVED_CONFIG_ID.read() {
                Ok(saved_id_guard) => {
                    log::debug!(
                        "Comparing saved config ID '{}' with current ID '{}'",
                        *saved_id_guard,
                        current_config_id
                    );
                    *saved_id_guard != current_config_id
                }
                Err(e) => {
                    error!("Failed to acquire read lock on SAVED_CONFIG_ID: {}. Assuming config changed.", e);
                    true // Assume change if we can't read
                }
            }
        };

        if !init && !config_changed {
            debug!(
                "Configuration ID unchanged ('{}'). Skipping rule population.",
                current_config_id
            );
            return; // No change detected
        }

        // Config ID has changed (or lock failed), proceed with update.
        // Log the old ID safely
        let old_config_id_str = match SAVED_CONFIG_ID.read() {
            Ok(guard) => guard.clone(),
            Err(_) => "<unknown: read lock failed>".to_string(),
        };
        info!(
            "Configuration change detected ({} -> {}). Reloading rules for source: {}",
            old_config_id_str, current_config_id, self.source
        );

        // Clear the route cache as rules are changing.
        self.route_cache.clear();

        // Load raw rule data from the configuration source.
        let gateway_nodes = match config::RoutingData::GatewayRouting.xget::<Vec<GatewayPath>>() {
            Some(nodes) if !nodes.is_empty() => nodes,
            _ => {
                warn!(
                    "No valid gateway routing rules found in configuration for source '{}'.",
                    self.source
                );
                // Update state even if no rules are found
                self.update_rules_and_config_id(Vec::new(), &current_config_id);
                return;
            }
        };

        // Process and compile rules relevant to *this* gateway instance's source.
        let mut applicable_rules = Vec::new();
        for node in gateway_nodes {
            log::debug!(
                "Processing node: addr_listen={}, addr_target={}, path_listen={}, path_target={}, targetd={}",
                node.addr_listen, node.addr_target, node.path_listen, node.path_target, self.source
            );
            if node.addr_listen != self.source {
                continue;
            }
            // Filter rules for the current listener source.
            log::debug!(
                "Processing rule for source: {}, target: {}",
                node.addr_target,
                self.source
            );

            log::debug!(
                "Path listen: {}, path target: {}",
                node.path_listen,
                node.path_target
            );

            // Determine if this is a plain string path, a wildcard path, or a regex pattern.
            // Process the pattern string to handle different formats
            let processed_pattern = if is_regex_pattern(&node.path_listen) {
                // Already a regex pattern (contains regex special chars other than * at the end)
                debug!("Processing as regex pattern: '{}'", node.path_listen);
                node.path_listen.clone()
            } else if node.path_listen.ends_with("/*") {
                // Wildcard pattern (e.g., "/api/*")
                debug!("Processing as wildcard pattern: '{}'", node.path_listen);
                // Convert "/api/*" to "^/api/.*$"
                let base_path = &node.path_listen[..node.path_listen.len() - 1];
                format!("^{}.*$", base_path)
            } else {
                // Plain string path (e.g., "/test")
                debug!("Processing as exact match pattern: '{}'", node.path_listen);
                // Convert "/test" to "^/test$"
                format!("^{}$", node.path_listen)
            };

            // Compile the processed regex pattern.
            let pattern = match Regex::new(&processed_pattern) {
                Ok(re) => re,
                Err(e) => {
                    warn!(
                        "Invalid regex pattern '{}' (from '{}') for source '{}': {}. Skipping rule.",
                        processed_pattern, node.path_listen, self.source, e
                    );
                    continue;
                }
            };

            // Create the target peer (use Arc for cheap sharing).
            // BasicPeer::new takes &str, so clone addr_target if needed or pass reference
            log::debug!(
                "Creating target peer for address: {}",
                node.addr_target
            );
            let target_peer = Arc::new(BasicPeer::new(&node.addr_target));

            applicable_rules.push(RedirectRule {
                pattern,
                target_template: node.path_target, // Store the template string
                _alt_listen: node.addr_listen,     // Already checked, but store for completeness
                alt_target: target_peer,
                priority: node.priority as usize,
            });
        }
        log::info!(
            "Found {} applicable rules for source: {}",
            applicable_rules.len(),
            self.source
        );
        if applicable_rules.is_empty() {
            info!(
                "No applicable redirect rules found for source: {}",
                self.source
            );
        } else {
            // Sort rules by priority (lower number = higher priority).
            // Use unstable sort as stability is not required.
            applicable_rules.sort_unstable_by_key(|rule| rule.priority);
            info!(
                "Loaded and sorted {} rules for source: {}",
                applicable_rules.len(),
                self.source
            );
        }

        // Update the shared state with the new rules and config ID.
        self.update_rules_and_config_id(applicable_rules, &current_config_id);
    }

    /// Atomically updates the REDIRECT_RULES and SAVED_CONFIG_ID.
    fn update_rules_and_config_id(&self, rules: Vec<RedirectRule>, new_config_id: &str) {
        // Acquire write locks to update the shared data.
        match REDIRECT_RULES.write() {
            Ok(mut rules_map_guard) => {
                // Store rules wrapped in Arc for efficient cloning on read.
                rules_map_guard.insert(self.source.clone(), Arc::new(rules));
            }
            Err(e) => {
                error!(
                    "Failed to acquire write lock on REDIRECT_RULES: {}. Rules not updated.",
                    e
                );
                // Decide if we should still try to update the config ID or return
                return; // Let's return early to avoid inconsistent state
            }
        }

        match SAVED_CONFIG_ID.write() {
            Ok(mut saved_id_guard) => {
                *saved_id_guard = new_config_id.to_string();
                debug!(
                    "Successfully updated rules and saved config ID: '{}'",
                    new_config_id
                );
            }
            Err(e) => {
                error!(
                    "Failed to acquire write lock on SAVED_CONFIG_ID: {}. Config ID not updated.",
                    e
                );
                // Rules were updated, but ID wasn't. This might cause repeated reloads.
            }
        }
    }

    /// Gets a clone of the rules relevant to this gateway instance.
    /// Cloning the Arc is cheap.
    #[inline]
    fn get_rules(&self) -> Arc<Vec<RedirectRule>> {
        match REDIRECT_RULES.read() {
            Ok(rules_map_guard) => {
                rules_map_guard
                    .get(&self.source)
                    .cloned()
                    .unwrap_or_else(|| {
                        log::warn!(
                            "No rules found for source '{}'. Returning empty ruleset.",
                            self.source
                        );
                        Arc::new(Vec::new())
                    })
            }
            Err(e) => {
                error!(
                    "Failed to acquire read lock on REDIRECT_RULES: {}. Returning empty ruleset.",
                    e
                );
                Arc::new(Vec::new()) // Return empty rules on error
            }
        }
    }

    /// Checks if the configuration should be reloaded based on time interval and ID change.
    fn check_and_reload_config_if_needed(&self) {
        let now = Instant::now();
        let needs_check = {
            match self.last_check_time.read() {
                Ok(last_check_guard) => {
                    now.duration_since(*last_check_guard) >= self.check_interval
                }
                Err(e) => {
                    error!("Failed to acquire read lock on last_check_time: {}. Assuming check needed.", e);
                    true // Assume check needed if lock fails
                }
            }
        };

        if needs_check {
            match self.last_check_time.write() {
                Ok(mut last_check_guard) => {
                    // Double-check in case another thread updated it between the read and write lock acquisition.
                    if now.duration_since(*last_check_guard) >= self.check_interval {
                        // Update last check time *before* potentially long-running populate_rules
                        *last_check_guard = now;
                        // Drop the lock before calling populate_rules to avoid holding it too long
                        drop(last_check_guard);
                        // Now perform the actual check and potential reload
                        debug!("Checking rules due to interval check...");
                        self.populate_rules(false);
                    }
                    // If the double-check fails, another thread already handled it.
                }
                Err(e) => {
                    error!("Failed to acquire write lock on last_check_time: {}. Check interval not updated.", e);
                    // Might lead to more frequent checks if this persists
                }
            }
        }
    }
}

/// Helper function to determine if a pattern string contains regex special characters.
///
/// This function checks if a string has regex special metacharacters that would
/// indicate it should be treated as a regex pattern rather than a literal string.
///
/// # Arguments
///
/// * `pattern` - The pattern string to check
///
/// # Returns
///
/// Returns `true` if the string contains regex special characters, `false` otherwise.
fn is_regex_pattern(pattern: &str) -> bool {
    // These are common regex metacharacters excluding the wildcard character at the end
    // which we handle specially
    let regex_special_chars = [
        '^', '$', '.', '+', '?', '(', ')', '[', ']', '{', '}', '|', '\\',
    ];

    // Check if the pattern contains any regex special characters
    for &c in &regex_special_chars {
        if pattern.contains(c) {
            return true;
        }
    }

    // If the pattern has a wildcard in the middle (not at the end), treat as regex
    if pattern.contains('*') && !pattern.ends_with("/*") {
        return true;
    }

    // Otherwise it's a plain string or a simple wildcard pattern
    false
}

#[async_trait]
impl ProxyHttp for GatewayApp {
    type CTX = (); // No context needed for this simple router

    fn new_ctx(&self) -> Self::CTX {}

    /// Core routing logic: checks cache, applies rules, updates request, returns upstream peer.
    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // Use pingora::Result

        // 1. Check and potentially reload configuration first.
        self.check_and_reload_config_if_needed();

        // 2. Prepare cache key (full path + query)
        // Avoid allocation if query is None
        let path = session.req_header().uri.path();
        let query = session.req_header().uri.query();
        // Use Cow for potential zero-allocation case when no query exists
        let cache_key = match query {
            Some(q) => format!("{}?{}", path, q), // Changed to String directly
            None => path.to_string(),             // Convert to String directly
        };

        // 3. Check cache using the String key
        if let Some((rewritten_path_query, peer_arc)) = self.route_cache.get(&cache_key) {
            // Cache Hit!
            debug!("Cache hit for key: {}", cache_key);
            // Update request URI using the cached rewritten path and query.
            match http::uri::PathAndQuery::from_maybe_shared(rewritten_path_query.clone()) {
                Ok(pq) => {
                    let mut parts = session.req_header_mut().uri.clone().into_parts();
                    parts.path_and_query = Some(pq);
                    match http::Uri::from_parts(parts) {
                        Ok(new_uri) => session.req_header_mut().uri = new_uri,
                        Err(e) => {
                            error!("Error rebuilding URI from cached parts: {}", e);
                            // Fallback on error
                            return Ok(DEFAULT_FALLBACK_PEER.clone()); // Clone the precomputed Box<HttpPeer>
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Invalid PathAndQuery in cache: '{}', error: {}",
                        rewritten_path_query, e
                    );
                    // Fallback on error
                    return Ok(DEFAULT_FALLBACK_PEER.clone());
                }
            }

            // Return the cached peer. Cloning Arc is cheap.
            let peer_address = &peer_arc._address; // Get address string directly
            let http_peer = HttpPeer::new(peer_address, false, String::new());
            return Ok(Box::new(http_peer));
        }

        // 4. Cache Miss - Apply routing rules
        debug!("Cache miss for key: {}", cache_key);
        debug!("Checking rules for path: {}", path);
        
        let rules = self.get_rules(); // Gets an Arc<Vec<RedirectRule>>

        for rule in rules.iter() {
            // ADD THIS LINE FOR DEBUGGING:
            debug!("Testing path '{}' against rule pattern: '{}' (priority: {})", path, rule.pattern, rule.priority);

            // Match against the path part only
            if let Some(captures) = rule.pattern.captures(path) {
                // Rule matches!
                debug!(
                    "Rule matched: pattern='{}', target='{}'",
                    rule.pattern, rule.target_template
                );

                // FIX: Use Captures::expand with a String buffer.
                let mut rewritten_path_buf = String::new(); // Use String buffer
                captures.expand(&rule.target_template, &mut rewritten_path_buf); // Pass &mut String

                // FIX: rewritten_path_buf is already a String, no need for from_utf8_lossy
                let rewritten_path = rewritten_path_buf; // Already a String

                // Combine rewritten path with original query string.
                let final_path_query = match query {
                    Some(q) => format!("{}?{}", rewritten_path, q),
                    None => rewritten_path, // Already a String
                };

                // Update request URI
                match http::uri::PathAndQuery::from_maybe_shared(final_path_query.clone()) {
                    Ok(pq) => {
                        let mut parts = session.req_header_mut().uri.clone().into_parts();
                        parts.path_and_query = Some(pq);
                        match http::Uri::from_parts(parts) {
                            Ok(new_uri) => session.req_header_mut().uri = new_uri,
                            Err(e) => {
                                error!("Error rebuilding URI after rewrite: {}", e);
                                // Fallback on error
                                return Ok(DEFAULT_FALLBACK_PEER.clone());
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "Invalid PathAndQuery after rewrite: '{}', error: {}",
                            final_path_query, e
                        );
                        // Fallback on error
                        return Ok(DEFAULT_FALLBACK_PEER.clone());
                    }
                }

                // Cache the result (cloning Arc is cheap)
                // Use into_owned() on cache_key if it was borrowed
                self.route_cache.insert(
                    cache_key.to_owned(),
                    (final_path_query, rule.alt_target.clone()),
                );
                debug!("Cached result for key used in insertion"); // Key might have been owned now
                // Return the target peer for this rule.
                // Use the address string from BasicPeer directly
                let peer_address = &rule.alt_target._address; // Get address string
                let http_peer = HttpPeer::new(peer_address, false, String::new()); // Create HttpPeer directly
                return Ok(Box::new(http_peer)); // Box the HttpPeer and return
            }
        }

        // 5. No rules matched - use the precomputed default fallback
        debug!(
            "No matching rules for path '{}', using default fallback.",
            path
        );
        // Clone the precomputed Box<HttpPeer>
        Ok(DEFAULT_FALLBACK_PEER.clone())
    }
    

    /// Logs request details after completion.
    async fn logging(&self, session: &mut Session, _e: Option<&Error>, _ctx: &mut Self::CTX) {
        let response_code = session
            .response_written()
            .map_or(0, |resp| resp.status.as_u16());
        let path = session.req_header().uri.path(); // Borrow path directly
        let body_size = session.body_bytes_sent();

        // Log using borrowed path to avoid allocation
        info!(
            "[GWX] | ID:{}, STATUS:{}, SIZE:{} |",
            path, response_code, body_size,
        );

        // Optional: Log cache stats periodically if needed (requires adding counters)
        // Consider adding AtomicUsize counters to ShardedLruCache if stats are needed.
    }
}
