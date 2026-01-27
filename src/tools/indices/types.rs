use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexAggregateBar {
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
pub struct IndexAggregatesResponse {
    pub ticker: Option<String>,
    #[serde(rename = "queryCount", default)]
    pub query_count: Option<i64>,
    #[serde(rename = "resultsCount", default)]
    pub results_count: Option<i64>,
    pub adjusted: Option<bool>,
    #[serde(default = "Vec::new")]
    pub results: Vec<IndexAggregateBar>,
    pub status: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexOpenCloseResponse {
    pub status: Option<String>,
    pub symbol: Option<String>,
    pub from: Option<String>,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    #[serde(rename = "afterHours")]
    pub after_hours: Option<f64>,
    #[serde(rename = "preMarket")]
    pub pre_market: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSession {
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub close: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub open: Option<f64>,
    pub previous_close: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexTicker {
    pub ticker: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub ticker_type: Option<String>,
    pub market_status: Option<String>,
    pub session: Option<IndexSession>,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSnapshotResponse {
    pub status: Option<String>,
    pub request_id: Option<String>,
    #[serde(default = "Vec::new")]
    pub results: Vec<IndexTicker>,
}
