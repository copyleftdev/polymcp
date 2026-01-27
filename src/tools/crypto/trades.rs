use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::CryptoTradesResponse;

pub struct GetCryptoTrades {
    client: Arc<PolygonClient>,
}

impl GetCryptoTrades {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let ticker = args
            .get("ticker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("ticker is required".to_string()))?;

        let mut params = Vec::new();

        if let Some(timestamp) = args.get("timestamp").and_then(|v| v.as_str()) {
            params.push(format!("timestamp={}", timestamp));
        }

        if let Some(timestamp_gte) = args.get("timestamp.gte").and_then(|v| v.as_str()) {
            params.push(format!("timestamp.gte={}", timestamp_gte));
        }

        if let Some(timestamp_lte) = args.get("timestamp.lte").and_then(|v| v.as_str()) {
            params.push(format!("timestamp.lte={}", timestamp_lte));
        }

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100);
        params.push(format!("limit={}", limit));

        if let Some(order) = args.get("order").and_then(|v| v.as_str()) {
            params.push(format!("order={}", order));
        }

        if let Some(sort) = args.get("sort").and_then(|v| v.as_str()) {
            params.push(format!("sort={}", sort));
        }

        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

        let url = format!("/v3/trades/{}{}", ticker.to_uppercase(), query);

        let response: CryptoTradesResponse = self.client.get(&url).await?;

        let trades: Vec<Value> = response
            .results
            .iter()
            .map(|t| {
                json!({
                    "price": t.price,
                    "size": t.size,
                    "exchange": t.exchange,
                    "timestamp": t.sip_timestamp.or(t.participant_timestamp),
                    "conditions": t.conditions
                })
            })
            .collect();

        let result = json!({
            "ticker": ticker.to_uppercase(),
            "count": trades.len(),
            "trades": trades
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetCryptoTrades {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_crypto_trades")
            .with_description(
                "Get recent trades for a cryptocurrency pair. \
                Returns trade price, size, exchange, and timestamp.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Crypto ticker with X: prefix (e.g., X:BTCUSD, X:ETHUSD)"
                        }),
                    ),
                    (
                        "timestamp".to_string(),
                        json!({
                            "type": "string",
                            "description": "Query by exact timestamp (nanoseconds)"
                        }),
                    ),
                    (
                        "timestamp.gte".to_string(),
                        json!({
                            "type": "string",
                            "description": "Trades at or after this timestamp"
                        }),
                    ),
                    (
                        "timestamp.lte".to_string(),
                        json!({
                            "type": "string",
                            "description": "Trades at or before this timestamp"
                        }),
                    ),
                    (
                        "limit".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Max results (default 100)",
                            "default": 100
                        }),
                    ),
                    (
                        "order".to_string(),
                        json!({
                            "type": "string",
                            "description": "Sort order: asc or desc",
                            "enum": ["asc", "desc"]
                        }),
                    ),
                    (
                        "sort".to_string(),
                        json!({
                            "type": "string",
                            "description": "Field to sort by: timestamp",
                            "enum": ["timestamp"]
                        }),
                    ),
                ])),
                required: Some(vec!["ticker".to_string()]),
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
