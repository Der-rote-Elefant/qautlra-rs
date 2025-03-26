use std::collections::HashSet;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::ffi::CString;

use actix::prelude::*;
use ctp_common::DepthMarketData;
use ctp_md::{GenericMdApi, MdApi};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use super::mdspi::CTPMDSPI;

/// Market data message
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct MarketData {
    pub trading_day: String,
    pub instrument_id: String,
    pub exchange_id: String,
    pub exchange_inst_id: String,
    pub last_price: f64,
    pub pre_settlement_price: f64,
    pub pre_close_price: f64,
    pub pre_open_interest: f64,
    pub open_price: f64,
    pub highest_price: f64,
    pub lowest_price: f64,
    pub volume: i32,
    pub turnover: f64,
    pub open_interest: f64,
    pub upper_limit_price: f64,
    pub lower_limit_price: f64,
    pub update_time: String,
    pub update_millisec: i32,
    pub bid_price1: f64,
    pub bid_volume1: i32,
    pub ask_price1: f64,
    pub ask_volume1: i32,
}

impl From<DepthMarketData> for MarketData {
    fn from(data: DepthMarketData) -> Self {
        Self {
            trading_day: data.TradingDay,
            instrument_id: data.InstrumentID.clone(),
            exchange_id: data.ExchangeID,
            exchange_inst_id: data.ExchangeInstID,
            last_price: data.LastPrice,
            pre_settlement_price: data.PreSettlementPrice,
            pre_close_price: data.PreClosePrice,
            pre_open_interest: data.PreOpenInterest,
            open_price: data.OpenPrice,
            highest_price: data.HighestPrice,
            lowest_price: data.LowestPrice,
            volume: data.Volume,
            turnover: data.Turnover,
            open_interest: data.OpenInterest,
            upper_limit_price: data.UpperLimitPrice,
            lower_limit_price: data.LowerLimitPrice,
            update_time: data.UpdateTime,
            update_millisec: data.UpdateMillisec,
            bid_price1: data.BidPrice1,
            bid_volume1: data.BidVolume1,
            ask_price1: data.AskPrice1,
            ask_volume1: data.AskVolume1,
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    pub subscribe: Vec<String>,
    pub client_id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UnSubscribe {
    pub unsubscribe: Vec<String>,
    pub client_id: usize,
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<MarketData>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Market data server that integrates CTP and WebSocket
pub struct MDServer {
    /// Market data API
    md_api: MdApi,
    /// Market data receiver
    rx: std::sync::mpsc::Receiver<DepthMarketData>,
    /// Connected sessions
    sessions: HashMap<usize, Recipient<MarketData>>,
    /// Subscriptions by instrument
    subscriptions: HashMap<String, HashSet<usize>>,
    /// Current front server
    front_server: String,
    /// User ID for login
    user_id: String,
    /// Password for login
    password: String,
    /// Broker ID for login
    broker_id: String,
}

impl MDServer {
    pub fn new(front_servers: Vec<&str>, user_id: &str, password: &str, broker_id: &str) -> Self {
        // Create channel to receive market data
        let (tx, rx) = channel();
        
        // Initialize the Market Data API with the first front server
        let front = if front_servers.is_empty() {
            "tcp://180.168.146.187:10131"
        } else {
            front_servers[0]
        };
        
        // Create SPI
        let md_spi = Box::new(CTPMDSPI::new(tx));
        
        // Configure and start the MD API
        let mut md_api = MdApi::new(CString::new("./flow/").unwrap(), false, false);
        md_api.register_front(CString::new(front).unwrap());
        md_api.register_spi(md_spi);
        md_api.init();
        
        println!("Starting Market Data Server with front server: {}", front);
        
        Self {
            md_api,
            rx,
            sessions: HashMap::new(),
            subscriptions: HashMap::new(),
            front_server: front.to_string(),
            user_id: user_id.to_string(),
            password: password.to_string(),
            broker_id: broker_id.to_string(),
        }
    }

    /// Send market data to subscribed clients
    fn send_market_data(&self, market_data: &MarketData) {
        if let Some(sessions) = self.subscriptions.get(&market_data.instrument_id) {
            for session_id in sessions {
                if let Some(recipient) = self.sessions.get(session_id) {
                    recipient.do_send(market_data.clone());
                }
            }
        }
    }
    
    /// Log in to the CTP server
    fn login(&mut self) {
        use ctp_common::ReqUserLoginField;
        
        let login_field = ReqUserLoginField {
            TradingDay: [0; 9],
            UserID: std::iter::FromIterator::from_iter(
                self.user_id.bytes().chain(std::iter::repeat(0).take(16 - self.user_id.len())),
            ),
            Password: std::iter::FromIterator::from_iter(
                self.password.bytes().chain(std::iter::repeat(0).take(41 - self.password.len())),
            ),
            BrokerID: std::iter::FromIterator::from_iter(
                self.broker_id.bytes().chain(std::iter::repeat(0).take(11 - self.broker_id.len())),
            ),
            UserProductInfo: [0; 11],
            InterfaceProductInfo: [0; 11],
            ProtocolInfo: [0; 11],
            MacAddress: [0; 21],
            OneTimePassword: [0; 41],
            ClientIPAddress: [0; 16],
            LoginRemark: [0; 36],
            ClientIPPort: 0,
        };
        
        if let Err(err) = self.md_api.req_user_login(&login_field, 1) {
            println!("Login request failed: {:?}", err);
        } else {
            println!("Login request sent");
        }
    }
    
    /// Subscribe to market data
    fn subscribe_market_data(&mut self, instruments: &[String]) {
        if instruments.is_empty() {
            return;
        }
        
        // Convert instrument IDs to CString
        let c_instruments: Vec<CString> = instruments
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();
        
        // Subscribe to market data
        if let Err(err) = self.md_api.subscribe_market_data(&c_instruments) {
            println!("Failed to subscribe to market data: {:?}", err);
        } else {
            println!("Subscribed to market data for instruments: {:?}", instruments);
        }
    }
    
    /// Unsubscribe from market data
    fn unsubscribe_market_data(&mut self, instruments: &[String]) {
        if instruments.is_empty() {
            return;
        }
        
        // Convert instrument IDs to CString
        let c_instruments: Vec<CString> = instruments
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();
        
        // Unsubscribe from market data
        if let Err(err) = self.md_api.unsubscribe_market_data(&c_instruments) {
            println!("Failed to unsubscribe from market data: {:?}", err);
        } else {
            println!("Unsubscribed from market data for instruments: {:?}", instruments);
        }
    }
}

impl Actor for MDServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Market Data Server started");
        
        // Login after a short delay
        ctx.run_later(Duration::from_secs(1), |act, _ctx| {
            act.login();
        });
        
        // Poll for market data
        ctx.run_interval(Duration::from_millis(2), |act, _ctx| {
            if let Ok(data) = act.rx.try_recv() {
                let market_data = MarketData::from(data);
                
                println!(
                    "MarketData: {} {} {}",
                    market_data.instrument_id, market_data.trading_day, market_data.update_time
                );
                
                // Send to subscribed clients
                act.send_market_data(&market_data);
            }
        });
        
        // Log active subscriptions periodically
        ctx.run_interval(Duration::from_secs(60), |act, _ctx| {
            println!("Active subscriptions: {:?}", act.subscriptions);
        });
    }
}

impl Handler<Subscribe> for MDServer {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _ctx: &mut Self::Context) -> Self::Result {
        println!("MDServer: handling Subscribe request");
        
        // Update subscriptions
        let mut new_instruments = Vec::new();
        
        for instrument in &msg.subscribe {
            let is_new = self.subscriptions
                .entry(instrument.clone())
                .or_insert_with(HashSet::new)
                .insert(msg.client_id);
                
            if is_new {
                new_instruments.push(instrument.clone());
            }
        }
        
        // Subscribe to new instruments
        if !new_instruments.is_empty() {
            self.subscribe_market_data(&new_instruments);
        }
    }
}

impl Handler<UnSubscribe> for MDServer {
    type Result = ();

