use qamd_rs::{MinuteMarketData, MinuteBar, InstrumentType};
use chrono::{TimeZone, Utc, NaiveDate};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("QAMD-RS Minute Market Data Example");
    println!("==================================");
    
    // Create sample market data for different instrument types
    
    // Create a stock minute bar
    let stock_time = Utc.with_ymd_and_hms(2023, 1, 10, 9, 31, 0).unwrap();
    let stock = MinuteBar::new_stock(
        stock_time,
        "000001.XSHG".to_string(), // Shanghai Index
        3155.85,
        3157.23,
        3155.56,
        3156.72,
        2057300.0,
        24085200.0,
    );
    
    // Create a futures minute bar with next day's trading date (night session)
    let future_time = Utc.with_ymd_and_hms(2023, 1, 10, 21, 1, 0).unwrap();
    let trading_date = NaiveDate::from_ymd_opt(2023, 1, 11).unwrap();
    let future = MinuteBar::new_future(
        future_time,
        trading_date,
        "IF2301.CFFEX".to_string(), // CSI 300 Index Futures
        3980.0,
        3981.2,
        3979.8,
        3980.6,
        50.0,
        2000000.0,
        25010.0,
    );
    
    // Create an index minute bar
    let index_time = Utc.with_ymd_and_hms(2023, 1, 10, 9, 31, 0).unwrap();
    let index = MinuteBar::new_index(
        index_time,
        "000300.XSHG".to_string(), // CSI 300 Index
        4020.0,
        4021.5,
        4019.8,
        4020.8,
        2000000.0,
        5000000.0,
    );
    
    // Display information using the common trait interface
    println!("\nStock Minute Bar ({})", stock.get_instrument_id());
    print_ohlc(&stock);
    
    println!("\nFutures Minute Bar ({})", future.get_instrument_id());
    print_ohlc(&future);
    println!("Trading Date: {}", future.get_trading_date().unwrap());
    println!("Open Interest: {}", future.open_interest().unwrap_or(0.0));
    
    println!("\nIndex Minute Bar ({})", index.get_instrument_id());
    print_ohlc(&index);
    
    // Working with a collection of different types using trait objects
    println!("\nWorking with trait objects:");
    let market_data: Vec<&dyn MinuteMarketData> = vec![&stock, &future, &index];
    
    for (i, data) in market_data.iter().enumerate() {
        println!("Bar {}: {} | {} | Close: {:.2}", 
            i + 1, 
            data.get_instrument_id(),
            data.get_datetime().format("%Y-%m-%d %H:%M:%S"),
            data.get_close()
        );
    }
    
    // Demonstrate analytics
    println!("\nAnalytics Examples:");
    println!("Stock Price Range: {:.2}", stock.range());
    println!("Stock Return: {:.4}%", stock.percent_change());
    println!("Futures Return: {:.4}%", future.percent_change());
    println!("Index Return: {:.4}%", index.percent_change());
    
    // Convert to JSON
    println!("\nJSON Example:");
    println!("{}", serde_json::to_string_pretty(&stock)?);
    
    Ok(())
}

// Helper function to print OHLC data using the trait interface
fn print_ohlc(data: &impl MinuteMarketData) {
    println!("Timestamp: {}", data.get_datetime().format("%Y-%m-%d %H:%M:%S"));
    println!("OHLC: {:.2} | {:.2} | {:.2} | {:.2}",
        data.get_open(),
        data.get_high(),
        data.get_low(),
        data.get_close()
    );
    println!("Volume: {:.2}", data.get_volume());
    println!("Turnover: {:.2}", data.get_total_turnover());
} 