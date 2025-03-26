use actix::prelude::*;
use ctp_common::{CThostFtdcDepthMarketDataField, CThostFtdcReqUserLoginField, CThostFtdcSpecificInstrumentField};
use ctp_md::{DisconnectionReason, MdApi, MdSpi, RspResult, GenericMdApi};
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

use crate::actors::messages::*;
use crate::config::BrokerConfig;
use crate::converter::convert_ctp_to_md_snapshot;
use crate::error::GatewayResult;

struct CtpMdSpiImpl {
    // We use the actor's address to send messages back to the actor from the CTP callbacks
    actor_addr: Addr<MarketDataActor>,
    subscribed_instruments: Arc<Mutex<HashSet<String>>>,
}

impl MdSpi for CtpMdSpiImpl {
    fn on_front_connected(&mut self) {
        info!("CTP MD Front connected - XCTP回调：前置连接已建立");
        self.actor_addr.do_send(MarketDataEvent::Connected);
    }

    fn on_front_disconnected(&mut self, reason: DisconnectionReason) {
        warn!("CTP MD Front disconnected: {:?} - XCTP回调：前置连接已断开，原因: {:?}", reason, reason);
        self.actor_addr.do_send(MarketDataEvent::Disconnected);
    }

    fn on_rsp_user_login(
        &mut self,
        rsp_user_login: Option<&ctp_common::CThostFtdcRspUserLoginField>,
        result: RspResult,
        request_id: i32,
        is_last: bool,
    ) {
        info!("XCTP回调：登录响应 RequestID={}, IsLast={}", request_id, is_last);
        
        if let Some(login_info) = rsp_user_login {
            let trading_day = String::from_utf8_lossy(&login_info.TradingDay);
            let login_time = String::from_utf8_lossy(&login_info.LoginTime);
            let broker_id = String::from_utf8_lossy(&login_info.BrokerID);
            let user_id = String::from_utf8_lossy(&login_info.UserID);
            
            info!(
                "CTP MD Logged in: Trading Day = {}, Login Time = {}, Broker ID = {}, User ID = {} - XCTP回调：登录成功",
                trading_day, login_time, broker_id, user_id
            );
            
            self.actor_addr.do_send(MarketDataEvent::LoggedIn);
        } else if let Some(error) = result.err() {
            let error_msg = format!(
                "CTP MD Login failed: Error = {} - XCTP回调：登录失败",
                error
            );
            error!("{}", error_msg);
            self.actor_addr.do_send(MarketDataEvent::Error(error_msg));
        }
    }

    fn on_rsp_sub_market_data(
        &mut self,
        specific_instrument: Option<&CThostFtdcSpecificInstrumentField>,
        result: RspResult,
        request_id: i32,
        is_last: bool,
    ) {
        info!("XCTP回调：订阅行情响应 RequestID={}, IsLast={}", request_id, is_last);
        
        if let Some(instrument) = specific_instrument {
            let instrument_id = String::from_utf8_lossy(&instrument.InstrumentID)
                .trim_end_matches('\0')
                .to_string();

            if result.is_ok() {
                info!("Subscribed to market data for {} - XCTP回调：订阅行情成功", instrument_id);
                
                // Save the subscription
                if let Ok(mut subscribed) = self.subscribed_instruments.lock() {
                    subscribed.insert(instrument_id.clone());
                }
                
                self.actor_addr.do_send(MarketDataEvent::SubscriptionSuccess(instrument_id));
            } else if let Some(error) = result.err() {
                let error_msg = format!(
                    "Failed to subscribe to market data for {}: Error = {} - XCTP回调：订阅行情失败",
                    instrument_id,
                    error
                );
                error!("{}", error_msg);
                self.actor_addr.do_send(MarketDataEvent::SubscriptionFailure(instrument_id, error_msg));
            }
        }
    }

    fn on_rtn_depth_market_data(
        &mut self,
        depth_market_data: Option<&CThostFtdcDepthMarketDataField>,
    ) {
        if let Some(market_data) = depth_market_data {
            // Clone the data to send it to the actor
            let market_data_owned = *market_data;
            self.actor_addr.do_send(MarketDataEvent::MarketData(market_data_owned));
        }
    }

