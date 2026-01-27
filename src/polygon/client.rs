//! Polygon.io HTTP client with authentication, rate limiting, and caching.
//!
//! The client handles API key authentication, automatic retries with exponential
//! backoff, proactive rate limiting, and optional response caching.
//!
//! # Example
//!
//! ```no_run
//! use polygon_mcp::PolygonClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = PolygonClient::from_env()?;
//!
//! // Make API requests
//! let response: serde_json::Value = client.get("/v2/aggs/ticker/AAPL/prev").await?;
//! # Ok(())
//! # }
//! ```

use std::env;
use std::time::Duration;

use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use tracing::{debug, trace, warn};
use url::Url;

use super::cache::{CacheConfig, ResponseCache};
use super::error::PolygonError;
use super::pagination::Paginator;
use super::rate_limit::{RateLimitConfig, RateLimiter};
use super::retry::RetryConfig;
use super::types::ErrorResponse;

const BASE_URL: &str = "https://api.polygon.io";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Builder for creating a [`PolygonClient`] with custom configuration.
///
/// # Example
///
/// ```no_run
/// use std::time::Duration;
/// use polygon_mcp::{PolygonClient, CacheConfig, RateLimitConfig};
///
/// let client = PolygonClient::builder()
///     .api_key("your-api-key")
///     .timeout(Duration::from_secs(60))
///     .cache(CacheConfig::enabled().with_ttl(Duration::from_secs(300)))
///     .rate_limit(RateLimitConfig::new(10))
///     .build()
///     .unwrap();
/// ```
pub struct PolygonClientBuilder {
    api_key: Option<String>,
    base_url: String,
    timeout: Duration,
    retry_config: RetryConfig,
    cache_config: CacheConfig,
    rate_limit_config: RateLimitConfig,
}

impl Default for PolygonClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PolygonClientBuilder {
    /// Creates a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: BASE_URL.to_string(),
            timeout: DEFAULT_TIMEOUT,
            retry_config: RetryConfig::default(),
            cache_config: CacheConfig::default(),
            rate_limit_config: RateLimitConfig::default(),
        }
    }

    /// Sets the API key for authentication.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Sets the base URL for the API.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Sets the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the maximum number of retry attempts.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.retry_config = self.retry_config.with_max_retries(max_retries);
        self
    }

    /// Sets the retry configuration.
    pub fn retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Sets the cache configuration.
    pub fn cache(mut self, config: CacheConfig) -> Self {
        self.cache_config = config;
        self
    }

    /// Sets the rate limit configuration.
    pub fn rate_limit(mut self, config: RateLimitConfig) -> Self {
        self.rate_limit_config = config;
        self
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Returns an error if no API key is provided and `POLYGON_API_KEY`
    /// environment variable is not set.
    pub fn build(self) -> Result<PolygonClient, PolygonError> {
        let api_key = self
            .api_key
            .or_else(|| env::var("POLYGON_API_KEY").ok())
            .ok_or(PolygonError::MissingApiKey)?;

        let client = Client::builder()
            .timeout(self.timeout)
            .pool_max_idle_per_host(10)
            .build()?;

        Ok(PolygonClient {
            client,
            api_key,
            base_url: self.base_url,
            retry_config: self.retry_config,
            cache: ResponseCache::new(&self.cache_config),
            rate_limiter: RateLimiter::new(self.rate_limit_config),
        })
    }
}

/// HTTP client for the Polygon.io API.
///
/// Provides authenticated access to the Polygon.io financial data API with
/// built-in rate limiting, caching, and automatic retries.
///
/// # Example
///
/// ```no_run
/// use polygon_mcp::PolygonClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create client from POLYGON_API_KEY environment variable
/// let client = PolygonClient::from_env()?;
///
/// // Or with explicit API key
/// let client = PolygonClient::with_key("your-api-key")?;
/// # Ok(())
/// # }
/// ```
pub struct PolygonClient {
    client: Client,
    api_key: String,
    base_url: String,
    retry_config: RetryConfig,
    cache: ResponseCache,
    rate_limiter: RateLimiter,
}

impl PolygonClient {
    /// Creates a new builder for configuring the client.
    pub fn builder() -> PolygonClientBuilder {
        PolygonClientBuilder::new()
    }

    /// Creates a client using the `POLYGON_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable is not set.
    pub fn from_env() -> Result<Self, PolygonError> {
        Self::builder().build()
    }

    /// Creates a client with the given API key.
    pub fn with_key(api_key: impl Into<String>) -> Result<Self, PolygonError> {
        Self::builder().api_key(api_key).build()
    }

    /// Makes a GET request to the given API path.
    ///
    /// The path should start with `/` and will be appended to the base URL.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use polygon_mcp::PolygonClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = PolygonClient::from_env()?;
    /// let response: serde_json::Value = client.get("/v2/aggs/ticker/AAPL/prev").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get<T>(&self, path: &str) -> Result<T, PolygonError>
    where
        T: DeserializeOwned,
    {
        let url = self.build_url(path)?;
        self.get_raw(&url).await
    }

    /// Makes a GET request to the given full URL.
    ///
    /// Use this for pagination when following `next_url` links.
    pub async fn get_raw<T>(&self, url: &str) -> Result<T, PolygonError>
    where
        T: DeserializeOwned,
    {
        let url = self.append_api_key(url)?;
        self.execute_with_retry(&url).await
    }

