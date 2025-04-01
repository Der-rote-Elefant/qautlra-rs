# QAUTLRA-RS

QAUTLRA-RS (Quantitative Analysis and Ultra-Low-latency Trading in Rust for Advanced Systems) 是一个基于Rust实现的高性能量化交易和市场数据处理平台，为金融市场分析和交易提供完整的技术基础设施。

![QAUTLRA-RS](qars/QUANTAXISRS.png)

## 🚀 项目概述

QAUTLRA-RS项目旨在构建一个完整的量化交易技术栈，充分利用Rust语言的高性能、内存安全和并发特性，为量化交易提供可靠的技术支持。平台由多个独立但协同工作的子系统组成，支持从市场数据接入、行情分发、策略执行到交易执行的完整量化交易流程。

## 🌟 核心特性

- **高性能架构**：基于Rust语言开发，实现毫秒级低延迟数据处理
- **多源数据接入**：支持CTP、新浪财经、腾讯财经等多种市场数据源
- **实时行情分发**：基于WebSocket的高效行情数据推送
- **增量数据更新**：首次连接发送全量数据，之后仅推送变化的字段，大幅减少带宽消耗
- **统一数据格式**：标准化不同来源的市场数据，提供一致的数据接口
- **高性能时序数据库**：基于Arrow和DataFusion的时序数据存储和查询引擎
- **分布式设计**：基于Actor模型的分布式并发处理架构
- **安全可靠**：充分利用Rust的内存安全特性，提供稳定可靠的运行环境
- **模块化组件**：各子系统可独立运行，也可协同工作

## 📚 系统组成

QAUTLRA-RS由以下主要组件构成：

### 1. QAUTLRA-RS - 核心集成组件

作为整个QAUTLRA生态系统的核心枢纽，连接市场数据网关、交易系统和客户端应用程序。提供高性能WebSocket服务器，用于实时市场数据分发和交易服务。

**核心功能**:
- 通过WebSocket实现实时市场数据分发
- 高吞吐量数据处理
- 直接集成CTP接口
- 会话管理和订阅处理
- WebSocket API支持数据流

[了解更多关于QAUTLRA-RS的信息](qautlra-rs/README.md)

### 2. QAMDGATEWAY - 市场数据网关

市场数据网关是整个系统的数据入口，负责连接各类市场数据源，并将数据转换为统一格式后分发给订阅者。

**核心功能**:
- 连接和管理多个市场数据源
- 将原始市场数据转换为标准QAMD数据结构
- 通过WebSocket实时推送行情数据
- 提供REST API用于订阅管理
- 支持增量数据更新，大幅提升性能和减少网络流量

**子组件**:
- **qamdgateway-ctp**: 连接CTP交易系统的市场数据网关
- **qamdgateway-qq**: 连接腾讯财经的市场数据网关
- **qamdgateway-sina**: 连接新浪财经的市场数据网关

### 3. QAREALTIMEPRO-RS - 实时数据处理

QAREALTIMEPRO-RS是一个高性能实时市场数据处理和分发服务。作为连接不同市场数据源的中心枢纽，实时处理数据并通过WebSocket和REST API分发给客户端。

**核心功能**:
- 对股票和期货市场的实时数据处理
- 集成交易账户以监控头寸和盈亏
- 提供数据访问的REST和WebSocket API
- 集成Redis实现高性能缓存
- 基于Actor模型的并发和弹性架构

[了解更多关于QAREALTIMEPRO-RS的信息](qarealtimepro-rs/README.md)

### 4. QATRADESERVER-RS - 交易网关

QATRADESERVER-RS是一个高性能交易服务器，作为中央交易网关管理与多个券商的连接，处理交易执行请求并提供实时交易更新。

**核心功能**:
- 多券商网关，实现对各交易所的集中访问
- 将订单路由到适当的券商
- 交易请求负载均衡
- 高可用性设计
- 提供交易操作的WebSocket和REST API
- 安全认证和限流保护

[了解更多关于QATRADESERVER-RS的信息](qatradeserver-rs/README.md)

