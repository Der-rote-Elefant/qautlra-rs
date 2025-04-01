# WebSocket 客户端开发指南

本指南详细介绍如何开发 WebSocket 客户端连接到 QADB-RS 服务，以及如何配置 QADB-RS 作为 WebSocket 客户端获取市场数据。

## 1. QADB-RS WebSocket 接口

QADB-RS 提供 WebSocket 服务器功能，允许客户端通过 WebSocket 协议进行实时数据交互。主要功能包括：

- 实时接收市场数据
- 订阅/取消订阅特定交易品种
- 发送查询请求
- 接收数据更新通知

## 2. 开发 WebSocket 客户端

### 2.1 连接规范

WebSocket 端点格式：`ws://<host>:<port>/<path>`

示例：`ws://localhost:18003/ws`

### 2.2 认证

如果 QADB-RS 启用了认证，需要在连接时提供用户名和密码。有两种方式：

1. **查询参数方式**：
   ```
   ws://localhost:18003/ws?username=admin&password=admin
   ```

2. **HTTP 头部方式**：
   在建立连接时设置 HTTP 头部：
   ```
   Authorization: Basic <base64编码的username:password>
   ```

### 2.3 消息格式

QADB-RS WebSocket 服务支持两种格式的消息：

1. **JSON 格式**：结构化数据，易于解析
2. **二进制格式**：高效率，低延迟

#### JSON 消息格式

```json
{
  "type": "数据类型",
  "stream": "流名称",
  "data": {
    // 数据内容，格式取决于type
  }
}
```

常见消息类型：
- `marketData`: 市场数据更新
- `subscribe`: 订阅请求
- `unsubscribe`: 取消订阅请求
- `query`: 查询请求
- `error`: 错误消息

#### 二进制消息格式

二进制消息采用特定的布局，头部包含消息类型和数据长度，后面是实际数据内容（通常是 Arrow 格式）。

### 2.4 订阅和取消订阅

订阅特定交易品种：

```json
{
  "type": "subscribe",
  "stream": "market_data",
  "data": {
    "instruments": ["au2412", "rb2412", "IF2406"]
  }
}
```

取消订阅：

```json
{
  "type": "unsubscribe",
  "stream": "market_data",
  "data": {
    "instruments": ["au2412"]
  }
}
```

### 2.5 错误处理

连接 WebSocket 服务器时应实现适当的错误处理和重连机制：

- 处理连接断开事件
- 实现指数退避重连策略
- 在重连后重新订阅
- 处理心跳消息确保连接活跃

## 3. 使用各种语言开发客户端

### 3.1 Python 客户端示例

使用 `websockets` 库：

```python
import asyncio
import json
import websockets

async def connect_qadbrs():
    uri = "ws://localhost:18003/ws"
    async with websockets.connect(uri) as websocket:
        # 订阅消息
        subscribe_msg = {
            "type": "subscribe",
            "stream": "market_data",
            "data": {
                "instruments": ["au2412", "rb2412", "IF2406"]
            }
        }
        await websocket.send(json.dumps(subscribe_msg))
        
        # 接收消息
        while True:
            try:
                message = await websocket.recv()
                data = json.loads(message)
                print(f"收到数据: {data}")
            except websockets.ConnectionClosed:
                print("连接已关闭")
                break

asyncio.run(connect_qadbrs())
```

### 3.2 JavaScript 客户端示例

使用浏览器 WebSocket API：

```javascript
function connectToQADBRS() {
    const socket = new WebSocket('ws://localhost:18003/ws');
    
    socket.onopen = function(e) {
        console.log("连接已建立");
        
        // 订阅消息
        const subscribeMsg = {
            type: "subscribe",
            stream: "market_data",
            data: {
                instruments: ["au2412", "rb2412", "IF2406"]
            }
        };
        
        socket.send(JSON.stringify(subscribeMsg));
    };
    
    socket.onmessage = function(event) {
        const data = JSON.parse(event.data);
        console.log(`收到数据: ${JSON.stringify(data)}`);
    };
    
    socket.onclose = function(event) {
        if (event.wasClean) {
            console.log(`连接已关闭: 代码=${event.code} 原因=${event.reason}`);
        } else {
            console.log('连接异常断开');
        }
    };
    
    socket.onerror = function(error) {
        console.log(`错误: ${error.message}`);
    };
    
    return socket;
}

const ws = connectToQADBRS();
```

