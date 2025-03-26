use actix::prelude::*;
use ctp_common::{CThostFtdcDepthMarketDataField, CThostFtdcReqUserLoginField, CThostFtdcSpecificInstrumentField};
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// 统一导入消息类型
use crate::actors::messages::*;
use crate::config::BrokerConfig;
use crate::converter::convert_ctp_to_md_snapshot;
use crate::error::GatewayResult;

// 特性标志条件导入
#[cfg(feature = "ctp")]
use ctp_md::{DisconnectionReason, MdApi, MdSpi, RspResult, GenericMdApi};
#[cfg(feature = "qq")]
use ctp_md_qq::{DisconnectionReason, MdApi, MdSpi, RspResult, GenericMdApi};
#[cfg(feature = "sina")]
use ctp_md_sina::{DisconnectionReason, MdApi, MdSpi, RspResult, GenericMdApi};

// 统一的SPI实现，用于回调处理
struct MarketDataSpiImpl {
    // 使用actor的地址将消息从CTP回调发送回actor
    actor_addr: Addr<MarketDataActor>,
    subscribed_instruments: Arc<Mutex<HashSet<String>>>,
}

// SPI接口实现，处理所有回调
impl MdSpi for MarketDataSpiImpl {
    fn on_front_connected(&mut self) {
        info!("MD Front connected");
        self.actor_addr.do_send(MarketDataEvent::Connected);
    }

    fn on_front_disconnected(&mut self, reason: DisconnectionReason) {
        warn!("MD Front disconnected: {:?}", reason);
        self.actor_addr.do_send(MarketDataEvent::Disconnected);
    }

