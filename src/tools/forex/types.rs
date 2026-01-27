use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexAggregateBar {
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
    #[serde(rename = "v", default)]
    pub volume: Option<f64>,
    #[serde(rename = "vw", default)]
    pub vwap: Option<f64>,
    #[serde(rename = "n", default)]
    pub transactions: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexAggregatesResponse {
    pub ticker: Option<String>,
    #[serde(rename = "queryCount", default)]
    pub query_count: Option<i64>,
    #[serde(rename = "resultsCount", default)]
    pub results_count: Option<i64>,
    pub adjusted: Option<bool>,
    #[serde(default = "Vec::new")]
    pub results: Vec<ForexAggregateBar>,
    pub status: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResponse {
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    #[serde(rename = "initialAmount")]
    pub initial_amount: Option<f64>,
    pub converted: Option<f64>,
    #[serde(rename = "lastTrade")]
    pub last_trade: Option<ForexLastTrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexLastTrade {
    pub ask: Option<f64>,
    pub bid: Option<f64>,
    pub exchange: Option<i32>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexTicker {
    pub ticker: Option<String>,
    pub todays_change: Option<f64>,
    pub todays_change_perc: Option<f64>,
    pub updated: Option<i64>,
    pub day: Option<ForexDay>,
    pub min: Option<ForexMin>,
    #[serde(rename = "prevDay")]
    pub prev_day: Option<ForexDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexDay {
    pub c: Option<f64>,
    pub h: Option<f64>,
    pub l: Option<f64>,
    pub o: Option<f64>,
    pub v: Option<f64>,
    pub vw: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexMin {
    pub av: Option<f64>,
    pub c: Option<f64>,
    pub h: Option<f64>,
    pub l: Option<f64>,
    pub o: Option<f64>,
    pub v: Option<f64>,
    pub vw: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexSnapshotResponse {
    pub status: Option<String>,
    #[serde(default = "Vec::new")]
    pub tickers: Vec<ForexTicker>,
}
