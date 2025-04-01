# QAUTLRA-RS

QAUTLRA-RS (Quantitative Analysis and Ultra-Low-latency Trading in Rust for Advanced Systems) is a high-performance quantitative trading and market data processing platform implemented in Rust, providing a complete technical infrastructure for financial market analysis and trading.

![QAUTLRA-RS](qars/QUANTAXISRS.png)

## 🚀 Project Overview

QAUTLRA-RS aims to build a complete quantitative trading technology stack, fully leveraging Rust's high performance, memory safety, and concurrency features to provide reliable technical support for quantitative trading. The platform consists of multiple independent but collaborative subsystems, supporting the entire quantitative trading process from market data access, market data distribution, strategy execution to trade execution.

## 🌟 Core Features

- **High-Performance Architecture**: Developed in Rust, achieving millisecond-level low-latency data processing
- **Multi-Source Data Access**: Support for various market data sources including CTP, Sina Finance, Tencent Finance, etc.
- **Real-time Market Data Distribution**: Efficient market data streaming based on WebSocket
- **Incremental Data Updates**: Send full data on first connection, then only push changed fields, greatly reducing bandwidth consumption
- **Unified Data Format**: Standardize market data from different sources, providing consistent data interfaces
- **High-Performance Time Series Database**: Time series data storage and query engine based on Apache Arrow and DataFusion
- **Distributed Design**: Distributed concurrent processing architecture based on the Actor model
- **Safe and Reliable**: Fully utilize Rust's memory safety features to provide a stable and reliable environment
- **Modular Components**: Subsystems can operate independently or collaboratively

## 📚 System Components

QAUTLRA-RS consists of the following major components:

### 1. QAUTLRA-RS - Core Integration Component

The central hub connecting various components of the QAUTLRA ecosystem, including market data gateways, trading systems, and client applications. It provides a high-performance WebSocket server for real-time market data distribution and trading services.

**Key Features**:
- Real-time market data distribution via WebSocket
- High-throughput data processing
- Direct CTP integration 
- Session management and subscription handling
- WebSocket API for data streaming

[Learn more about QAUTLRA-RS](qautlra-rs/README.md)

### 2. QAMDGATEWAY - Market Data Gateway

The market data gateway is the data entry point for the entire system, responsible for connecting various market data sources, converting the data into a unified format, and distributing it to subscribers.

**Key Features**:
- Connect and manage multiple market data sources
- Convert raw market data into standard QAMD data structures
- Real-time market data streaming via WebSocket
- Provide REST API for subscription management
- Support incremental data updates to significantly improve performance and reduce network traffic

**Sub-components**:
- **qamdgateway-ctp**: Market data gateway connecting to CTP trading system
- **qamdgateway-qq**: Market data gateway connecting to Tencent Finance
- **qamdgateway-sina**: Market data gateway connecting to Sina Finance

### 3. QAREALTIMEPRO-RS - Real-time Data Processing

QAREALTIMEPRO-RS is a high-performance real-time market data processing and distribution service. It serves as a central hub for connecting different market data sources, processing data in real-time, and distributing it to clients via WebSocket and REST APIs.

**Key Features**:
- Real-time data processing for both stock and futures markets
- Trading account integration for position and PnL monitoring
- REST and WebSocket APIs for data access
- Redis integration for high-performance caching
- Actor-based architecture for concurrency and resilience

[Learn more about QAREALTIMEPRO-RS](qarealtimepro-rs/README.md)

### 4. QATRADESERVER-RS - Trading Gateway

QATRADESERVER-RS is a high-performance trading server that acts as a centralized trading gateway, managing connections to multiple brokers, handling trade execution requests, and providing real-time trade updates.

**Key Features**:
- Multi-broker gateway for centralized access to various exchanges
- Order routing to appropriate brokers
- Load balancing for trading requests
- High availability design
- WebSocket and REST APIs for trade operations
- Authentication and rate limiting for security

[Learn more about QATRADESERVER-RS](qatradeserver-rs/README.md)

### 5. QATRADER-RS - Trading Engine

QATRADER-RS is a high-performance trading order management and execution system. It serves as the core trading engine, handling order routing, execution, risk management, and position tracking.

**Key Features**:
- High-performance trading with low-latency execution
- Support for multiple brokers and trading venues
- Event-driven architecture for concurrent processing
- Pre-trade and post-trade risk management
- Real-time position tracking and PnL monitoring
- Message queue integration for order flow

[Learn more about QATRADER-RS](qatrader-rs/README.md)

### 6. QAMAINTAINDB-RS - Financial Performance Analysis

QAMAINTAINDB-RS is a specialized tool for maintaining and analyzing financial product performance data. It provides a robust API for data ingestion, performance calculation, and comparison analysis.

