use actix::prelude::*;
use hashbrown::{HashMap, HashSet};
use log::{debug, error, info, warn};
use serde_json::json;
use uuid;

use crate::actors::messages::*;
use qamd_rs::{MDSnapshot, OptionalF64};

/// 市场数据分发器
/// 
/// 负责接收来自不同市场数据源的行情数据，
/// 并根据客户端订阅将数据转发给对应的接收者
pub struct MarketDataDistributor {
    // 保存客户端及其订阅关系
    subscribers: HashMap<String, Subscriber>,
    
    // 保存合约订阅关系 (合约ID -> 订阅客户端集合)
    instrument_subscribers: HashMap<String, HashSet<String>>,
    
    // 保存不同市场数据源的Actor地址
    #[cfg(feature = "ctp")]
    ctp_actors: HashMap<String, Addr<crate::actors::md_actor::MarketDataActor>>,
    
    #[cfg(feature = "qq")]
    qq_actors: HashMap<String, Addr<crate::actors::md_actor::MarketDataActor>>,
    
    #[cfg(feature = "sina")]
    sina_actors: HashMap<String, Addr<crate::actors::md_actor::MarketDataActor>>,
    
    // 最新的市场数据缓存 (合约ID -> 行情数据)
    market_data_cache: HashMap<String, qamd_rs::MDSnapshot>,
    
    // 来源标记 (合约ID -> 市场数据源)
    source_map: HashMap<String, MarketDataSource>,
}

/// 订阅者信息
struct Subscriber {
    // 客户端地址
    addr: Recipient<MarketDataUpdateMessage>,
    // 订阅的合约集合
    instruments: HashSet<String>,
}

impl Actor for MarketDataDistributor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("MarketDataDistributor started");
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("MarketDataDistributor stopped");
    }
}

