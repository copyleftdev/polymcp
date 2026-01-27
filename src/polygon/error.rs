use std::time::Duration;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PolygonError {
    #[error("rate limited, retry after {retry_after:?}")]
    RateLimit { retry_after: Duration },

    #[error("unauthorized: invalid or missing API key")]
    Unauthorized,

    #[error("API error {status}: {message}")]
    ApiError {
        status: u16,
        message: String,
        request_id: Option<String>,
    },

    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("missing API key: set POLYGON_API_KEY environment variable")]
    MissingApiKey,

    #[error("max retries exceeded after {attempts} attempts")]
    MaxRetriesExceeded { attempts: u32 },

    #[error("invalid parameter: {0}")]
    InvalidParams(String),
}

impl PolygonError {
    pub fn api_error(status: u16, message: impl Into<String>, request_id: Option<String>) -> Self {
        Self::ApiError {
            status,
            message: message.into(),
            request_id,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            PolygonError::RateLimit { .. }
                | PolygonError::ApiError {
                    status: 500..=599,
                    ..
                }
        )
    }
}
