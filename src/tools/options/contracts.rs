use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::ContractsResponse;

pub struct GetOptionsContracts {
    client: Arc<PolygonClient>,
}

impl GetOptionsContracts {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let mut params = Vec::new();

        if let Some(underlying) = args.get("underlying_ticker").and_then(|v| v.as_str()) {
            params.push(format!("underlying_ticker={}", underlying.to_uppercase()));
        }

        if let Some(contract_type) = args.get("contract_type").and_then(|v| v.as_str()) {
            params.push(format!("contract_type={}", contract_type));
        }

        if let Some(expiration) = args.get("expiration_date").and_then(|v| v.as_str()) {
            params.push(format!("expiration_date={}", expiration));
        }

        if let Some(strike) = args.get("strike_price").and_then(|v| v.as_f64()) {
            params.push(format!("strike_price={}", strike));
        }

        if let Some(expired) = args.get("expired").and_then(|v| v.as_bool()) {
            params.push(format!("expired={}", expired));
        }

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100);
        params.push(format!("limit={}", limit));

        if let Some(order) = args.get("order").and_then(|v| v.as_str()) {
            params.push(format!("order={}", order));
        }

        if let Some(sort) = args.get("sort").and_then(|v| v.as_str()) {
            params.push(format!("sort={}", sort));
        }

        let url = format!("/v3/reference/options/contracts?{}", params.join("&"));

        let response: ContractsResponse = self.client.get(&url).await?;

        let contracts: Vec<Value> = response
            .results
            .iter()
            .map(|c| {
                json!({
                    "ticker": c.ticker,
                    "underlyingTicker": c.underlying_ticker,
                    "contractType": c.contract_type,
                    "expirationDate": c.expiration_date,
                    "strikePrice": c.strike_price,
                    "exerciseStyle": c.exercise_style,
                    "sharesPerContract": c.shares_per_contract,
                    "primaryExchange": c.primary_exchange
                })
            })
            .collect();

        let result = json!({
            "contracts": contracts
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetOptionsContracts {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_options_contracts")
            .with_description(
                "Get options contracts for an underlying stock. Returns contract ticker, \
                expiration date, strike price, and contract type (call/put).",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "underlying_ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Underlying stock ticker (e.g., AAPL, TSLA)"
                        }),
                    ),
                    (
                        "contract_type".to_string(),
                        json!({
                            "type": "string",
                            "description": "Contract type: call or put",
                            "enum": ["call", "put"]
                        }),
                    ),
                    (
                        "expiration_date".to_string(),
                        json!({
                            "type": "string",
                            "description": "Expiration date (YYYY-MM-DD)"
                        }),
                    ),
                    (
                        "strike_price".to_string(),
                        json!({
                            "type": "number",
                            "description": "Strike price of the contract"
                        }),
                    ),
                    (
                        "expired".to_string(),
                        json!({
                            "type": "boolean",
                            "description": "Include expired contracts (default false)"
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
                            "description": "Field to sort by: expiration_date, strike_price, ticker"
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
