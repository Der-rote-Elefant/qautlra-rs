# qamd-rs

A market data protocol library for the QUANTAXIS project. This library provides standardized structures for interacting with market data sources, with support for snapshots, ticks, daily bars, minute bars, and order book information.

## Features

- Standardized representation of market data snapshots with `MDSnapshot`
- Support for both level 1 (top of book) and level 2 (order book depth) market data
- Simple tick data representation with the `Tick` structure
- Unified daily bar structure (`DailyBar`) for stocks, futures, indices, and ETFs
- Unified minute bar structure (`MinuteBar`) with support for stocks, futures, and indices
- Serialization and deserialization support via Serde
- Support for optional fields with special handling for market data "no data" values

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
qamd-rs = "0.1.0"
```

## Usage Examples

### Working with Market Data Snapshots

```rust
use qamd_rs::MDSnapshot;
use chrono::Utc;

// Create a market data snapshot
let snapshot = MDSnapshot {
    instrument_id: "SSE_688286".to_string(),
    amount: 1000000.0,
    ask_price1: 10.5,
    ask_volume1: 100,
    bid_price1: 10.4,
    bid_volume1: 150,
    last_price: 10.45,
    datetime: Utc::now(),
    highest: 10.6,
    lowest: 10.3,
    open: 10.35,
    close: OptionalF64::Value(10.5),
    volume: 25000,
    pre_close: 10.3,
    lower_limit: 9.3,
    upper_limit: 11.3,
    average: 10.45,
    // Set optional fields as needed
    ask_price2: Some(10.55),
    ask_volume2: Some(200),
    bid_price2: Some(10.35),
    bid_volume2: Some(250),
    // ... other fields...
};

// Check order book depth
if snapshot.has_level2_depth() {
    println!("This snapshot contains L2 market data");
}

// Calculate the bid-ask spread
let spread = snapshot.bid_ask_spread();
println!("Current bid-ask spread: {}", spread);

// Serialize to JSON
let json = serde_json::to_string_pretty(&snapshot).unwrap();
println!("{}", json);
```

### Working with Tick Data

```rust
use qamd_rs::Tick;
use chrono::Utc;

// Create a new tick directly
let tick = Tick {
    instrument_id: "SZSE_300750".to_string(),
    last_price: 55.67,
    volume: 1000,
    amount: 55670.0,
    datetime: Utc::now(),
};

// Or extract a tick from a market data snapshot
let tick_from_snapshot = Tick::from_snapshot(&snapshot);
```

### Working with Daily Market Data

The library provides a unified `DailyBar` type with factory methods for different instrument types:

```rust
use qamd_rs::{DailyBar, DailyMarketData, InstrumentType};
use chrono::NaiveDate;

let date = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();

// Create a stock daily bar
let stock = DailyBar::new_stock(
    date,
    "000001.XSHG".to_string(),  // Shanghai Index
    3150.85,  // open
    3160.23,  // high
    3130.56,  // low
    3155.22,  // close
    20573000000.0,  // volume
    240852000000.0,  // turnover
    1500000.0,  // num_trades
    3300.0,  // limit_up
    3000.0,  // limit_down
);

// Create a futures daily bar
let future = DailyBar::new_future(
    date,
    "IF2301.CFFEX".to_string(),  // CSI 300 Index Futures
    3950.0,  // open
    4010.0,  // high
    3940.0,  // low
    3980.0,  // close
    500000.0,  // volume
    20000000000.0,  // turnover
    4100.0,  // limit_up
    3800.0,  // limit_down
    25000.0,  // open_interest
    3960.0,  // prev_settlement
    3980.0,  // settlement
);

// Access common data via trait interface
println!("Stock close: {}", stock.get_close());

// Access instrument-specific data via methods
if let Some(oi) = future.open_interest() {
    println!("Future open interest: {}", oi);
}

// Check instrument type
if stock.is_stock() {
    println!("This is a stock instrument");
}

// Handle different instruments with a common interface
let instruments: Vec<&dyn DailyMarketData> = vec![&stock, &future];
for instr in &instruments {
    println!("{}: OHLC = {}/{}/{}/{}", 
        instr.get_instrument_id(),
        instr.get_open(),
        instr.get_high(),
        instr.get_low(),
        instr.get_close()
    );
}
```

### Working with Minute Market Data

The library provides a unified `MinuteBar` type with factory methods for different instrument types:

```rust
use qamd_rs::{MinuteBar, MinuteMarketData};
use chrono::{TimeZone, Utc, NaiveDate};

// Create a stock minute bar
let stock_time = Utc.with_ymd_and_hms(2023, 1, 10, 9, 31, 0).unwrap();
let stock = MinuteBar::new_stock(
    stock_time,
    "000001.XSHG".to_string(),  // Shanghai Index
    3155.85,  // open
    3157.23,  // high
    3155.56,  // low
    3156.72,  // close
    2057300.0,  // volume
    24085200.0,  // turnover
);

// Create a futures minute bar with a trading date (useful for night sessions)
let future_time = Utc.with_ymd_and_hms(2023, 1, 10, 21, 1, 0).unwrap();
let trading_date = NaiveDate::from_ymd_opt(2023, 1, 11).unwrap();
let future = MinuteBar::new_future(
    future_time,
    trading_date,
    "IF2301.CFFEX".to_string(),
    3980.0,
    3981.2,
    3979.8,
    3980.6,
    50.0,
    2000000.0,
    25010.0,  // open interest
);

// Access common data via trait interface
println!("Stock close: {}", stock.get_close());

// Access instrument-specific data
println!("Future timestamp: {}", future.get_datetime());
println!("Future trading date: {}", future.get_trading_date().unwrap());

// Calculate analytics on bars
println!("Price range: {}", stock.range());
println!("Percentage change: {}%", stock.percent_change());

// Handle different instruments with a common interface
let bars: Vec<&dyn MinuteMarketData> = vec![&stock, &future];
for bar in &bars {
    println!("{}: {} | OHLC = {}/{}/{}/{}", 
        bar.get_instrument_id(),
        bar.get_datetime(),
        bar.get_open(),
        bar.get_high(),
        bar.get_low(),
        bar.get_close()
    );
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 