    fn handle(&mut self, msg: UnSubscribe, _ctx: &mut Self::Context) -> Self::Result {
        println!("MDServer: handling UnSubscribe request");
        
        // Update subscriptions
        let mut instruments_to_unsubscribe = Vec::new();
        
        for instrument in &msg.unsubscribe {
            if let Some(sessions) = self.subscriptions.get_mut(instrument) {
                sessions.remove(&msg.client_id);
                
                // If no more subscribers, unsubscribe from the feed
                if sessions.is_empty() {
                    instruments_to_unsubscribe.push(instrument.clone());
                }
            }
        }
        
        // Unsubscribe from instruments with no subscribers
        if !instruments_to_unsubscribe.is_empty() {
            self.unsubscribe_market_data(&instruments_to_unsubscribe);
        }
    }
}

impl Handler<Connect> for MDServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        println!("MDServer: New client connected");
        
        // Generate a new session ID
        let id = self.sessions.len();
        // Store the client's recipient
        self.sessions.insert(id, msg.addr);
        
        id
    }
}

impl Handler<Disconnect> for MDServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        println!("MDServer: Client disconnected");
        
        // Remove from sessions
        if self.sessions.remove(&msg.id).is_some() {
            // Remove from all subscriptions
            for (_instrument, sessions) in &mut self.subscriptions {
                sessions.remove(&msg.id);
            }
            
            // Clean up empty subscriptions
            self.subscriptions.retain(|_, sessions| !sessions.is_empty());
        }
    }
} 