impl Default for MarketDataDistributor {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketDataDistributor {
    /// 创建一个新的市场数据分发器
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
            instrument_subscribers: HashMap::new(),
            #[cfg(feature = "ctp")]
            ctp_actors: HashMap::new(),
            #[cfg(feature = "qq")]
            qq_actors: HashMap::new(),
            #[cfg(feature = "sina")]
            sina_actors: HashMap::new(),
            market_data_cache: HashMap::new(),
            source_map: HashMap::new(),
        }
    }

    /// 添加订阅
    fn add_subscription(&mut self, client_id: &str, instruments: &[String]) {
        // Collect instruments with cached data for later use
        let mut instruments_with_data = Vec::new();
        
        if let Some(subscriber) = self.subscribers.get_mut(client_id) {
            // 更新现有订阅者的订阅
            for instrument in instruments {
                subscriber.instruments.insert(instrument.clone());
                
                // 更新合约订阅关系
                self.instrument_subscribers
                    .entry(instrument.clone())
                    .or_insert_with(HashSet::new)
                    .insert(client_id.to_string());
                
                // 缓存合约列表和数据，稍后发送
                if let Some(data) = self.market_data_cache.get(instrument) {
                    instruments_with_data.push((instrument.clone(), data.clone()));
                }
            }
        }
        
        // 发送缓存的行情数据
        for (instrument, data) in instruments_with_data {
            self.send_market_data_to_client(client_id, &instrument, &data);
        }
    }

    /// 删除订阅
    fn remove_subscription(&mut self, client_id: &str, instruments: &[String]) {
        if let Some(subscriber) = self.subscribers.get_mut(client_id) {
            // 从订阅者中移除订阅
            for instrument in instruments {
                subscriber.instruments.remove(instrument);
                
                // 更新合约订阅关系
                if let Some(subscribers) = self.instrument_subscribers.get_mut(instrument) {
                    subscribers.remove(client_id);
                    
                    // 如果没有订阅者了，则考虑取消订阅该合约（从行情源）
                    if subscribers.is_empty() {
                        self.instrument_subscribers.remove(instrument);
                        
                        // 根据数据来源取消订阅合约
                        if let Some(source) = self.source_map.get(instrument) {
                            match source {
                                #[cfg(feature = "ctp")]
                                MarketDataSource::CTP => {
                                    for (_, actor) in &self.ctp_actors {
                                        actor.do_send(Unsubscribe {
                                            id: uuid::Uuid::nil(),
                                            instruments: vec![instrument.clone()],
                                        });
                                    }
                                },
                                #[cfg(feature = "qq")]
                                MarketDataSource::QQ => {
                                    for (_, actor) in &self.qq_actors {
                                        actor.do_send(Unsubscribe {
                                            id: uuid::Uuid::nil(),
                                            instruments: vec![instrument.clone()],
                                        });
                                    }
                                },
                                #[cfg(feature = "sina")]
                                MarketDataSource::Sina => {
                                    for (_, actor) in &self.sina_actors {
                                        actor.do_send(Unsubscribe {
                                            id: uuid::Uuid::nil(),
                                            instruments: vec![instrument.clone()],
                                        });
                                    }
                                },
                                #[allow(unreachable_patterns)]
                                _ => {
                                    warn!("Unknown market data source for instrument {}", instrument);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// 向客户端发送市场数据
    fn send_market_data_to_client(&self, client_id: &str, instrument: &str, data: &qamd_rs::MDSnapshot) {
        if let Some(subscriber) = self.subscribers.get(client_id) {
            // 检查是否订阅了该合约
            if subscriber.instruments.contains(instrument) {
                // 将市场数据转换为JSON字符串
                let data_json = json!({
                    "instrument_id": data.instrument_id.clone(),
                    "last_price": data.last_price,
                    "pre_settlement": data.pre_settlement,
                    "pre_close": data.pre_close,
                    "pre_open_interest": data.pre_open_interest,
                    "open": data.open,
                    "highest": data.highest,
                    "lowest": data.lowest,
                    "volume": data.volume,
                    "amount": data.amount,
                    "open_interest": data.open_interest,
                    "close": data.close,
                    "settlement": data.settlement,
                    "upper_limit": data.upper_limit,
                    "lower_limit": data.lower_limit,
                    "bid_price1": data.bid_price1,
                    "bid_volume1": data.bid_volume1,
                    "ask_price1": data.ask_price1,
                    "ask_volume1": data.ask_volume1,
                    "bid_price2": data.bid_price2,
                    "bid_volume2": data.bid_volume2,
                    "ask_price2": data.ask_price2,
                    "ask_volume2": data.ask_volume2,
                    "bid_price3": data.bid_price3,
                    "bid_volume3": data.bid_volume3,
                    "ask_price3": data.ask_price3,
                    "ask_volume3": data.ask_volume3,
                    "bid_price4": data.bid_price4,
                    "bid_volume4": data.bid_volume4,
                    "ask_price4": data.ask_price4,
                    "ask_volume4": data.ask_volume4,
                    "bid_price5": data.bid_price5,
                    "bid_volume5": data.bid_volume5,
                    "ask_price5": data.ask_price5,
                    "ask_volume5": data.ask_volume5,
                    "average": data.average,
                    "datetime": data.datetime.clone()
                });
                
                // 构建市场数据更新消息
                let mut data_map = HashMap::new();
                data_map.insert(instrument.to_string(), data_json.to_string());
                
                let message = MarketDataUpdateMessage {
                    instruments: vec![instrument.to_string()],
                    data: data_map,
                };
                
                // 发送给订阅者
                match subscriber.addr.try_send(message) {
                    Err(e) => error!("Failed to send market data to client {}: {}", client_id, e),
                    _ => {}
                }
            }
        }
    }

    /// 发送市场数据更新
    fn broadcast_market_data(&self, data: &qamd_rs::MDSnapshot) {
        let instrument = &data.instrument_id;
        
        // 获取订阅该合约的客户端列表
        if let Some(subscribers) = self.instrument_subscribers.get(instrument) {
            for client_id in subscribers {
                self.send_market_data_to_client(client_id, instrument, data);
            }
        }
    }

    /// 查找合适的Actor处理订阅请求
    fn find_actor_for_instrument(&self, instrument: &str) -> Option<(Addr<crate::actors::md_actor::MarketDataActor>, MarketDataSource)> {
        // 首先检查该合约是否已经有数据源
        if let Some(source) = self.source_map.get(instrument) {
            match source {
                #[cfg(feature = "ctp")]
                MarketDataSource::CTP => {
                    if let Some((_, actor)) = self.ctp_actors.iter().next() {
                        return Some((actor.clone(), MarketDataSource::CTP));
                    }
                },
                #[cfg(feature = "qq")]
                MarketDataSource::QQ => {
                    if let Some((_, actor)) = self.qq_actors.iter().next() {
                        return Some((actor.clone(), MarketDataSource::QQ));
                    }
                },
                #[cfg(feature = "sina")]
                MarketDataSource::Sina => {
                    if let Some((_, actor)) = self.sina_actors.iter().next() {
                        return Some((actor.clone(), MarketDataSource::Sina));
                    }
                },
                #[allow(unreachable_patterns)]
                _ => {}
            }
        }
        
        // 简化：由于构建时只会启用一个feature，直接返回对应类型的第一个actor即可
        
        #[cfg(feature = "ctp")]
        if let Some((_, actor)) = self.ctp_actors.iter().next() {
            return Some((actor.clone(), MarketDataSource::CTP));
        }
        
        #[cfg(feature = "qq")]
        if let Some((_, actor)) = self.qq_actors.iter().next() {
            return Some((actor.clone(), MarketDataSource::QQ));
        }
        
        #[cfg(feature = "sina")]
        if let Some((_, actor)) = self.sina_actors.iter().next() {
            return Some((actor.clone(), MarketDataSource::Sina));
        }
        
        // 没有找到合适的数据源
        warn!("No suitable market data actor found for instrument: {}", instrument);
        None
    }
}

// 处理市场数据更新消息
impl Handler<MarketDataUpdate> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: MarketDataUpdate, _: &mut Self::Context) -> Self::Result {
        let (data, source) = (msg.0, msg.1);
        let instrument = data.instrument_id.clone();
        
        // 更新缓存
        self.market_data_cache.insert(instrument.clone(), data.clone());
        self.source_map.insert(instrument.clone(), source);
        
        // 广播市场数据
        self.broadcast_market_data(&data);
    }
}

// 处理客户端注册消息
impl Handler<RegisterDataReceiver> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: RegisterDataReceiver, _: &mut Self::Context) -> Self::Result {
        let client_id = msg.client_id.clone();
        
        // 创建新的订阅者
        let subscriber = Subscriber {
            addr: msg.addr,
            instruments: HashSet::new(),
        };
        
        // 保存订阅者信息
        self.subscribers.insert(client_id.clone(), subscriber);
        
        // 添加订阅
        if !msg.instruments.is_empty() {
            self.add_subscription(&client_id, &msg.instruments);
            
            // 处理每个合约的订阅
            for instrument in &msg.instruments {
                // 查找合适的Actor处理订阅请求
                if let Some((actor, source)) = self.find_actor_for_instrument(instrument) {
                    // 记录数据源
                    self.source_map.insert(instrument.clone(), source);
                    
                    // 发送订阅请求
                    actor.do_send(Subscribe {
                        id: uuid::Uuid::nil(),
                        instruments: vec![instrument.clone()],
                    });
                } else {
                    warn!("No suitable market data actor found for instrument {}", instrument);
                }
            }
        }
    }
}

// 处理客户端取消注册消息
impl Handler<UnregisterDataReceiver> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: UnregisterDataReceiver, _: &mut Self::Context) -> Self::Result {
        if let Some(subscriber) = self.subscribers.remove(&msg.client_id) {
            // 获取客户端订阅的所有合约
            let instruments: Vec<String> = subscriber.instruments.into_iter().collect();
            
            // 移除订阅
            if !instruments.is_empty() {
                self.remove_subscription(&msg.client_id, &instruments);
            }
        }
    }
}

// 处理订阅更新消息
impl Handler<UpdateSubscription> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: UpdateSubscription, _: &mut Self::Context) -> Self::Result {
        if let Some(subscriber) = self.subscribers.get(&msg.client_id) {
            // 获取当前订阅的合约列表
            let current_instruments: HashSet<String> = subscriber.instruments.clone();
            
            // 计算需要添加的合约
            let new_instruments: HashSet<String> = msg.instruments.iter().cloned().collect();
            let to_add: Vec<String> = new_instruments
                .difference(&current_instruments)
                .cloned()
                .collect();
            
            // 计算需要移除的合约
            let to_remove: Vec<String> = current_instruments
                .difference(&new_instruments)
                .cloned()
                .collect();
            
            // 添加新订阅
            if !to_add.is_empty() {
                self.add_subscription(&msg.client_id, &to_add);
                
                // 处理每个合约的订阅
                for instrument in &to_add {
                    // 查找合适的Actor处理订阅请求
                    if let Some((actor, source)) = self.find_actor_for_instrument(instrument) {
                        // 记录数据源
                        self.source_map.insert(instrument.clone(), source);
                        
                        // 发送订阅请求
                        actor.do_send(Subscribe {
                            id: uuid::Uuid::nil(),
                            instruments: vec![instrument.clone()],
                        });
                    } else {
                        warn!("No suitable market data actor found for instrument {}", instrument);
                    }
                }
            }
            
            // 移除旧订阅
            if !to_remove.is_empty() {
                self.remove_subscription(&msg.client_id, &to_remove);
            }
        }
    }
}

