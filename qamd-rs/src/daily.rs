use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

/// Instrument type enumeration to categorize market data
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstrumentType {
    /// Stock/Equity
    Stock,
    /// Future contract
    Future,
    /// Market index
    Index,
    /// Listed Open-ended Fund (ETF, LOF)
    Fund,
    /// Other instrument type
    Other,
}

/// Unified daily market data structure that works for all instrument types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyBar {
    /// Trading date
    pub date: NaiveDate,
    
    /// Instrument identifier (e.g. "000001.XSHE")
    pub order_book_id: String,
    
    /// Type of financial instrument 
    pub instrument_type: InstrumentType,
    
    /// Opening price
    pub open: f32,
    
    /// Highest price
    pub high: f32,
    
    /// Lowest price
    pub low: f32,
    
    /// Closing price
    pub close: f32,
    
    /// Trading volume
    pub volume: f32,
    
    /// Total turnover value (amount)
    pub total_turnover: f32,
    
    /// Number of trades (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_trades: Option<f32>,
    
    /// Upper limit price (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_up: Option<f32>,
    
    /// Lower limit price (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_down: Option<f32>,
    
    /// Open interest for futures/options (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<f32>,
    
    /// Previous settlement price for futures (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_settlement: Option<f32>,
    
    /// Settlement price for futures (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement: Option<f32>,
    
    /// Indicative Optimized Portfolio Value for ETFs (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iopv: Option<f32>,
}

/// Common trait for daily market data access
pub trait DailyMarketData {
    /// Get instrument identifier
    fn get_instrument_id(&self) -> &str;
    
    /// Get trading date
    fn get_date(&self) -> NaiveDate;
    
    /// Get open price
    fn get_open(&self) -> f32;
    
    /// Get high price
    fn get_high(&self) -> f32;
    
    /// Get low price
    fn get_low(&self) -> f32;
    
    /// Get close price
    fn get_close(&self) -> f32;
    
    /// Get trading volume
    fn get_volume(&self) -> f32;
    
    /// Get total turnover value
    fn get_total_turnover(&self) -> f32;
}

impl DailyMarketData for DailyBar {
    fn get_instrument_id(&self) -> &str {
        &self.order_book_id
    }
    
    fn get_date(&self) -> NaiveDate {
        self.date
    }
    
    fn get_open(&self) -> f32 {
        self.open
    }
    
    fn get_high(&self) -> f32 {
        self.high
    }
    
    fn get_low(&self) -> f32 {
        self.low
    }
    
    fn get_close(&self) -> f32 {
        self.close
    }
    
    fn get_volume(&self) -> f32 {
        self.volume
    }
    
    fn get_total_turnover(&self) -> f32 {
        self.total_turnover
    }
}

impl DailyBar {
    /// Create a new daily bar with the required fields
    pub fn new(
        date: NaiveDate,
        order_book_id: String,
        instrument_type: InstrumentType,
        open: f32,
        high: f32,
        low: f32,
        close: f32,
        volume: f32,
        total_turnover: f32,
    ) -> Self {
        Self {
            date,
            order_book_id,
            instrument_type,
            open,
            high,
            low,
            close,
            volume,
            total_turnover,
            num_trades: None,
            limit_up: None,
            limit_down: None,
            open_interest: None,
            prev_settlement: None,
            settlement: None,
            iopv: None,
        }
    }

    /// Create a stock daily record
    pub fn new_stock(
        date: NaiveDate,
        order_book_id: String,
        open: f32,
        high: f32,
        low: f32,
        close: f32,
        volume: f32,
        total_turnover: f32,
        num_trades: f32,
        limit_up: f32,
        limit_down: f32,
    ) -> Self {
        let mut bar = Self::new(
            date,
            order_book_id,
            InstrumentType::Stock,
            open,
            high,
            low,
            close,
            volume,
            total_turnover,
        );
        
        bar.num_trades = Some(num_trades);
        bar.limit_up = Some(limit_up);
        bar.limit_down = Some(limit_down);
        
        bar
    }
    
    /// Create a futures daily record
    pub fn new_future(
        date: NaiveDate,
        order_book_id: String,
        open: f32,
        high: f32,
        low: f32,
        close: f32,
        volume: f32,
        total_turnover: f32,
        limit_up: f32,
        limit_down: f32,
        open_interest: f32,
        prev_settlement: f32,
        settlement: f32,
    ) -> Self {
        let mut bar = Self::new(
            date,
            order_book_id,
            InstrumentType::Future,
            open,
            high,
            low,
            close,
            volume,
            total_turnover,
        );
        
        bar.limit_up = Some(limit_up);
        bar.limit_down = Some(limit_down);
        bar.open_interest = Some(open_interest);
        bar.prev_settlement = Some(prev_settlement);
        bar.settlement = Some(settlement);
        
        bar
    }
    
    /// Create an index daily record
    pub fn new_index(
        date: NaiveDate,
        order_book_id: String,
        open: f32,
        high: f32,
        low: f32,
        close: f32,
        volume: f32,
        total_turnover: f32,
        num_trades: f32,
    ) -> Self {
        let mut bar = Self::new(
            date,
            order_book_id,
            InstrumentType::Index,
            open,
            high,
            low,
            close,
            volume,
            total_turnover,
        );
        
        bar.num_trades = Some(num_trades);
        
        bar
    }
    
