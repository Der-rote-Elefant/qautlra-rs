use qamd_rs::{DailyMarketData, DailyBar, InstrumentType};
use chrono::NaiveDate;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("QAMD-RS Daily Market Data Example");
    println!("=================================");
    
    // Create sample market data for different instrument types
    let date = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
    
    // Create a stock daily record
    let stock = DailyBar::new_stock(
        date,
        "000001.XSHG".to_string(), // Shanghai Index
        3150.85,
        3160.23,
        3130.56,
        3155.22,
        20573000000.0,
        240852000000.0,
        1500000.0,
        3300.0,
        3000.0,
    );
    
    // Create a futures daily record
    let future = DailyBar::new_future(
        date,
        "IF2301.CFFEX".to_string(), // CSI 300 Index Futures
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
    
    // Create an index daily record
    let index = DailyBar::new_index(
        date,
        "000300.XSHG".to_string(), // CSI 300 Index
        4000.0,
        4050.0,
        3980.0,
        4020.0,
        2000000000.0,
        50000000000.0,
        10000.0,
    );
    
    // Create a LOF/ETF daily record
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
    
    // Display information using the common trait interface
    println!("\nStock Data ({})", stock.get_instrument_id());
    print_ohlc(&stock);
    println!("Number of Trades: {}", stock.num_trades().unwrap_or(0.0));
    
    if let Some((down, up)) = stock.price_limits() {
        println!("Price Limits: {} - {}", down, up);
    }
    
    println!("\nFutures Data ({})", future.get_instrument_id());
    print_ohlc(&future);
    println!("Open Interest: {}", future.open_interest().unwrap_or(0.0));
    println!("Settlement: {} (Previous: {})", 
        future.settlement().unwrap_or(0.0), 
        future.prev_settlement().unwrap_or(0.0)
    );
    
    println!("\nIndex Data ({})", index.get_instrument_id());
    print_ohlc(&index);
    println!("Number of Trades: {}", index.num_trades().unwrap_or(0.0));
    
    println!("\nLOF/ETF Data ({})", lof.get_instrument_id());
    print_ohlc(&lof);
    println!("IOPV: {}", lof.iopv().unwrap_or(0.0));
    println!("Number of Trades: {}", lof.num_trades().unwrap_or(0.0));
    
    // Working with a collection of different types using trait objects
    println!("\nWorking with trait objects:");
    let market_data: Vec<&dyn DailyMarketData> = vec![&stock, &future, &index, &lof];
    
    for (i, data) in market_data.iter().enumerate() {
        println!("Instrument {}: {} | Close: {}", 
            i + 1, 
            data.get_instrument_id(), 
            data.get_close()
        );
    }
    
    // Demonstrate type checking
    println!("\nType checking:");
    print_instrument_type(&stock);
    print_instrument_type(&future);
    print_instrument_type(&index);
    print_instrument_type(&lof);
    
    // Create a custom instrument data directly
    let custom = DailyBar::new(
        date,
        "CUSTOM_INSTRUMENT".to_string(),
        InstrumentType::Other,
        100.0,
        105.0,
        98.0,
        103.0,
        1000.0,
        100000.0
    );
    
    println!("\nCustom Instrument:");
    print_ohlc(&custom);
    
    // Convert to JSON
    println!("\nJSON Examples:");
    println!("Stock JSON: {}", serde_json::to_string_pretty(&stock)?);
    
    Ok(())
}

// Helper function to print OHLC data using the trait interface
fn print_ohlc(data: &impl DailyMarketData) {
    println!("Date: {}", data.get_date());
    println!("OHLC: {:.2} | {:.2} | {:.2} | {:.2}",
        data.get_open(),
        data.get_high(),
        data.get_low(),
        data.get_close()
    );
    println!("Volume: {:.2}", data.get_volume());
    println!("Turnover: {:.2}", data.get_total_turnover());
}

// Helper function to print instrument type
fn print_instrument_type(bar: &DailyBar) {
    let type_name = match bar.instrument_type {
        InstrumentType::Stock => "Stock",
        InstrumentType::Future => "Future",
        InstrumentType::Index => "Index",
        InstrumentType::Fund => "Fund/ETF",
        InstrumentType::Other => "Other",
    };
    
    println!("{}: {} is a {}", 
        bar.get_date(), 
        bar.get_instrument_id(), 
        type_name
    );
} 