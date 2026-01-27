use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::{AggregatesResponse, Timespan};

pub struct GetStockAggregates {
    client: Arc<PolygonClient>,
}

impl GetStockAggregates {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_url(
        &self,
        ticker: &str,
        multiplier: u32,
        timespan: Timespan,
        from: &str,
        to: &str,
        adjusted: bool,
        sort: &str,
        limit: u32,
    ) -> String {
        format!(
            "/v2/aggs/ticker/{}/range/{}/{}/{}/{}?adjusted={}&sort={}&limit={}",
            ticker.to_uppercase(),
            multiplier,
            timespan.as_str(),
            from,
            to,
            adjusted,
            sort,
            limit
        )
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

        let timespan_str = args
            .get("timespan")
            .and_then(|v| v.as_str())
            .unwrap_or("day");

        let timespan = match timespan_str {
            "second" => Timespan::Second,
            "minute" => Timespan::Minute,
            "hour" => Timespan::Hour,
            "day" => Timespan::Day,
            "week" => Timespan::Week,
            "month" => Timespan::Month,
            "quarter" => Timespan::Quarter,
            "year" => Timespan::Year,
            _ => Timespan::Day,
        };

        let multiplier = args.get("multiplier").and_then(|v| v.as_u64()).unwrap_or(1) as u32;

        let adjusted = args
            .get("adjusted")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let sort = args.get("sort").and_then(|v| v.as_str()).unwrap_or("asc");

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5000) as u32;

        let url = self.build_url(
            ticker, multiplier, timespan, from, to, adjusted, sort, limit,
        );

        let response: AggregatesResponse = self.client.get(&url).await?;

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

impl Tool for GetStockAggregates {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_stock_aggregates")
            .with_description(
                "Get aggregate bars (OHLCV) for a stock over a date range. \
                Returns open, high, low, close, volume, and VWAP for each time period.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Stock ticker symbol (e.g., AAPL, MSFT, TSLA)"
                        }),
                    ),
                    (
                        "from".to_string(),
                        json!({
                            "type": "string",
                            "description": "Start date in YYYY-MM-DD format"
                        }),
                    ),
                    (
                        "to".to_string(),
                        json!({
                            "type": "string",
                            "description": "End date in YYYY-MM-DD format"
                        }),
                    ),
                    (
                        "timespan".to_string(),
                        json!({
                            "type": "string",
                            "description": "Size of time window: second, minute, hour, day, week, month, quarter, year",
                            "enum": ["second", "minute", "hour", "day", "week", "month", "quarter", "year"],
                            "default": "day"
                        }),
                    ),
                    (
                        "multiplier".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Timespan multiplier (e.g., 5 with minute = 5-minute bars)",
                            "default": 1
                        }),
                    ),
                    (
                        "adjusted".to_string(),
                        json!({
                            "type": "boolean",
                            "description": "Whether results are adjusted for splits",
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
                            "description": "Max number of results (max 50000)",
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
