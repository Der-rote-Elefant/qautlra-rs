use actix::prelude::*;
use qamd_rs::MDSnapshot;
use ctp_common::CThostFtdcDepthMarketDataField;
use std::collections::HashSet;
use uuid::Uuid;

use crate::actors::qq_md_actor::QQMarketDataActor;

// Market Data Source Messages
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: Uuid,
    pub addr: Recipient<MarketDataUpdate>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    pub id: Uuid,
    pub instruments: Vec<String>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Unsubscribe {
    pub id: Uuid,
    pub instruments: Vec<String>,
}

#[derive(Message)]
#[rtype(result = "Vec<String>")]
pub struct GetSubscriptions {
    pub id: Uuid,
    pub callback: Option<Box<dyn FnOnce(Vec<String>) + Send>>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct MarketDataUpdate(pub MDSnapshot, pub MarketDataSource);

/// 市场数据来源类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarketDataSource {
    CTP,
    QQ,
    Sina
}

// CTP Market Data Events
#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub enum MarketDataEvent {
    Connected,
    Disconnected,
    LoggedIn,
    MarketData(CThostFtdcDepthMarketDataField),
    SubscriptionSuccess(String),
    SubscriptionFailure(String, String),
    Error(String),
}

// Control messages for the Market Data Manager
#[derive(Message)]
#[rtype(result = "()")]
pub struct StartMarketData {
    pub instruments: Vec<String>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopMarketData;

// New actor-specific messages
#[derive(Message)]
#[rtype(result = "()")]
pub struct InitMarketDataSource;

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct LoginMarketDataSource;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterDistributor {
    pub addr: Addr<crate::actors::md_distributor::MarketDataDistributor>,
}

#[derive(Message)]
#[rtype(result = "HashSet<String>")]
pub struct GetAllSubscriptions;

// WebSocket connection management messages
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct WebSocketConnect {
    pub id: Uuid,
    pub addr: Recipient<WebSocketMessage>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct WebSocketDisconnect {
    pub id: Uuid,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct WebSocketMessage(pub String);

// Subscription management messages
#[derive(Message)]
#[rtype(result = "()")]
pub struct AddSubscription {
    pub instrument: String,
    pub client_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RemoveSubscription {
    pub instrument: String,
    pub client_id: Uuid,
}

// Message for supervised actor restart
#[derive(Message)]
#[rtype(result = "()")]
pub struct RestartActor;

/// 注册QQ行情Actor的消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterQQMdActor {
    pub addr: Addr<crate::actors::qq_md_actor::QQMarketDataActor>,
}

/// 注册Sina行情Actor的消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterSinaMdActor {
    pub addr: Addr<crate::actors::sina_md_actor::SinaMarketDataActor>,
}

/// 订阅QQ行情消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct SubscribeQQ {
    pub id: Uuid,
    pub instruments: Vec<String>,
}

/// 取消订阅QQ行情消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct UnsubscribeQQ {
    pub id: Uuid,
    pub instruments: Vec<String>,
}

/// 订阅Sina行情消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct SubscribeSina {
    pub id: Uuid,
    pub instruments: Vec<String>,
}

/// 取消订阅Sina行情消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct UnsubscribeSina {
    pub id: Uuid,
    pub instruments: Vec<String>,
} 