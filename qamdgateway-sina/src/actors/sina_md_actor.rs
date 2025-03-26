use actix::prelude::*;
use ctp_common::{CThostFtdcDepthMarketDataField, CThostFtdcReqUserLoginField, CThostFtdcSpecificInstrumentField};
use ctp_md_sina::{MdApi, MdSpi, DisconnectionReason, RspResult, GenericMdApi};
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

use crate::actors::messages::*;
use crate::config::BrokerConfig;
use crate::converter::convert_ctp_to_md_snapshot;

/// Sina行情回调实现
struct SinaMdSpiImpl {
    /// Actor地址，用于发送消息回Actor
    actor_addr: Addr<SinaMarketDataActor>,
    /// 已订阅的合约列表
    subscribed_instruments: Arc<Mutex<HashSet<String>>>,
}

impl MdSpi for SinaMdSpiImpl {
    fn on_front_connected(&mut self) {
        info!("Sina MD Front connected - XCTP回调：前置连接已建立");
        self.actor_addr.do_send(MarketDataEvent::Connected);
    }

    fn on_front_disconnected(&mut self, reason: DisconnectionReason) {
        warn!("Sina MD Front disconnected: {:?} - XCTP回调：前置连接已断开，原因: {:?}", reason, reason);
        self.actor_addr.do_send(MarketDataEvent::Disconnected);
    }

