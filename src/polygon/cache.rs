//! Response caching for Polygon.io API calls.
//!
//! Provides an optional caching layer that stores API responses in memory
//! with configurable TTL (time-to-live) to reduce API calls and improve latency.

use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;

const DEFAULT_MAX_CAPACITY: u64 = 1000;
const DEFAULT_TTL: Duration = Duration::from_secs(60);

/// Configuration for the response cache.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache.
    pub max_capacity: u64,
    /// Time-to-live for cached entries.
    pub ttl: Duration,
    /// Whether caching is enabled.
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: DEFAULT_MAX_CAPACITY,
            ttl: DEFAULT_TTL,
            enabled: false,
        }
    }
}

impl CacheConfig {
    /// Creates a new cache configuration with caching enabled.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Sets the maximum capacity of the cache.
    pub fn with_max_capacity(mut self, capacity: u64) -> Self {
        self.max_capacity = capacity;
        self
    }

    /// Sets the TTL for cached entries.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }
}

/// A thread-safe cache for API responses.
#[derive(Clone)]
pub struct ResponseCache {
    cache: Arc<Cache<String, String>>,
    enabled: bool,
}

impl ResponseCache {
    /// Creates a new response cache with the given configuration.
    pub fn new(config: &CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.ttl)
            .build();

        Self {
            cache: Arc::new(cache),
            enabled: config.enabled,
        }
    }

    /// Gets a cached response for the given URL.
    pub async fn get(&self, url: &str) -> Option<String> {
        if !self.enabled {
            return None;
        }
        self.cache.get(url).await
    }

    /// Stores a response in the cache.
    pub async fn insert(&self, url: String, response: String) {
        if self.enabled {
            self.cache.insert(url, response).await;
        }
    }

    /// Returns whether caching is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the number of entries in the cache.
    pub async fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    /// Invalidates all cached entries.
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cache_disabled_by_default() {
        let config = CacheConfig::default();
        let cache = ResponseCache::new(&config);
        assert!(!cache.is_enabled());
    }

    #[tokio::test]
    async fn cache_stores_and_retrieves() {
        let config = CacheConfig::enabled();
        let cache = ResponseCache::new(&config);

        cache
            .insert(
                "https://api.test/data".to_string(),
                r#"{"result": 42}"#.to_string(),
            )
            .await;

        let result = cache.get("https://api.test/data").await;
        assert_eq!(result, Some(r#"{"result": 42}"#.to_string()));
    }

    #[tokio::test]
    async fn cache_returns_none_when_disabled() {
        let config = CacheConfig::default();
        let cache = ResponseCache::new(&config);

        cache
            .insert(
                "https://api.test/data".to_string(),
                r#"{"result": 42}"#.to_string(),
            )
            .await;

        let result = cache.get("https://api.test/data").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn cache_invalidate_all_clears_entries() {
        let config = CacheConfig::enabled();
        let cache = ResponseCache::new(&config);

        cache
            .insert(
                "https://api.test/data".to_string(),
                r#"{"result": 42}"#.to_string(),
            )
            .await;

        cache.invalidate_all();
        cache.cache.run_pending_tasks().await;

        let result = cache.get("https://api.test/data").await;
        assert_eq!(result, None);
    }
}