    /// Creates a paginator for iterating over paged results.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use polygon_mcp::PolygonClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = PolygonClient::from_env()?;
    /// let mut paginator = client.paginate::<serde_json::Value>("/v3/reference/tickers")?;
    ///
    /// while let Some(page) = paginator.next_page().await? {
    ///     // Process page
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn paginate<T>(&self, path: &str) -> Result<Paginator<'_, T>, PolygonError>
    where
        T: DeserializeOwned + Send,
    {
        let url = self.build_url(path)?;
        Ok(Paginator::new(self, url))
    }

    /// Returns the response cache.
    pub fn cache(&self) -> &ResponseCache {
        &self.cache
    }

    /// Returns the rate limiter.
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }

    fn build_url(&self, path: &str) -> Result<String, PolygonError> {
        let base = Url::parse(&self.base_url)?;
        let full = base.join(path)?;
        Ok(full.to_string())
    }

    fn append_api_key(&self, url: &str) -> Result<String, PolygonError> {
        let mut parsed = Url::parse(url)?;
        parsed
            .query_pairs_mut()
            .append_pair("apiKey", &self.api_key);
        Ok(parsed.to_string())
    }

    async fn execute_with_retry<T>(&self, url: &str) -> Result<T, PolygonError>
    where
        T: DeserializeOwned,
    {
        let cache_key = url.to_string();

        if let Some(cached) = self.cache.get(&cache_key).await {
            trace!(url = %url, "cache hit");
            return serde_json::from_str(&cached).map_err(PolygonError::from);
        }

        let mut attempts = 0;
        let mut last_error: Option<PolygonError> = None;
        let mut retry_after_hint: Option<Duration> = None;

        while attempts < self.retry_config.max_retries {
            attempts += 1;

            self.rate_limiter.acquire().await;

            match self.execute_request(url).await {
                Ok(response) => {
                    let result = self.handle_response::<T>(response, &cache_key).await;
                    return result;
                }
                Err(e) if e.is_retryable() && attempts < self.retry_config.max_retries => {
                    if let PolygonError::RateLimit { retry_after } = &e {
                        retry_after_hint = Some(*retry_after);
                    }

                    let delay = self
                        .retry_config
                        .calculate_delay_with_hint(attempts, retry_after_hint);

                    warn!(
                        error = %e,
                        attempt = attempts,
                        delay_ms = delay.as_millis(),
                        "request failed, retrying"
                    );

                    tokio::time::sleep(delay).await;
                    last_error = Some(e);
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or(PolygonError::MaxRetriesExceeded { attempts }))
    }

    async fn execute_request(&self, url: &str) -> Result<Response, PolygonError> {
        debug!(url = %url, "executing request");
        Ok(self.client.get(url).send().await?)
    }

    async fn handle_response<T>(
        &self,
        response: Response,
        cache_key: &str,
    ) -> Result<T, PolygonError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        debug!(status = %status, request_id = ?request_id, "received response");

        match status {
            StatusCode::OK => {
                let body = response.text().await?;

                self.cache.insert(cache_key.to_string(), body.clone()).await;

                serde_json::from_str(&body).map_err(PolygonError::from)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(Duration::from_secs)
                    .unwrap_or(Duration::from_secs(1));

                Err(PolygonError::RateLimit { retry_after })
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(PolygonError::Unauthorized),
            _ => {
                let error_body: ErrorResponse =
                    response.json().await.unwrap_or_else(|_| ErrorResponse {
                        status: Some(status.to_string()),
                        request_id: request_id.clone(),
                        error: Some(format!("HTTP {}", status.as_u16())),
                        message: None,
                    });

                Err(PolygonError::api_error(
                    status.as_u16(),
                    error_body.message(),
                    request_id,
                ))
            }
        }
    }

    #[cfg(test)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    #[cfg(test)]
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_client_with_explicit_key() {
        let client = PolygonClient::with_key("test-key").unwrap();
        assert_eq!(client.api_key(), "test-key");
    }

    #[test]
    fn builds_client_from_env() {
        unsafe { env::set_var("POLYGON_API_KEY", "env-test-key") };
        let client = PolygonClient::from_env().unwrap();
        assert_eq!(client.api_key(), "env-test-key");
        unsafe { env::remove_var("POLYGON_API_KEY") };
    }

    #[test]
    fn fails_without_api_key() {
        unsafe { env::remove_var("POLYGON_API_KEY") };
        let result = PolygonClient::builder().build();
        assert!(matches!(result, Err(PolygonError::MissingApiKey)));
    }

    #[test]
    fn builder_sets_custom_base_url() {
        let client = PolygonClient::builder()
            .api_key("test")
            .base_url("https://custom.api.com")
            .build()
            .unwrap();
        assert_eq!(client.base_url(), "https://custom.api.com");
    }

    #[test]
    fn appends_api_key_to_url() {
        let client = PolygonClient::with_key("secret").unwrap();
        let url = client
            .append_api_key("https://api.polygon.io/v2/tickers")
            .unwrap();
        assert!(url.contains("apiKey=secret"));
    }

    #[test]
    fn appends_api_key_preserving_existing_params() {
        let client = PolygonClient::with_key("secret").unwrap();
        let url = client
            .append_api_key("https://api.polygon.io/v2/tickers?limit=10")
            .unwrap();
        assert!(url.contains("limit=10"));
        assert!(url.contains("apiKey=secret"));
    }

    #[test]
    fn rate_limit_error_is_retryable() {
        let err = PolygonError::RateLimit {
            retry_after: Duration::from_secs(1),
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn server_error_is_retryable() {
        let err = PolygonError::api_error(503, "Service Unavailable", None);
        assert!(err.is_retryable());
    }

    #[test]
    fn client_error_is_not_retryable() {
        let err = PolygonError::api_error(400, "Bad Request", None);
        assert!(!err.is_retryable());
    }
}
