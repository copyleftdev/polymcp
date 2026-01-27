use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::IndexSnapshotResponse;

pub struct GetIndexSnapshot {
    client: Arc<PolygonClient>,
}

impl GetIndexSnapshot {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let tickers = args.get("tickers").and_then(|v| {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_uppercase()))
                        .collect::<Vec<_>>()
                })
                .or_else(|| v.as_str().map(|s| vec![s.to_uppercase()]))
        });

        let url = match &tickers {
            Some(t) if !t.is_empty() => {
                format!("/v3/snapshot/indices?ticker.any_of={}", t.join(","))
            }
            _ => "/v3/snapshot/indices".to_string(),
        };

        let response: IndexSnapshotResponse = self.client.get(&url).await?;

        let snapshots: Vec<Value> = response
            .results
            .iter()
            .map(|t| {
                let mut snapshot = json!({
                    "ticker": t.ticker,
                    "name": t.name,
                    "type": t.ticker_type,
                    "marketStatus": t.market_status,
                    "value": t.value
                });

                if let Some(session) = &t.session {
                    snapshot["session"] = json!({
                        "open": session.open,
                        "high": session.high,
                        "low": session.low,
                        "close": session.close,
                        "previousClose": session.previous_close,
                        "change": session.change,
                        "changePercent": session.change_percent
                    });
                }

                snapshot
            })
            .collect();

        let result = json!({
            "count": snapshots.len(),
            "indices": snapshots
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetIndexSnapshot {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_index_snapshot")
            .with_description(
                "Get real-time snapshot data for market indices. \
                Returns current values and change percentages for indices like S&P 500, NASDAQ, Dow Jones.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([(
                    "tickers".to_string(),
                    json!({
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of index tickers (e.g., [\"I:SPX\", \"I:NDX\", \"I:DJI\"]). Omit for all indices."
                    }),
                )])),
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
