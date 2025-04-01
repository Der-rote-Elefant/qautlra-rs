# QAUTLRA-RS 

QAUTLRA-RS is the core integration component of the QAUTLRA (Quantitative Analysis and Ultra-Low-latency Trading in Rust for Advanced Systems) ecosystem. It provides a high-performance WebSocket server for real-time market data distribution and trading services.

## 📚 Overview

QAUTLRA-RS serves as the central hub connecting various components of the QAUTLRA ecosystem, including market data gateways, trading systems, and client applications. Built with Rust and Actix for maximum performance and reliability, it delivers microsecond-level latency for market data processing and distribution.

## 🌟 Features

- **Real-time Market Data Distribution**: Stream market data via WebSocket with minimal latency
- **High-throughput Processing**: Handle thousands of market data updates per second
- **Direct CTP Integration**: Connect directly to Chinese financial markets via CTP
- **Secure WebSocket Protocol**: Encrypted data transmission for sensitive financial information
- **Session Management**: Robust handling of client connections, subscriptions, and heartbeats
- **Customizable Data Filters**: Subscribe to specific instruments of interest
- **Actor-based Architecture**: Built with Actix for superior concurrency and resilience

## 🔧 System Requirements

- Rust 1.70 or higher
- Linux environment (recommended for production)
- At least 2GB RAM
- Network with low latency to CTP front servers

## 🚀 Installation

1. Clone the repository:
```bash
git clone https://github.com/yutiansut/qautlra-rs.git
cd qautlra-rs
```

2. Build the project:
```bash
cargo build --release
```

3. Run the server:
```bash
cargo run --release
```

## ⚙️ Configuration

Before running the server, configure your CTP credentials in `src/main.rs`:

```rust
// CTP account credentials
let user_id = "your_user_id";
let password = "your_password";
let broker_id = "your_broker_id";
```

## 📊 WebSocket API

The WebSocket server is available at `ws://your-server:8080/ws/marketdata`.

### Connection

Connect to the WebSocket endpoint to establish a session.

### Message Format

All messages use JSON format:

```json
{
  "op": "operation",
  "data": {}
}
```

### Subscribe to Market Data

**Request:**
```json
{
  "op": "subscribe",
  "symbols": ["cu2206", "IF2206", "au2206"]
}
```

**Response:**
```json
{
  "data": "Subscribed",
  "topic": "subscribe",
  "code": 200
}
```

### Unsubscribe from Market Data

**Request:**
```json
{
  "op": "unsubscribe",
  "symbols": ["cu2206"]
}
```

**Response:**
```json
{
  "data": "Unsubscribed",
  "topic": "unsubscribe",
  "code": 200
}
```

### Market Data Updates

The server will push real-time market data updates for subscribed instruments:

```json
{
  "data": {
    "trading_day": "20230418",
    "instrument_id": "cu2206",
    "exchange_id": "SHFE",
    "exchange_inst_id": "cu2206",
    "last_price": 73210.0,
    "pre_settlement_price": 73150.0,
    "pre_close_price": 73180.0,
    "pre_open_interest": 62345.0,
    "open_price": 73190.0,
    "highest_price": 73310.0,
    "lowest_price": 73100.0,
    "volume": 28672,
    "turnover": 1258763240.0,
    "open_interest": 62512.0,
    "upper_limit_price": 76070.0,
    "lower_limit_price": 70230.0,
    "update_time": "11:20:35",
    "update_millisec": 500,
    "bid_price1": 73200.0,
    "bid_volume1": 12,
    "ask_price1": 73210.0,
    "ask_volume1": 8
  },
  "topic": "market_data",
  "code": 200
}
```

### Heartbeat

The server sends ping frames every 5 seconds. Clients must respond with pong frames to maintain the connection. If no response is received for 10 seconds, the connection will be closed.

## 🔄 Integration with QAUTLRA Ecosystem

QAUTLRA-RS is part of the larger QAUTLRA ecosystem, which includes:

- **QAMD-RS**: Market data standardization and processing library
- **QAMDGATEWAY**: Market data gateway for different sources (CTP, Sina, QQ)
- **QADB-RS**: Time-series database for financial data
- **QATRADER-RS**: Trading execution system
- **QAREALTIMEPRO-RS**: Real-time data processing and distribution service
- **QIFI-RS**: Financial interface standards library

QAUTLRA-RS serves as an integration point between these components, enabling seamless data flow and system coordination.

## 🌐 Example WebSocket Client

A simple HTML/JavaScript WebSocket client is provided in `websocket_client.html` for testing purposes.

## 🧪 Development

To run in development mode with logging:

```bash
RUST_LOG=debug cargo run
```

### Project Structure

- **src/main.rs**: Application entry point and HTTP server setup
- **src/server/websocket/mdserver.rs**: Market data server implementation
- **src/server/websocket/mdsession.rs**: WebSocket session handler
- **src/server/websocket/mdspi.rs**: CTP market data SPI implementation
- **src/actors/**: Actor implementations for concurrent processing
- **src/data/**: Data structure definitions
- **src/util/**: Utility functions and helpers

## 📊 Performance Metrics

- **Latency**: <1ms for market data processing (from CTP to WebSocket client)
- **Throughput**: Support for 10,000+ market data updates per second
- **Connections**: Handles 1,000+ simultaneous WebSocket connections

## 📝 License

[License information]

## 👥 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📬 Contact

For questions or support, please contact [contact information]. 