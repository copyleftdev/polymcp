use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::CryptoSnapshotResponse;

pub struct GetCryptoSnapshot {
    client: Arc<PolygonClient>,
}

impl GetCryptoSnapshot {
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
                format!(
                    "/v2/snapshot/locale/global/markets/crypto/tickers?tickers={}",
                    t.join(",")
                )
            }
            _ => "/v2/snapshot/locale/global/markets/crypto/tickers".to_string(),
        };

        let response: CryptoSnapshotResponse = self.client.get(&url).await?;

        let snapshots: Vec<Value> = response
            .tickers
            .iter()
            .map(|t| {
                let mut snapshot = json!({
                    "ticker": t.ticker,
                    "todaysChange": t.todays_change,
                    "todaysChangePercent": t.todays_change_perc,
                    "updated": t.updated
                });

                if let Some(day) = &t.day {
                    snapshot["day"] = json!({
                        "open": day.o,
                        "high": day.h,
                        "low": day.l,
                        "close": day.c,
                        "volume": day.v,
                        "vwap": day.vw
                    });
                }

                if let Some(min) = &t.min {
                    snapshot["lastMinute"] = json!({
                        "open": min.o,
                        "high": min.h,
                        "low": min.l,
                        "close": min.c,
                        "volume": min.v,
                        "vwap": min.vw
                    });
                }

                if let Some(prev) = &t.prev_day {
                    snapshot["previousDay"] = json!({
                        "open": prev.o,
                        "high": prev.h,
                        "low": prev.l,
                        "close": prev.c,
                        "volume": prev.v,
                        "vwap": prev.vw
                    });
                }

                if let Some(trade) = &t.last_trade {
                    snapshot["lastTrade"] = json!({
                        "price": trade.p,
                        "size": trade.s,
                        "exchange": trade.x,
                        "timestamp": trade.t
                    });
                }

                snapshot
            })
            .collect();

        let result = json!({
            "count": snapshots.len(),
            "tickers": snapshots
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetCryptoSnapshot {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_crypto_snapshot")
            .with_description(
                "Get real-time snapshot data for cryptocurrency pairs. \
                Returns current prices, 24h changes, and trading volume.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([(
                    "tickers".to_string(),
                    json!({
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of crypto tickers (e.g., [\"X:BTCUSD\", \"X:ETHUSD\"]). Omit for all pairs."
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