    fn on_rsp_un_sub_market_data(
        &mut self,
        specific_instrument: Option<&CThostFtdcSpecificInstrumentField>,
        result: RspResult,
        request_id: i32,
        is_last: bool,
    ) {
        info!("XCTP回调：取消订阅行情响应 RequestID={}, IsLast={}", request_id, is_last);
        
        if let Some(instrument) = specific_instrument {
            let instrument_id = String::from_utf8_lossy(&instrument.InstrumentID)
                .trim_end_matches('\0')
                .to_string();

            if result.is_ok() {
                info!("Unsubscribed from market data for {} - XCTP回调：取消订阅行情成功", instrument_id);
                
                // Remove the subscription
                if let Ok(mut subscribed) = self.subscribed_instruments.lock() {
                    subscribed.remove(&instrument_id);
                }
            } else if let Some(error) = result.err() {
                error!(
                    "Failed to unsubscribe from market data for {}: Error = {} - XCTP回调：取消订阅行情失败",
                    instrument_id,
                    error
                );
            }
        }
    }

    fn on_rsp_error(
        &mut self,
        result: RspResult,
        request_id: i32,
        is_last: bool,
    ) {
        if let Some(error) = result.err() {
            let error_msg = format!(
                "CTP error: Request ID = {}, Is Last = {}, Error = {} - XCTP回调：错误响应",
                request_id, is_last, error
            );
            error!("{}", error_msg);
            self.actor_addr.do_send(MarketDataEvent::Error(error_msg));
        }
    }
}

pub struct MarketDataActor {
    md_api: Option<MdApi>,
    subscribed_instruments: Arc<Mutex<HashSet<String>>>,
    broker_config: BrokerConfig,
    distributor: Option<Addr<crate::actors::md_distributor::MarketDataDistributor>>,
    front_addr: String,
    user_id: String,
    password: String,
    broker_id: String,
    is_connected: bool,
    is_logged_in: bool,
}

impl Actor for MarketDataActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("MarketDataActor started");
        
        // Schedule a heartbeat to check connection status
        ctx.run_interval(Duration::from_secs(30), |act, ctx| {
            if !act.is_connected {
                info!("MarketDataActor heartbeat: Not connected, attempting to reconnect");
                act.init_md_api(ctx);
            }
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("MarketDataActor stopped");
    }
}

impl MarketDataActor {
    pub fn new(config: BrokerConfig) -> Self {
        let front_addr = config.front_addr.clone();
        let user_id = config.user_id.clone();
        let password = config.password.clone();
        let broker_id = config.broker_id.clone();
        
        Self {
            md_api: None,
            subscribed_instruments: Arc::new(Mutex::new(HashSet::new())),
            broker_config: config,
            distributor: None,
            front_addr,
            user_id,
            password,
            broker_id,
            is_connected: false,
            is_logged_in: false,
        }
    }

    fn init_md_api(&mut self, ctx: &mut Context<Self>) {
        // CTP requires a flow path for data storage
        let flow_path = ::std::ffi::CString::new("").unwrap();//CString::new("./data_md_flow").unwrap();
        
        // Create the MdApi
        let mut md_api = MdApi::new(flow_path, false, false);
        
        // Create SPI
        let addr = ctx.address();
        let subscribed_instruments = self.subscribed_instruments.clone();
        let spi = Box::new(CtpMdSpiImpl {
            actor_addr: addr,
            subscribed_instruments,
        });
        
        // Register SPI
        md_api.register_spi(spi);
        
        // Connect
        let front_addr = CString::new(self.front_addr.clone()).unwrap();
        md_api.register_front(front_addr);
        
        // Initialize the API
        md_api.init();
        std::thread::sleep(std::time::Duration::from_secs(1));

        
        // Save the API
        self.md_api = Some(md_api);
    }

