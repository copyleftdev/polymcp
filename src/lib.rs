pub mod mcp;
pub mod polygon;
pub mod tools;

pub use mcp::{
    CallToolResult, ContentBlock, DynTool, McpError, McpServer, McpServerBuilder, PROTOCOL_VERSION,
    TextContent, Tool, ToolAnnotations, ToolCallFuture, ToolDefinition, ToolInputSchema,
};
pub use polygon::{PagedResponse, Paginator, PolygonClient, PolygonClientBuilder, PolygonError};
pub use tools::{
    GetEma, GetLastTrade, GetMacd, GetRsi, GetSma, GetStockAggregates, GetStockSnapshot,
};