    /// Create a fund (ETF/LOF) daily record
    pub fn new_fund(
        date: NaiveDate,
        order_book_id: String,
        open: f32,
        high: f32,
        low: f32,
        close: f32,
        volume: f32,
        total_turnover: f32,
        limit_up: f32,
        limit_down: f32,
        iopv: f32,
        num_trades: f32,
    ) -> Self {
        let mut bar = Self::new(
            date,
            order_book_id,
            InstrumentType::Fund,
            open,
            high,
            low,
            close,
            volume,
            total_turnover,
        );
        
        bar.limit_up = Some(limit_up);
        bar.limit_down = Some(limit_down);
        bar.iopv = Some(iopv);
        bar.num_trades = Some(num_trades);
        
        bar
    }
    
    /// Check if this is a stock
    pub fn is_stock(&self) -> bool {
        self.instrument_type == InstrumentType::Stock
    }
    
    /// Check if this is a future
    pub fn is_future(&self) -> bool {
        self.instrument_type == InstrumentType::Future
    }
    
    /// Check if this is an index
    pub fn is_index(&self) -> bool {
        self.instrument_type == InstrumentType::Index
    }
    
    /// Check if this is a fund (ETF/LOF)
    pub fn is_fund(&self) -> bool {
        self.instrument_type == InstrumentType::Fund
    }
    
    /// Get the number of trades if available
    pub fn num_trades(&self) -> Option<f32> {
        self.num_trades
    }
    
    /// Get price limits if available (returns a tuple of (lower, upper))
    pub fn price_limits(&self) -> Option<(f32, f32)> {
        match (self.limit_down, self.limit_up) {
            (Some(down), Some(up)) => Some((down, up)),
            _ => None
        }
    }
    
    /// Get open interest if available (mainly for futures)
    pub fn open_interest(&self) -> Option<f32> {
        self.open_interest
    }
    
    /// Get settlement price if available (mainly for futures)
    pub fn settlement(&self) -> Option<f32> {
        self.settlement
    }
    
    /// Get previous settlement price if available (mainly for futures)
    pub fn prev_settlement(&self) -> Option<f32> {
        self.prev_settlement
    }
    
    /// Get IOPV if available (mainly for ETFs)
    pub fn iopv(&self) -> Option<f32> {
        self.iopv
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use serde_json;

    #[test]
    fn test_stock_daily() {
        let date = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
        let stock = DailyBar::new_stock(
            date,
            "000001.XSHG".to_string(),
            10.5,
            11.2,
            10.3,
            10.9,
            1000000.0,
            11000000.0,
            5000.0,
            11.5,
            9.5,
        );

        assert_eq!(stock.get_instrument_id(), "000001.XSHG");
        assert_eq!(stock.get_date(), date);
        assert_eq!(stock.get_open(), 10.5);
        assert_eq!(stock.get_high(), 11.2);
        assert_eq!(stock.get_low(), 10.3);
        assert_eq!(stock.get_close(), 10.9);
        assert_eq!(stock.get_volume(), 1000000.0);
        assert_eq!(stock.get_total_turnover(), 11000000.0);
        assert_eq!(stock.num_trades(), Some(5000.0));
        assert_eq!(stock.price_limits(), Some((9.5, 11.5)));
        assert!(stock.is_stock());
        assert!(!stock.is_future());

        // Test JSON serialization/deserialization
        let json = serde_json::to_string(&stock).unwrap();
        let deserialized: DailyBar = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.get_instrument_id(), stock.get_instrument_id());
        assert_eq!(deserialized.get_close(), stock.get_close());
        assert_eq!(deserialized.num_trades(), stock.num_trades());
    }

    #[test]
    fn test_future_daily() {
        let date = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
        let future = DailyBar::new_future(
            date,
            "IF2301.CFFEX".to_string(),
            3950.0,
            4010.0,
            3940.0,
            3980.0,
            500000.0,
            20000000000.0,
            4100.0,
            3800.0,
            25000.0,
            3960.0,
            3980.0,
        );

        assert_eq!(future.get_instrument_id(), "IF2301.CFFEX");
        assert_eq!(future.get_date(), date);
        assert_eq!(future.get_close(), 3980.0);
        assert_eq!(future.open_interest(), Some(25000.0));
        assert_eq!(future.prev_settlement(), Some(3960.0));
        assert_eq!(future.settlement(), Some(3980.0));
        assert!(future.is_future());
    }

    #[test]
    fn test_index_daily() {
        let date = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
        let index = DailyBar::new_index(
            date,
            "000300.XSHG".to_string(), // CSI 300
            4000.0,
            4050.0,
            3980.0,
            4020.0,
            2000000000.0,
            50000000000.0,
            10000.0,
        );

        assert_eq!(index.get_instrument_id(), "000300.XSHG");
        assert_eq!(index.get_date(), date);
        assert_eq!(index.get_close(), 4020.0);
        assert_eq!(index.num_trades(), Some(10000.0));
        assert!(index.is_index());
    }

    #[test]
    fn test_lof_daily() {
        let date = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
        let lof = DailyBar::new_fund(
            date,
            "510050.XSHG".to_string(), // SSE 50 ETF
            3.5,
            3.6,
            3.48,
            3.55,
            500000000.0,
            1750000000.0,
            3.7,
            3.3,
            3.56,
            30000.0,
        );

        assert_eq!(lof.get_instrument_id(), "510050.XSHG");
        assert_eq!(lof.get_date(), date);
        assert_eq!(lof.get_close(), 3.55);
        assert_eq!(lof.iopv(), Some(3.56));
        assert_eq!(lof.num_trades(), Some(30000.0));
        assert!(lof.is_fund());
    }
} 