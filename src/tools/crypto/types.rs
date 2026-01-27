use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoAggregateBar {
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
pub struct CryptoAggregatesResponse {
    pub ticker: Option<String>,
    #[serde(rename = "queryCount", default)]
    pub query_count: Option<i64>,
    #[serde(rename = "resultsCount", default)]
    pub results_count: Option<i64>,
    pub adjusted: Option<bool>,
    #[serde(default = "Vec::new")]
    pub results: Vec<CryptoAggregateBar>,
    pub status: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoTrade {
    #[serde(default)]
    pub conditions: Vec<i32>,
    pub exchange: Option<i64>,
    pub price: Option<f64>,
    pub size: Option<f64>,
    #[serde(rename = "participant_timestamp")]
    pub participant_timestamp: Option<i64>,
    #[serde(rename = "sip_timestamp")]
    pub sip_timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoTradesResponse {
    #[serde(default = "Vec::new")]
    pub results: Vec<CryptoTrade>,
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoTicker {
    pub ticker: Option<String>,
    #[serde(rename = "todaysChange")]
    pub todays_change: Option<f64>,
    #[serde(rename = "todaysChangePerc")]
    pub todays_change_perc: Option<f64>,
    pub updated: Option<i64>,
    pub day: Option<CryptoDay>,
    pub min: Option<CryptoMin>,
    #[serde(rename = "prevDay")]
    pub prev_day: Option<CryptoDay>,
    #[serde(rename = "lastTrade")]
    pub last_trade: Option<CryptoLastTrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoDay {
    pub c: Option<f64>,
    pub h: Option<f64>,
    pub l: Option<f64>,
    pub o: Option<f64>,
    pub v: Option<f64>,
    pub vw: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoMin {
    pub av: Option<f64>,
    pub c: Option<f64>,
    pub h: Option<f64>,
    pub l: Option<f64>,
    pub o: Option<f64>,
    pub v: Option<f64>,
    pub vw: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoLastTrade {
    pub p: Option<f64>,
    pub s: Option<f64>,
    pub x: Option<i64>,
    pub t: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoSnapshotResponse {
    pub status: Option<String>,
    #[serde(default = "Vec::new")]
    pub tickers: Vec<CryptoTicker>,
}