### 5. QATRADER-RS - 交易引擎

QATRADER-RS是一个高性能交易订单管理和执行系统。作为核心交易引擎，处理订单路由、执行、风险管理和头寸跟踪。

**核心功能**:
- 低延迟高性能交易执行
- 支持多券商和交易场所
- 基于事件驱动架构实现并发处理
- 交易前和交易后的风险管理
- 实时头寸跟踪和盈亏监控
- 集成消息队列处理订单流

[了解更多关于QATRADER-RS的信息](qatrader-rs/README.md)

### 6. QAMAINTAINDB-RS - 金融绩效分析

QAMAINTAINDB-RS是一个专门用于维护和分析金融产品绩效数据的工具。提供强大的API用于数据摄取、绩效计算和比较分析。

**核心功能**:
- Excel数据导入功能，方便金融产品数据导入
- 全面的绩效指标计算
- 在特定时间段内比较多个金融产品
- 日期范围过滤分析
- 基于Actor的并行计算
- 集成MongoDB实现数据持久化

[了解更多关于QAMAINTAINDB-RS的信息](qamaintaindb-rs/README.md)

### 7. QADB-RS - 高性能时序数据库

QADB-RS是一个专为量化交易设计的高性能时序数据库系统，基于Apache Arrow和DataFusion构建，实现高效的金融市场数据存储和查询功能。

**核心功能**:
- 高效存储和管理大规模金融市场数据
- 通过WebSocket客户端直接接收市场数据
- 通过WebSocket服务端提供数据分发能力
- 支持Kafka连接器从消息队列中获取数据(可选)
- 自动管理动态数据模式(Schema)
- 提供基于HTTP的查询API
- 支持时间和自定义字段分区存储
- 与量化交易引擎无缝集成

**技术特点**:
- 基于Apache Arrow的柱状存储格式
- 使用DataFusion高性能查询引擎
- 支持毫秒级市场数据的高速写入和查询
- 优化的时间序列数据压缩算法
- 分区存储提升查询效率

### 8. QAMD-RS - 市场数据处理库

市场数据处理库提供了标准化的数据结构和处理工具，用于处理和转换各类市场数据。

**核心功能**:
- 定义标准化的市场数据结构
- 提供数据转换和处理工具
- 支持不同来源数据的格式转换

### 9. QARS - 量化交易引擎核心库

QARS是系统的核心计算引擎，提供量化分析和交易策略执行的功能。

**核心功能**:
- 高性能回测引擎
- 因子计算和分析工具
- 策略执行引擎
- 支持与Python交互的接口

### 10. QIFI-RS - 金融交易接口标准

QIFI-RS是Quantaxis Financial Interface的Rust实现，提供标准化的金融交易接口协议。

**核心功能**:
- 定义标准账户数据结构
- 提供快速的数据序列化和反序列化
- 支持从JSON/BSON格式加载和转换数据
- 兼容MongoDB数据库操作

### 11. CTP-MD - CTP市场数据接口

CTP市场数据接口提供了连接中国金融期货交易所CTP系统的Rust绑定。

**核心功能**:
- 提供CTP API的Rust封装
- 实现CTP市场数据的接收和处理
- 支持多种期货合约的订阅

### 12. QAOMS - 量化交易前端系统

量化交易前端系统提供了用户交互界面，包括行情显示、交易操作和策略管理等功能。

**核心功能**:
- 行情数据可视化
- 交易操作界面
- 策略管理和监控
- 账户和持仓展示

## 🔧 系统架构

