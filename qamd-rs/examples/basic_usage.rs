use qamd_rs::{MDSnapshot, OptionalF64, Tick};
use chrono::Utc;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("QAMD-RS Basic Usage Example");
    println!("==========================");
    
    // Create a market data snapshot
    let snapshot = create_example_snapshot();
    
    // Display snapshot information
    println!("\nSnapshot Information:");
    println!("Instrument: {}", snapshot.instrument_id);
    println!("Last Price: {}", snapshot.last_price);
    println!("Bid/Ask: {}/{} (spread: {})", 
        snapshot.bid_price1, 
        snapshot.ask_price1, 
        snapshot.bid_ask_spread());
    println!("Volume: {}", snapshot.volume);
    println!("Amount: {}", snapshot.amount);
    
    // Check for level 2 data
    if snapshot.has_level2_depth() {
        println!("\nLevel 2 Market Data Available:");
        println!("Ask Level 2: {} @ {}", 
            snapshot.ask_volume2.unwrap_or(0), 
            snapshot.ask_price2.unwrap_or(0.0));
        println!("Bid Level 2: {} @ {}", 
            snapshot.bid_volume2.unwrap_or(0), 
            snapshot.bid_price2.unwrap_or(0.0));
    }
    
    // Create a tick from the snapshot
    let tick = Tick::from_snapshot(&snapshot);
    println!("\nTick Data:");
    println!("Instrument: {}", tick.instrument_id);
    println!("Last Price: {}", tick.last_price);
    println!("Volume: {}", tick.volume);
    println!("Time: {}", tick.datetime);
    
    // Convert to JSON
    let json = serde_json::to_string_pretty(&snapshot)?;
    println!("\nJSON Representation Sample (truncated):");
    println!("{:.500}...", json);
    
    Ok(())
}

// Helper function to create an example snapshot
fn create_example_snapshot() -> MDSnapshot {
    MDSnapshot {
        instrument_id: "SSE_688286".to_string(),
        amount: 1_234_567.89,
        ask_price1: 55.67,
        ask_volume1: 500,
        bid_price1: 55.65,
        bid_volume1: 300,
        last_price: 55.66,
        datetime: Utc::now(),
        highest: 56.0,
        lowest: 55.2,
        open: 55.5,
        close: OptionalF64::Value(55.66),
        volume: 10_000,
        pre_close: 55.4,
        lower_limit: 50.0,
        upper_limit: 61.0,
        average: 55.65,
        // Optional fields
        ask_price2: Some(55.68),
        ask_volume2: Some(800),
        bid_price2: Some(55.64),
        bid_volume2: Some(600),
        // Other level 2 fields
        ask_price3: Some(55.69),
        ask_volume3: Some(1200),
        bid_price3: Some(55.63),
        bid_volume3: Some(900),
        // Remaining fields set to None
        ask_price4: None,
        ask_price5: None,
        ask_price6: None,
        ask_price7: None,
        ask_price8: None,
        ask_price9: None,
        ask_price10: None,
        ask_volume4: None,
        ask_volume5: None,
        ask_volume6: None,
        ask_volume7: None,
        ask_volume8: None,
        ask_volume9: None,
        ask_volume10: None,
        bid_price4: None,
        bid_price5: None,
        bid_price6: None,
        bid_price7: None,
        bid_price8: None,
        bid_price9: None,
        bid_price10: None,
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
    }
} 