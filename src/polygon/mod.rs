//! Polygon.io API client with rate limiting, caching, and retry support.
//!
//! This module provides a robust HTTP client for the Polygon.io financial data API,
//! with built-in support for:
//!
//! - **Rate limiting**: Token bucket algorithm to avoid 429 errors
//! - **Caching**: Optional in-memory response caching with configurable TTL
//! - **Retries**: Exponential backoff with jitter for transient failures
//! - **Pagination**: Cursor-based pagination for large result sets

pub mod cache;
pub mod client;
pub mod error;
pub mod pagination;
pub mod rate_limit;
pub mod retry;
pub mod types;

pub use cache::{CacheConfig, ResponseCache};
pub use client::{PolygonClient, PolygonClientBuilder};
pub use error::PolygonError;
pub use pagination::{PagedResponse, Paginator};
pub use rate_limit::{RateLimitConfig, RateLimiter};
pub use retry::RetryConfig;
pub use types::{ApiResponse, ErrorResponse};
