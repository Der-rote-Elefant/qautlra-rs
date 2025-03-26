# QAMD Gateway

A gateway service that connects CTP market data sources to QAMD data structures, with real-time distribution via WebSocket.

## Features

- Connects to CTP market data providers
- Converts CTP market data to QAMD MDSnapshot format
- Real-time market data distribution via WebSocket
- RESTful API for subscription management
- Configurable broker connections
- Support for multiple instrument subscriptions

## Architecture

```
   +----------------+        +----------------+        +----------------+
   |                |        |                |        |                |
   |  CTP Market    |------->|  QAMD Gateway  |------->|  WebSocket    |
   |  Data Source   |        |  Converter     |        |  Clients      |
   |                |        |                |        |                |
   +----------------+        +----------------+        +----------------+
                                    |
                                    v
                             +----------------+
                             |                |
                             |  REST API      |
                             |  Clients       |
                             |                |
                             +----------------+
```

## Prerequisites

- Rust 1.70+
- CTP API libraries (provided in the repo)
- Network access to CTP front addresses

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yutiansut/qautlra-rs.git
   cd qautlra-rs
   ```

2. Build the QAMD Gateway:
   ```bash
   cargo build --release --package qamdgateway
   ```

3. Update the `config.json` with your broker information.

4. Run the gateway:
   ```bash
   ./target/release/qamdgateway
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
      // Other fields...
    }
  }
}
```

## License

MIT License 