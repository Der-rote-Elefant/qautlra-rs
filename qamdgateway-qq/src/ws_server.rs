use actix::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use uuid::Uuid;
use log::{info, debug};

use crate::actors::messages::{WebSocketMessage, WebSocketConnect, WebSocketDisconnect};
use crate::actors::md_connector::MarketDataConnector;
use crate::actors::messages::{
    Subscribe, Unsubscribe, RegisterQQMdActor, SubscribeQQ, UnsubscribeQQ, RegisterSinaMdActor, SubscribeSina, UnsubscribeSina,
};
use crate::actors::qq_md_actor::QQMarketDataActor;
use crate::config::BrokerConfig;

// Interval for sending ping frames to keep the connection alive (10 seconds)
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
// Terminate connection if client doesn't respond to ping for this period (30 seconds)
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

/// WebSocket message types that can be received from clients
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WsClientMessage {
    /// TradingView格式订阅行情
    #[serde(rename_all = "snake_case")]
    TvSubscribeQuote {
        aid: String,
        ins_list: String,
    },
    /// 老格式兼容
    LegacyMessage(LegacyClientMessage),
    /// Peek message
    #[serde(rename_all = "snake_case")]
    PeekMessage {
        aid: String,
    },
}

/// 兼容旧版本的消息格式
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum LegacyClientMessage {
    /// Subscribe to one or more instruments
    #[serde(rename = "subscribe")]
    Subscribe { instruments: Vec<String> },
    /// Unsubscribe from one or more instruments
    #[serde(rename = "unsubscribe")]
    Unsubscribe { instruments: Vec<String> },
    /// Get all subscribed instruments
    #[serde(rename = "subscriptions")]
    Subscriptions,
    /// Authenticate (if required)
    #[serde(rename = "auth")]
    Auth { token: String },
    /// Ping message to keep connection alive
    #[serde(rename = "ping")]
    Ping,
}

/// WebSocket message types that can be sent to clients
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WsServerMessage {
    /// TradingView格式的行情数据
    TvMarketData {
        aid: String,
        data: Vec<TvMarketDataItem>,
    },
    /// 旧版格式
    LegacyMessage(LegacyServerMessage),
    /// Peek message response
    PeekMessageResponse {
        aid: String,
        ins_list: String,
    },
}

/// TradingView格式的行情数据项
#[derive(Debug, Serialize, Deserialize)]
pub struct TvMarketDataItem {
    pub quotes: HashMap<String, TvQuote>,
}

/// TradingView格式的行情数据
#[derive(Debug, Serialize, Deserialize)]
pub struct TvQuote {
    pub instrument_id: String,
    pub datetime: String,
    pub last_price: f64,
    pub volume: i64,
    pub amount: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub bid_price1: f64,
    pub bid_volume1: i64,
    pub ask_price1: f64,
    pub ask_volume1: i64,
    pub volume_multiple: i32,
    pub price_tick: f64,
    #[serde(default)]
    pub price_decs: i32,
    #[serde(default)]
    pub max_market_order_volume: i64,
    #[serde(default)]
    pub min_market_order_volume: i64,
    #[serde(default)]
    pub max_limit_order_volume: i64,
    #[serde(default)]
    pub min_limit_order_volume: i64,
    #[serde(default)]
    pub margin: f64,
    #[serde(default)]
    pub commission: f64,
    #[serde(default)]
    pub upper_limit: f64,
    #[serde(default)]
    pub lower_limit: f64,
    #[serde(default)]
    pub pre_close: f64,
    #[serde(default)]
    pub pre_settlement: f64,
    #[serde(default)]
    pub pre_open_interest: i64,
    #[serde(default)]
    pub open_interest: i64,
    #[serde(default)]
    pub close: f64,
    #[serde(default)]
    pub settlement: f64,
    #[serde(default)]
    pub average: f64,
}

/// 旧版消息格式
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum LegacyServerMessage {
    /// Market data update
    #[serde(rename = "market_data")]
    MarketData {
        data: qamd_rs::MDSnapshot,
    },
    /// System message
    #[serde(rename = "system")]
    System {
        message: String,
    },
    /// Error message
    #[serde(rename = "error")]
    Error {
        message: String,
    },
    /// Response to subscriptions request
    #[serde(rename = "subscriptions")]
    Subscriptions {
        instruments: Vec<String>,
    },
    /// Pong response to ping
    #[serde(rename = "pong")]
    Pong,
}

