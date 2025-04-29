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
use std::sync::{LazyLock, RwLock, Arc};
use std::time::{Duration, Instant};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::borrow::Cow;
use lazy_static::lazy_static;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashSet;
use std::cell::RefCell;

// Number of cache shards to reduce lock contention
const CACHE_SHARDS: usize = 32; // Increased from 16 to 32 for better sharding
const MAX_CACHE_ENTRIES: usize = 8192; // Increased total cache size

// Pre-allocate static buffers for common string operations
thread_local! {
    // Buffer for path transformations to avoid frequent allocations
    static PATH_BUFFER: std::cell::RefCell<String> = std::cell::RefCell::new(String::with_capacity(256));
    
    // Buffer for cache key generation
    static CACHE_KEY_BUFFER: std::cell::RefCell<String> = std::cell::RefCell::new(String::with_capacity(512));
}

// Sharded LRU cache implementation for high-concurrency scenarios
struct ShardedCache<K, V> {
    shards: Vec<RwLock<LruCache<K, V>>>,
    total_hits: AtomicUsize,
    total_misses: AtomicUsize,
    // Track last eviction time per shard to distribute evictions
    last_evictions: Vec<RwLock<Instant>>,
}

impl<K: Hash + Eq + Clone, V: Clone> ShardedCache<K, V> {
    fn new(shard_capacity: usize) -> Self {
        let mut shards = Vec::with_capacity(CACHE_SHARDS);
        let mut last_evictions = Vec::with_capacity(CACHE_SHARDS);
        
        for i in 0..CACHE_SHARDS {
            shards.push(RwLock::new(LruCache::new(shard_capacity)));
            // Stagger initial eviction times to avoid simultaneous evictions across shards
            last_evictions.push(RwLock::new(Instant::now() - Duration::from_millis((i * 100) as u64)));
        }
        
        Self {
            shards,
            total_hits: AtomicUsize::new(0),
            total_misses: AtomicUsize::new(0),
            last_evictions,
        }
    }
    
