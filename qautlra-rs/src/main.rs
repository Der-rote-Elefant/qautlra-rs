use std::time::Instant;

use actix::*;
use actix_web::middleware::Logger;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use env_logger;

use crate::server::websocket::mdserver::MDServer;
use crate::server::websocket::mdsession::MDSession;

/// WebSocket connection handler for market data
async fn ws_market_data_handler(
    req: HttpRequest,
    stream: web::Payload,
    md_server: web::Data<Addr<MDServer>>,
) -> Result<HttpResponse, Error> {
    // Create a new WebSocket session
    ws::start(
        MDSession {
            id: 0,
            hb: Instant::now(),
            room: "main".to_owned(),
            md_addr: md_server.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Create a new Actix Arbiter for the market data server
    let arbiter = Arbiter::new();
    
    // CTP front server addresses
    let front_servers = vec![
        "tcp://180.168.146.187:10131",
        "tcp://180.168.146.187:10130",
        "tcp://218.202.237.33:10112",
    ];
    
    // CTP account credentials
    let user_id = "your_user_id";
    let password = "your_password";
    let broker_id = "your_broker_id";
    
    // Start the market data server in its own thread
    let md_server = MDServer::start_in_arbiter(&arbiter.handle(), move |_| {
        MDServer::new(front_servers, user_id, password, broker_id)
    });
    
    // Start the HTTP server with WebSocket support
    println!("Starting WebSocket market data server on 0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(md_server.clone()))
            .service(web::resource("/ws/marketdata").route(web::get().to(ws_market_data_handler)))
            .wrap(Logger::default())
    })
    .workers(4)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
