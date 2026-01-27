use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsContract {
    pub ticker: Option<String>,
    pub underlying_ticker: Option<String>,
    pub cfi: Option<String>,
    pub contract_type: Option<String>,
    pub exercise_style: Option<String>,
    pub expiration_date: Option<String>,
    pub primary_exchange: Option<String>,
    pub shares_per_contract: Option<f64>,
    pub strike_price: Option<f64>,
    pub additional_underlyings: Option<Vec<AdditionalUnderlying>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalUnderlying {
    #[serde(rename = "type")]
    pub underlying_type: Option<String>,
    pub underlying: Option<String>,
    pub amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractsResponse {
    #[serde(default)]
    pub results: Vec<OptionsContract>,
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsAggregateBar {
    #[serde(rename = "t")]
    pub timestamp: i64,
    #[serde(rename = "o")]
    pub open: f64,
    #[serde(rename = "h")]
    pub high: f64,
    #[serde(rename = "l")]
    pub low: f64,
    #[serde(rename = "c")]
    pub close: f64,
    #[serde(rename = "v")]
    pub volume: f64,
    #[serde(rename = "vw", default)]
    pub vwap: Option<f64>,
    #[serde(rename = "n", default)]
    pub transactions: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsAggregatesResponse {
    pub ticker: Option<String>,
    #[serde(rename = "queryCount", default)]
    pub query_count: Option<i64>,
    #[serde(rename = "resultsCount", default)]
    pub results_count: Option<i64>,
    pub adjusted: Option<bool>,
    #[serde(default)]
    pub results: Vec<OptionsAggregateBar>,
    pub status: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Greeks {
    pub delta: Option<f64>,
    pub gamma: Option<f64>,
    pub theta: Option<f64>,
    pub vega: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsDay {
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub close: Option<f64>,
    pub high: Option<f64>,
    pub last_updated: Option<i64>,
    pub low: Option<f64>,
    pub open: Option<f64>,
    pub previous_close: Option<f64>,
    pub volume: Option<f64>,
    pub vwap: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsLastQuote {
    pub ask: Option<f64>,
    pub ask_size: Option<f64>,
    pub bid: Option<f64>,
    pub bid_size: Option<f64>,
    pub last_updated: Option<i64>,
    pub midpoint: Option<f64>,
    pub timeframe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsLastTrade {
    #[serde(default)]
    pub conditions: Vec<i32>,
    pub exchange: Option<i32>,
    pub price: Option<f64>,
    pub sip_timestamp: Option<i64>,
    pub size: Option<i64>,
    pub timeframe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderlyingAsset {
    pub change_to_break_even: Option<f64>,
    pub last_updated: Option<i64>,
    pub price: Option<f64>,
    pub ticker: Option<String>,
    pub timeframe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsContractSnapshot {
    pub break_even_price: Option<f64>,
    pub day: Option<OptionsDay>,
    pub details: Option<OptionsContract>,
    pub greeks: Option<Greeks>,
    pub implied_volatility: Option<f64>,
    pub last_quote: Option<OptionsLastQuote>,
    pub last_trade: Option<OptionsLastTrade>,
    pub open_interest: Option<f64>,
    pub underlying_asset: Option<UnderlyingAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsSnapshotResponse {
    #[serde(default)]
    pub results: Vec<OptionsContractSnapshot>,
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub next_url: Option<String>,
}
