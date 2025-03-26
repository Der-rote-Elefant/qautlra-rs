use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use actix_rt;
use tungstenite::{connect, Message};
use url::Url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start a separate thread for the input handling
    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();
    
    // Handle keyboard input in a separate thread
    thread::spawn(move || {
        println!("Commands:");
        println!("  s, sub <instrument_id> - Subscribe to an instrument");
        println!("  u, unsub <instrument_id> - Unsubscribe from an instrument");
        println!("  l, list - List current subscriptions");
        println!("  q, quit - Exit the program");
        
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            match parts[0] {
                "q" | "quit" => {
                    println!("Exiting...");
                    let _ = tx.send(None); // Signal to quit
                    break;
                },
                "s" | "sub" => {
                    if parts.len() < 2 {
                        println!("Usage: sub <instrument_id>");
                        continue;
                    }
                    
                    let instrument = parts[1];
                    let subscribe_msg = json!({
                        "aid": "subscribe_quote",
                        "ins_list": instrument,
                        "data_type": "MARKET"
                    });

                    println!("Sending subscription for: {}", instrument);
                    if let Err(e) = tx.send(Some(Message::Text(subscribe_msg.to_string()))) {
                        eprintln!("Error queueing subscribe request: {}", e);
                    }
                },
                "u" | "unsub" => {
                    if parts.len() < 2 {
                        println!("Usage: unsub <instrument_id>");
                        continue;
                    }
                    
                    let instrument = parts[1];
                    let unsubscribe_msg = json!({
                        "action": "unsubscribe",
                        "instrument": instrument,
                        "data_type": "MARKET"
                    });
                    
                    println!("Sending unsubscription for: {}", instrument);
                    if let Err(e) = tx.send(Some(Message::Text(unsubscribe_msg.to_string()))) {
                        eprintln!("Error queueing unsubscribe request: {}", e);
                    }
                },
                "l" | "list" => {
                    println!("Requesting subscription list");
                    let list_msg = json!({
                        "action": "list_subscriptions"
                    });
                    
                    if let Err(e) = tx.send(Some(Message::Text(list_msg.to_string()))) {
                        eprintln!("Error queueing list request: {}", e);
                    }
                },
                _ => {
                    println!("Unknown command: {}", parts[0]);
                }
            }
        }
    });
    
    // Ping thread
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(30));
            
            let ping_msg = json!({
                "action": "ping",
                "data": chrono::Utc::now().timestamp_millis()
            });
            
            if let Err(e) = tx_clone.send(Some(Message::Text(ping_msg.to_string()))) {
                eprintln!("Error queueing ping: {}", e);
                break;
            }
        }
    });
    
    // Connect to the WebSocket server (blocking version)
    let url = Url::parse("ws://localhost:8012/ws/qq/market")?;
    println!("Connecting to {}", url);
    
    let (mut socket, _) = connect(url)?;
    println!("WebSocket connected");
    
    // Main loop - this is simpler as we're using blocking calls
    loop {
        // Check for user input
        if let Ok(msg_opt) = rx.try_recv() {
            match msg_opt {
                Some(msg) => {
                    if let Err(e) = socket.write_message(msg) {
                        eprintln!("Error sending message: {}", e);
                        break;
                    }
                },
                None => {
                    // User requested to quit
                    break;
                }
            }
        }
        
        // Check for incoming messages
        match socket.read_message() {
            Ok(Message::Text(text)) => {
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    if json["action"] == "market_data" {
                        // Format and display market data
                        if let Some(data) = json.get("data") {
                            println!(
                                "Market data: Instrument={}, Ask={}, Bid={}, Last={}, Time={}",
                                json.get("instrument").and_then(|v| v.as_str()).unwrap_or("unknown"),
                                data.get("ask_price1").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                data.get("bid_price1").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                data.get("last_price").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                data.get("update_time").and_then(|v| v.as_str()).unwrap_or("unknown")
                            );
                        } else {
                            println!("Received  QQ market data: {}", text);
                        }
                    } else if json["action"] == "subscription_list" {
                        println!("Subscription list: {:?}", json["data"]);
                    } else if json["action"] == "error" {
                        eprintln!("Error from server: {}", json["data"]);
                    } else if json["action"] == "pong" {
                        println!("Pong received: latency {}ms", json["data"]);
                    } else {
                        println!("Other message: {}", text);
                    }
                } else {
                    println!("Received non-JSON text: {}", text);
                }
            },
            Ok(Message::Binary(data)) => {
                println!("Received binary data: {} bytes", data.len());
            },
            Ok(Message::Ping(_)) => {
                println!("Received ping");
            },
            Ok(Message::Pong(_)) => {
                println!("Received pong");
            },
            Ok(Message::Close(_)) => {
                println!("WebSocket closed");
                break;
            },
            Ok(_) => {
                println!("Received other message type");
            },
            Err(e) => {
                // For non-blocking, we would check for WouldBlock here
                eprintln!("Error receiving message: {}", e);
                break;
            },
        }
    }
    
    println!("WebSocket client terminated");
    Ok(())
} 