use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::{MarketHoliday, MarketStatus};

pub struct GetMarketStatus {
    client: Arc<PolygonClient>,
}

impl GetMarketStatus {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, _args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let response: MarketStatus = self.client.get("/v1/marketstatus/now").await?;

        let result = json!({
            "market": response.market,
            "serverTime": response.server_time,
            "earlyHours": response.early_hours,
            "afterHours": response.after_hours,
            "exchanges": response.exchanges.as_ref().map(|e| json!({
                "nasdaq": e.nasdaq,
                "nyse": e.nyse,
                "otc": e.otc
            })),
            "currencies": response.currencies.as_ref().map(|c| json!({
                "crypto": c.crypto,
                "fx": c.fx
            }))
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetMarketStatus {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_market_status")
            .with_description(
                "Get the current trading status of exchanges and financial markets. \
                Returns whether markets are open, closed, in early/after hours trading.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: None,
                required: None,
            })
    }

    fn call(&self, arguments: HashMap<String, Value>) -> ToolCallFuture {
        let this = Self {
            client: self.client.clone(),
        };
        Box::pin(async move {
            this.execute(arguments)
                .await
                .map_err(|e| crate::mcp::McpError::InternalError {
                    message: e.to_string(),
                })
        })
    }
}

pub struct GetMarketHolidays {
    client: Arc<PolygonClient>,
}

impl GetMarketHolidays {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, _args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let response: Vec<MarketHoliday> = self.client.get("/v1/marketstatus/upcoming").await?;

        let holidays: Vec<Value> = response
            .iter()
            .map(|h| {
                json!({
                    "date": h.date,
                    "exchange": h.exchange,
                    "name": h.name,
                    "status": h.status,
                    "open": h.open,
                    "close": h.close
                })
            })
            .collect();

        let result = json!({
            "holidays": holidays
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetMarketHolidays {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_market_holidays")
            .with_description(
                "Get upcoming market holidays with date, exchange, name, and status. \
                Useful for planning around market closures.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: None,
                required: None,
            })
    }

    fn call(&self, arguments: HashMap<String, Value>) -> ToolCallFuture {
        let this = Self {
            client: self.client.clone(),
        };
        Box::pin(async move {
            this.execute(arguments)
                .await
                .map_err(|e| crate::mcp::McpError::InternalError {
                    message: e.to_string(),
                })
        })
    }
}
