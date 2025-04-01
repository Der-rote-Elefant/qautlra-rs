# 市场数据网关开发指南

## 简介

市场数据网关(QAMDGATEWAY)是QAUTLRA-RS生态系统中负责连接和转换不同数据源的关键组件，它将各种市场数据源的数据转换为统一格式，并通过WebSocket实时推送给订阅者。本指南将帮助开发者理解和扩展市场数据网关的功能。

## 架构概述

QAMDGATEWAY基于Actor模型设计，主要组件包括：

1. **Market Data Source**：负责连接具体的市场数据源(如CTP、新浪财经、腾讯财经等)
2. **Market Data Connector**：管理多个市场数据源，处理连接状态和数据流
3. **Market Data Distributor**：将标准化后的市场数据分发给订阅客户端
4. **WebSocket Server**：提供WebSocket服务，允许客户端订阅和接收市场数据
5. **REST API Server**：提供HTTP API接口，用于管理订阅关系和查询网关状态

```
                    ┌─────────────────┐
                    │                 │
                    │ Market Data     │
                    │ Distributor     │
                    │                 │
                    └─────────────────┘
                          ▲     │
                          │     ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│                 │  │                 │  │                 │
│ Market Data     │─▶│ Market Data     │◀─│ WebSocket       │
│ Source          │  │ Connector       │  │ Clients         │
│                 │  │                 │  │                 │
└─────────────────┘  └─────────────────┘  └─────────────────┘
                          ▲
                          │
                    ┌─────────────────┐
                    │                 │
                    │ REST API        │
                    │ Clients         │
                    │                 │
                    └─────────────────┘
```

## 技术栈

- **Rust语言**：核心开发语言
- **actix/actix-web**：Actor模型实现和Web服务器框架
- **tokio**：异步运行时
- **CTP API**：中国金融期货交易所API(通过FFI绑定)
- **serde**：序列化/反序列化
- **tracing**：日志和跟踪
- **clap**：命令行参数解析

## 数据流程

1. Market Data Source连接到外部数据源(如CTP、新浪财经等)
2. 接收到原始市场数据后，转换为统一的`MarketData`结构
3. 转发给Market Data Connector进行聚合和预处理
4. Market Data Distributor根据订阅关系分发数据
5. 通过WebSocket推送给订阅客户端，支持增量更新

## 自定义市场数据源

要添加新的市场数据源，需要完成以下步骤：

### 1. 创建数据源Actor实现

```rust
use crate::market_data::MarketData;
use actix::{Actor, Context, Handler, Message, Supervised, SystemService};

pub struct MyDataSourceActor {
    // 添加必要的字段
    connector_addr: Addr<MarketDataConnector>,
}

impl Actor for MyDataSourceActor {
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        // 初始化逻辑，例如连接到数据源
        self.connect();
    }
}

// 实现必要的消息处理
impl Handler<SubscribeMsg> for MyDataSourceActor {
    type Result = ();
    
    fn handle(&mut self, msg: SubscribeMsg, _: &mut Context<Self>) {
        // 处理订阅请求
    }
}
```

### 2. 实现数据转换逻辑

```rust
impl MyDataSourceActor {
    fn process_raw_data(&self, raw_data: &RawDataType) -> MarketData {
        // 将原始数据转换为标准MarketData结构
        MarketData {
            instrument_id: raw_data.symbol.clone(),
            exchange_id: self.extract_exchange(raw_data),
            last_price: raw_data.price,
            // ... 填充其他字段
            source: "MyDataSource".to_string(),
        }
    }
    
    fn extract_exchange(&self, raw_data: &RawDataType) -> String {
        // 提取交易所ID的逻辑
    }
}
```

### 3. 注册到Market Data Connector

```rust
impl MyDataSourceActor {
    fn register(&self, ctx: &mut Context<Self>) {
        // 向MarketDataConnector注册
        self.connector_addr.do_send(RegisterSource {
            name: "MyDataSource".to_string(),
            addr: ctx.address(),
        });
    }
}
```

### 4. 更新配置处理

在`config.rs`中添加新数据源的配置结构：

```rust
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MyDataSourceConfig {
    pub enabled: bool,
    pub api_key: String,
    pub api_secret: String,
    // ... 其他配置字段
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    // ... 现有配置
    pub my_data_source: Option<MyDataSourceConfig>,
}
```

### 5. 更新Main函数

在`main.rs`中添加条件初始化：

```rust
fn main() -> Result<()> {
    // ... 现有代码
    
    if let Some(config) = &config.my_data_source {
        if config.enabled {
            let my_data_source = MyDataSourceActor::new(config.clone(), connector_addr.clone());
            let _ = my_data_source.start();
        }
    }
    
    // ... 现有代码
}
```

## 增量数据更新

QAMDGATEWAY支持增量数据更新，可以大幅减少网络流量。实现步骤：

1. 保存每个客户端的上次发送状态
2. 计算新数据与上次发送数据的差异
3. 只发送变化的字段

```rust
fn calculate_diff(prev: &MarketData, current: &MarketData) -> HashMap<String, Value> {
    let mut diff = HashMap::new();
    
    // 比较并添加变化的字段
    if prev.last_price != current.last_price {
        diff.insert("last_price".to_string(), json!(current.last_price));
    }
    
    // ... 检查其他字段
    
    diff
}
```

## WebSocket接口

QAMDGATEWAY的WebSocket接口支持以下消息类型：

### 订阅请求

```json
{
  "aid": "subscribe_quote",
  "ins_list": "SHFE.au2412,DCE.a2405"
}
```

### 取消订阅请求

```json
{
  "aid": "unsubscribe_quote",
  "ins_list": "SHFE.au2412"
}
```

### 市场数据响应

