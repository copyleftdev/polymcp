use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::LastTradeResponse;

pub struct GetLastTrade {
    client: Arc<PolygonClient>,
}

impl GetLastTrade {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let ticker = args
            .get("ticker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("ticker is required".to_string()))?;

        let url = format!("/v2/last/trade/{}", ticker.to_uppercase());

        let response: LastTradeResponse = self.client.get(&url).await?;
        let trade = &response.results;

        let result = json!({
            "ticker": trade.ticker.as_deref().unwrap_or(ticker),
            "price": trade.price,
            "size": trade.size,
            "exchange": trade.exchange,
            "timestamp": trade.sip_timestamp,
            "conditions": trade.conditions,
            "tradeId": trade.trade_id,
            "tape": trade.tape
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetLastTrade {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_last_trade")
            .with_description(
                "Get the most recent trade for a stock. \
                Returns price, size, exchange, timestamp, and trade conditions.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([(
                    "ticker".to_string(),
                    json!({
                        "type": "string",
                        "description": "Stock ticker symbol (e.g., AAPL, MSFT, TSLA)"
                    }),
                )])),
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
