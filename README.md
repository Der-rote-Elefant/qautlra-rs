# QAUTLRA-RS

QAUTLRA-RS是一个基于Rust实现的高性能量化交易和市场数据处理平台，为金融市场分析和交易提供完整的技术基础设施。

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

### 1. QAMDGATEWAY - 市场数据网关

```
写在前面：

首先致谢罗总的openctp提供的标准的行情接入 更多openctp的相关可以参考

openctp 官网 [http://www.openctp.cn/]

openctp 的github  [https://github.com/openctp/openctp]

@yutiansut 

```

市场数据网关是整个系统的数据入口，负责连接各类市场数据源，并将数据转换为统一格式后分发给订阅者。

**主要功能**:
- 连接和管理多个市场数据源
- 将原始市场数据转换为标准QAMD数据结构
- 通过WebSocket实时推送行情数据
- 提供REST API用于订阅管理
- 支持增量数据更新，大幅提升性能和减少网络流量

**子组件**:
- **qamdgateway-ctp**: 连接CTP交易系统的市场数据网关
- **qamdgateway-qq**: 连接腾讯财经的市场数据网关
- **qamdgateway-sina**: 连接新浪财经的市场数据网关

### 2. QADB-RS - 高性能时序数据库

QADB-RS是一个专为量化交易设计的高性能时序数据库系统，基于Apache Arrow和DataFusion构建，实现了高效的金融市场数据存储和查询功能。

**主要功能**:
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

### 3. QAMD-RS - 市场数据处理库

市场数据处理库提供了标准化的数据结构和处理工具，用于处理和转换各类市场数据。

**主要功能**:
- 定义标准化的市场数据结构
- 提供数据转换和处理工具
- 支持不同来源数据的格式转换

### 4. QARS - 量化交易引擎核心库

QARS是系统的核心计算引擎，提供量化分析和交易策略执行的功能。

**主要功能**:
- 高性能回测引擎
- 因子计算和分析工具
- 策略执行引擎
- 支持与Python交互的接口

### 5. QIFI-RS - 金融交易接口标准

QIFI-RS是Quantaxis Financial Interface的Rust实现，提供标准化的金融交易接口协议。

**主要功能**:
- 定义标准账户数据结构
- 提供快速的数据序列化和反序列化
- 支持从JSON/BSON格式加载和转换数据
- 兼容MongoDB数据库操作

### 6. CTP-MD - CTP市场数据接口

CTP市场数据接口提供了连接中国金融期货交易所CTP系统的Rust绑定。

**主要功能**:
- 提供CTP API的Rust封装
- 实现CTP市场数据的接收和处理
- 支持多种期货合约的订阅

### 7. QAOMS - 量化交易前端系统

量化交易前端系统提供了用户交互界面，包括行情显示、交易操作和策略管理等功能。

**主要功能**:
- 行情数据可视化
- 交易操作界面
- 策略管理和监控
- 账户和持仓展示

## 🔧 系统架构

```
                                  ┌───────────────────┐
                                  │                   │
                                  │  Market Data      │
                                  │  Distributor      │
                                  │  (Actor)          │
                                  │                   │
                                  └───────────────────┘
                                    ▲      │
                                    │      ▼
┌─────────────────┐  ┌───────────────────┐  ┌──────────────────┐  ┌─────────────────┐
│                 │  │                   │  │                  │  │                 │
│ CTP Market Data │─▶│ Market Data       │◀─│ WebSocket        │─▶│ QADB-RS Time    │
│ Source (Actor)  │  │ Connector         │  │ Clients          │  │ Series Database │
│                 │  │ (Actor)           │  │                  │  │                 │
└─────────────────┘  └───────────────────┘  └──────────────────┘  └─────────────────┘
                        ▲     ▲                                      ▲
┌─────────────────┐     │     │     ┌──────────────────┐             │
│                 │     │     │     │                  │             │
│ QQ Market Data  │────▶│     │◀────│ REST API         │             │
│ Source (Actor)  │           │     │ Clients          │             │
│                 │           │     │                  │             │
└─────────────────┘           │     └──────────────────┘             │
                              │                                      │
┌─────────────────┐           │     ┌──────────────────┐             │
│                 │           │     │                  │             │
│ Sina Market Data│──────────▶│     │ QARS Trading     │─────────────┘
│ Source (Actor)  │                 │ Engine           │
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

# 构建支持QQ财经的市场数据网关
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

4. 构建前端系统：

```bash
cd qaoms/web
npm install
npm run dev
```

## ⚙️ 配置

配置文件位于项目根目录的`config.json`及其变体文件中，包含以下主要配置：

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
  "storage": {
    "data_dir": "/path/to/data",
    "retention_days": 30,
    "compression": "zstd"
  },
  "websocket": {
    "enabled": true,
    "address": "0.0.0.0",
    "port": 8765,
    "path": "/ws",
    "stream_name": "market_data",
    "client_enabled": true,
    "client_host": "localhost",
    "client_port": 8014,
    "client_path": "/ws/market",
    "client_instruments": ["au2512", "rb2512"]
  },
  "server": {
    "host": "0.0.0.0",
    "port": 8080
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

#### 处理增量数据更新

```javascript
// 客户端维护完整的行情快照
const marketDataSnapshots = {};

ws.onmessage = function(evt) {
  const data = JSON.parse(evt.data);
  
  // 处理每个合约的数据
  for (const instrument in data) {
    if (instrument === "aid") continue;
    
    // 如果是新合约，创建快照
    if (!marketDataSnapshots[instrument]) {
      marketDataSnapshots[instrument] = {};
    }
    
    // 更新快照中的字段
    for (const field in data[instrument]) {
      marketDataSnapshots[instrument][field] = data[instrument][field];
    }
    
    // 使用更新后的完整快照
    processMarketData(instrument, marketDataSnapshots[instrument]);
  }
};
```

## 📊 性能指标

- 市场数据网关可处理每秒10000+条行情更新
- QADB-RS支持每秒100000+条时序数据点的写入
- 查询性能可达亚毫秒级别响应时间（取决于查询复杂度和数据量）

## 🔍 进一步了解

- [QAdb-rs详细文档](qadb-rs/README.md)
- [市场数据网关开发指南](docs/mdgateway-development.md)
- [量化策略开发指南](docs/strategy-development.md)

## 📄 许可证

QAUTLRA-RS采用GNU Affero通用公共许可证v3.0（AGPL-3.0）进行许可。