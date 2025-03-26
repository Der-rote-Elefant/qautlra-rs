use url::Url;
use env_logger;
use tokio;
use tokio_tungstenite;
use futures::SinkExt;
use futures::StreamExt;
use serde_json::json;
use std::env;

/// 使用tokio和tokio-tungstenite创建异步WebSocket客户端示例
/// 支持CTP、QQ和Sina三种行情源
#[tokio::main]
async fn main() {
    // 设置日志级别
    env_logger::init();

    // 获取命令行参数，决定使用哪种行情源
    // 使用方法: cargo run --example ws_client [ctp|qq|sina]
    let args: Vec<String> = env::args().collect();
    let source_type = if args.len() > 1 {
        args[1].to_lowercase()
    } else {
        "ctp".to_string() // 默认使用CTP行情
    };

    // 根据行情源类型确定WebSocket URL和订阅合约
    let (url, instruments) = match source_type.as_str() {
        "qq" => {
            // QQ行情WebSocket URL和行情合约（股票代码格式无特殊要求）
            (
                "ws://localhost:8012/ws/qq/market", 
                vec!["000001", "600000", "00700"]
            )
        },
        "sina" => {
            // 新浪行情WebSocket URL和行情合约（股票代码格式无特殊要求）
            (
                "ws://localhost:8012/ws/sina/market", 
                vec!["600000", "000001", "AAPL"]
            )
        },
        _ => {
            // CTP行情WebSocket URL和行情合约（期货合约格式）
            (
                "ws://localhost:8012/ws/market", 
                vec!["SHFE.au2512", "SHFE.rb2512", "CFFEX.IF2506"]
            )
        }
    };
    
    // 连接到WebSocket服务器
    println!("使用 {} 行情源连接到 {}", source_type, url);
    
    let ws_url = Url::parse(url).unwrap();
    let (ws_stream, _) = tokio_tungstenite::connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket server");
    
    println!("已连接到服务器");
    
    // 拆分WebSocket流为发送和接收部分
    let (mut write, read) = ws_stream.split();
    
    // 在一个单独的任务中处理来自服务器的消息
    let handle_server_messages = tokio::spawn(async move {
        read.for_each(|message| async {
            match message {
                Ok(msg) => {
                    if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                        // 如果收到的是取消订阅消息，打印内容以便调试
                        if text.contains("unsubscribe") {
                            println!("收到取消订阅响应: {}", text);
                        } else {
                            println!("收到消息: {}", text);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("接收消息错误: {}", e);
                }
            }
        }).await;
    });
    
    // 创建订阅请求
    // 打印将要订阅的合约
    println!("订阅合约: {:?}", instruments);
    
    // 使用Trading View格式创建订阅请求
    let ins_list = instruments.join(",");
    let subscribe_msg = json!({
        "aid": "subscribe_quote",
        "ins_list": ins_list
    });
    
    // 打印订阅消息
    println!("订阅消息: {}", subscribe_msg.to_string());
    
    // 发送订阅请求
    let msg = tokio_tungstenite::tungstenite::Message::Text(subscribe_msg.to_string());
    if let Err(e) = write.send(msg).await {
        eprintln!("发送订阅请求错误: {:?}", e);
    } else {
        println!("订阅请求已发送");
    }
    
    // 添加一些交互功能
    println!("\n可用命令:");
    println!("  按 Ctrl+C 退出");
    
    // 等待服务器消息处理任务完成
    handle_server_messages.await.unwrap();
} 