**Key Features**:
- Excel data import for financial product data
- Comprehensive performance metrics calculation
- Product comparison over specified time periods
- Date range filtering for analysis
- Actor-based parallel computation
- MongoDB integration for data persistence

[Learn more about QAMAINTAINDB-RS](qamaintaindb-rs/README.md)

### 7. QADB-RS - High-Performance Time Series Database

QADB-RS is a high-performance time series database system designed specifically for quantitative trading, built on Apache Arrow and DataFusion, enabling efficient storage and querying of financial market data.

**Key Features**:
- Efficient storage and management of large-scale financial market data
- Receive market data directly via WebSocket client
- Distribute data via WebSocket server
- Support Kafka connector to receive data from message queues (optional)
- Automatic management of dynamic data schemas
- Provide HTTP-based query API
- Support time and custom field partitioned storage
- Seamless integration with quantitative trading engine

**Technical Highlights**:
- Columnar storage format based on Apache Arrow
- High-performance query engine with DataFusion
- High-speed writing and querying of millisecond-level market data
- Optimized time series data compression algorithms
- Partitioned storage to improve query efficiency

### 8. QAMD-RS - Market Data Processing Library

The market data processing library provides standardized data structures and processing tools for handling and converting various types of market data.

**Key Features**:
- Define standardized market data structures
- Provide data conversion and processing tools
- Support format conversion for data from different sources

### 9. QARS - Quantitative Trading Engine Core Library

QARS is the core computation engine of the system, providing quantitative analysis and trading strategy execution functions.

**Key Features**:
- High-performance backtesting engine
- Factor calculation and analysis tools
- Strategy execution engine
- Interface supporting Python interaction

### 10. QIFI-RS - Financial Interface Standard

QIFI-RS is the Rust implementation of Quantaxis Financial Interface, providing a standardized financial trading interface protocol.

**Key Features**:
- Define standard account data structures
- Provide fast data serialization and deserialization
- Support loading and converting data from JSON/BSON format
- Compatible with MongoDB database operations

### 11. CTP-MD - CTP Market Data Interface

The CTP market data interface provides Rust bindings for connecting to the CTP system of China Financial Futures Exchange.

**Key Features**:
- Provide Rust wrapper for CTP API
- Implement reception and processing of CTP market data
- Support subscription to various futures contracts

### 12. QAOMS - Quantitative Trading Frontend System

The quantitative trading frontend system provides a user interface, including market data display, trading operations, and strategy management.

**Key Features**:
- Market data visualization
- Trading operation interface
- Strategy management and monitoring
- Account and position display

## 🔧 System Architecture

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

## 📦 Installation Guide

### Requirements

- Rust (recommended version: 1.70+)
- Cargo
- CMake (for compiling CTP-related dependencies)
- GCC/Clang (C++ compiler)
- Node.js (for frontend system)
- MongoDB
- Redis
- RabbitMQ

### Building from Source

1. Clone the repository:

```bash
git clone https://github.com/yutiansut/qautlra-rs.git
cd qautlra-rs
```

2. Build the market data gateway:

```bash
# Build market data gateway with CTP support
QAMDGATEWAY_CONFIG_PATH=config_ctp.json cargo run -p qamdgateway --no-default-features --features="ctp"

# Build market data gateway with Tencent Finance support
QAMDGATEWAY_CONFIG_PATH=config_qq.json cargo run -p qamdgateway --no-default-features --features="qq"

# Build market data gateway with Sina Finance support
QAMDGATEWAY_CONFIG_PATH=config_sina.json cargo run -p qamdgateway --no-default-features --features="sina"
```

3. Build and run the QADB-RS time series database:

```bash
# Build with full functionality
cargo build -p qadb-rs --release

# Build with WebSocket functionality only
cargo build -p qadb-rs --release --features="websocket"

# Run QADB-RS service
QADB_DATA_DIR=/path/to/data ./target/release/qadb-rs --mode all --websocket-enabled
```

4. Build and run QARealTimePro-RS:

```bash
cargo build -p qarealtimepro-rs --release
cargo run -p qarealtimepro-rs --release
```

5. Build and run QATrader-RS:

```bash
cargo build -p qatrader-rs --release
cargo run -p qatrader-rs --release
```

6. Build and run QATradeServer-RS:

```bash
cargo build -p qatradeserver-rs --release
cargo run -p qatradeserver-rs --release
```

7. Build and run QAMaintainDB-RS:

```bash
cargo build -p qamaintaindb-rs --release
cargo run -p qamaintaindb-rs --release
```

8. Build the frontend system:

```bash
cd qaoms/web
npm install
npm run dev
```

## ⚙️ Configuration

Configuration files are located in the root directory of the project, including `config.json` and its variants, containing the following main configurations:

### CTP Market Data Gateway Configuration Example

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

### QADB-RS Configuration Example

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

## 📝 License

[License information]

## 👥 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📬 Contact

For questions or support, please contact [contact information]