    fn login(&mut self) -> Result<(), String> {
        if let Some(ref mut md_api) = self.md_api {
            let mut req = CThostFtdcReqUserLoginField::default();
            
            // Fill login request - safely handling buffer sizes
            if !self.broker_id.is_empty() {
                // Ensure we only copy what fits in the destination buffer
                let broker_bytes = self.broker_id.as_bytes();
                let copy_len = std::cmp::min(broker_bytes.len(), req.BrokerID.len() - 1); // Leave room for null terminator
                req.BrokerID[..copy_len].copy_from_slice(&broker_bytes[..copy_len]);
                req.BrokerID[copy_len] = 0; // Null terminator
            }
            
            if !self.user_id.is_empty() {
                // Ensure we only copy what fits in the destination buffer
                let user_bytes = self.user_id.as_bytes();
                let copy_len = std::cmp::min(user_bytes.len(), req.UserID.len() - 1); // Leave room for null terminator
                req.UserID[..copy_len].copy_from_slice(&user_bytes[..copy_len]);
                req.UserID[copy_len] = 0; // Null terminator
            }
            
            if !self.password.is_empty() {
                // Ensure we only copy what fits in the destination buffer
                let pass_bytes = self.password.as_bytes();
                let copy_len = std::cmp::min(pass_bytes.len(), req.Password.len() - 1); // Leave room for null terminator
                req.Password[..copy_len].copy_from_slice(&pass_bytes[..copy_len]);
                req.Password[copy_len] = 0; // Null terminator
            }
            
            // Perform login
            let result = md_api.req_user_login(&req, 1);
            
            match result {
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    Ok(())},
                Err(e) => {
                    let error_msg = format!("Failed to send login request: {:?}", e);
                    error!("{}", error_msg);
                    Err(error_msg)
                }
            }
        } else {
            Err("Market data API not initialized".to_string())
        }
    }

    fn subscribe_instruments(&mut self, instruments: &[String]) -> Result<(), String> {
        if !self.is_logged_in {
            return Err("Not logged in".to_string());
        }

        if let Some(ref mut md_api) = self.md_api {
            // Convert instruments to CString
            // let mut instrument_cstrings = Vec::new();
            // for instr in instruments {
            //     match CString::new(instr.clone()) {
            //         Ok(cstr) => instrument_cstrings.push(cstr),
            //         Err(e) => return Err(format!("Invalid instrument ID: {}, error: {}", instr, e)),
            //     }
            // }
            let instrument_cstrings: Vec<CString> = instruments
            .iter()
            .map(|s| {
                // 股票代码可能不含交易所前缀，需要处理
                let instrument_code = s.split('.').last().unwrap_or(s);
                
                // // 对于纯数字的股票代码，检查长度，可能需要添加前导零
                // let code = if instrument_code.chars().all(char::is_numeric) && instrument_code.len() <= 6 {
                //     // 确保股票代码长度为6位
                //     format!("{:0>6}", instrument_code)
                // } else {
                //     instrument_code.to_string()
                // };
                let code =instrument_code.to_string();

                println!("CTP Subscribing to instrument: {}", code);
                CString::new(code).unwrap()
            })
            .collect();
            // Subscribe
            let result = md_api.subscribe_market_data(&instrument_cstrings);
            
            match result {
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    // 股票代码可能不含交易所前缀，需要处理
                    Ok(())},
                Err(e) => Err(format!("Failed to subscribe to instruments, error: {:?}", e))
            }
        } else {
            Err("MD API not initialized".to_string())
        }
    }

    fn unsubscribe_instruments(&mut self, instruments: &[String]) -> Result<(), String> {
        if !self.is_logged_in {
            return Err("Not logged in".to_string());
        }

        if let Some(ref mut md_api) = self.md_api {
            // Convert instruments to CString
            let mut instrument_cstrings = Vec::new();
            for instr in instruments {
                match CString::new(instr.clone()) {
                    Ok(cstr) => instrument_cstrings.push(cstr),
                    Err(e) => return Err(format!("Invalid instrument ID: {}, error: {}", instr, e)),
                }
            }
            
            // Unsubscribe
            let result = md_api.unsubscribe_market_data(&instrument_cstrings);
            
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to unsubscribe from instruments, error: {:?}", e))
            }
        } else {
            Err("MD API not initialized".to_string())
        }
    }
}

impl Handler<InitMarketDataSource> for MarketDataActor {
    type Result = ();

    fn handle(&mut self, _: InitMarketDataSource, ctx: &mut Self::Context) -> Self::Result {
        self.init_md_api(ctx);
    }
}

impl Handler<LoginMarketDataSource> for MarketDataActor {
    type Result = Result<(), String>;

    fn handle(&mut self, _: LoginMarketDataSource, _: &mut Self::Context) -> Self::Result {
        self.login()
    }
}

impl Handler<Subscribe> for MarketDataActor {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) -> Self::Result {
        if let Err(e) = self.subscribe_instruments(&msg.instruments) {
            error!("Failed to subscribe to instruments: {}", e);
        }
    }
}

impl Handler<Unsubscribe> for MarketDataActor {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _: &mut Self::Context) -> Self::Result {
        if let Err(e) = self.unsubscribe_instruments(&msg.instruments) {
            error!("Failed to unsubscribe from instruments: {}", e);
        }
    }
}

impl Handler<GetSubscriptions> for MarketDataActor {
    type Result = Vec<String>;