    fn on_rsp_user_login(
        &mut self,
        rsp_user_login: Option<&ctp_common::CThostFtdcRspUserLoginField>,
        result: RspResult,
        request_id: i32,
        is_last: bool,
    ) {
        info!("Login response: RequestID={}, IsLast={}", request_id, is_last);
        
        if let Some(login_info) = rsp_user_login {
            let trading_day = String::from_utf8_lossy(&login_info.TradingDay);
            let login_time = String::from_utf8_lossy(&login_info.LoginTime);
            let broker_id = String::from_utf8_lossy(&login_info.BrokerID);
            let user_id = String::from_utf8_lossy(&login_info.UserID);
            
            info!(
                "MD Logged in: Trading Day = {}, Login Time = {}, Broker ID = {}, User ID = {}",
                trading_day, login_time, broker_id, user_id
            );
            
            self.actor_addr.do_send(MarketDataEvent::LoggedIn);
        } else if let Some(error) = result.err() {
            let error_msg = format!(
                "MD Login failed: Error = {}",
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
        info!("Subscribe response: RequestID={}, IsLast={}", request_id, is_last);
        
        if let Some(instrument) = specific_instrument {
            let instrument_id = String::from_utf8_lossy(&instrument.InstrumentID)
                .trim_end_matches('\0')
                .to_string();

            if result.is_ok() {
                info!("Subscribed to market data for {}", instrument_id);
                
                // 保存订阅信息
                if let Ok(mut subscribed) = self.subscribed_instruments.lock() {
                    subscribed.insert(instrument_id.clone());
                }
                
                self.actor_addr.do_send(MarketDataEvent::SubscriptionSuccess(instrument_id));
            } else if let Some(error) = result.err() {
                let error_msg = format!(
                    "Failed to subscribe to market data for {}: Error = {}",
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
            // 将数据克隆后发送给actor
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
        info!("Unsubscribe response: RequestID={}, IsLast={}", request_id, is_last);
        
        if let Some(instrument) = specific_instrument {
            let instrument_id = String::from_utf8_lossy(&instrument.InstrumentID)
                .trim_end_matches('\0')
                .to_string();

            if result.is_ok() {
                info!("Unsubscribed from market data for {}", instrument_id);
                
                // 移除订阅信息
                if let Ok(mut subscribed) = self.subscribed_instruments.lock() {
                    subscribed.remove(&instrument_id);
                }
            } else if let Some(error) = result.err() {
                error!(
                    "Failed to unsubscribe from market data for {}: Error = {}",
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
                "MD error: Request ID = {}, Is Last = {}, Error = {}",
                request_id, is_last, error
            );
            error!("{}", error_msg);
            self.actor_addr.do_send(MarketDataEvent::Error(error_msg));
        }
    }
}

// 统一的MarketDataActor结构，通过feature flags选择实际的API实现
pub struct MarketDataActor {
    #[cfg(feature = "ctp")]
    md_api: Option<ctp_md::MdApi>,
    #[cfg(feature = "qq")]
    md_api: Option<ctp_md_qq::MdApi>,
    #[cfg(feature = "sina")]
    md_api: Option<ctp_md_sina::MdApi>,
    #[cfg(not(any(feature = "ctp", feature = "qq", feature = "sina")))]
    md_api: Option<()>, // 当没有特性被启用时的占位符
    
    subscribed_instruments: Arc<Mutex<HashSet<String>>>,
    broker_config: BrokerConfig,
    distributor: Option<Addr<crate::actors::md_distributor::MarketDataDistributor>>,
    front_addr: String,
    user_id: String,
    password: String,
    broker_id: String,
    is_connected: bool,
    is_logged_in: bool,
    
    // 数据源类型(便于标识)
    #[cfg(feature = "ctp")]
    source_type: MarketDataSource,
    #[cfg(feature = "qq")]
    source_type: MarketDataSource,
    #[cfg(feature = "sina")]
    source_type: MarketDataSource,
    #[cfg(not(any(feature = "ctp", feature = "qq", feature = "sina")))]
    source_type: MarketDataSource,
}

impl Actor for MarketDataActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("MarketDataActor started");
        
        // 调度心跳以检查连接状态
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
    // 创建新的市场数据Actor，可根据编译时特性决定具体行为
    #[cfg(feature = "ctp")]
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
            source_type: MarketDataSource::CTP,
        }
    }

    #[cfg(feature = "qq")]
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
            source_type: MarketDataSource::QQ,
        }
    }

    #[cfg(feature = "sina")]
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
            source_type: MarketDataSource::Sina,
        }
    }

    #[cfg(not(any(feature = "ctp", feature = "qq", feature = "sina")))]
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
            source_type: MarketDataSource::CTP, // 默认值
        }
    }

    // 初始化市场数据API，根据编译时特性选择不同实现
    fn init_md_api(&mut self, ctx: &mut Context<Self>) {
        let flow_path = CString::new("").unwrap();
        
        #[cfg(feature = "ctp")]
        {
            // 创建CTP的MdApi
            let mut md_api = ctp_md::MdApi::new(flow_path, false, false);
            
            // 创建SPI并注册
            let addr = ctx.address();
            let subscribed_instruments = self.subscribed_instruments.clone();
            let spi = Box::new(MarketDataSpiImpl {
                actor_addr: addr,
                subscribed_instruments,
            });
            
            md_api.register_spi(spi);
            
            // 连接
            let front_addr = CString::new(self.front_addr.clone()).unwrap();
            md_api.register_front(front_addr);
            
            // 初始化API
            md_api.init();
            std::thread::sleep(std::time::Duration::from_secs(1));
            
            // 保存API
            self.md_api = Some(md_api);
        }
        
        #[cfg(feature = "qq")]
        {
            // 创建QQ的MdApi
            let mut md_api = ctp_md_qq::MdApi::new(flow_path, false, false);
            
            // 创建SPI并注册
            let addr = ctx.address();
            let subscribed_instruments = self.subscribed_instruments.clone();
            let spi = Box::new(MarketDataSpiImpl {
                actor_addr: addr,
                subscribed_instruments,
            });
            
            md_api.register_spi(spi);
            
            // 连接
            let front_addr = CString::new(self.front_addr.clone()).unwrap();
            md_api.register_front(front_addr);
            
            // 初始化API
            md_api.init();
            std::thread::sleep(std::time::Duration::from_secs(1));
            
            // 保存API
            self.md_api = Some(md_api);
        }
        
        #[cfg(feature = "sina")]
        {
            // 创建Sina的MdApi
            let mut md_api = ctp_md_sina::MdApi::new(flow_path, false, false);
            
            // 创建SPI并注册
            let addr = ctx.address();
            let subscribed_instruments = self.subscribed_instruments.clone();
            let spi = Box::new(MarketDataSpiImpl {
                actor_addr: addr,
                subscribed_instruments,
            });
            
            md_api.register_spi(spi);
            
            // 连接
            let front_addr = CString::new(self.front_addr.clone()).unwrap();
            md_api.register_front(front_addr);
            
            // 初始化API
            md_api.init();
            std::thread::sleep(std::time::Duration::from_secs(1));
            
            // 保存API
            self.md_api = Some(md_api);
        }
    }

    // 登录方法，根据编译时特性选择不同实现
    fn login(&mut self) -> Result<(), String> {
        #[cfg(any(feature = "ctp", feature = "qq", feature = "sina"))]
        if let Some(ref mut md_api) = self.md_api {
            let mut req = CThostFtdcReqUserLoginField::default();
            
            // 填充登录请求
            if !self.broker_id.is_empty() {
                let broker_bytes = self.broker_id.as_bytes();
                let copy_len = std::cmp::min(broker_bytes.len(), req.BrokerID.len() - 1);
                req.BrokerID[..copy_len].copy_from_slice(&broker_bytes[..copy_len]);
                req.BrokerID[copy_len] = 0; // 空终止符
            }
            
            if !self.user_id.is_empty() {
                let user_bytes = self.user_id.as_bytes();
                let copy_len = std::cmp::min(user_bytes.len(), req.UserID.len() - 1);
                req.UserID[..copy_len].copy_from_slice(&user_bytes[..copy_len]);
                req.UserID[copy_len] = 0;
            }
            
            if !self.password.is_empty() {
                let pass_bytes = self.password.as_bytes();
                let copy_len = std::cmp::min(pass_bytes.len(), req.Password.len() - 1);
                req.Password[..copy_len].copy_from_slice(&pass_bytes[..copy_len]);
                req.Password[copy_len] = 0;
            }
            
            // 执行登录
            let result = md_api.req_user_login(&req, 1);
            
            match result {
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    Ok(())
                },
                Err(e) => {
                    let error_msg = format!("Failed to send login request: {:?}", e);
                    error!("{}", error_msg);
                    Err(error_msg)
                }
            }
        } else {
            Err("Market data API not initialized".to_string())
        }

        #[cfg(not(any(feature = "ctp", feature = "qq", feature = "sina")))]
        Err("No market data provider enabled".to_string())
    }

    // 订阅合约方法
    fn subscribe_instruments(&mut self, instruments: &[String]) -> Result<(), String> {
        if !self.is_logged_in {
            return Err("Not logged in".to_string());
        }

        #[cfg(any(feature = "ctp", feature = "qq", feature = "sina"))]
        if let Some(ref mut md_api) = self.md_api {
            // 将合约ID转换为CString
            let instrument_cstrings: Vec<CString> = instruments
                .iter()
                .map(|s| {
                    // 股票代码可能不含交易所前缀，需要处理
                    let instrument_code = s.split('.').last().unwrap_or(s);
                    let code = instrument_code.to_string();
                    info!("Subscribing to instrument: {}", code);
                    CString::new(code).unwrap()
                })
                .collect();
                
            // 执行订阅
            let result = md_api.subscribe_market_data(&instrument_cstrings);
            
            match result {
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    Ok(())
                },
                Err(e) => Err(format!("Failed to subscribe to instruments, error: {:?}", e))
            }
        } else {
            Err("MD API not initialized".to_string())
        }

        #[cfg(not(any(feature = "ctp", feature = "qq", feature = "sina")))]
        Err("No market data provider enabled".to_string())
    }

    // 取消订阅合约方法
    fn unsubscribe_instruments(&mut self, instruments: &[String]) -> Result<(), String> {
        if !self.is_logged_in {
            return Err("Not logged in".to_string());
        }

        #[cfg(any(feature = "ctp", feature = "qq", feature = "sina"))]
        if let Some(ref mut md_api) = self.md_api {
            // 将合约ID转换为CString
            let instrument_cstrings: Vec<CString> = instruments
                .iter()
                .map(|s| {
                    let instrument_code = s.split('.').last().unwrap_or(s);
                    CString::new(instrument_code.to_string()).unwrap()
                })
                .collect();
            
            // 执行取消订阅
            let result = md_api.unsubscribe_market_data(&instrument_cstrings);
            
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to unsubscribe from instruments, error: {:?}", e))
            }
        } else {
            Err("MD API not initialized".to_string())
        }

        #[cfg(not(any(feature = "ctp", feature = "qq", feature = "sina")))]
        Err("No market data provider enabled".to_string())
    }
}

