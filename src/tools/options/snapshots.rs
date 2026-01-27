use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::OptionsSnapshotResponse;

pub struct GetOptionsSnapshot {
    client: Arc<PolygonClient>,
}

impl GetOptionsSnapshot {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let underlying = args
            .get("underlying_asset")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PolygonError::InvalidParams("underlying_asset is required".to_string())
            })?;

        let mut params = Vec::new();

        if let Some(contract_type) = args.get("contract_type").and_then(|v| v.as_str()) {
            params.push(format!("contract_type={}", contract_type));
        }

        if let Some(expiration) = args.get("expiration_date").and_then(|v| v.as_str()) {
            params.push(format!("expiration_date={}", expiration));
        }

        if let Some(strike) = args.get("strike_price").and_then(|v| v.as_f64()) {
            params.push(format!("strike_price={}", strike));
        }

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(250);
        params.push(format!("limit={}", limit));

        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

        let url = format!(
            "/v3/snapshot/options/{}{}",
            underlying.to_uppercase(),
            query
        );

        let response: OptionsSnapshotResponse = self.client.get(&url).await?;

        let snapshots: Vec<Value> = response
            .results
            .iter()
            .map(|s| {
                let mut snapshot = json!({
                    "breakEvenPrice": s.break_even_price,
                    "impliedVolatility": s.implied_volatility,
                    "openInterest": s.open_interest
                });

                if let Some(details) = &s.details {
                    snapshot["details"] = json!({
                        "ticker": details.ticker,
                        "underlyingTicker": details.underlying_ticker,
                        "contractType": details.contract_type,
                        "expirationDate": details.expiration_date,
                        "strikePrice": details.strike_price,
                        "exerciseStyle": details.exercise_style
                    });
                }

                if let Some(greeks) = &s.greeks {
                    snapshot["greeks"] = json!({
                        "delta": greeks.delta,
                        "gamma": greeks.gamma,
                        "theta": greeks.theta,
                        "vega": greeks.vega
                    });
                }

                if let Some(day) = &s.day {
                    snapshot["day"] = json!({
                        "open": day.open,
                        "high": day.high,
                        "low": day.low,
                        "close": day.close,
                        "volume": day.volume,
                        "vwap": day.vwap,
                        "change": day.change,
                        "changePercent": day.change_percent
                    });
                }

                if let Some(quote) = &s.last_quote {
                    snapshot["lastQuote"] = json!({
                        "bid": quote.bid,
                        "bidSize": quote.bid_size,
                        "ask": quote.ask,
                        "askSize": quote.ask_size,
                        "midpoint": quote.midpoint
                    });
                }

                if let Some(trade) = &s.last_trade {
                    snapshot["lastTrade"] = json!({
                        "price": trade.price,
                        "size": trade.size,
                        "exchange": trade.exchange
                    });
                }

                if let Some(underlying) = &s.underlying_asset {
                    snapshot["underlyingAsset"] = json!({
                        "ticker": underlying.ticker,
                        "price": underlying.price,
                        "changeToBreakEven": underlying.change_to_break_even
                    });
                }

                snapshot
            })
            .collect();

        let result = json!({
            "underlying": underlying.to_uppercase(),
            "count": snapshots.len(),
            "snapshots": snapshots
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetOptionsSnapshot {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_options_snapshot")
            .with_description(
                "Get real-time snapshot data for options contracts on an underlying asset. \
                Returns greeks, implied volatility, open interest, and pricing for each contract.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "underlying_asset".to_string(),
                        json!({
                            "type": "string",
                            "description": "Underlying stock ticker (e.g., AAPL, TSLA)"
                        }),
                    ),
                    (
                        "contract_type".to_string(),
                        json!({
                            "type": "string",
                            "description": "Filter by contract type: call or put",
                            "enum": ["call", "put"]
                        }),
                    ),
                    (
                        "expiration_date".to_string(),
                        json!({
                            "type": "string",
                            "description": "Filter by expiration date (YYYY-MM-DD)"
                        }),
                    ),
                    (
                        "strike_price".to_string(),
                        json!({
                            "type": "number",
                            "description": "Filter by strike price"
                        }),
                    ),
                    (
                        "limit".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Max results (default 250)",
                            "default": 250
                        }),
                    ),
                ])),
                required: Some(vec!["underlying_asset".to_string()]),
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
