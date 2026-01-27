use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Timespan {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl Timespan {
    pub fn as_str(&self) -> &'static str {
        match self {
            Timespan::Second => "second",
            Timespan::Minute => "minute",
            Timespan::Hour => "hour",
            Timespan::Day => "day",
            Timespan::Week => "week",
            Timespan::Month => "month",
            Timespan::Quarter => "quarter",
            Timespan::Year => "year",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateBar {
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
pub struct AggregatesResponse {
    pub ticker: Option<String>,
    #[serde(rename = "queryCount", default)]
    pub query_count: Option<i64>,
    #[serde(rename = "resultsCount", default)]
    pub results_count: Option<i64>,
    pub adjusted: Option<bool>,
    #[serde(default)]
    pub results: Vec<AggregateBar>,
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastTrade {
    #[serde(rename = "T")]
    pub ticker: Option<String>,
    #[serde(rename = "t")]
    pub sip_timestamp: i64,
    #[serde(rename = "y", default)]
    pub participant_timestamp: Option<i64>,
    #[serde(rename = "f", default)]
    pub trf_timestamp: Option<i64>,
    #[serde(rename = "q", default)]
    pub sequence_number: Option<i64>,
    #[serde(rename = "i")]
    pub trade_id: Option<String>,
    #[serde(rename = "x")]
    pub exchange: i32,
    #[serde(rename = "s")]
    pub size: i64,
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "c", default)]
    pub conditions: Vec<i32>,
    #[serde(rename = "z", default)]
    pub tape: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastTradeResponse {
    pub results: LastTrade,
    pub status: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastQuote {
    #[serde(rename = "P")]
    pub ask_price: f64,
    #[serde(rename = "S")]
    pub ask_size: i64,
    #[serde(rename = "p")]
    pub bid_price: f64,
    #[serde(rename = "s")]
    pub bid_size: i64,
    #[serde(rename = "t")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotAgg {
    #[serde(rename = "o", default)]
    pub open: Option<f64>,
    #[serde(rename = "h", default)]
    pub high: Option<f64>,
    #[serde(rename = "l", default)]
    pub low: Option<f64>,
    #[serde(rename = "c", default)]
    pub close: Option<f64>,
    #[serde(rename = "v", default)]
    pub volume: Option<f64>,
    #[serde(rename = "vw", default)]
    pub vwap: Option<f64>,
    #[serde(rename = "t", default)]
    pub timestamp: Option<i64>,
    #[serde(rename = "n", default)]
    pub transactions: Option<i64>,
    #[serde(rename = "av", default)]
    pub accumulated_volume: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotTrade {
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "s")]
    pub size: i64,
    #[serde(rename = "x")]
    pub exchange: i32,
    #[serde(rename = "t")]
    pub timestamp: i64,
    #[serde(rename = "c", default)]
    pub conditions: Vec<i32>,
    #[serde(rename = "i", default)]
    pub trade_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerSnapshot {
    pub ticker: String,
    #[serde(rename = "todaysChange", default)]
    pub todays_change: Option<f64>,
    #[serde(rename = "todaysChangePerc", default)]
    pub todays_change_perc: Option<f64>,
    pub updated: Option<i64>,
    pub day: Option<SnapshotAgg>,
    #[serde(rename = "prevDay")]
    pub prev_day: Option<SnapshotAgg>,
    pub min: Option<SnapshotAgg>,
    #[serde(rename = "lastTrade")]
    pub last_trade: Option<SnapshotTrade>,
    #[serde(rename = "lastQuote")]
    pub last_quote: Option<LastQuote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotResponse {
    pub ticker: TickerSnapshot,
    pub status: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorValue {
    pub timestamp: i64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorResults {
    #[serde(default)]
    pub values: Vec<IndicatorValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorUnderlying {
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmaResponse {
    pub results: Option<IndicatorResults>,
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub next_url: Option<String>,
}