/// Actor message for distributing market data to connected clients
struct DistributeMarketData(qamd_rs::MDSnapshot);

/// WebSocket session state
struct WsSession {
    /// Unique session id
    id: Uuid,
    /// Client heartbeat status
    heartbeat: Instant,
    /// Address of the market data connector actor
    md_connector: actix::Addr<MarketDataConnector>,
    /// Subscribed instruments for this session
    subscriptions: HashSet<String>,
    /// 是否为QQ行情会话
    is_qq_session: bool,
    /// 是否为Sina行情会话
    is_sina_session: bool,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Start heartbeat process
        self.start_heartbeat(ctx);

        // Register with the market data connector
        let addr = ctx.address();
        
        // 我们需要创建一个接收 MarketDataUpdate 消息的 handler，但这个消息我们不再处理
        // 让我们只注册 WebSocketConnect 而不是 Connect
        self.md_connector.do_send(WebSocketConnect {
            id: self.id,
            addr: addr.recipient(),
        });

        // Send welcome message
        let msg = WsServerMessage::LegacyMessage(LegacyServerMessage::System {
            message: format!("Connected to QAMD Gateway WebSocket. Session ID: {}", self.id),
        });
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        // Unregister from market data connector
        self.md_connector.do_send(WebSocketDisconnect {
            id: self.id,
        });
        actix::Running::Stop
    }
}

impl WsSession {
    /// Create a new WebSocket session
    pub fn new(md_connector: actix::Addr<MarketDataConnector>) -> Self {
        Self {
            id: Uuid::new_v4(),
            heartbeat: Instant::now(),
            md_connector: md_connector,
            subscriptions: HashSet::new(),
            is_qq_session: false,
            is_sina_session: false,
        }
    }

    /// Create a new QQ WebSocket session
    pub fn new_qq(md_connector: actix::Addr<MarketDataConnector>) -> Self {
        Self {
            id: Uuid::new_v4(),
            heartbeat: Instant::now(),
            md_connector: md_connector,
            subscriptions: HashSet::new(),
            is_qq_session: true,
            is_sina_session: false,
        }
    }

    /// Create a new Sina WebSocket session
    pub fn new_sina(md_connector: actix::Addr<MarketDataConnector>) -> Self {
        Self {
            id: Uuid::new_v4(),
            heartbeat: Instant::now(),
            md_connector: md_connector,
            subscriptions: HashSet::new(),
            is_qq_session: false,
            is_sina_session: true,
        }
    }

