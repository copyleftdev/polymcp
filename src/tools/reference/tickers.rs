use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::{TickerDetailsResponse, TickerSearchResponse};

pub struct GetTickerDetails {
    client: Arc<PolygonClient>,
}

impl GetTickerDetails {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let ticker = args
            .get("ticker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("ticker is required".to_string()))?;

        let mut url = format!("/v3/reference/tickers/{}", ticker.to_uppercase());

        if let Some(date) = args.get("date").and_then(|v| v.as_str()) {
            url.push_str(&format!("?date={}", date));
        }

        let response: TickerDetailsResponse = self.client.get(&url).await?;
        let details = &response.results;

        let result = json!({
            "ticker": details.ticker,
            "name": details.name,
            "market": details.market,
            "locale": details.locale,
            "type": details.ticker_type,
            "active": details.active,
            "currency": details.currency_name,
            "description": details.description,
            "homepageUrl": details.homepage_url,
            "listDate": details.list_date,
            "totalEmployees": details.total_employees,
            "primaryExchange": details.primary_exchange,
            "marketCap": details.market_cap,
            "sicCode": details.sic_code,
            "sicDescription": details.sic_description,
            "cik": details.cik,
            "compositeFigi": details.composite_figi,
            "address": details.address.as_ref().map(|a| json!({
                "address1": a.address1,
                "city": a.city,
                "state": a.state,
                "postalCode": a.postal_code
            })),
            "branding": details.branding.as_ref().map(|b| json!({
                "logoUrl": b.logo_url,
                "iconUrl": b.icon_url
            }))
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetTickerDetails {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_ticker_details")
            .with_description(
                "Get detailed information about a ticker symbol including company name, \
                market, locale, type, currency, description, homepage URL, and branding.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Ticker symbol (e.g., AAPL, MSFT, TSLA)"
                        }),
                    ),
                    (
                        "date".to_string(),
                        json!({
                            "type": "string",
                            "description": "Optional date (YYYY-MM-DD) for historical data"
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

pub struct SearchTickers {
    client: Arc<PolygonClient>,
}

impl SearchTickers {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let mut params = Vec::new();

        if let Some(search) = args.get("search").and_then(|v| v.as_str()) {
            params.push(format!("search={}", search));
        }

        if let Some(ticker) = args.get("ticker").and_then(|v| v.as_str()) {
            params.push(format!("ticker={}", ticker.to_uppercase()));
        }

        if let Some(market) = args.get("market").and_then(|v| v.as_str()) {
            params.push(format!("market={}", market));
        }

        if let Some(ticker_type) = args.get("type").and_then(|v| v.as_str()) {
            params.push(format!("type={}", ticker_type));
        }

        if let Some(active) = args.get("active").and_then(|v| v.as_bool()) {
            params.push(format!("active={}", active));
        }

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100);
        params.push(format!("limit={}", limit));

        let url = if params.is_empty() {
            "/v3/reference/tickers".to_string()
        } else {
            format!("/v3/reference/tickers?{}", params.join("&"))
        };

        let response: TickerSearchResponse = self.client.get(&url).await?;

        let tickers: Vec<Value> = response
            .results
            .iter()
            .map(|t| {
                json!({
                    "ticker": t.ticker,
                    "name": t.name,
                    "market": t.market,
                    "locale": t.locale,
                    "type": t.ticker_type,
                    "active": t.active,
                    "primaryExchange": t.primary_exchange,
                    "currency": t.currency_name,
                    "lastUpdated": t.last_updated_utc
                })
            })
            .collect();

        let result = json!({
            "count": response.count,
            "tickers": tickers
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for SearchTickers {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("search_tickers")
            .with_description(
                "Search for tickers by name or symbol. Returns matching tickers with \
                name, market, type, and active status.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "search".to_string(),
                        json!({
                            "type": "string",
                            "description": "Search query (company name or partial ticker)"
                        }),
                    ),
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Filter by specific ticker prefix"
                        }),
                    ),
                    (
                        "market".to_string(),
                        json!({
                            "type": "string",
                            "description": "Filter by market: stocks, crypto, fx, otc",
                            "enum": ["stocks", "crypto", "fx", "otc"]
                        }),
                    ),
                    (
                        "type".to_string(),
                        json!({
                            "type": "string",
                            "description": "Filter by ticker type: CS (common stock), ETF, etc."
                        }),
                    ),
                    (
                        "active".to_string(),
                        json!({
                            "type": "boolean",
                            "description": "Filter by active status"
                        }),
                    ),
                    (
                        "limit".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Max results to return (default 100)",
                            "default": 100
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
