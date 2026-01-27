use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub address1: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branding {
    pub logo_url: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerDetails {
    pub ticker: String,
    pub name: Option<String>,
    pub market: Option<String>,
    pub locale: Option<String>,
    #[serde(rename = "type")]
    pub ticker_type: Option<String>,
    pub active: Option<bool>,
    pub currency_name: Option<String>,
    pub cik: Option<String>,
    pub composite_figi: Option<String>,
    pub share_class_figi: Option<String>,
    pub description: Option<String>,
    pub homepage_url: Option<String>,
    pub list_date: Option<String>,
    pub total_employees: Option<i64>,
    pub primary_exchange: Option<String>,
    pub sic_code: Option<String>,
    pub sic_description: Option<String>,
    pub market_cap: Option<f64>,
    pub address: Option<Address>,
    pub branding: Option<Branding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerDetailsResponse {
    pub results: TickerDetails,
    pub status: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerSearchResult {
    pub ticker: String,
    pub name: Option<String>,
    pub market: Option<String>,
    pub locale: Option<String>,
    #[serde(rename = "type")]
    pub ticker_type: Option<String>,
    pub active: Option<bool>,
    pub currency_name: Option<String>,
    pub primary_exchange: Option<String>,
    pub last_updated_utc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerSearchResponse {
    #[serde(default)]
    pub results: Vec<TickerSearchResult>,
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub count: Option<i64>,
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyStatus {
    pub crypto: Option<String>,
    pub fx: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeStatus {
    pub nasdaq: Option<String>,
    pub nyse: Option<String>,
    pub otc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketStatus {
    pub market: Option<String>,
    pub server_time: Option<String>,
    pub exchanges: Option<ExchangeStatus>,
    pub currencies: Option<CurrencyStatus>,
    pub early_hours: Option<bool>,
    pub after_hours: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketHoliday {
    pub date: Option<String>,
    pub exchange: Option<String>,
    pub name: Option<String>,
    pub status: Option<String>,
    pub open: Option<String>,
    pub close: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dividend {
    pub ticker: Option<String>,
    pub ex_dividend_date: Option<String>,
    pub record_date: Option<String>,
    pub pay_date: Option<String>,
    pub declaration_date: Option<String>,
    pub cash_amount: Option<f64>,
    pub currency: Option<String>,
    pub frequency: Option<i32>,
    pub dividend_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendsResponse {
    #[serde(default)]
    pub results: Vec<Dividend>,
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    pub name: Option<String>,
    pub homepage_url: Option<String>,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub id: Option<String>,
    pub publisher: Option<Publisher>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub published_utc: Option<String>,
    pub article_url: Option<String>,
    #[serde(default)]
    pub tickers: Vec<String>,
    pub image_url: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsResponse {
    #[serde(default)]
    pub results: Vec<NewsArticle>,
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub count: Option<i64>,
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSplit {
    pub ticker: Option<String>,
    pub execution_date: Option<String>,
    pub split_from: Option<f64>,
    pub split_to: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitsResponse {
    #[serde(default)]
    pub results: Vec<StockSplit>,
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub next_url: Option<String>,
}