    #[inline]
    fn get(&self, key: &K) -> Option<V> {
        // Calculate which shard this key belongs to
        let shard_index = self.get_shard_index(key);
        
        // Get a read lock on only that shard (reduced lock contention)
        if let Ok(shard) = self.shards[shard_index].read() {
            // Try to get the value from the cache
            match shard.get(key) {
                Some(value) => {
                    self.total_hits.fetch_add(1, Ordering::Relaxed);
                    return Some(value.clone());
                },
                None => {
                    self.total_misses.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        None
    }
    
    #[inline]
    fn insert(&self, key: K, value: V) {
        // Calculate which shard this key belongs to
        let shard_index = self.get_shard_index(&key);
        
        // Get a write lock on only that shard (other shards remain available)
        if let Ok(mut shard) = self.shards[shard_index].write() {
            // Check if eviction might be needed
            if shard.is_near_capacity() {
                // Time-based gradual eviction to avoid spikes
                if let Ok(mut last_time) = self.last_evictions[shard_index].write() {
                    let now = Instant::now();
                    let elapsed = now.duration_since(*last_time);
                    
                    // Only perform eviction if sufficient time has passed (100ms minimum)
                    if elapsed > Duration::from_millis(100) {
                        // Evict just a few items (max 2) to spread out the eviction work
                        shard.evict_oldest(2);
                        *last_time = now;
                    }
                }
            }
            
            // Insert into the cache
            shard.insert(key, value);
        }
    }
    
    #[inline]
    fn get_shard_index(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % CACHE_SHARDS
    }
    
    // Clear all entries from all cache shards
    fn clear(&self) {
        for shard in &self.shards {
            if let Ok(mut shard_guard) = shard.write() {
                shard_guard.cache.clear();
            }
        }
        log::debug!("Cleared all entries from route cache");
    }
    
    // Partial clear - only clear entries that match a predicate
    fn clear_matching<F>(&self, predicate: F) where F: Fn(&K) -> bool + Send + Sync {
        for shard in &self.shards {
            if let Ok(mut shard_guard) = shard.write() {
                shard_guard.clear_matching(&predicate);
            }
        }
        log::debug!("Selectively cleared matching entries from route cache");
    }
    
    // Get cache hit rate statistics
    #[allow(dead_code)]
    fn stats(&self) -> (usize, usize, f64) {
        let hits = self.total_hits.load(Ordering::Relaxed);
        let misses = self.total_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
        (hits, misses, hit_rate)
    }
}

// Simple LRU cache implementation with optimized eviction policy
struct LruCache<K, V> {
    cache: HashMap<K, (V, Instant)>,
    capacity: usize,
    hits: AtomicUsize,
    misses: AtomicUsize,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            capacity,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn get(&self, key: &K) -> Option<&V> {
        match self.cache.get(key) {
            Some((v, _)) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(v)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    #[inline]
    fn insert(&mut self, key: K, value: V) {
        // Fast path: If under capacity, just insert
        if self.cache.len() < self.capacity {
            self.cache.insert(key, (value, Instant::now()));
            return;
        }
        
        // If we need to evict, call the evict_oldest method
        if self.cache.len() >= self.capacity {
            self.evict_oldest(1); // Evict at least one entry
        }
        
        // Now insert the new entry
        self.cache.insert(key, (value, Instant::now()));
    }
    
    // Check if cache is near capacity
    #[inline]
    fn is_near_capacity(&self) -> bool {
        self.cache.len() >= (self.capacity * 90) / 100 // 90% full
    }
    
    // Evict a specific number of the oldest entries
    #[inline]
    fn evict_oldest(&mut self, count: usize) {
        if count == 0 || self.cache.is_empty() {
            return;
        }
        
        // Find the oldest entries
        let mut entries: Vec<(K, Instant)> = self.cache
            .iter()
            .map(|(k, (_, t))| (k.clone(), *t))
            .collect();
        
        // Only sort as many entries as we need to evict
        if entries.len() > count {
            // Use partial_sort to only sort the oldest 'count' entries - more efficient
            entries.sort_by_key(|(_, timestamp)| *timestamp);
            entries.truncate(count);
        } else {
            entries.sort_by_key(|(_, timestamp)| *timestamp);
        }
        
        // Remove the oldest entries
        for (key, _) in entries {
            self.cache.remove(&key);
        }
    }
    
    // Clear entries matching a predicate
    fn clear_matching<F>(&mut self, predicate: &F) where F: Fn(&K) -> bool {
        self.cache.retain(|k, _| !predicate(k));
    }
    
    // Get cache hit rate statistics
    #[allow(dead_code)]
    fn stats(&self) -> (usize, usize, f64) {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
        (hits, misses, hit_rate)
    }
}

// Precompiled regex patterns for better performance
lazy_static! {
    static ref DOLLAR_SIGN_PATTERN: Regex = Regex::new(r"\$(\d+)").unwrap();
}

// Thread-local cache for common replacement patterns
thread_local! {
    // Pre-computed replacement templates for common patterns
    static REPLACEMENT_CACHE: RefCell<HashMap<String, Vec<(usize, usize, usize)>>> = RefCell::new(HashMap::with_capacity(64));
    
    // Lookup structure for fast replacements
    static DOLLAR_POSITIONS: RefCell<Vec<(usize, usize, usize)>> = RefCell::new(Vec::with_capacity(8));
}

use crate::config::{self, GatewayNode, DEFAULT_PORT};

/// # Fast pattern matcher for pre-filtering
#[derive(Clone, Debug)]
struct FastMatcher {
    regex: Regex,
}

impl FastMatcher {
    fn new(pattern: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let regex = Regex::new(pattern)?;
        
        Ok(FastMatcher {
            regex,
        })
    }
    
    #[inline]
    fn is_match(&self, input: &str) -> bool {
        self.regex.is_match(input)
    }
}

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
    fast_matcher: Option<Arc<FastMatcher>>, // Add a fast matcher for initial filtering
    target: String,
    alt_listen: String,
    alt_target: Option<BasicPeer>,
    target_addr_str: String, // Pre-formatted target address string
    priority: usize,
    has_replacements: bool, // Flag to optimize path transformation
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
    // Cache Value: (Transformed Path+Query, Target Peer, Target Address String)
    route_cache: Arc<ShardedCache<String, (String, BasicPeer, String)>>, 
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
        
        // Calculate shard capacity - distribute cache entries across shards
        let per_shard_capacity = MAX_CACHE_ENTRIES / CACHE_SHARDS;
        
        let app = GatewayApp {
            source: alt_source.to_string(),
            last_check_time: RwLock::new(Instant::now()),
            check_interval: Duration::from_secs(5), // Check config every 5 seconds
            route_cache: Arc::new(ShardedCache::new(per_shard_capacity)),
        };
        app.populate();
        app
    }

    fn populate(&self) {
        log::debug!("Populating redirect rules for source: {}", self.source);
        // Get read access to check if we need to update
        let config_id = config::RoutingData::GatewayID.get();
        let current_rules_map: HashMap<String, Vec<RedirectRule>>;
        let saved_id: String;

        log::debug!("Current config ID: {}", config_id);

        // Fast path: First use a read lock to check if update is needed
        {
            let saved_id_guard = SAVED_CONFIG_ID.read().unwrap();
            saved_id = saved_id_guard.clone();
            
            if saved_id == config_id {
                log::debug!("No changes in routing rules, skipping population");
                return;
            }
            
            // Store current rules for incremental update
            let rules_map = REDIRECT_RULES.read().unwrap();
            if rules_map.contains_key(&self.source) {
                current_rules_map = rules_map
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
            } else {
                current_rules_map = HashMap::new();
            }
        }

        let node = config::RoutingData::GatewayRouting.xget::<Vec<GatewayNode>>();
        
        // Create empty rules vector for this source - will be used if no valid rules are found
        let empty_rules = Vec::new();
        
        // Early return if no rules exist - but still update SAVED_CONFIG_ID
        let rules = match node {
            Some(rules) => rules,
            None => {
                log::debug!("No redirect rules found in configuration");
                
                // Only selectively clear cache if previous rules existed
                if current_rules_map.contains_key(&self.source) {
                    // Clear cache for this source as all rules are gone
                    self.route_cache.clear();
                }
                
                // Update SAVED_CONFIG_ID even when no rules are found
                // First clear any existing rules for this source
                {
                    let mut rules_map = REDIRECT_RULES.write().unwrap();
                    rules_map.insert(self.source.clone(), empty_rules);
                }
                
                // Update the saved config ID
                {
                    let mut saved_id_guard = SAVED_CONFIG_ID.write().unwrap();
                    *saved_id_guard = config_id.clone();
                }
                
                log::debug!("Saved empty rules with config ID: {}", config_id);
                return;
            }
        };
        
        if rules.is_empty() {
            log::debug!("Empty redirect rules array");
            
            // Only selectively clear cache if previous rules existed
            if current_rules_map.contains_key(&self.source) {
                // Clear cache for this source as all rules are gone
                self.route_cache.clear();
            }
            
            // Update SAVED_CONFIG_ID even when rules are empty
            // First clear any existing rules for this source
            {
                let mut rules_map = REDIRECT_RULES.write().unwrap();
                rules_map.insert(self.source.clone(), empty_rules);
            }
            
            // Update the saved config ID
            {
                let mut saved_id_guard = SAVED_CONFIG_ID.write().unwrap();
                *saved_id_guard = config_id.clone();
            }
            
            log::debug!("Saved empty rules with config ID: {}", config_id);
            return;
        }

        // Pre-allocate with capacity for better performance
        let mut redirects = Vec::with_capacity(rules.len());

        // Process gateway rules
        let mut host_patterns: HashSet<String> = HashSet::new();
        
        for rule in rules {
            // Skip invalid patterns early
            let pattern = match Regex::new(&rule.path_listen) {
                Ok(pattern) => pattern,
                Err(e) => {
                    log::warn!("Invalid regex pattern '{}': {}", rule.path_listen, e);
                    continue;
                }
            };
            
            // Try to create a fast matcher as well
            let fast_matcher = match FastMatcher::new(&rule.path_listen) {
                Ok(matcher) => Some(Arc::new(matcher)),
                Err(_) => None,
            };
            
            let alt_target = rule.addr_target.clone();
            let has_replacements = rule.path_target.contains('$');
            
            // Track host patterns for cache invalidation
            host_patterns.insert(rule.path_listen.clone());
            
            // Pre-calculate the address string
            let target_peer = BasicPeer::new(&alt_target);
            let target_addr_str = target_peer._address.to_string();

            redirects.push(RedirectRule {
                pattern,
                fast_matcher,
                target: rule.path_target.clone(),
                alt_listen: rule.addr_listen.clone(),
                alt_target: Some(target_peer),
                target_addr_str,
                priority: rule.priority as usize,
                has_replacements,
            });
        }

        log::debug!("Redirect rules loaded: {}", redirects.len());

        if redirects.is_empty() {
            log::debug!("No valid redirect rules found");
            
            // Only selectively clear cache if previous rules existed
            if current_rules_map.contains_key(&self.source) {
                // Clear cache for this source as all rules are gone
                self.route_cache.clear();
            }
            
            // Update SAVED_CONFIG_ID even when no valid rules are found
            // First clear any existing rules for this source
            {
                let mut rules_map = REDIRECT_RULES.write().unwrap();
                rules_map.insert(self.source.clone(), empty_rules);
            }
            
            // Update the saved config ID
            {
                let mut saved_id_guard = SAVED_CONFIG_ID.write().unwrap();
                *saved_id_guard = config_id.clone();
            }
            
            log::debug!("Saved empty rules with config ID: {}", config_id);
            return;
        }

        // Process and store rules for each source - filter with exact match for better performance
        let my_source = &self.source;
        let mut source_rules: Vec<RedirectRule> = redirects
            .into_iter()
            .filter(|rule| rule.alt_listen == *my_source)
            .collect();

        if source_rules.is_empty() {
            log::debug!("No redirect rules found for source: {}", self.source);
            
            // Only selectively clear cache if previous rules existed
            if current_rules_map.contains_key(&self.source) {
                // Clear cache for this source as all rules are gone
                self.route_cache.clear();
            }
            
            // Update SAVED_CONFIG_ID even when no rules match this source
            // First clear any existing rules for this source
            {
                let mut rules_map = REDIRECT_RULES.write().unwrap();
                rules_map.insert(self.source.clone(), empty_rules);
            }
            
            // Update the saved config ID
            {
                let mut saved_id_guard = SAVED_CONFIG_ID.write().unwrap();
                *saved_id_guard = config_id.clone();
            }
            
            log::debug!("Saved empty rules with config ID: {}", config_id);
            return;
        }
            
        // Sort by priority - lower values have higher priority
        // Use unstable sort for better performance since we don't need stability
        source_rules.sort_unstable_by_key(|rule| rule.priority);

        // Determine if we need full cache clear or selective invalidation
        let need_full_clear = if let Some(old_rules) = current_rules_map.get(&self.source) {
            // Check if rule count changed significantly
            if (old_rules.len() as i32 - source_rules.len() as i32).abs() > 5 {
                true
            } else {
                // Check if priority ordering changed significantly
                // This is a heuristic: if many rules changed priority, we should clear the cache
                let old_paths: Vec<String> = old_rules.iter()
                    .map(|r| r.target.clone())
                    .collect();
                
                let new_paths: Vec<String> = source_rules.iter()
                    .map(|r| r.target.clone())
                    .collect();
                
                // If paths are very different, do a full clear
                old_paths.len() / 2 < old_paths.iter().filter(|p| !new_paths.contains(p)).count()
            }
        } else {
            // No previous rules, no need for a full clear
            false
        };

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
            *saved_id_guard = config_id.clone();
        }

        // Clear cache when appropriate
        if need_full_clear {
            log::debug!("Configuration changed significantly, clearing entire route cache");
            self.route_cache.clear();
        } else {
            // Selective cache invalidation - invalidate entries that could be affected by pattern changes
            log::debug!("Configuration changed incrementally, selectively clearing route cache");
            
            // We can be more clever here, but for now just invalidate cache based on hostname patterns
            // In a more advanced implementation, we could track which rules changed and only invalidate
            // those cache entries that match the changed rules
            
            // This is a simple approach - for paths looking like /api/.* or similar,
            // we strip the regex parts and use that as a prefix check
            let patterns_for_invalidation: Vec<String> = host_patterns.into_iter()
                .map(|p| p.replace("(.*)", "").replace(".*", "").replace("[^/]+", ""))
                .collect();
            
            self.route_cache.clear_matching(|key: &String| {
                // Check if any pattern is a prefix of this cache key
                for pattern in &patterns_for_invalidation {
                    if !pattern.is_empty() && key.starts_with(pattern) {
                        return true;
                    }
                }
                false
            });
        }

        log::debug!("Saved config ID: {}", config_id);
    }

    // Helper to get the rules for this instance's source
    fn get_rules(&self) -> Vec<RedirectRule> {
        // Only need read access here
        let rules_map = REDIRECT_RULES.read().unwrap();
        rules_map.get(&self.source).cloned().unwrap_or_default()
    }

    // Check if we should refresh configuration based on time interval and update the timestamp
    #[inline]
    fn should_check_config(&self) -> bool {
        let now = Instant::now();
        
        // First check if enough time has passed since last check - this is a lightweight operation
        let time_elapsed = {
            let last_check = self.last_check_time.read().unwrap();
            now.duration_since(*last_check) >= self.check_interval
        };
        
        // If enough time has passed, do a full check
        if time_elapsed {
            // Always update the timestamp first to prevent repeated checks
            {
                let mut last_check = self.last_check_time.write().unwrap();
                *last_check = now;
            }
            
            // Now check if configuration has actually changed
            let current_config_id = config::RoutingData::GatewayID.get();
            let saved_id = {
                let saved_id_guard = SAVED_CONFIG_ID.read().unwrap();
                saved_id_guard.clone()
            };
            
            if saved_id != current_config_id {
                log::debug!("Config changed: Saved ID '{}' vs Current ID '{}'", saved_id, current_config_id);
                return true;
            }
            
            // Even if IDs match, check if we need to refresh based on actual configuration
            let has_rules = {
                let rules_map = REDIRECT_RULES.read().unwrap();
                rules_map.contains_key(&self.source) && !rules_map.get(&self.source).unwrap_or(&vec![]).is_empty()
            };
            
            if !has_rules {
                log::debug!("No rules found for source {}, attempting refresh", self.source);
                return true;
            }
        }
        
        false
    }
    
    // Fast path transformation with minimum allocations
    #[inline]
    fn transform_path<'a>(&self, rule: &'a RedirectRule, _path: &'a str, captures: regex::Captures<'a>) -> Cow<'a, str> {
        // Fast path for simple substitutions
        if !rule.has_replacements {
            return Cow::Borrowed(&rule.target);
        }
        
        // Use thread-local buffer to avoid allocations
        PATH_BUFFER.with(|buffer| {
            let mut buf = buffer.borrow_mut();
            buf.clear();
            
            // Try to use cached replacement plan if available
            let replacement_plan = REPLACEMENT_CACHE.with(|cache| {
                let cache = cache.borrow();
                cache.get(&rule.target).cloned()
            });
            
            if let Some(plan) = replacement_plan {
                // We have a cached plan for this target pattern
                // Apply the plan directly (faster than searching)
                buf.push_str(&rule.target);
                
                // Apply replacements in reverse order to avoid invalidating positions
                for &(start, end, capture_idx) in plan.iter().rev() {
                    if let Some(capture) = captures.get(capture_idx) {
                        buf.replace_range(start..end, capture.as_str());
                    }
                }
            } else {
                // No cached plan - compute one
                buf.push_str(&rule.target);
                
                // Find all dollar sign replacements
                let mut dollar_positions = DOLLAR_POSITIONS.with(|positions| {
                    let mut positions = positions.borrow_mut();
                    positions.clear();
                    
                    // Find all $n patterns in the target string
                    let mut pos = 0;
                    while let Some(m) = DOLLAR_SIGN_PATTERN.find_at(&rule.target, pos) {
                        let start = m.start();
                        let end = m.end();
                        if let Some(capture_num) = rule.target[start+1..end].parse::<usize>().ok() {
                            positions.push((start, end, capture_num));
                        }
                        pos = end;
                    }
                    
                    positions.clone()
                });
                
                // Apply replacements in reverse order to avoid invalidating positions
                dollar_positions.sort_by(|a, b| b.0.cmp(&a.0));
                
                for &(start, end, capture_idx) in &dollar_positions {
                    if let Some(capture) = captures.get(capture_idx) {
                        buf.replace_range(start..end, capture.as_str());
                    }
                }
                
                // Cache the replacement plan for future use (reverse back to original order)
                dollar_positions.reverse();
                
                // Only cache if it's not too complex
                if dollar_positions.len() <= 8 {
                    REPLACEMENT_CACHE.with(|cache| {
                        // Only grow the cache to a reasonable size
                        let mut cache = cache.borrow_mut();
                        if cache.len() < 64 {
                            cache.insert(rule.target.clone(), dollar_positions);
                        }
                    });
                }
            }
            
            Cow::Owned(buf.clone())
        })
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
        // Get request information before any mutable borrows
        let path = session.req_header().uri.path();
        let query = session.req_header().uri.query();
        
        // Create a cache key combining path and query using thread-local buffer
        let cache_key = CACHE_KEY_BUFFER.with(|buffer| {
            let mut buf = buffer.borrow_mut();
            buf.clear();
            buf.push_str(path);
            if let Some(q) = query {
                buf.push('?');
                buf.push_str(q);
            }
            buf.clone()
        });
        
        // First check if config needs refreshing before looking in cache
        // This ensures we don't use stale cache entries after config changes
        if self.should_check_config() {
            self.populate(); // This clears the cache if config has changed
        }
        
        // Now check cache (fast path) - this avoids expensive regex operations
        // with confidence that cache has been properly cleared if needed
        if let Some((path_and_query, _peer, target_addr_str)) = self.route_cache.get(&cache_key) {
            // We found a cached route result - reuse the cached path and query string
            
            // Update the request URI with the cached path and query
            let mut parts = session.req_header_mut().uri.clone().into_parts();
            parts.path_and_query = Some(
                http::uri::PathAndQuery::from_maybe_shared(path_and_query.clone().into_bytes())
                    .expect("Valid URI"),
            );
            session.req_header_mut().uri = http::Uri::from_parts(parts).expect("Valid URI");
            
            // Return the cached peer using the pre-formatted address string - zero allocations
            return Ok(Box::new(HttpPeer::new(target_addr_str, false, String::new())));
        }

        // Process rules with optimized matching
        let rules = self.get_rules();

        // Try to match path against our redirect rules with optimized loop
        for rule in &rules {
            // Fast pre-check with DFA matcher if available - huge performance win
            if let Some(fast_matcher) = &rule.fast_matcher {
                if !fast_matcher.is_match(path) {
                    continue;
                }
            }
            
            // Use standard regex for captures
            if let Some(captures) = rule.pattern.captures(path) {
                if let Some(alt_target) = &rule.alt_target {
                    // Transform path with minimum allocations
                    let final_path = self.transform_path(rule, path, captures);

                    // Use thread-local buffer for path+query construction
                    let new_path_and_query = PATH_BUFFER.with(|buffer| {
                        let mut buf = buffer.borrow_mut();
                        buf.clear();
                        buf.push_str(final_path.as_ref());
                        if let Some(q) = query {
                            buf.push('?');
                            buf.push_str(q);
                        }
                        buf.clone()
                    });

                    // Rebuild URI efficiently
                    let mut parts = session.req_header_mut().uri.clone().into_parts();
                    parts.path_and_query = Some(
                        http::uri::PathAndQuery::from_maybe_shared(new_path_and_query.clone().into_bytes())
                            .expect("Valid URI"),
                    );
                    session.req_header_mut().uri = http::Uri::from_parts(parts).expect("Valid URI");

                    // Use the pre-formatted address string - zero allocation
                    let new_peer = HttpPeer::new(&rule.target_addr_str, false, String::new());
                    
                    // Store in cache
                    self.route_cache.insert(cache_key, (new_path_and_query, alt_target.clone(), rule.target_addr_str.clone()));
                    
                    return Ok(Box::new(new_peer));
                }
            }
        }

        // Default fallback if no rules match - Precomputed default for fewer allocations
        static DEFAULT_PEER: LazyLock<(String, Box<HttpPeer>)> = LazyLock::new(|| {
            let port_str = DEFAULT_PORT.p404;
            let parts: Vec<&str> = port_str.split(':').collect();
            let addr = (parts[0], parts[1].parse::<u16>().unwrap_or(80));
            let peer_string = format!("{}:{}", addr.0, addr.1);
            (peer_string.clone(), Box::new(HttpPeer::new(addr, false, String::new())))
        });
        
        log::debug!("No matching rules, connecting to default {}", DEFAULT_PEER.0);
        Ok(DEFAULT_PEER.1.clone())
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
