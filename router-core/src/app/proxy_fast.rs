// Copyright 2025 Cloudflare, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use async_trait::async_trait;
use log::{debug, error, warn};

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::select;

use pingora::apps::ServerApp;
use pingora::connectors::TransportConnector;
use pingora::protocols::Stream;
use pingora::server::ShutdownWatch;
use pingora::upstreams::peer::BasicPeer;
use regex_automata::meta::Regex;
use std::num::NonZeroUsize;
use std::sync::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use lru::LruCache;

use crate::config::{self, GatewayPath};

// Number of cache shards to reduce lock contention
const CACHE_SHARDS: usize = 8;
// Default capacity per shard if not otherwise specified
const DEFAULT_PER_SHARD_CAPACITY: usize = 100; // ~800 total routes

#[derive(Debug)]
struct RewriteRule {
    pattern: Regex,
    replacement: String,
}

// Sharded LRU Cache Implementation
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
        // Use read lock for concurrent access
        match self.shards[shard_index].read() {
            Ok(shard) => shard.peek(key).cloned(),
            Err(e) => {
                error!("Failed to acquire read lock on cache shard {}: {}", shard_index, e);
                None // Handle poisoned lock
            }
        }
    }

    /// Inserts a value into the cache, potentially evicting the least recently used item.
    fn insert(&self, key: K, value: V) {
        let shard_index = self.get_shard_index(&key);
        // Use write lock for mutation
        match self.shards[shard_index].write() {
            Ok(mut shard) => {
                shard.put(key, value); // Discard the return value of put
            },
            Err(e) => {
                error!("Failed to acquire write lock on cache shard {}: {}", shard_index, e);
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
                    error!("Failed to acquire write lock on cache shard {} for clearing: {}", i, e);
                    // Handle poisoned lock - cannot clear this shard
                }
            }
        }
        debug!("Cleared entries from rewrite cache (potentially skipping poisoned shards)");
    }
}

pub struct ProxyApp {
    client_connector: TransportConnector,
    proxy_to: BasicPeer,
    path_rewrites: Arc<RwLock<Vec<RewriteRule>>>,
    // Cache for rewritten requests: key = original request line, value = rewritten request
    rewrite_cache: Arc<ShardedLruCache<String, String>>,
    // Last time config was checked
    last_check_time: RwLock<std::time::Instant>,
    // Recheck interval
    check_interval: std::time::Duration,
}

enum DuplexEvent {
    DownstreamRead(usize),
    UpstreamRead(usize),
}

impl ProxyApp {
    pub fn new(proxy_to: BasicPeer) -> Self {
        let path_rewrites = Self::fetch_config(proxy_to.clone());

        ProxyApp {
            client_connector: TransportConnector::new(None),
            proxy_to,
            path_rewrites: Arc::new(RwLock::new(path_rewrites)),
            rewrite_cache: Arc::new(ShardedLruCache::new(DEFAULT_PER_SHARD_CAPACITY)),
            last_check_time: RwLock::new(std::time::Instant::now()),
            check_interval: std::time::Duration::from_secs(5), // Check config every 5 seconds
        }
    }