    fn on_rsp_user_login(
        &mut self,
        rsp_user_login: Option<&ctp_common::CThostFtdcRspUserLoginField>,
        result: RspResult,
        request_id: i32,
        is_last: bool,
    ) {
        info!("XCTP回调：Sina登录响应 RequestID={}, IsLast={}", request_id, is_last);
        
        if let Some(login_info) = rsp_user_login {
            let trading_day = String::from_utf8_lossy(&login_info.TradingDay);
            let login_time = String::from_utf8_lossy(&login_info.LoginTime);
            let broker_id = String::from_utf8_lossy(&login_info.BrokerID);
            let user_id = String::from_utf8_lossy(&login_info.UserID);
            
            info!(
                "Sina MD Logged in: Trading Day = {}, Login Time = {}, Broker ID = {}, User ID = {} - XCTP回调：登录成功",
                trading_day, login_time, broker_id, user_id
            );
            
            self.actor_addr.do_send(MarketDataEvent::LoggedIn);
        } else if let Some(error) = result.err() {
            let error_msg = format!(
                "Sina MD Login failed: Error = {} - XCTP回调：登录失败",
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
        info!("XCTP回调：Sina订阅行情响应 RequestID={}, IsLast={}", request_id, is_last);
        
        if let Some(instrument) = specific_instrument {
            let instrument_id = String::from_utf8_lossy(&instrument.InstrumentID)
                .trim_end_matches('\0')
                .to_string();

            if result.is_ok() {
                info!("Sina Subscribed to market data for {} - XCTP回调：订阅行情成功", instrument_id);
                
                // 保存订阅
                if let Ok(mut subscribed) = self.subscribed_instruments.lock() {
                    subscribed.insert(instrument_id.clone());
                }
                
                self.actor_addr.do_send(MarketDataEvent::SubscriptionSuccess(instrument_id));
            } else if let Some(error) = result.err() {
                let error_msg = format!(
                    "Sina Failed to subscribe to market data for {}: Error = {} - XCTP回调：订阅行情失败",
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
        info!("Sina on_rtn_depth_market_data depth_market_data received");
        if let Some(market_data) = depth_market_data {
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
        info!("XCTP回调：Sina取消订阅行情响应 RequestID={}, IsLast={}", request_id, is_last);
        
        if let Some(instrument) = specific_instrument {
            let instrument_id = String::from_utf8_lossy(&instrument.InstrumentID)
                .trim_end_matches('\0')
                .to_string();

            if result.is_ok() {
                info!("Sina Unsubscribed from market data for {} - XCTP回调：取消订阅行情成功", instrument_id);
                
                // 移除订阅
                if let Ok(mut subscribed) = self.subscribed_instruments.lock() {
                    subscribed.remove(&instrument_id);
                }
            } else if let Some(error) = result.err() {
                error!(
                    "Sina Failed to unsubscribe from market data for {}: Error = {} - XCTP回调：取消订阅行情失败",
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
                "Sina CTP error: Request ID = {}, Is Last = {}, Error = {} - XCTP回调：错误响应",
                request_id, is_last, error
            );
            error!("{}", error_msg);
            self.actor_addr.do_send(MarketDataEvent::Error(error_msg));
        }
    }
}

/// Sina行情数据Actor
pub struct SinaMarketDataActor {
    /// Sina行情API实例
    md_api: Option<MdApi>,
    /// 已订阅的合约集合
    subscribed_instruments: Arc<Mutex<HashSet<String>>>,
    /// 行情分发器地址
    distributor: Option<Addr<crate::actors::md_distributor::MarketDataDistributor>>,
    /// 前置地址
    front_addr: String,
    /// 用户ID (可能为空)
    user_id: String,
    /// 密码 (可能为空)
    password: String,
    /// 经纪商ID (可能为空)
    broker_id: String,
    /// 是否已连接
    is_connected: bool,
    /// 是否已登录
    is_logged_in: bool,
}

impl Actor for SinaMarketDataActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("SinaMarketDataActor started");
        
        // Initialize API right away
        self.init_md_api(ctx);
        
        // Try to login immediately and then again after a delay to ensure we're connected
        ctx.run_later(Duration::from_secs(2), |act, _| {
            if !act.is_logged_in {
                info!("SinaMarketDataActor delayed login attempt");
                if let Err(e) = act.login() {
                    error!("Sina Failed to login during startup: {}", e);
                }
            }
        });
        
        // 定期检查连接状态
        ctx.run_interval(Duration::from_secs(30), |act, ctx| {
            if !act.is_connected {
                info!("SinaMarketDataActor heartbeat: Not connected, attempting to reconnect");
                act.init_md_api(ctx);
            } else if !act.is_logged_in {
                info!("SinaMarketDataActor heartbeat: Connected but not logged in, attempting to login");
                if let Err(e) = act.login() {
                    error!("Sina Failed to login during heartbeat: {}", e);
                }
            }
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("SinaMarketDataActor stopped");
    }
}

impl SinaMarketDataActor {
    /// 创建新的Sina行情Actor
    pub fn new(config: BrokerConfig) -> Self {
        let front_addr = config.front_addr.clone();
        let user_id = config.user_id.clone();
        let password = config.password.clone();
        let broker_id = config.broker_id.clone();
        
        Self {
            md_api: None,
            subscribed_instruments: Arc::new(Mutex::new(HashSet::new())),
            distributor: None,
            front_addr,
            user_id,
            password,
            broker_id,
            is_connected: false,
            is_logged_in: false,
        }
    }

    /// 初始化Sina行情API
    fn init_md_api(&mut self, ctx: &mut Context<Self>) {
        // 创建数据流路径
        let flow_path = CString::new("./data_sina_md_flow").unwrap();
        
        // 创建Sina行情API
        let mut md_api = MdApi::new(flow_path, false, false);
        
        // 创建SPI
        let addr = ctx.address();
        let subscribed_instruments = self.subscribed_instruments.clone();
        let spi = Box::new(SinaMdSpiImpl {
            actor_addr: addr,
            subscribed_instruments,
        });
        
        // 注册SPI
        md_api.register_spi(spi);
        
        // 连接前置
        let front_addr = CString::new(self.front_addr.clone()).unwrap();
        md_api.register_front(front_addr);
        
        // 初始化API
        md_api.init();
        
        // 保存API实例
        self.md_api = Some(md_api);
    }

    /// 登录到行情服务器
    fn login(&mut self) -> Result<(), String> {
        if let Some(ref mut md_api) = self.md_api {
            // 使用默认登录请求，简化登录过程
            let result = md_api.req_user_login(&Default::default(), 1);
            
            match result {
                Ok(_) => {
                    info!("Sina login request sent");
                    Ok(())
                },
                Err(e) => {
                    let error_msg = format!("Sina Failed to send login request: {:?}", e);
                    error!("{}", error_msg);
                    Err(error_msg)
                }
            }
        } else {
            Err("Sina Market data API not initialized".to_string())
        }
    }

    /// 订阅合约
    fn subscribe_instruments(&mut self, instruments: &[String]) -> Result<(), String> {
        if !self.is_logged_in {
            return Err("Sina Not logged in".to_string());
        }

        if let Some(ref mut md_api) = self.md_api {
            // 将合约ID转换为CString
            let instrument_cstrings: Vec<CString> = instruments
                .iter()
                .map(|s| {
                    // 股票代码可能不含交易所前缀，需要处理
                    let instrument_code = s.split('.').last().unwrap_or(s);
                    
                    // 对于纯数字的股票代码，检查长度并可能添加前导零
                    let code = if instrument_code.chars().all(char::is_numeric) && instrument_code.len() <= 6 {
                        // 确保股票代码长度为6位
                        format!("{:0>6}", instrument_code)
                    } else {
                        instrument_code.to_string()
                    };

                    info!("Sina Subscribing to instrument: {}", code);
                    CString::new(code).unwrap()
                })
                .collect();
            
            info!("Sina Subscribing to instruments: {:?}", instruments);
            info!("Sina Converted instrument codes: {:?}", instrument_cstrings);
            
            // 订阅所有合约
            let result = md_api.subscribe_market_data(&instrument_cstrings);
            match result {
                Ok(_) => {
                    info!("Sina subscribe_market_data request sent");
                    Ok(())
                },
                Err(e) => Err(format!("Sina Failed to subscribe to instruments, error: {:?}", e))
            }
        } else {
            Err("Sina Market data API not initialized".to_string())
        }
    }

    /// 取消订阅合约
    fn unsubscribe_instruments(&mut self, instruments: &[String]) -> Result<(), String> {
        if !self.is_logged_in {
            return Err("Sina Not logged in".to_string());
        }

        if let Some(ref mut md_api) = self.md_api {
            // 将合约ID转换为CString
            let instrument_cstrings: Vec<CString> = instruments
                .iter()
                .map(|s| {
                    // 股票代码可能不含交易所前缀，需要处理
                    let instrument_code = s.split('.').last().unwrap_or(s);
                    
                    // 对于纯数字的股票代码，检查长度并可能添加前导零
                    let code = if instrument_code.chars().all(char::is_numeric) && instrument_code.len() <= 6 {
                        // 确保股票代码长度为6位
                        format!("{:0>6}", instrument_code)
                    } else {
                        instrument_code.to_string()
                    };
                    
                    CString::new(code).unwrap()
                })
                .collect();
            
            // 取消订阅
            let result = md_api.unsubscribe_market_data(&instrument_cstrings);
            
            match result {
                Ok(_) => {
                    info!("Sina unsubscribe_market_data request sent");
                    Ok(())
                },
                Err(e) => Err(format!("Sina Failed to unsubscribe from instruments, error: {:?}", e))
            }
        } else {
            Err("Sina MD API not initialized".to_string())
        }
    }
}

// 实现消息处理程序
impl Handler<InitMarketDataSource> for SinaMarketDataActor {
    type Result = ();

    fn handle(&mut self, _: InitMarketDataSource, ctx: &mut Self::Context) -> Self::Result {
        self.init_md_api(ctx);
    }
}

impl Handler<LoginMarketDataSource> for SinaMarketDataActor {
    type Result = Result<(), String>;

    fn handle(&mut self, _: LoginMarketDataSource, _: &mut Self::Context) -> Self::Result {
        self.login()
    }
}

impl Handler<Subscribe> for SinaMarketDataActor {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) -> Self::Result {
        if let Err(e) = self.subscribe_instruments(&msg.instruments) {
            error!("Sina Failed to subscribe to instruments: {}", e);
        }
    }
}

impl Handler<Unsubscribe> for SinaMarketDataActor {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _: &mut Self::Context) -> Self::Result {
        if let Err(e) = self.unsubscribe_instruments(&msg.instruments) {
            error!("Sina Failed to unsubscribe from instruments: {}", e);
        }
    }
}

impl Handler<GetSubscriptions> for SinaMarketDataActor {
    type Result = Vec<String>;

    fn handle(&mut self, msg: GetSubscriptions, _: &mut Self::Context) -> Self::Result {
        let subscriptions = if let Ok(subscribed) = self.subscribed_instruments.lock() {
            subscribed.iter().cloned().collect()
        } else {
            Vec::new()
        };
        
        // 如果提供了回调，执行它
        if let Some(callback) = msg.callback {
            callback(subscriptions.clone());
        }
        
        subscriptions
    }
}

impl Handler<MarketDataEvent> for SinaMarketDataActor {
    type Result = ();

    fn handle(&mut self, msg: MarketDataEvent, _: &mut Self::Context) -> Self::Result {
        match msg {
            MarketDataEvent::Connected => {
                info!("Sina Market data source connected");
                self.is_connected = true;
                
                // 自动登录
                if let Err(e) = self.login() {
                    error!("Sina Failed to login: {}", e);
                }
            },
            MarketDataEvent::Disconnected => {
                warn!("Sina Market data source disconnected");
                self.is_connected = false;
                self.is_logged_in = false;
            },
            MarketDataEvent::LoggedIn => {
                info!("Sina Market data source logged in");
                self.is_logged_in = true;
                
                // 重新订阅所有合约
                let instruments = {
                    if let Ok(subscribed) = self.subscribed_instruments.lock() {
                        subscribed.iter().cloned().collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    }
                };
                
                if !instruments.is_empty() {
                    if let Err(e) = self.subscribe_instruments(&instruments) {
                        error!("Sina Failed to resubscribe to instruments: {}", e);
                    }
                }
            },
            MarketDataEvent::MarketData(md) => {
                // Convert to MDSnapshot
                debug!("Sina Received market data");
                match convert_ctp_to_md_snapshot(&md) {
                    Ok(snapshot) => {
                        debug!("Sina Received market data for {}", snapshot.instrument_id);

                        // Forward to distributor
                        if let Some(distributor) = &self.distributor {
                            distributor.do_send(MarketDataUpdate(snapshot, MarketDataSource::Sina));
                        }
                    },
                    Err(e) => {
                        println!("Failed to convert Sina market data: {}", e);
                    }
                }
            },
            MarketDataEvent::SubscriptionSuccess(instrument) => {
                info!("Sina Successfully subscribed to {}", instrument);
            },
            MarketDataEvent::SubscriptionFailure(instrument, error) => {
                error!("Sina Failed to subscribe to {}: {}", instrument, error);
            },
            MarketDataEvent::Error(error) => {
                error!("Sina Market data error: {}", error);
            },
        }
    }
}

impl Handler<RegisterDistributor> for SinaMarketDataActor {
    type Result = ();

    fn handle(&mut self, msg: RegisterDistributor, _: &mut Self::Context) -> Self::Result {
        self.distributor = Some(msg.addr);
        info!("Sina Market data distributor registered");
    }
}

impl Handler<StartMarketData> for SinaMarketDataActor {
    type Result = ();

    fn handle(&mut self, msg: StartMarketData, ctx: &mut Self::Context) -> Self::Result {
        // 如果未初始化，则初始化
        if self.md_api.is_none() {
            self.init_md_api(ctx);
        }
        
        // 订阅合约
        if !msg.instruments.is_empty() {
            if let Err(e) = self.subscribe_instruments(&msg.instruments) {
                error!("Sina Failed to subscribe to initial instruments: {}", e);
            }
        }
    }
}

impl Handler<StopMarketData> for SinaMarketDataActor {
    type Result = ();

    fn handle(&mut self, _: StopMarketData, _: &mut Self::Context) -> Self::Result {
        // 取消订阅所有合约
        let instruments = {
            if let Ok(subscribed) = self.subscribed_instruments.lock() {
                subscribed.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };
        
        if !instruments.is_empty() {
            if let Err(e) = self.unsubscribe_instruments(&instruments) {
                error!("Sina Failed to unsubscribe from instruments: {}", e);
            }
        }
    }
}

impl Handler<RestartActor> for SinaMarketDataActor {
    type Result = ();

    fn handle(&mut self, _: RestartActor, ctx: &mut Self::Context) -> Self::Result {
        // 只有在未连接或未登录状态下才重启
        if !self.is_connected || !self.is_logged_in {
            info!("Sina Restarting market data actor for broker {}", self.broker_id);
            
            // 重新初始化
            if self.md_api.is_none() {
                self.init_md_api(ctx);
            }
            
            // 尝试重新登录
            if let Err(e) = self.login() {
                error!("Sina Failed to login during restart: {}", e);
            }
        }
    }
} 