# QAMD Gateway

A flexible gateway service that connects various market data sources (CTP, QQ, Sina) to QAMD data structures, with real-time distribution via WebSocket.

## Features

- Supports multiple market data sources:
  - CTP (China Financial Futures Exchange)
  - QQ Finance (腾讯财经)
  - Sina Finance (新浪财经)
- Converts market data to unified QAMD MDSnapshot format
- Real-time market data distribution via WebSocket
- RESTful API for subscription management
- Actor-based architecture for high concurrency and fault tolerance
- Configurable broker connections
- Support for multiple instrument subscriptions

## Architecture

```
                                  +-------------------+
                                  |                   |
                                  |  Market Data      |
                                  |  Distributor      |
                                  |  (Actor)          |
                                  |                   |
                                  +-------------------+
                                    ^      |
                                    |      v
+-----------------+  +---------------+  +------------------+
|                 |  |               |  |                  |
| CTP Market Data |->| Market Data   |<-| WebSocket       |
| Source (Actor)  |  | Connector     |  | Clients         |
|                 |  | (Actor)       |  |                  |
+-----------------+  +---------------+  +------------------+
                        ^     ^
+-----------------+     |     |     +------------------+
|                 |     |     |     |                  |
| QQ Market Data  |---->|     |<----| REST API         |
| Source (Actor)  |           |     | Clients          |
|                 |           |     |                  |
+-----------------+           |     +------------------+
                              |
+-----------------+           |
|                 |           |
| Sina Market Data|---------->|
| Source (Actor)  |
|                 |
+-----------------+
```

## Prerequisites

- Rust 1.70+
- API libraries for data sources (provided in the repo)
- Network access to market data front addresses

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yutiansut/qautlra-rs.git
   cd qautlra-rs
   ```

2. Build the QAMD Gateway with your desired market data sources:
   ```bash
   # Build with CTP support only (default)
   cargo build --release --package qamdgateway

   # Build with QQ finance support
   cargo build --release --package qamdgateway --features qq

   # Build with Sina finance support
   cargo build --release --package qamdgateway --features sina

   # Build with all market data sources
   cargo build --release --package qamdgateway --features all
   ```

3. Update the `config.json` with your broker information.

4. Run the gateway:
   ```bash
   ./target/release/qamdgateway


   QAMDGATEWAY_CONFIG_PATH=config_ctp.json cargo run -p qamdgateway --no-default-features --features="ctp"

   QAMDGATEWAY_CONFIG_PATH=config_qq.json cargo run -p qamdgateway --no-default-features --features="qq"
   
   QAMDGATEWAY_CONFIG_PATH=config_sina.json cargo run -p qamdgateway --no-default-features --features="sina"

   ```

## Configuration

The gateway is configured using a `config.json` file. Example:

```json
{
  "brokers": {
    "testbroker": {
      "name": "Test Broker",
      "front_addr": "tcp://180.166.103.21:57213",
      "user_id": "",
      "password": "",
      "broker_id": ""
    },
    "qqbroker": {
      "name": "QQ Finance",
      "front_addr": "tcp://quotes.qq.com/path",
      "user_id": "",
      "password": "",
      "broker_id": "qq"
    },
    "sinabroker": {
      "name": "Sina Finance",
      "front_addr": "tcp://hq.sinajs.cn/path",
      "user_id": "",
      "password": "",
      "broker_id": "sina"
    }
  },
  "default_broker": "testbroker",
  "websocket": {
    "host": "0.0.0.0",
    "port": 8081,
    "path": "/ws/market"
  },
  "rest_api": {
    "host": "0.0.0.0",
    "port": 8080,
    "cors": {
      "allow_all": true,
      "allowed_origins": [],
      "allow_credentials": true
    }
  },
  "subscription": {
    "default_instruments": [
      "au2412",
      "rb2412"
    ]
  }
}
```

## Actor System

The gateway uses an actor-based architecture for high concurrency and fault tolerance:

- **MarketDataActor**: Represents a connection to a specific market data source (CTP, QQ, Sina)
- **MarketDataConnector**: Manages multiple market data sources and forwards data from sources to distributor
- **MarketDataDistributor**: Distributes market data to subscribed clients
- **WebSocket Sessions**: Manages client connections and subscriptions

## API Usage

### REST API

#### Get Subscriptions
```
GET /api/subscriptions
```

#### Subscribe to Instruments
```
POST /api/subscribe
Content-Type: application/json

{
  "instruments": ["au2412", "rb2412"]
}
```

#### Unsubscribe from Instruments
```
POST /api/unsubscribe
Content-Type: application/json

{
  "instruments": ["au2412"]
}
```

### WebSocket API

Connect to WebSocket endpoint:
```
ws://localhost:8081/ws/market
```

#### Subscribe Message
```json
{
  "type": "subscribe",
  "payload": {
    "instruments": ["au2412", "rb2412"]
  }
}
```

#### Unsubscribe Message
```json
{
  "type": "unsubscribe",
  "payload": {
    "instruments": ["au2412"]
  }
}
```

#### Get Subscriptions
```json
{
  "type": "subscriptions"
}
```

#### Market Data Message (Received)
```json
{
  "type": "market_data",
  "payload": {
    "data": {
      "instrument_id": "SHFE_au2412",
      "last_price": 400.5,
      "source": "CTP",  // or "QQ" or "Sina"
      // Other fields...
    }
  }
}
```

## Feature Flags

- `ctp`: Enable CTP market data source (default)
- `qq`: Enable QQ Finance market data source
- `sina`: Enable Sina Finance market data source
- `all`: Enable all market data sources

## License

MIT License 