    fn handle(&mut self, msg: GetSubscriptions, _: &mut Self::Context) -> Self::Result {
        let subscriptions = if let Ok(subscribed) = self.subscribed_instruments.lock() {
            subscribed.iter().cloned().collect()
        } else {
            Vec::new()
        };
        
        // If a callback was provided, execute it with the subscriptions
        if let Some(callback) = msg.callback {
            callback(subscriptions.clone());
        }
        
        subscriptions
    }
}

impl Handler<MarketDataEvent> for MarketDataActor {
    type Result = ();

    fn handle(&mut self, msg: MarketDataEvent, _: &mut Self::Context) -> Self::Result {
        match msg {
            MarketDataEvent::Connected => {
                info!("Market data source connected");
                self.is_connected = true;
                
                // Automatically login when connected
                if let Err(e) = self.login() {
                    error!("Failed to login: {}", e);
                }
            },
            MarketDataEvent::Disconnected => {
                warn!("Market data source disconnected");
                self.is_connected = false;
                self.is_logged_in = false;
            },
            MarketDataEvent::LoggedIn => {
                info!("Market data source logged in");
                self.is_logged_in = true;
                
                // Resubscribe to all instruments
                let instruments = {
                    if let Ok(subscribed) = self.subscribed_instruments.lock() {
                        subscribed.iter().cloned().collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    }
                };
                
                if !instruments.is_empty() {
                    if let Err(e) = self.subscribe_instruments(&instruments) {
                        error!("Failed to resubscribe to instruments: {}", e);
                    }
                }
            },
            MarketDataEvent::MarketData(md) => {
                // Convert to MDSnapshot
                
                match convert_ctp_to_md_snapshot(&md) {
                    Ok(snapshot) => {
                        println!("self.distributor: {:?}", self.distributor);
                        // Forward to distributor
                        if let Some(distributor) = &self.distributor {
                            distributor.do_send(MarketDataUpdate(snapshot, MarketDataSource::CTP));
                        }
                    },
                    Err(e) => {
                        println!("Failed to convert market data: {}", e);
                    }
                }
            },
            MarketDataEvent::SubscriptionSuccess(instrument) => {
                info!("Successfully subscribed to {}", instrument);
            },
            MarketDataEvent::SubscriptionFailure(instrument, error) => {
                error!("Failed to subscribe to {}: {}", instrument, error);
            },
            MarketDataEvent::Error(error) => {
                error!("Market data error: {}", error);
            },
        }
    }
}

impl Handler<RegisterDistributor> for MarketDataActor {
    type Result = ();

    fn handle(&mut self, msg: RegisterDistributor, _: &mut Self::Context) -> Self::Result {
        self.distributor = Some(msg.addr);
        info!("Market data distributor registered");
    }
}

impl Handler<StartMarketData> for MarketDataActor {
    type Result = ();

    fn handle(&mut self, msg: StartMarketData, ctx: &mut Self::Context) -> Self::Result {
        // Initialize if not already done
        if self.md_api.is_none() {
            self.init_md_api(ctx);
        }
        
        // Subscribe to instruments
        if !msg.instruments.is_empty() {
            if let Err(e) = self.subscribe_instruments(&msg.instruments) {
                error!("Failed to subscribe to initial instruments: {}", e);
            }
        }
    }
}

impl Handler<StopMarketData> for MarketDataActor {
    type Result = ();

    fn handle(&mut self, _: StopMarketData, _: &mut Self::Context) -> Self::Result {
        // Unsubscribe from all instruments
        let instruments = {
            if let Ok(subscribed) = self.subscribed_instruments.lock() {
                subscribed.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };
        
        if !instruments.is_empty() {
            if let Err(e) = self.unsubscribe_instruments(&instruments) {
                error!("Failed to unsubscribe from instruments: {}", e);
            }
        }
    }
}

impl Handler<RestartActor> for MarketDataActor {
    type Result = ();

    fn handle(&mut self, _: RestartActor, ctx: &mut Self::Context) -> Self::Result {
        // Only restart if not connected or not logged in
        if !self.is_connected || !self.is_logged_in {
            info!("Restarting market data actor for broker {}", self.broker_id);
            
            // Re-initialize
            if self.md_api.is_none() {
                self.init_md_api(ctx);
            }
            
            // Try to login again
            if let Err(e) = self.login() {
                error!("Failed to login during restart: {}", e);
            }
        }
    }
} 