```json
{
  "aid": "rtn_data",
  "data": [
    {
      "quotes": {
        "SHFE.au2412": {
          "instrument_id": "SHFE.au2412",
          "exchange_id": "SHFE",
          "last_price": 2056.5,
          "volume": 12500,
          // ... 其他数据字段
        }
      }
    }
  ]
}
```

## REST API接口

QAMDGATEWAY提供以下REST API接口：

### 获取订阅列表

```
GET /api/subscriptions
```

### 添加订阅

```
POST /api/subscribe
Content-Type: application/json

{
  "instruments": ["SHFE.au2412", "DCE.a2405"]
}
```

### 取消订阅

```
POST /api/unsubscribe
Content-Type: application/json

{
  "instruments": ["SHFE.au2412"]
}
```

### 获取系统状态

```
GET /api/status
```

## 错误处理和日志

使用`tracing`库进行日志记录和错误跟踪：

```rust
use tracing::{info, error, warn, debug};

fn connect_to_source(&mut self) -> Result<()> {
    info!("Connecting to data source: {}", self.name);
    
    match perform_connection() {
        Ok(_) => {
            info!("Successfully connected to {}", self.name);
            Ok(())
        }
        Err(e) => {
            error!("Failed to connect to {}: {}", self.name, e);
            Err(e.into())
        }
    }
}
```

## 性能优化

1. **批量处理**：将多个市场数据更新批量处理和发送
2. **缓存策略**：缓存频繁访问的数据
3. **内存优化**：减少数据复制，使用引用和智能指针
4. **线程池调优**：根据实际负载调整工作线程数量

## 测试

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_market_data_conversion() {
        let raw_data = MockRawData { /* ... */ };
        let actor = MyDataSourceActor::new(/* ... */);
        
        let market_data = actor.process_raw_data(&raw_data);
        
        assert_eq!(market_data.instrument_id, "SHFE.au2412");
        assert_eq!(market_data.last_price, 2056.5);
        // ... 其他断言
    }
}
```

### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[actix_rt::test]
    async fn test_subscription_flow() {
        // 启动测试系统
        let sys = System::new();
        
        // 创建测试组件
        let connector = MarketDataConnector::new().start();
        let source = MyDataSourceActor::new(connector.clone()).start();
        
        // 执行订阅
        let res = connector.send(SubscribeMsg {
            instruments: vec!["SHFE.au2412".to_string()],
        }).await;
        
        assert!(res.is_ok());
        
        // 验证数据分发
        // ...
    }
}
```

## 部署

QAMDGATEWAY支持多种部署方式：

### Docker部署

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin qamdgateway

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/qamdgateway /usr/local/bin/
COPY config.json /etc/qamdgateway/
CMD ["qamdgateway", "--config", "/etc/qamdgateway/config.json"]
```

### 系统服务部署

```
[Unit]
Description=QA Market Data Gateway
After=network.target

[Service]
ExecStart=/usr/local/bin/qamdgateway --config /etc/qamdgateway/config.json
Restart=on-failure
User=qauser

[Install]
WantedBy=multi-user.target
```

## 监控

集成Prometheus监控：

```rust
use prometheus::{IntCounter, register_int_counter};

lazy_static! {
    static ref MARKET_DATA_RECEIVED: IntCounter = register_int_counter!(
        "qamdgateway_market_data_received_total",
        "Total number of market data messages received"
    ).unwrap();
    
    static ref MARKET_DATA_SENT: IntCounter = register_int_counter!(
        "qamdgateway_market_data_sent_total",
        "Total number of market data messages sent to clients"
    ).unwrap();
}

// 在代码中记录指标
fn process_market_data(&self, data: MarketData) {
    MARKET_DATA_RECEIVED.inc();
    
    // 处理逻辑...
    
    MARKET_DATA_SENT.inc_by(client_count as u64);
}
```

## 常见问题

### 1. 连接断开如何处理？

实现重连机制：

```rust
fn handle_disconnection(&mut self, ctx: &mut Context<Self>) {
    if self.reconnect_attempts < self.max_reconnect_attempts {
        self.reconnect_attempts += 1;
        let backoff = Duration::from_secs(2u64.pow(self.reconnect_attempts as u32));
        
        ctx.run_later(backoff, |actor, ctx| {
            info!("Attempting to reconnect ({}/{})", 
                  actor.reconnect_attempts, actor.max_reconnect_attempts);
            actor.connect();
        });
    } else {
        error!("Max reconnection attempts reached. Giving up.");
    }
}
```

### 2. 如何处理大量订阅？

优化订阅管理：

```rust
// 使用高效的数据结构存储订阅关系
use dashmap::DashMap;

struct SubscriptionManager {
    // 使用线程安全的哈希表
    subscriptions: DashMap<String, HashSet<Addr<WebSocketSession>>>,
}

impl SubscriptionManager {
    fn add_subscription(&self, instrument: &str, client: Addr<WebSocketSession>) {
        self.subscriptions
            .entry(instrument.to_string())
            .or_insert_with(HashSet::new)
            .insert(client);
    }
    
    fn get_subscribers(&self, instrument: &str) -> Vec<Addr<WebSocketSession>> {
        if let Some(subscribers) = self.subscriptions.get(instrument) {
            subscribers.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }
}
```

## 结论

开发一个高性能的市场数据网关需要关注数据处理效率、连接可靠性和扩展性。通过遵循本指南中的最佳实践，可以构建一个稳定、高效的市场数据网关，为量化交易提供可靠的数据基础。

## 参考资源

- [Actix文档](https://actix.rs/)
- [Tokio文档](https://tokio.rs/)
- [CTP API文档](http://www.sfit.com.cn/5_2_DocumentDown.htm)
- [OpenCTP项目](https://github.com/openctp/openctp) 