use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::IndexOpenCloseResponse;

pub struct GetIndexOpenClose {
    client: Arc<PolygonClient>,
}

impl GetIndexOpenClose {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let ticker = args
            .get("ticker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("ticker is required".to_string()))?;

        let date = args
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("date is required".to_string()))?;

        let url = format!("/v1/open-close/{}/{}", ticker.to_uppercase(), date);

        let response: IndexOpenCloseResponse = self.client.get(&url).await?;

        let result = json!({
            "symbol": response.symbol,
            "date": response.from,
            "open": response.open,
            "high": response.high,
            "low": response.low,
            "close": response.close,
            "afterHours": response.after_hours,
            "preMarket": response.pre_market
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetIndexOpenClose {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_index_open_close")
            .with_description(
                "Get daily open, high, low, and close values for a market index on a specific date.",
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
                        "date".to_string(),
                        json!({
                            "type": "string",
                            "description": "Date (YYYY-MM-DD)"
                        }),
                    ),
                ])),
                required: Some(vec!["ticker".to_string(), "date".to_string()]),
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
