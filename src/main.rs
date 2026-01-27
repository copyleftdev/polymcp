use std::sync::Arc;

use polygon_mcp::{
    GetDividends, GetEma, GetLastTrade, GetMacd, GetMarketHolidays, GetMarketStatus, GetNews,
    GetRsi, GetSma, GetStockAggregates, GetStockSnapshot, GetStockSplits, GetTickerDetails,
    McpServer, PolygonClient, SearchTickers,
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let polygon = Arc::new(PolygonClient::from_env()?);

    let server = McpServer::builder("polygon-mcp", env!("CARGO_PKG_VERSION"))
        .description("MCP server providing Polygon.io financial data API as tools")
        .tool(GetStockAggregates::new(polygon.clone()))
        .tool(GetLastTrade::new(polygon.clone()))
        .tool(GetStockSnapshot::new(polygon.clone()))
        .tool(GetSma::new(polygon.clone()))
        .tool(GetEma::new(polygon.clone()))
        .tool(GetRsi::new(polygon.clone()))
        .tool(GetMacd::new(polygon.clone()))
        .tool(GetTickerDetails::new(polygon.clone()))
        .tool(SearchTickers::new(polygon.clone()))
        .tool(GetMarketStatus::new(polygon.clone()))
        .tool(GetMarketHolidays::new(polygon.clone()))
        .tool(GetDividends::new(polygon.clone()))
        .tool(GetNews::new(polygon.clone()))
        .tool(GetStockSplits::new(polygon.clone()))
        .build();

    server.run().await?;

    Ok(())
}
