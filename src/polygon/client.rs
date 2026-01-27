use std::env;
use std::time::Duration;

use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use tracing::{debug, warn};
use url::Url;

use super::error::PolygonError;
use super::pagination::Paginator;
use super::types::ErrorResponse;

const BASE_URL: &str = "https://api.polygon.io";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_RETRY_DELAY: Duration = Duration::from_secs(1);

pub struct PolygonClientBuilder {
    api_key: Option<String>,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
}

impl Default for PolygonClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PolygonClientBuilder {
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: BASE_URL.to_string(),
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

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
            max_retries: self.max_retries,
        })
    }
}

pub struct PolygonClient {
    client: Client,
    api_key: String,
    base_url: String,
    max_retries: u32,
}

impl PolygonClient {
    pub fn builder() -> PolygonClientBuilder {
        PolygonClientBuilder::new()
    }

    pub fn from_env() -> Result<Self, PolygonError> {
        Self::builder().build()
    }

    pub fn with_key(api_key: impl Into<String>) -> Result<Self, PolygonError> {
        Self::builder().api_key(api_key).build()
    }

    pub async fn get<T>(&self, path: &str) -> Result<T, PolygonError>
    where
        T: DeserializeOwned,
    {
        let url = self.build_url(path)?;
        self.get_raw(&url).await
    }

    pub async fn get_raw<T>(&self, url: &str) -> Result<T, PolygonError>
    where
        T: DeserializeOwned,
    {
        let url = self.append_api_key(url)?;
        self.execute_with_retry(&url).await
    }

    pub fn paginate<T>(&self, path: &str) -> Result<Paginator<'_, T>, PolygonError>
    where
        T: DeserializeOwned + Send,
    {
        let url = self.build_url(path)?;
        Ok(Paginator::new(self, url))
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
        let mut attempts = 0;
        let mut last_error: Option<PolygonError> = None;

        while attempts < self.max_retries {
            attempts += 1;

            match self.execute_request(url).await {
                Ok(response) => return self.handle_response(response).await,
                Err(e) if e.is_retryable() && attempts < self.max_retries => {
                    let delay = self.calculate_retry_delay(&e, attempts);
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

    async fn handle_response<T>(&self, response: Response) -> Result<T, PolygonError>
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
                serde_json::from_str(&body).map_err(PolygonError::from)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(Duration::from_secs)
                    .unwrap_or(DEFAULT_RETRY_DELAY);

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

    fn calculate_retry_delay(&self, error: &PolygonError, attempt: u32) -> Duration {
        match error {
            PolygonError::RateLimit { retry_after } => *retry_after,
            _ => DEFAULT_RETRY_DELAY * attempt,
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
