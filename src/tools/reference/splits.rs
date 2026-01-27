use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::SplitsResponse;

pub struct GetStockSplits {
    client: Arc<PolygonClient>,
}

impl GetStockSplits {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let mut params = Vec::new();

        if let Some(ticker) = args.get("ticker").and_then(|v| v.as_str()) {
            params.push(format!("ticker={}", ticker.to_uppercase()));
        }

        if let Some(execution_date) = args.get("execution_date").and_then(|v| v.as_str()) {
            params.push(format!("execution_date={}", execution_date));
        }

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100);
        params.push(format!("limit={}", limit));

        if let Some(order) = args.get("order").and_then(|v| v.as_str()) {
            params.push(format!("order={}", order));
        }

        let url = format!("/v3/reference/splits?{}", params.join("&"));

        let response: SplitsResponse = self.client.get(&url).await?;

        let splits: Vec<Value> = response
            .results
            .iter()
            .map(|s| {
                json!({
                    "ticker": s.ticker,
                    "executionDate": s.execution_date,
                    "splitFrom": s.split_from,
                    "splitTo": s.split_to,
                    "ratio": s.split_to.zip(s.split_from).map(|(to, from)| to / from)
                })
            })
            .collect();

        let result = json!({
            "splits": splits
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetStockSplits {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_stock_splits")
            .with_description(
                "Get stock split history. Returns execution date, split ratio (from/to), \
                useful for understanding historical price adjustments.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Stock ticker symbol (e.g., AAPL, TSLA)"
                        }),
                    ),
                    (
                        "execution_date".to_string(),
                        json!({
                            "type": "string",
                            "description": "Filter by split execution date (YYYY-MM-DD)"
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
                ])),
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