```
                                  ┌───────────────────┐
                                  │                   │
                                  │  市场数据分发器    │
                                  │  (Actor)          │
                                  │                   │
                                  └───────────────────┘
                                    ▲      │
                                    │      ▼
┌─────────────────┐  ┌───────────────────┐  ┌──────────────────┐  ┌─────────────────┐
│                 │  │                   │  │                  │  │                 │
│ CTP市场数据源   │─▶│ 市场数据连接器     │◀─│ WebSocket客户端   │─▶│ QADB-RS时序    │
│ (Actor)         │  │ (Actor)           │  │                  │  │ 数据库          │
│                 │  │                   │  │                  │  │                 │
└─────────────────┘  └───────────────────┘  └──────────────────┘  └─────────────────┘
                        ▲     ▲                                      ▲
┌─────────────────┐     │     │     ┌──────────────────┐             │
│                 │     │     │     │                  │             │
│ 腾讯市场数据源   │────▶│     │◀────│ REST API客户端    │             │
│ (Actor)         │           │     │                  │             │
│                 │           │     │                  │             │
└─────────────────┘           │     └──────────────────┘             │
                              │                                      │
┌─────────────────┐           │     ┌──────────────────┐             │
│                 │           │     │                  │             │
│ 新浪市场数据源   │──────────▶│     │ QARS交易引擎      │─────────────┘
│ (Actor)         │                 │                  │
│                 │                 │                  │
└─────────────────┘                 └──────────────────┘
```

## 📦 安装指南

### 环境要求

- Rust (推荐版本: 1.70+)
- Cargo
- CMake (用于编译CTP相关依赖)
- GCC/Clang (C++编译器)
- Node.js (用于前端系统)
- MongoDB
- Redis
- RabbitMQ

### 从源码构建

1. 克隆仓库：

```bash
git clone https://github.com/yutiansut/qautlra-rs.git
cd qautlra-rs
```

2. 构建市场数据网关：

```bash
# 构建支持CTP的市场数据网关
QAMDGATEWAY_CONFIG_PATH=config_ctp.json cargo run -p qamdgateway --no-default-features --features="ctp"

# 构建支持腾讯财经的市场数据网关
QAMDGATEWAY_CONFIG_PATH=config_qq.json cargo run -p qamdgateway --no-default-features --features="qq"

# 构建支持新浪财经的市场数据网关
QAMDGATEWAY_CONFIG_PATH=config_sina.json cargo run -p qamdgateway --no-default-features --features="sina"
```

3. 构建并运行QADB-RS时序数据库：

```bash
# 构建完整功能
cargo build -p qadb-rs --release

# 只启用WebSocket功能
cargo build -p qadb-rs --release --features="websocket"

# 运行QADB-RS服务
QADB_DATA_DIR=/path/to/data ./target/release/qadb-rs --mode all --websocket-enabled
```

4. 构建并运行QARealTimePro-RS：

```bash
cargo build -p qarealtimepro-rs --release
cargo run -p qarealtimepro-rs --release
```

5. 构建并运行QATrader-RS：

```bash
cargo build -p qatrader-rs --release
cargo run -p qatrader-rs --release
```

6. 构建并运行QATradeServer-RS：

```bash
cargo build -p qatradeserver-rs --release
cargo run -p qatradeserver-rs --release
```

7. 构建并运行QAMaintainDB-RS：

```bash
cargo build -p qamaintaindb-rs --release
cargo run -p qamaintaindb-rs --release
```

8. 构建前端系统：

```bash
cd qaoms/web
npm install
npm run dev
```

## ⚙️ 配置

配置文件位于项目根目录，包含`config.json`及其变种文件，主要配置如下：

### CTP市场数据网关配置示例

```json
{
  "brokers": {
    "simnow724": {
      "name": "simnow724",
      "front_addr": "tcp://180.168.146.187:10131",
      "user_id": "YOUR_USER_ID",
      "password": "YOUR_PASSWORD",
      "broker_id": "9999"
    }
  },
  "default_broker": "simnow724",
  "websocket": {
    "host": "0.0.0.0",
    "port": 8014,
    "path": "/ws/market"
  },
  "rest_api": {
    "host": "0.0.0.0",
    "port": 8015
  },
  "incremental_updates": {
    "enabled": true,
    "batch_interval_ms": 100,
    "batch_size_threshold": 50
  }
}
```

### QADB-RS配置示例

