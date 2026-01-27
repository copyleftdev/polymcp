//! MCP server providing Polygon.io financial data API as tools for AI assistants.
//!
//! This crate implements an MCP (Model Context Protocol) server that exposes
//! Polygon.io's financial data API as tools that can be called by AI assistants.
//!
//! # Features
//!
//! - **26 financial data tools** covering stocks, options, forex, crypto, and indices
//! - **Rate limiting** with token bucket algorithm to avoid API throttling
//! - **Response caching** with configurable TTL for improved performance
//! - **Automatic retries** with exponential backoff and jitter
//! - **MCP protocol support** over stdio transport
//!
//! # Quick Start
//!
//! ```no_run
//! use std::sync::Arc;
//! use polygon_mcp::{McpServer, PolygonClient, GetStockAggregates};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let polygon = Arc::new(PolygonClient::from_env()?);
//!
//! let server = McpServer::builder("my-server", "1.0.0")
//!     .tool(GetStockAggregates::new(polygon.clone()))
//!     .build();
//!
//! server.run().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! The client can be configured with caching and rate limiting:
//!
//! ```no_run
//! use std::time::Duration;
//! use polygon_mcp::{PolygonClient, CacheConfig, RateLimitConfig};
//!
//! let client = PolygonClient::builder()
//!     .api_key("your-api-key")
//!     .cache(CacheConfig::enabled().with_ttl(Duration::from_secs(60)))
//!     .rate_limit(RateLimitConfig::new(10))
//!     .build()
//!     .unwrap();
//! ```

pub mod mcp;
pub mod polygon;
pub mod tools;

pub use mcp::{
    CallToolResult, ContentBlock, DynTool, McpError, McpServer, McpServerBuilder, PROTOCOL_VERSION,
    TextContent, Tool, ToolAnnotations, ToolCallFuture, ToolDefinition, ToolInputSchema,
};
pub use polygon::{
    CacheConfig, PagedResponse, Paginator, PolygonClient, PolygonClientBuilder, PolygonError,
    RateLimitConfig, ResponseCache, RetryConfig,
};
pub use tools::{
    ConvertCurrency, GetCryptoAggregates, GetCryptoSnapshot, GetCryptoTrades, GetDividends, GetEma,
    GetForexAggregates, GetForexSnapshot, GetIndexAggregates, GetIndexOpenClose, GetIndexSnapshot,
    GetLastTrade, GetMacd, GetMarketHolidays, GetMarketStatus, GetNews, GetOptionsAggregates,
    GetOptionsContracts, GetOptionsSnapshot, GetRsi, GetSma, GetStockAggregates, GetStockSnapshot,
    GetStockSplits, GetTickerDetails, SearchTickers,
};