    /// Start the heartbeat process
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Check if client has responded to previous heartbeat
            if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
                log::info!("WebSocket client timeout: {}", act.id);
                ctx.stop();
                return;
            }

            // Send ping frame
            ctx.ping(b"");
        });
    }

    /// Handle subscription request
    fn handle_subscribe(&mut self, instruments: Vec<String>, ctx: &mut ws::WebsocketContext<Self>) {
        if instruments.is_empty() {
            let msg = WsServerMessage::LegacyMessage(LegacyServerMessage::Error {
                message: "No instruments specified".to_string(),
            });
            if let Ok(json) = serde_json::to_string(&msg) {
                ctx.text(json);
            }
            return;
        }

        // Update local subscriptions
        for instrument in &instruments {
            self.subscriptions.insert(instrument.clone());
        }

        // 根据会话类型选择订阅方式
        if self.is_qq_session {
            // QQ行情订阅
            println!("Subscribing to QQ market data: {:?}", instruments);
            self.md_connector.do_send(SubscribeQQ {
                id: self.id,
                instruments: instruments.clone(),
            });
        } else if self.is_sina_session {
            // Sina行情订阅
            self.md_connector.do_send(SubscribeSina {
                id: self.id,
                instruments: instruments.clone(),
            });
        } else {
            // 普通行情订阅
            self.md_connector.do_send(Subscribe {
                id: self.id,
                instruments: instruments.clone(),
            });
        }

        // Send confirmation
        let ins_list = instruments.join(",");
        let response = json!({
            "aid": "rtn_data",
            "data": [{"trade": {"status": "READY"}}],
            "ins_list": ins_list,
        });
        ctx.text(response.to_string());
    }

    /// Handle unsubscription request
    fn handle_unsubscribe(&mut self, instruments: Vec<String>, ctx: &mut ws::WebsocketContext<Self>) {
        println!("handle_unsubscribe");
        if instruments.is_empty() {
            let msg = WsServerMessage::LegacyMessage(LegacyServerMessage::Error {
                message: "No instruments specified".to_string(),
            });
            if let Ok(json) = serde_json::to_string(&msg) {
                ctx.text(json);
            }
            return;
        }

        // Update local subscriptions
        for instrument in &instruments {
            self.subscriptions.remove(instrument);
        }

        // 根据会话类型选择取消订阅方式
        if self.is_qq_session {
            // QQ行情取消订阅
            println!("Unsubscribing from QQ market data: {:?}", instruments);
            self.md_connector.do_send(UnsubscribeQQ {
                id: self.id,
                instruments: instruments.clone(),
            });
        } else if self.is_sina_session {
            // Sina行情取消订阅
            self.md_connector.do_send(UnsubscribeSina {
                id: self.id,
                instruments: instruments.clone(),
            });
        } else {
            // 普通行情取消订阅
            self.md_connector.do_send(Unsubscribe {
                id: self.id,
                instruments: instruments.clone(),
            });
        }

        // Send confirmation
        let ins_list = instruments.join(",");
        let response = json!({
            "aid": "rtn_data",
            "data": [{"trade": {"status": "READY"}}],
            "ins_list": ins_list,
        });
        ctx.text(response.to_string());
    }

    /// Handle get subscriptions request
    fn handle_get_subscriptions(&self, ctx: &mut ws::WebsocketContext<Self>) {
        // Send current subscriptions
        let instruments: Vec<String> = self.subscriptions.iter().cloned().collect();
        let msg = WsServerMessage::LegacyMessage(LegacyServerMessage::Subscriptions {
            instruments,
        });
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                self.heartbeat = Instant::now();
                
                // Try to parse as JSON
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    // 处理TradingView格式的请求
                    if let Some(aid) = json.get("aid").and_then(|a| a.as_str()) {
                        match aid {
                            "subscribe_quote" => {
                                // 处理订阅请求
                                if let Some(ins_list) = json.get("ins_list").and_then(|i| i.as_str()) {
                                    let instruments: Vec<String> = ins_list
                                        .split(',')
                                        .filter(|s| !s.is_empty())
                                        .map(|s| s.to_string())
                                        .collect();
                                    println!("!!!!!!!!!!!instruments: {:?}", instruments);
                                    self.handle_subscribe(instruments, ctx);
                                } else {
                                    println!("!!!!!!!!!!!empty ins_list");
                                    // 空的ins_list表示清空所有订阅
                                    let current_subs: Vec<String> = self.subscriptions.iter().cloned().collect();
                                    self.handle_unsubscribe(current_subs, ctx);
                                }
                            }
                            "peek_message" => {
                                // 返回当前订阅列表
                                let instruments: Vec<String> = self.subscriptions.iter().cloned().collect();
                                let ins_list = instruments.join(",");
                                let msg = WsServerMessage::PeekMessageResponse {
                                    aid: "peek_message".to_string(),
                                    ins_list,
                                };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    ctx.text(json);
                                }
                            }
                            _ => {
                                // 未知的aid类型
                                let err_msg = LegacyServerMessage::Error {
                                    message: format!("Unknown aid type: {}", aid),
                                };
                                let msg = WsServerMessage::LegacyMessage(err_msg);
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    ctx.text(json);
                                }
                            }
                        }
                    } else {
                        // 尝试解析为旧格式
                        if let Ok(client_msg) = serde_json::from_value::<LegacyClientMessage>(json.clone()) {
                            match client_msg {
                                LegacyClientMessage::Subscribe { instruments } => {
                                    self.handle_subscribe(instruments, ctx);
                                }
                                LegacyClientMessage::Unsubscribe { instruments } => {
                                    self.handle_unsubscribe(instruments, ctx);
                                }
                                LegacyClientMessage::Subscriptions => {
                                    self.handle_get_subscriptions(ctx);
                                }
                                LegacyClientMessage::Ping => {
                                    let msg = WsServerMessage::LegacyMessage(LegacyServerMessage::Pong);
                                    if let Ok(json) = serde_json::to_string(&msg) {
                                        ctx.text(json);
                                    }
                                }
                                LegacyClientMessage::Auth { token: _ } => {
                                    // 目前不需要验证
                                    let msg = WsServerMessage::LegacyMessage(LegacyServerMessage::System {
                                        message: "Authentication not required".to_string(),
                                    });
                                    if let Ok(json) = serde_json::to_string(&msg) {
                                        ctx.text(json);
                                    }
                                }
                            }
                        } else {
                            // 无法解析的消息
                            let err_msg = LegacyServerMessage::Error {
                                message: "Invalid message format".to_string(),
                            };
                            let msg = WsServerMessage::LegacyMessage(err_msg);
                            if let Ok(json) = serde_json::to_string(&msg) {
                                ctx.text(json);
                            }
                        }
                    }
                } else {
                    // 无效的JSON
                    let err_msg = LegacyServerMessage::Error {
                        message: "Invalid JSON".to_string(),
                    };
                    let msg = WsServerMessage::LegacyMessage(err_msg);
                    if let Ok(json) = serde_json::to_string(&msg) {
                        ctx.text(json);
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                log::warn!("Binary messages not supported");
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl Handler<WebSocketMessage> for WsSession {
    type Result = ();
    
    fn handle(&mut self, msg: WebSocketMessage, ctx: &mut Self::Context) -> Self::Result {
        debug!("WsSession received WebSocketMessage for client {}", self.id);
        // 这里将消息发送到客户端
        ctx.text(msg.0);
    }
}

/// WebSocket route handler
pub async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    md_connector: web::Data<actix::Addr<MarketDataConnector>>,
) -> Result<HttpResponse, Error> {
    let session = WsSession::new(md_connector.get_ref().clone());
    let resp = ws::start(session, &req, stream)?;
    Ok(resp)
}

/// WebSocket route handler for QQ market data
pub async fn ws_qq_index(
    req: HttpRequest,
    stream: web::Payload,
    md_connector: web::Data<actix::Addr<MarketDataConnector>>,
) -> Result<HttpResponse, Error> {
    // 创建一个新的 QQ 行情会话
    info!("Creating a new QQ market data session");
    let config = BrokerConfig {
        name: "qq".to_string(),
        front_addr: "tcp://120.136.160.67:33441".to_string(),  // 使用8013端口，避免与服务端口冲突
        user_id: "".to_string(),
        password: "".to_string(),
        broker_id: "qq".to_string(),
        app_id: "".to_string(),
        auth_code: "".to_string(),
        source_type: Some("qq".to_string()),
    };
    println!("!!!QQ!!!!config: {:?}", config);
    // 创建 QQ 行情 Actor
    let qq_md_actor = QQMarketDataActor::new(config).start();
    
    // 将 QQ 行情 Actor 注册到分发器
    md_connector.do_send(RegisterQQMdActor {
        addr: qq_md_actor.clone(),
    });
    
    // 创建 WebSocket 会话 - 使用QQ会话
    let session = WsSession::new_qq(md_connector.get_ref().clone());
    let resp = ws::start(session, &req, stream)?;
    Ok(resp)
}

/// WebSocket route handler for Sina market data
pub async fn ws_sina_index(
    req: HttpRequest,
    stream: web::Payload,
    md_connector: web::Data<actix::Addr<MarketDataConnector>>,
) -> Result<HttpResponse, Error> {
    // 创建一个新的 Sina 行情会话
    info!("Creating a new Sina market data session");
    let config = BrokerConfig {
        name: "sina".to_string(),
        front_addr: "tcp://hq2fuhq.client.tdx.com.cn:7709".to_string(),
        user_id: "".to_string(),
        password: "".to_string(),
        broker_id: "sina".to_string(),
        app_id: "".to_string(),
        auth_code: "".to_string(),
        source_type: Some("sina".to_string()),
    };
    println!("!!!Sina!!!!config: {:?}", config);
    
    // 创建 Sina 行情 Actor
    let sina_md_actor = crate::actors::sina_md_actor::SinaMarketDataActor::new(config).start();
    
    // 将 Sina 行情 Actor 注册到分发器
    md_connector.do_send(RegisterSinaMdActor {
        addr: sina_md_actor.clone(),
    });
    
    // 创建 WebSocket 会话 - 使用Sina会话
    let session = WsSession::new_sina(md_connector.get_ref().clone());
    let resp = ws::start(session, &req, stream)?;
    Ok(resp)
}

/// WebSocket route handler for Sina market data
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/ws/market")
            .route(web::get().to(ws_index))
    );
    cfg.service(
        web::resource("/ws/qq/market")
            .route(web::get().to(ws_qq_index))
    );
    cfg.service(
        web::resource("/ws/sina/market")
            .route(web::get().to(ws_sina_index))
    );
} 