    fn fetch_config(proxy_to: BasicPeer) -> Vec<RewriteRule> {
        let current_addr = proxy_to._address.to_string();
        let config: Option<Vec<GatewayPath>> =
            config::RoutingData::GatewayRouting.xget::<Vec<GatewayPath>>();
        let mut new_rewrites = Vec::new();
        if let Some(cfg) = config {
            for node in cfg {
                // high speed only
                if node.addr_target == current_addr {
                    // Determine if this is a plain string path, a wildcard path, or a regex pattern
                    let processed_pattern = if Self::is_regex_pattern(&node.path_listen) {
                        // Already a regex pattern (contains regex special chars other than * at the end)
                        log::debug!("Processing as regex pattern: '{}'", node.path_listen);
                        node.path_listen.clone()
                    } else if node.path_listen.ends_with("/*") {
                        // Wildcard pattern (e.g., "/api/*")
                        log::debug!("Processing as wildcard pattern: '{}'", node.path_listen);
                        // Convert "/api/*" to "^/api/.*$"
                        let base_path = &node.path_listen[..node.path_listen.len() - 1];
                        format!("^{}.*$", base_path)
                    } else {
                        // Plain string path (e.g., "/test")
                        log::debug!("Processing as exact match pattern: '{}'", node.path_listen);
                        // Convert "/test" to "^/test$"
                        format!("^{}$", node.path_listen)
                    };

                    // Compile the processed regex pattern
                    let rgx = match Regex::new(&processed_pattern) {
                        Ok(rgx) => rgx,
                        Err(e) => {
                            log::error!(
                                "Failed to compile regex pattern '{}' (from '{}'): {}",
                                processed_pattern,
                                node.path_listen,
                                e
                            );
                            continue;
                        }
                    };

                    new_rewrites.push(RewriteRule {
                        pattern: rgx,
                        replacement: node.path_target,
                    });
                }
            }
        }
        new_rewrites
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

    // Process a replacement string, handling $1, $2, etc. references
    fn process_replacement(&self, captures: &[&str], template: &str) -> String {
        let mut result = String::new();
        let mut i = 0;
        let template_chars: Vec<char> = template.chars().collect(); // Collect chars for safe indexing

        while i < template_chars.len() {
            if template_chars[i] == '$' && i + 1 < template_chars.len() {
                if let Some(digit) = template_chars[i + 1].to_digit(10) {
                    let capture_index = digit as usize;
                    // $0 is not a valid capture, $1 corresponds to captures[0]
                    if capture_index > 0 && capture_index <= captures.len() {
                        result.push_str(captures[capture_index - 1]);
                        i += 2; // Skip '$' and the digit
                        continue;
                    }
                }
            }

            // Not a valid capture reference or index, add current char
            result.push(template_chars[i]);
            i += 1;
        }

        result
    }

    // Regex-based HTTP request line parser and rewriter
    fn rewrite_http_request(&self, buffer: &mut [u8], length: usize) -> usize {
        // First convert the buffer to a string for processing
        let request_str = match std::str::from_utf8(&buffer[..length]) {
            Ok(s) => s,
            Err(_) => return length, // Not valid UTF-8, return unchanged
        };

        // Check if this looks like an HTTP request
        if !request_str.starts_with("GET ")
            && !request_str.starts_with("POST ")
            && !request_str.starts_with("PUT ")
            && !request_str.starts_with("DELETE ")
            && !request_str.starts_with("CONNECT ")
            && !request_str.starts_with("OPTIONS ")
        {
            return length;
        }

        // First check for configuration changes at regular intervals
        self.check_and_reload_config_if_needed();

        // Check if we already have this request in the cache
        if let Some(cached_request) = self.rewrite_cache.get(&request_str.to_string()) {
            debug!("Cache hit for request rewrite");
            let new_bytes = cached_request.as_bytes();
            let new_len = new_bytes.len();

            // Make sure we don't overflow the buffer
            if new_len <= buffer.len() {
                buffer[..new_len].copy_from_slice(new_bytes);
                return new_len;
            } else {
                debug!("Cached rewritten request too large for buffer");
                return length;
            }
        }

        debug!("Cache miss for request rewrite");

        // Flag to track if this is a WebSocket upgrade request
        let is_websocket = request_str.contains("Upgrade: websocket")
            || request_str.contains("Upgrade: WebSocket");

        // Find the first line of the request (the request line)
        let line_end = match request_str.find("\r\n") {
            Some(pos) => pos,
            None => return length, // Not a complete HTTP request line
        };

        let request_line = &request_str[..line_end];
        let rest_of_request = &request_str[line_end..];

        // Log the full request line for debugging
        debug!("Received request line: '{}'", request_line);

        // Parse the request line to extract Method, Path, and Protocol
        let parts: Vec<&str> = request_line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            debug!("Malformed request line: '{}'", request_line);
            return length; // Malformed, return original
        }
        let method = parts[0];
        let request_path = parts[1];
        let protocol = parts[2];
        debug!("Parsed request path: '{}'", request_path);

        // Try each rewrite rule
        let rules_guard = match self.path_rewrites.read() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire read lock on path_rewrites: {}", e);
                return length; // Return original length if lock acquisition fails
            }
        };
        for rule in rules_guard.iter() {
            // Find all matches in the request line
            let mut matches = Vec::new();
            let mut captures = Vec::new();

            log::debug!("Checking rewrite rule pattern for request_path: '{}'", request_path);

            // Use regex-automata to find matches in the path
            for mat in rule.pattern.find_iter(request_path.as_bytes()) {
                matches.push((mat.start(), mat.end()));

                // Extract capture groups
                // This is simplified since regex-automata's Match doesn't directly provide captures
                // In a real implementation, you'd need to extract captures based on the match bounds
                let matched_text = &request_path[mat.start()..mat.end()];
                captures.push(matched_text);
            }

            // If we have a match, perform the rewrite
            if let Some((start, end)) = matches.first() {
                debug!("Matched regex pattern for rewrite");

                // Get parts of the *path* before and after the match
                let before = &request_path[..*start];
                let after = &request_path[*end..];

                // Process replacement template with capture references
                let replacement = self.process_replacement(
                    &captures.iter().map(|s| *s).collect::<Vec<&str>>(),
                    &rule.replacement,
                );

                // Create the new *path* and then the new request line
                let new_request_path = format!("{}{}{}", before, replacement, after);
                let new_request_line = format!("{} {} {}", method, new_request_path, protocol);
                let new_request = format!("{}{}", new_request_line, rest_of_request);

                // Log rewrite information with special note for WebSocket upgrades
                if is_websocket {
                    debug!("Rewrote WebSocket upgrade request: {} -> {}", request_line, new_request_line);
                } else {
                    debug!("Rewrote request: {} -> {}", request_line, new_request_line);
                }

                // Store the rewritten request in the cache
                self.rewrite_cache.insert(request_str.to_string(), new_request.clone());
                debug!("Stored rewritten request in cache");

                // Convert back to bytes and copy to the buffer
                let new_bytes = new_request.as_bytes();
                let new_len = new_bytes.len();

                // Make sure we don't overflow the buffer
                if new_len <= buffer.len() {
                    // Copy the new request into the buffer
                    buffer[..new_len].copy_from_slice(new_bytes);
                    return new_len;
                } else {
                    debug!("Rewritten request too large for buffer");
                    return length; // Return original length if new request is too large
                }
            }
        }

        // No rewrite performed
        debug!("No rewrite rule matched for request path: {}", request_path);
        // should close if no match
        0
    }

    /// Checks if the configuration should be reloaded based on time interval.
    fn check_and_reload_config_if_needed(&self) {
        let now = std::time::Instant::now();
        let needs_check = {
            // Scoped read lock
            match self.last_check_time.read() {
                Ok(last_check_guard) => now.duration_since(*last_check_guard) >= self.check_interval,
                Err(e) => {
                    error!("Failed to acquire read lock on last_check_time: {}", e);
                    false // Don't check if lock is poisoned
                }
            }
            // Read lock is dropped here
        };

        if needs_check {
            // Acquire write lock only if the time interval has passed.
            match self.last_check_time.write() {
                Ok(mut last_check_guard) => {
                    // Double-check in case another thread updated it between the read and write lock acquisition.
                    if now.duration_since(*last_check_guard) >= self.check_interval {
                        // Update last check time *before* potentially long-running fetch_config
                        *last_check_guard = now;
                        // Drop the lock before calling fetch_config to avoid holding it too long
                        drop(last_check_guard);

                        // Now perform the actual check and potential reload
                        debug!("Checking rules due to interval check...");
                        let new_rewrites = Self::fetch_config(self.proxy_to.clone());

                        // Compare current rules count with new rules count
                        let current_rules_count = match self.path_rewrites.read() {
                             Ok(rules_guard) => rules_guard.len(),
                             Err(e) => {
                                 error!("Failed to acquire read lock on path_rewrites for count: {}", e);
                                 // Cannot compare if lock is poisoned, assume no change needed for safety
                                 // Or potentially return a sentinel value like usize::MAX
                                 return; // Exit the check function
                             }
                        };

                        // Only update if the rules have changed
                        if current_rules_count != new_rewrites.len() {
                            log::info!(
                                "Configuration changed. Reloading rules for proxy: {} (old: {}, new: {})",
                                self.proxy_to._address,
                                current_rules_count,
                                new_rewrites.len()
                            );

                            // Clear the rewrite cache as rules are changing
                            self.rewrite_cache.clear();

                            // Update the rules with write lock
                            match self.path_rewrites.write() {
                                Ok(mut rules_guard) => *rules_guard = new_rewrites,
                                Err(e) => {
                                    error!("Failed to acquire write lock on path_rewrites for update: {}", e);
                                    // Failed to update rules, cache might be inconsistent now.
                                    // Consider logging a more severe warning or taking other action.
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                     error!("Failed to acquire write lock on last_check_time: {}", e);
                     // Cannot update last check time if lock is poisoned
                }
            }
            // Write lock is dropped here
        }
    }

    async fn duplex(&self, mut server_session: Stream, mut client_session: Stream) {
        let mut upstream_buf = [0; 4096]; // Increased buffer size for HTTP headers
        let mut downstream_buf = [0; 4096];
        let timeout_duration = std::time::Duration::from_secs(60);

        loop {
            let downstream_read =
                tokio::time::timeout(timeout_duration, server_session.read(&mut upstream_buf));
            let upstream_read =
                tokio::time::timeout(timeout_duration, client_session.read(&mut downstream_buf));
            let event: DuplexEvent;

            select! {
                result = downstream_read => match result {
                    Ok(Ok(n)) => event = DuplexEvent::DownstreamRead(n),
                    Ok(Err(e)) => {
                        log::debug!("Error reading from downstream: {}", e);
                        return;
                    },
                    Err(_) => {
                        log::debug!("Timeout reading from downstream");
                        return;
                    }
                },
                result = upstream_read => match result {
                    Ok(Ok(n)) => event = DuplexEvent::UpstreamRead(n),
                    Ok(Err(e)) => {
                        log::debug!("Error reading from upstream: {}", e);
                        return;
                    },
                    Err(_) => {
                        log::debug!("Timeout reading from upstream");
                        return;
                    }
                },
            }
            match event {
                DuplexEvent::DownstreamRead(0) => {
                    debug!("downstream session closing");
                    return;
                }
                DuplexEvent::UpstreamRead(0) => {
                    debug!("upstream session closing");
                    return;
                }
                DuplexEvent::DownstreamRead(n) => {
                    // Try to rewrite the request if it's HTTP
                    let write_len = self.rewrite_http_request(&mut upstream_buf, n);
                    if write_len == 0 {
                        debug!("Request rewrite failed, closing connection");
                        return; // Close connection on rewrite failure
                    }
                    if let Err(e) = client_session
                        .write_all(&upstream_buf[0..write_len])
                        .await {
                            debug!("Error writing to upstream client: {}", e);
                            return; // Close connection on write error
                        }
                    if let Err(e) = client_session.flush().await {
                        debug!("Error flushing upstream client: {}", e);
                        return; // Close connection on flush error
                    }
                }
                DuplexEvent::UpstreamRead(n) => {
                    log::debug!("Incoming data from upstream: {}", n);
                     if let Err(e) = server_session
                        .write_all(&downstream_buf[0..n])
                        .await {
                            debug!("Error writing to downstream server: {}", e);
                            return; // Close connection on write error
                        }
                    if let Err(e) = server_session.flush().await {
                        debug!("Error flushing downstream server: {}", e);
                        return; // Close connection on flush error
                    }
                }
            }
        }
    }
}

#[async_trait]
impl ServerApp for ProxyApp {
    async fn process_new(
        self: &Arc<Self>,
        io: Stream,
        _shutdown: &ShutdownWatch,
    ) -> Option<Stream> {
        let client_session = self.client_connector.new_stream(&self.proxy_to).await;

        match client_session {
            Ok(client_session) => {
                self.duplex(io, client_session).await;
                None
            }
            Err(e) => {
                debug!("Failed to create client session: {}", e);
                None
            }
        }
    }
}
