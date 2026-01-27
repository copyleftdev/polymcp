use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::ConversionResponse;

pub struct ConvertCurrency {
    client: Arc<PolygonClient>,
}

impl ConvertCurrency {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let from = args
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("from currency is required".to_string()))?;

        let to = args
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("to currency is required".to_string()))?;

        let amount = args.get("amount").and_then(|v| v.as_f64()).unwrap_or(1.0);

        let precision = args.get("precision").and_then(|v| v.as_u64()).unwrap_or(2);

        let url = format!(
            "/v1/conversion/{}/{}?amount={}&precision={}",
            from.to_uppercase(),
            to.to_uppercase(),
            amount,
            precision
        );

        let response: ConversionResponse = self.client.get(&url).await?;

        let mut result = json!({
            "from": response.from,
            "to": response.to,
            "initialAmount": response.initial_amount,
            "convertedAmount": response.converted
        });

        if let Some(trade) = &response.last_trade {
            result["exchangeRate"] = json!({
                "ask": trade.ask,
                "bid": trade.bid,
                "timestamp": trade.timestamp
            });
        }

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for ConvertCurrency {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("convert_currency")
            .with_description(
                "Convert an amount from one currency to another using real-time exchange rates.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "from".to_string(),
                        json!({
                            "type": "string",
                            "description": "Source currency code (e.g., USD, EUR, GBP)"
                        }),
                    ),
                    (
                        "to".to_string(),
                        json!({
                            "type": "string",
                            "description": "Target currency code (e.g., EUR, JPY, GBP)"
                        }),
                    ),
                    (
                        "amount".to_string(),
                        json!({
                            "type": "number",
                            "description": "Amount to convert (default 1.0)",
                            "default": 1.0
                        }),
                    ),
                    (
                        "precision".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Decimal precision for result (default 2)",
                            "default": 2
                        }),
                    ),
                ])),
                required: Some(vec!["from".to_string(), "to".to_string()]),
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
