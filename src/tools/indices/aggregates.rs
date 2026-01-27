use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::IndexAggregatesResponse;

pub struct GetIndexAggregates {
    client: Arc<PolygonClient>,
}

impl GetIndexAggregates {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let ticker = args
            .get("ticker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("ticker is required".to_string()))?;

        let from = args
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("from date is required".to_string()))?;

        let to = args
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("to date is required".to_string()))?;

        let timespan = args
            .get("timespan")
            .and_then(|v| v.as_str())
            .unwrap_or("day");

        let multiplier = args.get("multiplier").and_then(|v| v.as_u64()).unwrap_or(1);

        let adjusted = args
            .get("adjusted")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let sort = args.get("sort").and_then(|v| v.as_str()).unwrap_or("asc");

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5000);

        let url = format!(
            "/v2/aggs/ticker/{}/range/{}/{}/{}/{}?adjusted={}&sort={}&limit={}",
            ticker.to_uppercase(),
            multiplier,
            timespan,
            from,
            to,
            adjusted,
            sort,
            limit
        );

        let response: IndexAggregatesResponse = self.client.get(&url).await?;

        let bars: Vec<Value> = response
            .results
            .iter()
            .map(|bar| {
                json!({
                    "timestamp": bar.timestamp,
                    "open": bar.open,
                    "high": bar.high,
                    "low": bar.low,
                    "close": bar.close,
                    "volume": bar.volume,
                    "vwap": bar.vwap,
                    "transactions": bar.transactions
                })
            })
            .collect();

        let result = json!({
            "ticker": response.ticker,
            "queryCount": response.query_count,
            "resultsCount": response.results_count,
            "adjusted": response.adjusted,
            "bars": bars
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetIndexAggregates {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_index_aggregates")
            .with_description(
                "Get OHLC aggregate bars for a market index. \
                Use I: prefix for index tickers (e.g., I:SPX for S&P 500, I:NDX for NASDAQ-100, I:DJI for Dow Jones).",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Index ticker with I: prefix (e.g., I:SPX, I:NDX, I:DJI)"
                        }),
                    ),
                    (
                        "from".to_string(),
                        json!({
                            "type": "string",
                            "description": "Start date (YYYY-MM-DD)"
                        }),
                    ),
                    (
                        "to".to_string(),
                        json!({
                            "type": "string",
                            "description": "End date (YYYY-MM-DD)"
                        }),
                    ),
                    (
                        "timespan".to_string(),
                        json!({
                            "type": "string",
                            "description": "Time window: minute, hour, day, week, month",
                            "enum": ["minute", "hour", "day", "week", "month"],
                            "default": "day"
                        }),
                    ),
                    (
                        "multiplier".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Timespan multiplier",
                            "default": 1
                        }),
                    ),
                    (
                        "adjusted".to_string(),
                        json!({
                            "type": "boolean",
                            "description": "Adjust for splits",
                            "default": true
                        }),
                    ),
                    (
                        "sort".to_string(),
                        json!({
                            "type": "string",
                            "description": "Sort order: asc or desc",
                            "enum": ["asc", "desc"],
                            "default": "asc"
                        }),
                    ),
                    (
                        "limit".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Max results",
                            "default": 5000
                        }),
                    ),
                ])),
                required: Some(vec![
                    "ticker".to_string(),
                    "from".to_string(),
                    "to".to_string(),
                ]),
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
