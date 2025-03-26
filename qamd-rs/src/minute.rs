use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};
use crate::daily::InstrumentType;

/// Unified minute-level market data structure for all instrument types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MinuteBar {
    /// Timestamp of the minute bar
    pub datetime: DateTime<Utc>,
    
    /// Trading date (may differ from datetime date for night sessions)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trading_date: Option<NaiveDate>,
    
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
    
    /// Open interest for futures/options (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<f32>,
}

/// Common trait for minute market data access
pub trait MinuteMarketData {
    /// Get instrument identifier
    fn get_instrument_id(&self) -> &str;
    
    /// Get timestamp
    fn get_datetime(&self) -> DateTime<Utc>;
    
    /// Get trading date if available
    fn get_trading_date(&self) -> Option<NaiveDate>;
    
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

impl MinuteMarketData for MinuteBar {
    fn get_instrument_id(&self) -> &str {
        &self.order_book_id
    }
    
    fn get_datetime(&self) -> DateTime<Utc> {
        self.datetime
    }
    
    fn get_trading_date(&self) -> Option<NaiveDate> {
        self.trading_date
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

impl MinuteBar {
    /// Create a new minute bar with the required fields
    pub fn new(
        datetime: DateTime<Utc>,
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
            datetime,
            trading_date: None,
            order_book_id,
            instrument_type,
            open,
            high,
            low,
            close,
            volume,
            total_turnover,
            open_interest: None,
        }
    }

    /// Create a stock minute bar
    pub fn new_stock(
        datetime: DateTime<Utc>,
        order_book_id: String,
        open: f32,
        high: f32,
        low: f32,
        close: f32,
        volume: f32,
        total_turnover: f32,
    ) -> Self {
        Self::new(
            datetime,
            order_book_id,
            InstrumentType::Stock,
            open,
            high,
            low,
            close,
            volume,
            total_turnover,
        )
    }
    
    /// Create a futures minute bar
    pub fn new_future(
        datetime: DateTime<Utc>,
        trading_date: NaiveDate,
        order_book_id: String,
        open: f32,
        high: f32,
        low: f32,
        close: f32,
        volume: f32,
        total_turnover: f32,
        open_interest: f32,
    ) -> Self {
        let mut bar = Self::new(
            datetime,
            order_book_id,
            InstrumentType::Future,
            open,
            high,
            low,
            close,
            volume,
            total_turnover,
        );
        
        bar.trading_date = Some(trading_date);
        bar.open_interest = Some(open_interest);
        
        bar
    }
    
    /// Create an index minute bar
    pub fn new_index(
        datetime: DateTime<Utc>,
        order_book_id: String,
        open: f32,
        high: f32,
        low: f32,
        close: f32,
        volume: f32,
        total_turnover: f32,
    ) -> Self {
        Self::new(
            datetime,
            order_book_id,
            InstrumentType::Index,
            open,
            high,
            low,
            close,
            volume,
            total_turnover,
        )
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
    
    /// Get open interest if available (mainly for futures)
    pub fn open_interest(&self) -> Option<f32> {
        self.open_interest
    }
    
    /// Calculate the range (high - low)
    pub fn range(&self) -> f32 {
        self.high - self.low
    }
    
    /// Calculate the return for this minute bar
    pub fn returns(&self) -> f32 {
        if self.open == 0.0 {
            0.0
        } else {
            (self.close - self.open) / self.open
        }
    }
    
    /// Calculate the percentage change for this minute bar
    pub fn percent_change(&self) -> f32 {
        self.returns() * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json;

    #[test]
    fn test_stock_minute() {
        let datetime = Utc.with_ymd_and_hms(2023, 1, 10, 9, 30, 0).unwrap();
        let stock = MinuteBar::new_stock(
            datetime,
            "000001.XSHG".to_string(),
            3150.85,
            3155.23,
            3150.56,
            3153.22,
            20500000.0,
            240000000.0,
        );

        assert_eq!(stock.get_instrument_id(), "000001.XSHG");
        assert_eq!(stock.get_datetime(), datetime);
        assert_eq!(stock.get_open(), 3150.85);
        assert_eq!(stock.get_high(), 3155.23);
        assert_eq!(stock.get_low(), 3150.56);
        assert_eq!(stock.get_close(), 3153.22);
        assert_eq!(stock.get_volume(), 20500000.0);
        assert_eq!(stock.get_total_turnover(), 240000000.0);
        assert_eq!(stock.get_trading_date(), None);
        assert!(stock.is_stock());
        assert!(!stock.is_future());

        // Test JSON serialization/deserialization
        let json = serde_json::to_string(&stock).unwrap();
        let deserialized: MinuteBar = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.get_instrument_id(), stock.get_instrument_id());
        assert_eq!(deserialized.get_close(), stock.get_close());
    }

    #[test]
    fn test_future_minute() {
        let datetime = Utc.with_ymd_and_hms(2023, 1, 10, 21, 0, 0).unwrap();
        let trading_date = NaiveDate::from_ymd_opt(2023, 1, 11).unwrap(); // Next day's trading date for night session
        let future = MinuteBar::new_future(
            datetime,
            trading_date,
            "IF2301.CFFEX".to_string(),
            3950.0,
            3953.0,
            3948.0,
            3952.0,
            200.0,
            8000000.0,
            25000.0,
        );

        assert_eq!(future.get_instrument_id(), "IF2301.CFFEX");
        assert_eq!(future.get_datetime(), datetime);
        assert_eq!(future.get_trading_date(), Some(trading_date));
        assert_eq!(future.get_close(), 3952.0);
        assert_eq!(future.open_interest(), Some(25000.0));
        assert!(future.is_future());
        
        // Test range calculation
        assert_eq!(future.range(), 5.0);
        
        // Test returns calculation
        assert!(future.returns() > 0.0);
    }

    #[test]
    fn test_index_minute() {
        let datetime = Utc.with_ymd_and_hms(2023, 1, 10, 9, 30, 0).unwrap();
        let index = MinuteBar::new_index(
            datetime,
            "000300.XSHG".to_string(), // CSI 300
            4000.0,
            4005.0,
            3995.0,
            4003.0,
            2000000.0,
            50000000.0,
        );

        assert_eq!(index.get_instrument_id(), "000300.XSHG");
        assert_eq!(index.get_datetime(), datetime);
        assert_eq!(index.get_close(), 4003.0);
        assert!(index.is_index());
        
        // Test percent change
        let expected_percent = (4003.0 - 4000.0) / 4000.0 * 100.0;
        assert!((index.percent_change() - expected_percent).abs() < f32::EPSILON);
    }
} 