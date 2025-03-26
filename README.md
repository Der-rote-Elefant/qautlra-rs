# QAUTLRA-RS

QAUTLRA-RS是一个基于Rust实现的高性能量化交易和市场数据处理平台，为金融市场分析和交易提供完整的技术基础设施。

![QAUTLRA-RS](qars/QUANTAXISRS.png)

## 🚀 项目概述

QAUTLRA-RS项目旨在构建一个完整的量化交易技术栈，充分利用Rust语言的高性能、内存安全和并发特性，为量化交易提供可靠的技术支持。平台由多个独立但协同工作的子系统组成，支持从市场数据接入、行情分发、策略执行到交易执行的完整量化交易流程。

## 🌟 核心特性

- **高性能架构**：基于Rust语言开发，实现毫秒级低延迟数据处理
- **多源数据接入**：支持CTP、新浪财经、腾讯财经等多种市场数据源
- **实时行情分发**：基于WebSocket的高效行情数据推送
- **统一数据格式**：标准化不同来源的市场数据，提供一致的数据接口
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

**子组件**:
- **qamdgateway-ctp**: 连接CTP交易系统的市场数据网关
- **qamdgateway-qq**: 连接腾讯财经的市场数据网关
- **qamdgateway-sina**: 连接新浪财经的市场数据网关

### 2. QAMD-RS - 市场数据处理库

市场数据处理库提供了标准化的数据结构和处理工具，用于处理和转换各类市场数据。

**主要功能**:
- 定义标准化的市场数据结构
- 提供数据转换和处理工具
- 支持不同来源数据的格式转换

### 3. QARS - 量化交易引擎核心库

QARS是系统的核心计算引擎，提供量化分析和交易策略执行的功能。

**主要功能**:
- 高性能回测引擎
- 因子计算和分析工具
- 策略执行引擎
- 支持与Python交互的接口

### 4. QIFI-RS - 金融交易接口标准

QIFI-RS是Quantaxis Financial Interface的Rust实现，提供标准化的金融交易接口协议。

**主要功能**:
- 定义标准账户数据结构
- 提供快速的数据序列化和反序列化
- 支持从JSON/BSON格式加载和转换数据
- 兼容MongoDB数据库操作

### 5. CTP-MD - CTP市场数据接口

CTP市场数据接口提供了连接中国金融期货交易所CTP系统的Rust绑定。

**主要功能**:
- 提供CTP API的Rust封装
- 实现CTP市场数据的接收和处理
- 支持多种期货合约的订阅

### 6. QAOMS - 量化交易前端系统

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
┌─────────────────┐  ┌───────────────────┐  ┌──────────────────┐
│                 │  │                   │  │                  │
│ CTP Market Data │─▶│ Market Data       │◀─│ WebSocket        │
│ Source (Actor)  │  │ Connector         │  │ Clients          │
│                 │  │ (Actor)           │  │                  │
└─────────────────┘  └───────────────────┘  └──────────────────┘
                        ▲     ▲
┌─────────────────┐     │     │     ┌──────────────────┐
│                 │     │     │     │                  │
│ QQ Market Data  │────▶│     │◀────│ REST API         │
│ Source (Actor)  │           │     │ Clients          │
│                 │           │     │                  │
└─────────────────┘           │     └──────────────────┘
                              │
┌─────────────────┐           │
│                 │           │
│ Sina Market Data│──────────▶│
│ Source (Actor)  │
│                 │
└─────────────────┘
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

3. 构建前端系统：

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

### REST API使用示例

```bash
# 获取当前订阅信息
curl http://localhost:8015/api/subscriptions

# 订阅新合约
curl -X POST http://localhost:8015/api/subscribe \
  -H "Content-Type: application/json" \
  -d '{"instruments": ["au2512", "rb2512"]}'

# 取消订阅
curl -X POST http://localhost:8015/api/unsubscribe \
  -H "Content-Type: application/json" \
  -d '{"instruments": ["au2512"]}'
```

## 📊 数据结构

### 标准市场数据格式

```json
{
  "type": "market_data",
  "payload": {
    "data": {
      "instrument_id": "SHFE.au2512",
      "exchange_id": "SHFE",
      "instrument_name": "黄金2512",
      "datetime": "2023-05-20T10:30:00.000Z",
      "last_price": 2056.5,
      "open": 2050.0,
      "high": 2060.0,
      "low": 2048.5,
      "volume": 12500,
      "open_interest": 25678,
      "bid_price1": 2056.0,
      "bid_volume1": 56,
      "ask_price1": 2057.0,
      "ask_volume1": 78,
      "source": "CTP"
    }
  }
}
```

### QIFI账户数据结构

QIFI-RS提供了标准化的账户数据结构，包括账户资金、持仓、订单和交易记录等信息。

## 🧩 组件详情

### QAMDGATEWAY

市场数据网关基于Actix框架实现Actor模型，主要组件包括：

- **MarketDataConnector**：负责连接和管理多个市场数据源
- **MarketDataDistributor**：负责将市场数据分发给订阅的客户端
- **MarketDataActor**：处理各类行情源的Actor实现
- **WebSocket Server**：通过WebSocket协议对外提供行情数据

### QARS

量化交易引擎核心库提供了多种量化分析和交易功能：

- 标准账户协议（股票/期货）
- 投资组合协议（多账户管理）
- 标准市场数据接口
- 标准连接协议（MongoDB、ClickHouse、RabbitMQ）
- 本地文件系统接口

### QAOMS

前端系统基于Vue.js开发，提供了友好的用户界面：

- 行情数据展示
- K线图表
- 交易操作界面
- 账户管理

## 🤝 贡献指南

欢迎贡献代码或提出建议！请遵循以下步骤：

1. Fork 仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建Pull Request

## 📄 开源协议

本项目采用MIT许可证。详情请参见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

感谢所有为本项目做出贡献的社区成员。