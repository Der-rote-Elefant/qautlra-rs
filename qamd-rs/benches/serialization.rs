use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qamd_rs::{MDSnapshot, OptionalF64, Tick};
use chrono::Utc;
use serde_json;

fn create_test_snapshot() -> MDSnapshot {
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
        // Optional fields for level 2
        ask_price2: Some(55.68),
        ask_volume2: Some(800),
        bid_price2: Some(55.64),
        bid_volume2: Some(600),
        ask_price3: Some(55.69),
        ask_volume3: Some(1200),
        bid_price3: Some(55.63),
        bid_volume3: Some(900),
        // Remaining fields set to None to benchmark realistic use case
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

fn criterion_benchmark(c: &mut Criterion) {
    // Generate a test snapshot once to use in all benchmarks
    let snapshot = create_test_snapshot();
    
    // Serialize the snapshot to JSON
    c.bench_function("serialize_snapshot_to_json", |b| {
        b.iter(|| {
            serde_json::to_string(black_box(&snapshot)).unwrap()
        })
    });
    
    // Pre-serialize to test deserialization performance
    let json = serde_json::to_string(&snapshot).unwrap();
    
    // Deserialize JSON into snapshot
    c.bench_function("deserialize_json_to_snapshot", |b| {
        b.iter(|| {
            let _snapshot: MDSnapshot = serde_json::from_str(black_box(&json)).unwrap();
        })
    });
    
    // Create Tick from snapshot
    c.bench_function("create_tick_from_snapshot", |b| {
        b.iter(|| {
            Tick::from_snapshot(black_box(&snapshot))
        })
    });
    
    // Serialize a tick
    let tick = Tick::from_snapshot(&snapshot);
    c.bench_function("serialize_tick_to_json", |b| {
        b.iter(|| {
            serde_json::to_string(black_box(&tick)).unwrap()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches); 