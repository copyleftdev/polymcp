use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::SnapshotResponse;

pub struct GetStockSnapshot {
    client: Arc<PolygonClient>,
}

impl GetStockSnapshot {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let ticker = args
            .get("ticker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("ticker is required".to_string()))?;

        let url = format!(
            "/v2/snapshot/locale/us/markets/stocks/tickers/{}",
            ticker.to_uppercase()
        );

        let response: SnapshotResponse = self.client.get(&url).await?;
        let snap = &response.ticker;

        let day = snap.day.as_ref().map(|d| {
            json!({
                "open": d.open,
                "high": d.high,
                "low": d.low,
                "close": d.close,
                "volume": d.volume,
                "vwap": d.vwap
            })
        });

        let prev_day = snap.prev_day.as_ref().map(|d| {
            json!({
                "open": d.open,
                "high": d.high,
                "low": d.low,
                "close": d.close,
                "volume": d.volume,
                "vwap": d.vwap
            })
        });

        let min_agg = snap.min.as_ref().map(|m| {
            json!({
                "open": m.open,
                "high": m.high,
                "low": m.low,
                "close": m.close,
                "volume": m.volume,
                "vwap": m.vwap,
                "timestamp": m.timestamp,
                "accumulatedVolume": m.accumulated_volume
            })
        });

        let last_trade = snap.last_trade.as_ref().map(|t| {
            json!({
                "price": t.price,
                "size": t.size,
                "exchange": t.exchange,
                "timestamp": t.timestamp,
                "conditions": t.conditions
            })
        });

        let last_quote = snap.last_quote.as_ref().map(|q| {
            json!({
                "bidPrice": q.bid_price,
                "bidSize": q.bid_size,
                "askPrice": q.ask_price,
                "askSize": q.ask_size,
                "timestamp": q.timestamp
            })
        });

        let result = json!({
            "ticker": snap.ticker,
            "todaysChange": snap.todays_change,
            "todaysChangePercent": snap.todays_change_perc,
            "updated": snap.updated,
            "day": day,
            "prevDay": prev_day,
            "min": min_agg,
            "lastTrade": last_trade,
            "lastQuote": last_quote
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetStockSnapshot {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_stock_snapshot")
            .with_description(
                "Get the current snapshot for a stock including day/prevDay aggregates, \
                last trade, last quote, and minute aggregate data.",
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
