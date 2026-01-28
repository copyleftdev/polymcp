use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::SmaResponse;

pub struct GetSma {
    client: Arc<PolygonClient>,
}

impl GetSma {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let ticker = args
            .get("ticker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("ticker is required".to_string()))?;

        let window = args.get("window").and_then(|v| v.as_u64()).unwrap_or(20);

        let timespan = args
            .get("timespan")
            .and_then(|v| v.as_str())
            .unwrap_or("day");

        let series_type = args
            .get("series_type")
            .and_then(|v| v.as_str())
            .unwrap_or("close");

        let adjusted = args
            .get("adjusted")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50);

        let mut url = format!(
            "/v1/indicators/sma/{}?window={}&timespan={}&series_type={}&adjusted={}&limit={}",
            ticker.to_uppercase(),
            window,
            timespan,
            series_type,
            adjusted,
            limit
        );

        if let Some(order) = args.get("order").and_then(|v| v.as_str()) {
            url.push_str(&format!("&order={}", order));
        }

        let response: SmaResponse = self.client.get(&url).await?;

        let values: Vec<Value> = response
            .results
            .as_ref()
            .map(|r| {
                r.values
                    .iter()
                    .map(|v| {
                        json!({
                            "timestamp": v.timestamp,
                            "value": v.value
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let result = json!({
            "ticker": ticker.to_uppercase(),
            "window": window,
            "timespan": timespan,
            "seriesType": series_type,
            "values": values
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetSma {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_sma")
            .with_description(
                "Get Simple Moving Average (SMA) for a stock. \
                Returns SMA values calculated over the specified window and timespan.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Stock ticker symbol (e.g., AAPL, MSFT, TSLA)"
                        }),
                    ),
                    (
                        "window".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Window size for SMA calculation (e.g., 20 for 20-period SMA)",
                            "default": 20
                        }),
                    ),
                    (
                        "timespan".to_string(),
                        json!({
                            "type": "string",
                            "description": "Aggregate timespan: second, minute, hour, day, week, month, quarter, year",
                            "enum": ["second", "minute", "hour", "day", "week", "month", "quarter", "year"],
                            "default": "day"
                        }),
                    ),
                    (
                        "series_type".to_string(),
                        json!({
                            "type": "string",
                            "description": "Price type: open, high, low, close",
                            "enum": ["open", "high", "low", "close"],
                            "default": "close"
                        }),
                    ),
                    (
                        "adjusted".to_string(),
                        json!({
                            "type": "boolean",
                            "description": "Whether results are adjusted for splits",
                            "default": true
                        }),
                    ),
                    (
                        "limit".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Number of results to return",
                            "default": 50
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

pub struct GetEma {
    client: Arc<PolygonClient>,
}

impl GetEma {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let ticker = args
            .get("ticker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("ticker is required".to_string()))?;

        let window = args.get("window").and_then(|v| v.as_u64()).unwrap_or(20);

        let timespan = args
            .get("timespan")
            .and_then(|v| v.as_str())
            .unwrap_or("day");

        let series_type = args
            .get("series_type")
            .and_then(|v| v.as_str())
            .unwrap_or("close");

        let adjusted = args
            .get("adjusted")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50);

        let url = format!(
            "/v1/indicators/ema/{}?window={}&timespan={}&series_type={}&adjusted={}&limit={}",
            ticker.to_uppercase(),
            window,
            timespan,
            series_type,
            adjusted,
            limit
        );

        let response: SmaResponse = self.client.get(&url).await?;

        let values: Vec<Value> = response
            .results
            .as_ref()
            .map(|r| {
                r.values
                    .iter()
                    .map(|v| {
                        json!({
                            "timestamp": v.timestamp,
                            "value": v.value
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let result = json!({
            "ticker": ticker.to_uppercase(),
            "window": window,
            "timespan": timespan,
            "seriesType": series_type,
            "values": values
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetEma {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_ema")
            .with_description(
                "Get Exponential Moving Average (EMA) for a stock. \
                Returns EMA values calculated over the specified window and timespan.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Stock ticker symbol (e.g., AAPL, MSFT, TSLA)"
                        }),
                    ),
                    (
                        "window".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Window size for EMA calculation",
                            "default": 20
                        }),
                    ),
                    (
                        "timespan".to_string(),
                        json!({
                            "type": "string",
                            "description": "Aggregate timespan: second, minute, hour, day, week, month, quarter, year",
                            "enum": ["second", "minute", "hour", "day", "week", "month", "quarter", "year"],
                            "default": "day"
                        }),
                    ),
                    (
                        "series_type".to_string(),
                        json!({
                            "type": "string",
                            "description": "Price type: open, high, low, close",
                            "enum": ["open", "high", "low", "close"],
                            "default": "close"
                        }),
                    ),
                    (
                        "adjusted".to_string(),
                        json!({
                            "type": "boolean",
                            "description": "Whether results are adjusted for splits",
                            "default": true
                        }),
                    ),
                    (
                        "limit".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Number of results to return",
                            "default": 50
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

pub struct GetRsi {
    client: Arc<PolygonClient>,
}

impl GetRsi {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let ticker = args
            .get("ticker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("ticker is required".to_string()))?;

        let window = args.get("window").and_then(|v| v.as_u64()).unwrap_or(14);

        let timespan = args
            .get("timespan")
            .and_then(|v| v.as_str())
            .unwrap_or("day");

        let series_type = args
            .get("series_type")
            .and_then(|v| v.as_str())
            .unwrap_or("close");

        let adjusted = args
            .get("adjusted")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50);

        let url = format!(
            "/v1/indicators/rsi/{}?window={}&timespan={}&series_type={}&adjusted={}&limit={}",
            ticker.to_uppercase(),
            window,
            timespan,
            series_type,
            adjusted,
            limit
        );

        let response: SmaResponse = self.client.get(&url).await?;

        let values: Vec<Value> = response
            .results
            .as_ref()
            .map(|r| {
                r.values
                    .iter()
                    .map(|v| {
                        json!({
                            "timestamp": v.timestamp,
                            "value": v.value
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let result = json!({
            "ticker": ticker.to_uppercase(),
            "window": window,
            "timespan": timespan,
            "seriesType": series_type,
            "values": values
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetRsi {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_rsi")
            .with_description(
                "Get Relative Strength Index (RSI) for a stock. \
                RSI measures momentum on a scale of 0-100. Values above 70 indicate overbought, below 30 oversold.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Stock ticker symbol (e.g., AAPL, MSFT, TSLA)"
                        }),
                    ),
                    (
                        "window".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Window size for RSI calculation (typically 14)",
                            "default": 14
                        }),
                    ),
                    (
                        "timespan".to_string(),
                        json!({
                            "type": "string",
                            "description": "Aggregate timespan: second, minute, hour, day, week, month, quarter, year",
                            "enum": ["second", "minute", "hour", "day", "week", "month", "quarter", "year"],
                            "default": "day"
                        }),
                    ),
                    (
                        "series_type".to_string(),
                        json!({
                            "type": "string",
                            "description": "Price type: open, high, low, close",
                            "enum": ["open", "high", "low", "close"],
                            "default": "close"
                        }),
                    ),
                    (
                        "adjusted".to_string(),
                        json!({
                            "type": "boolean",
                            "description": "Whether results are adjusted for splits",
                            "default": true
                        }),
                    ),
                    (
                        "limit".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Number of results to return",
                            "default": 50
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

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct MacdResponse {
    results: Option<MacdResults>,
    status: Option<String>,
    request_id: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct MacdResults {
    #[serde(default)]
    values: Vec<MacdValue>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct MacdValue {
    timestamp: i64,
    value: f64,
    signal: f64,
    histogram: f64,
}

pub struct GetMacd {
    client: Arc<PolygonClient>,
}

impl GetMacd {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let ticker = args
            .get("ticker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PolygonError::InvalidParams("ticker is required".to_string()))?;

        let short_window = args
            .get("short_window")
            .and_then(|v| v.as_u64())
            .unwrap_or(12);
        let long_window = args
            .get("long_window")
            .and_then(|v| v.as_u64())
            .unwrap_or(26);
        let signal_window = args
            .get("signal_window")
            .and_then(|v| v.as_u64())
            .unwrap_or(9);

        let timespan = args
            .get("timespan")
            .and_then(|v| v.as_str())
            .unwrap_or("day");

        let series_type = args
            .get("series_type")
            .and_then(|v| v.as_str())
            .unwrap_or("close");

        let adjusted = args
            .get("adjusted")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50);

        let url = format!(
            "/v1/indicators/macd/{}?short_window={}&long_window={}&signal_window={}&timespan={}&series_type={}&adjusted={}&limit={}",
            ticker.to_uppercase(),
            short_window,
            long_window,
            signal_window,
            timespan,
            series_type,
            adjusted,
            limit
        );

        let response: MacdResponse = self.client.get(&url).await?;

        let values: Vec<Value> = response
            .results
            .as_ref()
            .map(|r| {
                r.values
                    .iter()
                    .map(|v| {
                        json!({
                            "timestamp": v.timestamp,
                            "macd": v.value,
                            "signal": v.signal,
                            "histogram": v.histogram
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let result = json!({
            "ticker": ticker.to_uppercase(),
            "shortWindow": short_window,
            "longWindow": long_window,
            "signalWindow": signal_window,
            "timespan": timespan,
            "seriesType": series_type,
            "values": values
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetMacd {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_macd")
            .with_description(
                "Get MACD (Moving Average Convergence/Divergence) for a stock. \
                Returns MACD line, signal line, and histogram values.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Stock ticker symbol (e.g., AAPL, MSFT, TSLA)"
                        }),
                    ),
                    (
                        "short_window".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Short EMA window (typically 12)",
                            "default": 12
                        }),
                    ),
                    (
                        "long_window".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Long EMA window (typically 26)",
                            "default": 26
                        }),
                    ),
                    (
                        "signal_window".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Signal line EMA window (typically 9)",
                            "default": 9
                        }),
                    ),
                    (
                        "timespan".to_string(),
                        json!({
                            "type": "string",
                            "description": "Aggregate timespan: second, minute, hour, day, week, month, quarter, year",
                            "enum": ["second", "minute", "hour", "day", "week", "month", "quarter", "year"],
                            "default": "day"
                        }),
                    ),
                    (
                        "series_type".to_string(),
                        json!({
                            "type": "string",
                            "description": "Price type: open, high, low, close",
                            "enum": ["open", "high", "low", "close"],
                            "default": "close"
                        }),
                    ),
                    (
                        "adjusted".to_string(),
                        json!({
                            "type": "boolean",
                            "description": "Whether results are adjusted for splits",
                            "default": true
                        }),
                    ),
                    (
                        "limit".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Number of results to return",
                            "default": 50
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
