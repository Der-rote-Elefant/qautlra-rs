use actix::prelude::*;
use log::{info, error, debug};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use serde_json::json;

use crate::actors::messages::*;
use qamd_rs::MDSnapshot;

/// Market data distributor actor
pub struct MarketDataDistributor {
    /// Clients connected to this distributor
    clients: HashMap<Uuid, Recipient<WebSocketMessage>>,
    /// Map of instruments to clients subscribed to them
    subscriptions: HashMap<String, HashSet<Uuid>>,
}

impl Default for MarketDataDistributor {
    fn default() -> Self {
        Self {
            clients: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }
}

impl Actor for MarketDataDistributor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("MarketDataDistributor started");
    }
}

impl Handler<WebSocketConnect> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: WebSocketConnect, _: &mut Self::Context) -> Self::Result {
        info!("Client connected: {}", msg.id);
        self.clients.insert(msg.id, msg.addr);
    }
}

impl Handler<WebSocketDisconnect> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: WebSocketDisconnect, _: &mut Self::Context) -> Self::Result {
        info!("Client disconnected: {}", msg.id);
        
        // Remove client
        self.clients.remove(&msg.id);
        
        // Remove client from all subscriptions
        for subscribers in self.subscriptions.values_mut() {
            subscribers.remove(&msg.id);
        }
        
        // Remove empty subscription sets
        self.subscriptions.retain(|_, subscribers| !subscribers.is_empty());
    }
}

impl Handler<AddSubscription> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: AddSubscription, _: &mut Self::Context) -> Self::Result {
        debug!("Client {} subscribing to {}", msg.client_id, msg.instrument);
        
        // Add client to subscribers for this instrument
        self.subscriptions
            .entry(msg.instrument)
            .or_insert_with(HashSet::new)
            .insert(msg.client_id);
    }
}

impl Handler<RemoveSubscription> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: RemoveSubscription, _: &mut Self::Context) -> Self::Result {
        debug!("Client {} unsubscribing from {}", msg.client_id, msg.instrument);
        
        // Remove client from subscribers for this instrument
        if let Some(subscribers) = self.subscriptions.get_mut(&msg.instrument) {
            subscribers.remove(&msg.client_id);
            
            // If no subscribers left, remove the instrument
            if subscribers.is_empty() {
                self.subscriptions.remove(&msg.instrument);
            }
        }
    }
}

impl Handler<GetAllSubscriptions> for MarketDataDistributor {
    type Result = MessageResult<GetAllSubscriptions>;

    fn handle(&mut self, _: GetAllSubscriptions, _: &mut Self::Context) -> Self::Result {
        MessageResult(self.subscriptions.keys().cloned().collect())
    }
}

impl Handler<MarketDataUpdate> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: MarketDataUpdate, _: &mut Self::Context) -> Self::Result {
        let snapshot = msg.0;
        let instrument_id = snapshot.instrument_id.clone();
        println!("Distributor received market data for: {}", instrument_id);
        println!("Current subscriptions: {:?}", self.subscriptions);

        // Find clients subscribed to this instrument
        if let Some(subscribers) = self.subscriptions.get(&instrument_id) {
            println!("Found {} subscribers for {}", subscribers.len(), instrument_id);
            
            // 1. 创建传统格式消息
            let legacy_message = json!({
                "type": "market_data",
                "payload": {
                    "data": snapshot
                }
            }).to_string();
            
            // 2. 创建 TradingView 格式消息
            use std::collections::HashMap;
            use qamd_rs::types::OptionalF64;
            
            // Convert open_interest from OptionalNumeric to i64
            let open_interest = match &snapshot.open_interest {
                OptionalF64::Value(val) => *val as i64,
                _ => 0,
            };
            
            let mut tv_quote = HashMap::new();
            let quote = json!({
                "instrument_id": snapshot.instrument_id,
                "datetime": snapshot.datetime.to_rfc3339(),
                "last_price": snapshot.last_price,
                "volume": snapshot.volume,
                "amount": snapshot.amount,
                "open": snapshot.open,
                "high": snapshot.highest,
                "low": snapshot.lowest,
                "bid_price1": snapshot.bid_price1,
                "bid_volume1": snapshot.bid_volume1,
                "ask_price1": snapshot.ask_price1,
                "ask_volume1": snapshot.ask_volume1,
                "volume_multiple": 1,
                "price_tick": 0.01,
                "price_decs": 2,
                "open_interest": open_interest,
                // 其他字段设为默认值
                "max_market_order_volume": 0,
                "min_market_order_volume": 0,
                "max_limit_order_volume": 0,
                "min_limit_order_volume": 0,
                "margin": 0.0,
                "commission": 0.0,
                "upper_limit": 0.0,
                "lower_limit": 0.0,
                "pre_close": 0.0,
                "pre_settlement": 0.0,
                "pre_open_interest": 0,
                "close": 0.0,
                "settlement": 0.0,
                "average": 0.0
            });
            tv_quote.insert(snapshot.instrument_id.clone(), quote);
            
            let tv_message = json!({
                "aid": "rtn_data",
                "data": [{
                    "quotes": tv_quote
                }]
            }).to_string();
            
            // 发送给所有订阅者
            for client_id in subscribers.iter() {
                if let Some(client) = self.clients.get(client_id) {
                    // 发送传统格式
                    client.do_send(WebSocketMessage(legacy_message.clone()));
                    
                    // 发送 TradingView 格式
                    client.do_send(WebSocketMessage(tv_message.clone()));
                }
            }
        } else {
            println!("No subscribers found for {}", instrument_id);
        }
    }
} 