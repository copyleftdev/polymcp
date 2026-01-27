use std::sync::Arc;

use polygon_mcp::{
    ConvertCurrency, GetCryptoAggregates, GetCryptoSnapshot, GetCryptoTrades, GetDividends, GetEma,
    GetForexAggregates, GetForexSnapshot, GetIndexAggregates, GetIndexOpenClose, GetIndexSnapshot,
    GetLastTrade, GetMacd, GetMarketHolidays, GetMarketStatus, GetNews, GetOptionsAggregates,
    GetOptionsContracts, GetOptionsSnapshot, GetRsi, GetSma, GetStockAggregates, GetStockSnapshot,
    GetStockSplits, GetTickerDetails, McpServer, PolygonClient, SearchTickers,
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
        .tool(GetOptionsContracts::new(polygon.clone()))
        .tool(GetOptionsAggregates::new(polygon.clone()))
        .tool(GetOptionsSnapshot::new(polygon.clone()))
        .tool(GetForexAggregates::new(polygon.clone()))
        .tool(ConvertCurrency::new(polygon.clone()))
        .tool(GetForexSnapshot::new(polygon.clone()))
        .tool(GetCryptoAggregates::new(polygon.clone()))
        .tool(GetCryptoTrades::new(polygon.clone()))
        .tool(GetCryptoSnapshot::new(polygon.clone()))
        .tool(GetIndexAggregates::new(polygon.clone()))
        .tool(GetIndexOpenClose::new(polygon.clone()))
        .tool(GetIndexSnapshot::new(polygon.clone()))
        .build();

    server.run().await?;

    Ok(())
}