```json
{
  "data_directory": "/path/to/data",
  "websocket": {
    "enabled": true,
    "host": "0.0.0.0",
    "port": 8016,
    "path": "/ws/db"
  },
  "http_api": {
    "enabled": true,
    "host": "0.0.0.0",
    "port": 8017
  },
  "kafka": {
    "enabled": false,
    "brokers": ["localhost:9092"],
    "topics": ["market_data"]
  },
  "schema_management": {
    "auto_detect": true,
    "schema_cache_size": 100
  },
  "storage": {
    "partition_by": "date",
    "compression": "zstd",
    "retention_days": 365
  }
}
```

## 🚀 使用指南

### 订阅市场数据

平台提供了三种WebSocket连接端点：

1. CTP行情：`ws://localhost:8014/ws/market`
2. QQ行情：`ws://localhost:8016/ws/market`
3. 新浪行情：`ws://localhost:8012/ws/market`

#### WebSocket订阅示例

```javascript
// 创建WebSocket连接
const ws = new WebSocket("ws://localhost:8014/ws/market");

// 订阅合约
ws.onopen = function() {
  const subscribeMsg = {
    "aid": "subscribe_quote",
    "ins_list": "au2512,rb2512,IF2506"
  };
  ws.send(JSON.stringify(subscribeMsg));
};

// 接收行情数据
ws.onmessage = function(evt) {
  const data = JSON.parse(evt.data);
  console.log("接收到行情数据:", data);
};
```

### 使用QADB-RS存储和查询数据

#### 配置QADB-RS作为WebSocket客户端

```bash
# 通过环境变量配置
export QADB_WS_CLIENT_ENABLED=true
export QADB_WS_CLIENT_HOST=localhost
export QADB_WS_CLIENT_PORT=8014
export QADB_WS_CLIENT_PATH=/ws/market
export QADB_WS_CLIENT_STREAM=futures_market
export QADB_WS_CLIENT_INSTRUMENTS=au2512,rb2512,IF2506

# 启动QADB-RS
./target/release/qadb-rs --mode ingest
```

#### 通过HTTP API查询数据

```bash
# 查询最近1小时的期货行情数据
curl -X GET "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -d '{
    "stream": "futures_market",
    "start_time": "1h-ago",
    "end_time": "now",
    "limit": 1000,
    "filter": "instrument='au2512'"
  }'
```

### 使用QATrader-RS进行交易

#### 配置交易账户

编辑`conf/config.toml`文件，设置交易账户信息：

```toml
[common]
log_level = "info"
version = "1.0"

[account]
account_cookie = "YOUR_ACCOUNT"
password = "YOUR_PASSWORD"
broker_id = "YOUR_BROKER_ID"
td_server = "tcp://YOUR_SERVER:PORT"
appid = "YOUR_APP_ID"
auth_code = "YOUR_AUTH_CODE"

[websocket]
market_ws = "ws://localhost:8014/ws/market"

[mq]
uri = "amqp://admin:admin@localhost:5672"
exchange = "qaorder"
model = "fanout"
routing_key = ""
```

#### 启动交易引擎

```bash
cargo run -p qatrader-rs --release -- -c conf/config.toml
```

#### 发送交易指令

通过消息队列发送交易指令：

```json
{
  "topic": "order",
  "action": "send_order",
  "account_cookie": "acc001",
  "data": {
    "instrument_id": "cu2109",
    "exchange_id": "SHFE",
    "price": 75000,
    "volume": 1,
    "direction": "BUY",
    "offset": "OPEN"
  }
}
```

## 📊 性能指标

- 市场数据网关可处理每秒10000+条行情更新
- QADB-RS支持每秒100000+条时序数据点的写入
- 查询性能可达亚毫秒级别响应时间（取决于查询复杂度和数据量）
- 交易订单处理延迟<1ms
- 支持每秒1000+笔订单处理
- 关键交易功能99.99%的可用性

## 📄 许可证

[版权信息]

## 👥 贡献指南

欢迎提交Pull Request贡献代码！

## 📬 联系方式

如有问题或需要支持，请联系[联系方式]