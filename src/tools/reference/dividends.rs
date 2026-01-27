use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::DividendsResponse;

pub struct GetDividends {
    client: Arc<PolygonClient>,
}

impl GetDividends {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let mut params = Vec::new();

        if let Some(ticker) = args.get("ticker").and_then(|v| v.as_str()) {
            params.push(format!("ticker={}", ticker.to_uppercase()));
        }

        if let Some(ex_date) = args.get("ex_dividend_date").and_then(|v| v.as_str()) {
            params.push(format!("ex_dividend_date={}", ex_date));
        }

        if let Some(record_date) = args.get("record_date").and_then(|v| v.as_str()) {
            params.push(format!("record_date={}", record_date));
        }

        if let Some(pay_date) = args.get("pay_date").and_then(|v| v.as_str()) {
            params.push(format!("pay_date={}", pay_date));
        }

        if let Some(frequency) = args.get("frequency").and_then(|v| v.as_i64()) {
            params.push(format!("frequency={}", frequency));
        }

        if let Some(dividend_type) = args.get("dividend_type").and_then(|v| v.as_str()) {
            params.push(format!("dividend_type={}", dividend_type));
        }

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100);
        params.push(format!("limit={}", limit));

        if let Some(order) = args.get("order").and_then(|v| v.as_str()) {
            params.push(format!("order={}", order));
        }

        let url = format!("/v3/reference/dividends?{}", params.join("&"));

        let response: DividendsResponse = self.client.get(&url).await?;

        let dividends: Vec<Value> = response
            .results
            .iter()
            .map(|d| {
                json!({
                    "ticker": d.ticker,
                    "exDividendDate": d.ex_dividend_date,
                    "recordDate": d.record_date,
                    "payDate": d.pay_date,
                    "declarationDate": d.declaration_date,
                    "cashAmount": d.cash_amount,
                    "currency": d.currency,
                    "frequency": d.frequency,
                    "dividendType": d.dividend_type
                })
            })
            .collect();

        let result = json!({
            "dividends": dividends
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetDividends {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_dividends")
            .with_description(
                "Get dividend history for a stock. Returns ex-dividend date, pay date, \
                cash amount, currency, and frequency.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Stock ticker symbol (e.g., AAPL, MSFT)"
                        }),
                    ),
                    (
                        "ex_dividend_date".to_string(),
                        json!({
                            "type": "string",
                            "description": "Filter by ex-dividend date (YYYY-MM-DD)"
                        }),
                    ),
                    (
                        "record_date".to_string(),
                        json!({
                            "type": "string",
                            "description": "Filter by record date (YYYY-MM-DD)"
                        }),
                    ),
                    (
                        "pay_date".to_string(),
                        json!({
                            "type": "string",
                            "description": "Filter by pay date (YYYY-MM-DD)"
                        }),
                    ),
                    (
                        "frequency".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Dividend frequency: 1=annual, 2=bi-annual, 4=quarterly, 12=monthly"
                        }),
                    ),
                    (
                        "dividend_type".to_string(),
                        json!({
                            "type": "string",
                            "description": "Type: CD (cash), SC (stock), LT (long-term), ST (short-term)"
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
