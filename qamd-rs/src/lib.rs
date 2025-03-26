//! # QAMD-RS
//! 
//! Market data protocol library for QUANTAXIS systems.
//! Provides standardized types for market data handling and exchange.

pub mod error;
pub mod snapshot;
pub mod tick;
pub mod constants;
pub mod types;
pub mod daily;
pub mod minute;

pub use snapshot::MDSnapshot;
pub use tick::Tick;
pub use error::QAMDError;
pub use types::*;
pub use daily::{
    DailyMarketData, 
    DailyBar,
    InstrumentType,
};
pub use minute::{
    MinuteMarketData,
    MinuteBar,
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json;

    #[test]
    fn test_create_snapshot() {
        let now = Utc::now();
        let snapshot = MDSnapshot {
            instrument_id: "SSE_688286".to_string(),
            amount: 1000000.0,
            ask_price1: 10.5,
            ask_volume1: 100,
            bid_price1: 10.4,
            bid_volume1: 150,
            last_price: 10.45,
            datetime: now,
            highest: 10.6,
            lowest: 10.3,
            open: 10.35,
            close: OptionalF64::Value(10.5),
            volume: 25000,
            pre_close: 10.3,
            lower_limit: 9.3,
            upper_limit: 11.3,
            average: 10.45,
            // Set optional fields
            ask_price2: Some(10.55),
            ask_volume2: Some(200),
            bid_price2: Some(10.35),
            bid_volume2: Some(250),
            // Other optional fields defaulted
            ask_price3: None,
            ask_price4: None,
            ask_price5: None,
            ask_price6: None,
            ask_price7: None,
            ask_price8: None,
            ask_price9: None,
            ask_price10: None,
            ask_volume3: None,
            ask_volume4: None,
            ask_volume5: None,
            ask_volume6: None,
            ask_volume7: None,
            ask_volume8: None,
            ask_volume9: None,
            ask_volume10: None,
            bid_price3: None,
            bid_price4: None,
            bid_price5: None,
            bid_price6: None,
            bid_price7: None,
            bid_price8: None,
            bid_price9: None,
            bid_price10: None,
            bid_volume3: None,
            bid_volume4: None,
            bid_volume5: None,
            bid_volume6: None,
            bid_volume7: None,
            bid_volume8: None,
            bid_volume9: None,
            bid_volume10: None,
            open_interest: OptionalF64::String("-".to_string()),
            pre_open_interest: OptionalF64::String("-".to_string()),
            pre_settlement: OptionalF64::String("-".to_string()),
            settlement: OptionalF64::String("-".to_string()),
            iopv: OptionalF64::Null,
        };

        assert_eq!(snapshot.instrument_id, "SSE_688286");
        assert_eq!(snapshot.last_price, 10.45);
        let spread = snapshot.bid_ask_spread();
        assert!((spread - 0.1).abs() < f64::EPSILON * 100.0, "Expected spread to be approximately 0.1, got {}", spread);
        assert!(snapshot.has_level2_depth());
        assert!(!snapshot.is_futures_or_options());
        assert!(!snapshot.is_etf());
    }

    #[test]
    fn test_create_tick_from_snapshot() {
        let now = Utc::now();
        let snapshot = MDSnapshot {
            instrument_id: "SZSE_300750".to_string(),
            amount: 500000.0,
            ask_price1: 55.67,
            ask_volume1: 500,
            bid_price1: 55.65,
            bid_volume1: 300,
            last_price: 55.66,
            datetime: now,
            highest: 56.0,
            lowest: 55.2,
            open: 55.5,
            close: OptionalF64::Value(55.66),
            volume: 10000,
            pre_close: 55.4,
            lower_limit: 50.0,
            upper_limit: 61.0,
            average: 55.65,
            // All other fields defaulted to None
            ask_price2: None,
            ask_price3: None,
            ask_price4: None,
            ask_price5: None,
            ask_price6: None,
            ask_price7: None,
            ask_price8: None,
            ask_price9: None,
            ask_price10: None,
            ask_volume2: None,
            ask_volume3: None,
            ask_volume4: None,
            ask_volume5: None,
            ask_volume6: None,
            ask_volume7: None,
            ask_volume8: None,
            ask_volume9: None,
            ask_volume10: None,
            bid_price2: None,
            bid_price3: None,
            bid_price4: None,
            bid_price5: None,
            bid_price6: None,
            bid_price7: None,
            bid_price8: None,
            bid_price9: None,
            bid_price10: None,
            bid_volume2: None,
            bid_volume3: None,
            bid_volume4: None,
            bid_volume5: None,
            bid_volume6: None,
            bid_volume7: None,
            bid_volume8: None,
            bid_volume9: None,
            bid_volume10: None,
            open_interest: OptionalF64::String("-".to_string()),
            pre_open_interest: OptionalF64::String("-".to_string()),
            pre_settlement: OptionalF64::String("-".to_string()),
            settlement: OptionalF64::String("-".to_string()),
            iopv: OptionalF64::Null,
        };

        let tick = Tick::from_snapshot(&snapshot);
        
        assert_eq!(tick.instrument_id, "SZSE_300750");
        assert_eq!(tick.last_price, 55.66);
        assert_eq!(tick.volume, 10000);
        assert_eq!(tick.amount, 500000.0);
        assert_eq!(tick.datetime, now);
    }

    #[test]
    fn test_serialization() {
        let now = Utc::now();
        let snapshot = MDSnapshot {
            instrument_id: "SSE_688286".to_string(),
            amount: 1000000.0,
            ask_price1: 10.5,
            ask_volume1: 100,
            bid_price1: 10.4,
            bid_volume1: 150,
            last_price: 10.45,
            datetime: now,
            highest: 10.6,
            lowest: 10.3,
            open: 10.35,
            close: OptionalF64::Value(10.5),
            volume: 25000,
            pre_close: 10.3,
            lower_limit: 9.3,
            upper_limit: 11.3,
            average: 10.45,
            // Minimal required fields
            ask_price2: None,
            ask_price3: None,
            ask_price4: None,
            ask_price5: None,
            ask_price6: None,
            ask_price7: None,
            ask_price8: None,
            ask_price9: None,
            ask_price10: None,
            ask_volume2: None,
            ask_volume3: None,
            ask_volume4: None,
            ask_volume5: None,
            ask_volume6: None,
            ask_volume7: None,
            ask_volume8: None,
            ask_volume9: None,
            ask_volume10: None,
            bid_price2: None,
            bid_price3: None,
            bid_price4: None,
            bid_price5: None,
            bid_price6: None,
            bid_price7: None,
            bid_price8: None,
            bid_price9: None,
            bid_price10: None,
            bid_volume2: None,
            bid_volume3: None,
            bid_volume4: None,
            bid_volume5: None,
            bid_volume6: None,
            bid_volume7: None,
            bid_volume8: None,
            bid_volume9: None,
            bid_volume10: None,
            open_interest: OptionalF64::String("-".to_string()),
            pre_open_interest: OptionalF64::String("-".to_string()),
            pre_settlement: OptionalF64::String("-".to_string()),
            settlement: OptionalF64::String("-".to_string()),
            iopv: OptionalF64::Null,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: MDSnapshot = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.instrument_id, snapshot.instrument_id);
        assert_eq!(deserialized.last_price, snapshot.last_price);
        assert_eq!(deserialized.bid_ask_spread(), snapshot.bid_ask_spread());
    }
}
