use polygon_mcp::McpServer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let server = McpServer::builder("polygon-mcp", env!("CARGO_PKG_VERSION"))
        .description("MCP server providing Polygon.io financial data API as tools")
        .build();

    server.run().await?;

    Ok(())
}