// 统一的Actor消息处理实现
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
        
        // 如果提供了回调，则执行回调
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
                
                // 连接后自动登录
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
                        error!("Failed to resubscribe to instruments: {}", e);
                    }
                }
            },
            MarketDataEvent::MarketData(md) => {
                // 转换为MDSnapshot
                match convert_ctp_to_md_snapshot(&md) {
                    Ok(snapshot) => {
                        debug!("Received market data for {}", snapshot.instrument_id);
                        // 转发给distributor
                        if let Some(distributor) = &self.distributor {
                            #[cfg(feature = "ctp")]
                            distributor.do_send(MarketDataUpdate(snapshot, MarketDataSource::CTP));
                            
                            #[cfg(feature = "qq")]
                            distributor.do_send(MarketDataUpdate(snapshot, MarketDataSource::QQ));
                            
                            #[cfg(feature = "sina")]
                            distributor.do_send(MarketDataUpdate(snapshot, MarketDataSource::Sina));
                            
                            #[cfg(not(any(feature = "ctp", feature = "qq", feature = "sina")))]
                            distributor.do_send(MarketDataUpdate(snapshot, MarketDataSource::CTP));
                        }
                    },
                    Err(e) => {
                        error!("Failed to convert market data: {}", e);
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
        // 如果API未初始化，则初始化
        if self.md_api.is_none() {
            self.init_md_api(ctx);
        }
        
        // 订阅合约
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
                error!("Failed to unsubscribe from instruments: {}", e);
            }
        }
    }
}

impl Handler<RestartActor> for MarketDataActor {
    type Result = ();

    fn handle(&mut self, _: RestartActor, ctx: &mut Self::Context) -> Self::Result {
        // 只有未连接或未登录时才重启
        if !self.is_connected || !self.is_logged_in {
            info!("Restarting market data actor for broker {}", self.broker_id);
            
            // 重新初始化
            if self.md_api.is_none() {
                self.init_md_api(ctx);
            }
            
            // 尝试重新登录
            if let Err(e) = self.login() {
                error!("Failed to login during restart: {}", e);
            }
        }
    }
} 