/// Market data source identifiers
pub mod source {
    /// Shanghai Stock Exchange
    pub const SSE: &str = "SSE";
    /// Shenzhen Stock Exchange
    pub const SZSE: &str = "SZSE";
    /// China Financial Futures Exchange
    pub const CFFEX: &str = "CFFEX";
    /// Shanghai Futures Exchange
    pub const SHFE: &str = "SHFE";
    /// Dalian Commodity Exchange
    pub const DCE: &str = "DCE";
    /// Zhengzhou Commodity Exchange
    pub const CZCE: &str = "CZCE";
    /// Hong Kong Stock Exchange
    pub const HKEX: &str = "HKEX";
    /// Interactive Brokers
    pub const IB: &str = "IB";
}

/// Market data field identifiers
pub mod fields {
    /// Instrument ID field
    pub const INSTRUMENT_ID: &str = "instrument_id";
    /// Last price field
    pub const LAST_PRICE: &str = "last_price";
    /// Trading volume field
    pub const VOLUME: &str = "volume";
    /// Trading amount field
    pub const AMOUNT: &str = "amount";
    /// Datetime field
    pub const DATETIME: &str = "datetime";
}

/// Default values
pub mod defaults {
    /// Represents a missing value in text format
    pub const MISSING_VALUE: &str = "-";
} 