// 处理订阅查询消息
impl Handler<QuerySubscription> for MarketDataDistributor {
    type Result = Vec<String>;

    fn handle(&mut self, msg: QuerySubscription, _: &mut Self::Context) -> Self::Result {
        if let Some(subscriber) = self.subscribers.get(&msg.client_id) {
            subscriber.instruments.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

// 处理CTP市场数据Actor注册消息
#[cfg(feature = "ctp")]
impl Handler<RegisterCTPMdActor> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: RegisterCTPMdActor, ctx: &mut Self::Context) -> Self::Result {
        // 注册CTP市场数据Actor
        let broker_id = msg.broker_id.clone();
        self.ctp_actors.insert(broker_id.clone(), msg.addr.clone());
        
        // 将分发器地址注册到Actor
        msg.addr.do_send(RegisterDistributor {
            addr: ctx.address(),
        });
        
        info!("Registered CTP market data actor for broker {}", broker_id);
    }
}

// 处理QQ市场数据Actor注册消息
#[cfg(feature = "qq")]
impl Handler<RegisterQQMdActor> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: RegisterQQMdActor, ctx: &mut Self::Context) -> Self::Result {
        // 注册QQ市场数据Actor
        let broker_id = msg.broker_id.clone();
        self.qq_actors.insert(broker_id.clone(), msg.addr.clone());
        
