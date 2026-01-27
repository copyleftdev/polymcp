//! Rate limiting to avoid 429 errors from Polygon.io.
//!
//! Implements a token bucket algorithm for proactive request throttling.
//! This helps avoid hitting rate limits and maintains consistent performance.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

const DEFAULT_REQUESTS_PER_SECOND: u32 = 5;
const DEFAULT_BURST_SIZE: u32 = 10;

/// Configuration for rate limiting.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per second.
    pub requests_per_second: u32,
    /// Maximum burst size (tokens in bucket).
    pub burst_size: u32,
    /// Whether rate limiting is enabled.
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: DEFAULT_REQUESTS_PER_SECOND,
            burst_size: DEFAULT_BURST_SIZE,
            enabled: true,
        }
    }
}

impl RateLimitConfig {
    /// Creates a new rate limit configuration.
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            requests_per_second,
            burst_size: requests_per_second * 2,
            enabled: true,
        }
    }

    /// Sets the burst size.
    pub fn with_burst_size(mut self, size: u32) -> Self {
        self.burst_size = size;
        self
    }

    /// Disables rate limiting.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }
}

struct TokenBucketState {
    tokens: f64,
    last_update: Instant,
}

/// A token bucket rate limiter.
#[derive(Clone)]
pub struct RateLimiter {
    state: Arc<Mutex<TokenBucketState>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Creates a new rate limiter with the given configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(TokenBucketState {
                tokens: config.burst_size as f64,
                last_update: Instant::now(),
            })),
            config,
        }
    }

    /// Acquires a token, waiting if necessary.
    ///
    /// Returns immediately if a token is available, otherwise waits
    /// until a token becomes available.
    pub async fn acquire(&self) {
        if !self.config.enabled {
            return;
        }

        loop {
            let wait_time = {
                let mut state = self.state.lock().await;
                self.refill_tokens(&mut state);

                if state.tokens >= 1.0 {
                    state.tokens -= 1.0;
                    return;
                }

                let tokens_needed = 1.0 - state.tokens;
                let seconds_to_wait = tokens_needed / self.config.requests_per_second as f64;
                Duration::from_secs_f64(seconds_to_wait)
            };

            tokio::time::sleep(wait_time).await;
        }
    }

    /// Attempts to acquire a token without waiting.
    ///
    /// Returns `true` if a token was acquired, `false` otherwise.
    pub async fn try_acquire(&self) -> bool {
        if !self.config.enabled {
            return true;
        }

        let mut state = self.state.lock().await;
        self.refill_tokens(&mut state);

        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn refill_tokens(&self, state: &mut TokenBucketState) {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_update);
        let new_tokens = elapsed.as_secs_f64() * self.config.requests_per_second as f64;

        state.tokens = (state.tokens + new_tokens).min(self.config.burst_size as f64);
        state.last_update = now;
    }

    /// Returns whether rate limiting is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Returns the current number of available tokens.
    pub async fn available_tokens(&self) -> f64 {
        let mut state = self.state.lock().await;
        self.refill_tokens(&mut state);
        state.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allows_burst_requests() {
        let config = RateLimitConfig::new(5).with_burst_size(3);
        let limiter = RateLimiter::new(config);

        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        assert!(!limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn tokens_refill_over_time() {
        let config = RateLimitConfig::new(100).with_burst_size(1);
        let limiter = RateLimiter::new(config);

        assert!(limiter.try_acquire().await);
        assert!(!limiter.try_acquire().await);

        tokio::time::sleep(Duration::from_millis(20)).await;

        assert!(limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn disabled_limiter_always_allows() {
        let config = RateLimitConfig::disabled();
        let limiter = RateLimiter::new(config);

        for _ in 0..100 {
            assert!(limiter.try_acquire().await);
        }
    }

    #[tokio::test]
    async fn acquire_waits_for_token() {
        let config = RateLimitConfig::new(100).with_burst_size(1);
        let limiter = RateLimiter::new(config);

        limiter.try_acquire().await;

        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(5));
    }
}
