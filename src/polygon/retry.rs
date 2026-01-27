//! Retry logic with exponential backoff and jitter.
//!
//! Implements robust retry handling for transient failures with configurable
//! backoff strategy to avoid thundering herd problems.

use std::time::Duration;

use rand::Rng;

const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(500);
const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(30);
const DEFAULT_MAX_RETRIES: u32 = 3;
const JITTER_FACTOR: f64 = 0.3;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Base delay for exponential backoff.
    pub base_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Maximum number of retry attempts.
    pub max_retries: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            base_delay: DEFAULT_BASE_DELAY,
            max_delay: DEFAULT_MAX_DELAY,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }
}

impl RetryConfig {
    /// Creates a new retry configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the base delay for exponential backoff.
    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Sets the maximum delay between retries.
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Sets the maximum number of retry attempts.
    pub fn with_max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// Calculates the delay for a given attempt using exponential backoff with jitter.
    ///
    /// The delay is calculated as: base_delay * 2^(attempt-1) + random_jitter
    /// The result is capped at max_delay.
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let exponential = self
            .base_delay
            .saturating_mul(2u32.saturating_pow(attempt.saturating_sub(1)));

        let base_ms = exponential.as_millis().min(self.max_delay.as_millis()) as f64;

        let jitter_range = base_ms * JITTER_FACTOR;
        let jitter = rand::thread_rng().gen_range(-jitter_range..jitter_range);

        let delay_ms = (base_ms + jitter).max(0.0) as u64;
        Duration::from_millis(delay_ms).min(self.max_delay)
    }

    /// Calculates delay respecting a server-provided retry-after hint.
    pub fn calculate_delay_with_hint(
        &self,
        attempt: u32,
        retry_after: Option<Duration>,
    ) -> Duration {
        match retry_after {
            Some(hint) => hint.max(self.calculate_delay(attempt)),
            None => self.calculate_delay(attempt),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_reasonable_values() {
        let config = RetryConfig::default();
        assert_eq!(config.base_delay, Duration::from_millis(500));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn delay_increases_exponentially() {
        let config = RetryConfig::new().with_base_delay(Duration::from_millis(100));

        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(2);
        let delay3 = config.calculate_delay(3);

        assert!(delay1.as_millis() < 200);
        assert!(delay2.as_millis() > delay1.as_millis());
        assert!(delay3.as_millis() > delay2.as_millis());
    }

    #[test]
    fn delay_capped_at_max() {
        let config = RetryConfig::new()
            .with_base_delay(Duration::from_secs(1))
            .with_max_delay(Duration::from_secs(5));

        let delay = config.calculate_delay(10);
        assert!(delay <= Duration::from_secs(5));
    }

    #[test]
    fn retry_after_hint_is_respected() {
        let config = RetryConfig::new().with_base_delay(Duration::from_millis(100));

        let hint = Duration::from_secs(10);
        let delay = config.calculate_delay_with_hint(1, Some(hint));

        assert!(delay >= hint);
    }

    #[test]
    fn jitter_adds_variation() {
        let config = RetryConfig::new().with_base_delay(Duration::from_millis(1000));

        let delays: Vec<_> = (0..10).map(|_| config.calculate_delay(1)).collect();

        let all_same = delays.windows(2).all(|w| w[0] == w[1]);
        assert!(!all_same, "jitter should add variation to delays");
    }
}