### 3.3 Rust 客户端示例

使用 `tokio-tungstenite` 库：

```rust
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://localhost:18003/ws";
    
    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket 已连接");
    
    let (mut write, mut read) = ws_stream.split();
    
    // 发送订阅消息
    let subscribe_msg = json!({
        "type": "subscribe",
        "stream": "market_data",
        "data": {
            "instruments": ["au2412", "rb2412", "IF2406"]
        }
    });
    
    write.send(Message::Text(subscribe_msg.to_string())).await?;
    
    // 接收消息
    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    let data: Value = serde_json::from_str(&text)?;
                    println!("收到数据: {}", data);
                }
            },
            Err(e) => {
                eprintln!("错误: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}
```

## 4. 配置 QADB-RS 作为 WebSocket 客户端

QADB-RS 本身可以作为 WebSocket 客户端连接到外部数据源（如交易所或行情供应商），并将接收到的数据存储到指定流中。

### 4.1 通过环境变量配置

```bash
export QADB_WS_CLIENT_ENABLED=true
export QADB_WS_CLIENT_HOST=192.168.2.115
export QADB_WS_CLIENT_PORT=8014
export QADB_WS_CLIENT_PATH=/ws/market
export QADB_WS_CLIENT_STREAM=futures_market
export QADB_WS_CLIENT_INSTRUMENTS=au2412,rb2412,IF2406

./target/release/qadbrs local-store --address 0.0.0.0:18000 --data-dir /path/to/data
```

### 4.2 通过命令行参数配置（如果后续版本支持）

```bash
./target/release/qadbrs local-store \
  --address 0.0.0.0:18000 \
  --data-dir /path/to/data \
  --ws-client-enabled \
  --ws-client-host 192.168.2.115 \
  --ws-client-port 8014 \
  --ws-client-path /ws/market \
  --ws-client-stream futures_market \
  --ws-client-instruments au2412,rb2412,IF2406
```

### 4.3 配置参数说明

| 参数 | 环境变量 | 描述 |
|------|---------|------|
| `ws-client-enabled` | `QADB_WS_CLIENT_ENABLED` | 启用 WebSocket 客户端功能 |
| `ws-client-host` | `QADB_WS_CLIENT_HOST` | 要连接的 WebSocket 服务器主机名或 IP |
| `ws-client-port` | `QADB_WS_CLIENT_PORT` | 要连接的 WebSocket 服务器端口 |
| `ws-client-path` | `QADB_WS_CLIENT_PATH` | WebSocket 路径，默认为 `/ws` |
| `ws-client-stream` | `QADB_WS_CLIENT_STREAM` | 数据将保存到的流名称 |
| `ws-client-instruments` | `QADB_WS_CLIENT_INSTRUMENTS` | 要订阅的交易品种，逗号分隔 |
| `ws-client-username` | `QADB_WS_CLIENT_USERNAME` | 认证用户名（如需要） |
| `ws-client-password` | `QADB_WS_CLIENT_PASSWORD` | 认证密码（如需要） |

## 5. 常见问题排查

### 5.1 连接问题

- **连接被拒绝**：检查服务器地址、端口和路径是否正确
- **认证失败**：确认用户名和密码正确，检查认证方式
- **连接超时**：检查网络连接和防火墙设置

### 5.2 数据问题

- **没有收到数据**：确认已正确订阅所需的交易品种
- **数据格式错误**：检查发送的消息格式是否符合要求
- **数据延迟**：检查网络质量和服务器负载

### 5.3 性能优化

- 使用二进制消息格式而非 JSON 可减少网络带宽并提高性能
- 只订阅必要的交易品种，减少数据量
- 实现适当的重连和错误处理机制，提高稳定性
- 对于高频数据，考虑使用多个并行连接来分散负载

## 6. 扩展阅读

- [QADB-RS 使用指南](./qadbrs-usage-guide.md)
- [市场数据网关开发指南](./market-data-gateway-guide.md)
- [量化策略开发指南](./quant-strategy-guide.md) 