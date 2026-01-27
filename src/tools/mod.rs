pub mod crypto;
pub mod forex;
pub mod indices;
pub mod options;
pub mod reference;
pub mod stocks;

pub use crypto::{GetCryptoAggregates, GetCryptoSnapshot, GetCryptoTrades};
pub use forex::{ConvertCurrency, GetForexAggregates, GetForexSnapshot};
pub use indices::{GetIndexAggregates, GetIndexOpenClose, GetIndexSnapshot};
pub use options::{GetOptionsAggregates, GetOptionsContracts, GetOptionsSnapshot};
pub use reference::{
    GetDividends, GetMarketHolidays, GetMarketStatus, GetNews, GetStockSplits, GetTickerDetails,
    SearchTickers,
};
pub use stocks::{
    GetEma, GetLastTrade, GetMacd, GetRsi, GetSma, GetStockAggregates, GetStockSnapshot,
};
