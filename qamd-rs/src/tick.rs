use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::snapshot::MDSnapshot;

/// Basic tick data representing a single price update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tick {
    /// Unique identifier for the instrument (e.g., "SSE_688286")
    pub instrument_id: String,
    
    /// Last traded price
    pub last_price: f64,
    
    /// Total trading volume
    pub volume: i64,
    
    /// Total turnover value
    pub amount: f64,
    
    /// Timestamp of the tick
    pub datetime: DateTime<Utc>,
}

impl Tick {
    /// Create a new tick with the given values
    pub fn new(
        instrument_id: String, 
        last_price: f64, 
        volume: i64, 
        amount: f64, 
        datetime: DateTime<Utc>
    ) -> Self {
        Self {
            instrument_id,
            last_price,
            volume,
            amount,
            datetime,
        }
    }
    
    /// Extract a tick from a market data snapshot
    pub fn from_snapshot(snapshot: &MDSnapshot) -> Self {
        Self {
            instrument_id: snapshot.instrument_id.clone(),
            last_price: snapshot.last_price,
            volume: snapshot.volume,
            amount: snapshot.amount,
            datetime: snapshot.datetime,
        }
    }
} 