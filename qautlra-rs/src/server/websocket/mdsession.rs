use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use serde::Serialize;
use serde_json::value::Value;

use super::mdserver::MDServer;
use crate::server::websocket::mdserver::{Connect, Disconnect, Subscribe, UnSubscribe};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Serialize)]
pub struct WebSocketResponse<T> {
    data: T,
    topic: String,
    code: i32,
}

impl<T: 'static> WebSocketResponse<T>
where
    T: Serialize,
{
    pub fn ok(data: T, topic: &str) -> Self {
        Self {
            code: 200,
            data,
            topic: topic.to_owned(),
        }
    }

    pub fn fail(data: T, topic: &str) -> Self {
        Self {
            code: 400,
            data,
            topic: topic.to_owned(),
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug)]
pub struct MDSession {
    /// unique session id
    pub id: usize,
    /// Client must send ping at least once per CLIENT_TIMEOUT seconds,
    /// otherwise we drop connection.
    pub hb: Instant,
    /// Joined room/channel
    pub room: String,
    /// Market data server
    pub md_addr: Addr<MDServer>,
}

impl MDSession {
    /// Send heartbeat ping to client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for MDSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Start the heartbeat process
        self.hb(ctx);

        // Register self in market data server
        let addr = ctx.address();
        self.md_addr
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with market data server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // Notify market data server when disconnecting
        self.md_addr.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MDSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let request: Value = match serde_json::from_str(&text) {
                    Ok(x) => x,
                    Err(e) => {
                        ctx.text(WebSocketResponse::fail(e.to_string(), "error").to_string());
                        return;
                    }
                };
                
                self.hb = Instant::now();
                
                // Handle subscription requests
                if let Some(op) = request.get("op").and_then(|v| v.as_str()) {
                    match op {
                        "subscribe" => {
                            if let Some(symbols) = request.get("symbols").and_then(|v| v.as_array()) {
                                let symbols: Vec<String> = symbols
                                    .iter()
                                    .filter_map(|s| s.as_str().map(|s| s.to_string()))
                                    .collect();
                                
                                if !symbols.is_empty() {
                                    self.md_addr.do_send(Subscribe {
                                        client_id: self.id,
                                        subscribe: symbols,
                                    });
                                    ctx.text(WebSocketResponse::ok("Subscribed", "subscribe").to_string());
                                }
                            }
                        }
                        "unsubscribe" => {
                            if let Some(symbols) = request.get("symbols").and_then(|v| v.as_array()) {
                                let symbols: Vec<String> = symbols
                                    .iter()
                                    .filter_map(|s| s.as_str().map(|s| s.to_string()))
                                    .collect();
                                
                                if !symbols.is_empty() {
                                    self.md_addr.do_send(UnSubscribe {
                                        client_id: self.id,
                                        unsubscribe: symbols,
                                    });
                                    ctx.text(WebSocketResponse::ok("Unsubscribed", "unsubscribe").to_string());
                                }
                            }
                        }
                        _ => {
                            ctx.text(WebSocketResponse::fail("Unknown operation", "error").to_string());
                        }
                    }
                } else {
                    ctx.text(WebSocketResponse::fail("Missing operation", "error").to_string());
                }
            }
            ws::Message::Binary(_) => {
                println!("Unexpected binary message");
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

/// Handle market data messages sent from the server
impl<T> Handler<T> for MDSession
where
    T: Message + Send + Serialize + 'static,
    T::Result: Send,
{
    type Result = ();

    fn handle(&mut self, msg: T, ctx: &mut Self::Context) {
        self.hb = Instant::now();
        // Send market data to client
        ctx.text(WebSocketResponse::ok(msg, "marketdata").to_string());
    }
} 