        // 将分发器地址注册到Actor
        msg.addr.do_send(RegisterDistributor {
            addr: ctx.address(),
        });
        
        info!("Registered QQ market data actor for broker {}", broker_id);
    }
}

// 处理Sina市场数据Actor注册消息
#[cfg(feature = "sina")]
impl Handler<RegisterSinaMdActor> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: RegisterSinaMdActor, ctx: &mut Self::Context) -> Self::Result {
        // 注册Sina市场数据Actor
        let broker_id = msg.broker_id.clone();
        self.sina_actors.insert(broker_id.clone(), msg.addr.clone());
        
        // 将分发器地址注册到Actor
        msg.addr.do_send(RegisterDistributor {
            addr: ctx.address(),
        });
        
        info!("Registered Sina market data actor for broker {}", broker_id);
    }
}

// 处理通用市场数据Actor注册消息
impl Handler<RegisterMdActor> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: RegisterMdActor, ctx: &mut Self::Context) -> Self::Result {
        // 根据数据源类型注册到不同的集合
        let broker_id = msg.broker_id.clone();
        match msg.source_type {
            #[cfg(feature = "ctp")]
            MarketDataSource::CTP => {
                self.ctp_actors.insert(broker_id.clone(), msg.addr.clone());
                info!("Registered CTP market data actor for broker {}", broker_id);
            },
            #[cfg(feature = "qq")]
            MarketDataSource::QQ => {
                self.qq_actors.insert(broker_id.clone(), msg.addr.clone());
                info!("Registered QQ market data actor for broker {}", broker_id);
            },
            #[cfg(feature = "sina")]
            MarketDataSource::Sina => {
                self.sina_actors.insert(broker_id.clone(), msg.addr.clone());
                info!("Registered Sina market data actor for broker {}", broker_id);
            },
            #[allow(unreachable_patterns)]
            _ => {
                warn!("Unknown market data source type {:?}", msg.source_type);
            }
        }
        
        // 将分发器地址注册到Actor
        msg.addr.do_send(RegisterDistributor {
            addr: ctx.address(),
        });
    }
}

// 处理获取所有订阅的消息
impl Handler<GetAllSubscriptions> for MarketDataDistributor {
    type Result = Vec<String>;

    fn handle(&mut self, _: GetAllSubscriptions, _: &mut Self::Context) -> Self::Result {
        // 返回所有已订阅的合约列表
        let mut result = HashSet::new();
        
        // 从所有合约订阅关系中收集合约
        for instrument in self.instrument_subscribers.keys() {
            result.insert(instrument.clone());
        }
        
        result.into_iter().collect()
    }
}

// 处理添加单个订阅消息
impl Handler<AddSubscription> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: AddSubscription, _: &mut Self::Context) -> Self::Result {
        // 添加订阅
        self.add_subscription(&msg.client_id.to_string(), &[msg.instrument.clone()]);
    }
}

// 处理移除单个订阅消息
impl Handler<RemoveSubscription> for MarketDataDistributor {
    type Result = ();

    fn handle(&mut self, msg: RemoveSubscription, _: &mut Self::Context) -> Self::Result {
        // 移除订阅
        self.remove_subscription(&msg.client_id.to_string(), &[msg.instrument.clone()]);
    }
}
