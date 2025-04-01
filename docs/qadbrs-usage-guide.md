# QADB-RS 使用指南

`qadbrs` 是一个高性能时序数据库和数据摄取服务，专为量化交易场景设计。本指南详细说明了如何配置和运行 QADB-RS 服务。

## 1. 基本命令格式

QADB-RS 的基本命令格式为：

```bash
qadbrs <存储类型> [选项...]
```

其中 `<存储类型>` 必须是以下三种之一：
- `local-store`：使用本地文件系统存储
- `s3-store`：使用 S3 对象存储
- `blob-store`：使用 Azure Blob 存储

## 2. 获取帮助

获取所有可用的命令和选项：

```bash
# 查看主帮助
./target/release/qadbrs --help

# 查看特定存储类型的帮助
./target/release/qadbrs local-store --help
```

## 3. 常用配置示例

### 3.1 使用本地文件系统（最简单配置）

```bash
./target/release/qadbrs local-store --address 0.0.0.0:18000 --data-dir /path/to/data
```

这将启动一个监听在 `0.0.0.0:18000` 的服务器，数据存储在 `/path/to/data` 目录中。

### 3.2 选择运行模式

QADB-RS 支持三种运行模式：

- `all`：全功能模式，包括数据摄取和查询（默认）
- `ingest`：仅数据摄取模式
- `query`：仅数据查询模式

```bash
# 仅摄取模式
./target/release/qadbrs local-store --address 0.0.0.0:18000 --data-dir /path/to/data --mode ingest

# 仅查询模式
./target/release/qadbrs local-store --address 0.0.0.0:18000 --data-dir /path/to/data --mode query
```

### 3.3 配置 WebSocket 服务器（需要 websocket feature）

如果您使用 `--features websocket` 编译了 QADB-RS，可以启用 WebSocket 服务器功能：

```bash
./target/release/qadbrs local-store \
  --address 0.0.0.0:18000 \
  --data-dir /path/to/data \
  --websocket-enabled \
  --websocket-port 8765 \
  --websocket-path /ws \
  --websocket-stream market_data
```

### 3.4 配置 WebSocket 客户端（需要 websocket feature）

QADB-RS 可以作为 WebSocket 客户端连接到数据源：

```bash
QADB_WS_CLIENT_ENABLED=true \
QADB_WS_CLIENT_HOST=192.168.2.115 \
QADB_WS_CLIENT_PORT=8014 \
QADB_WS_CLIENT_PATH=/ws/market \
QADB_WS_CLIENT_STREAM=futures_market \
QADB_WS_CLIENT_INSTRUMENTS=au2412,rb2412,IF2406 \
./target/release/qadbrs local-store --address 0.0.0.0:18000 --data-dir /path/to/data --mode ingest
```

### 3.5 配置 Kafka 连接器（需要 kafka feature）

如果您使用 `--features kafka` 编译了 QADB-RS，可以配置 Kafka 连接器：

```bash
./target/release/qadbrs local-store \
  --address 0.0.0.0:18000 \
  --data-dir /path/to/data \
  --kafka-brokers kafka1:9092,kafka2:9092 \
  --kafka-group-id qadbrs-consumer \
  --kafka-topics logs,metrics \
  --kafka-auto-offset-reset earliest
```

### 3.6 TLS/HTTPS 配置

要启用 HTTPS，需要提供 TLS 证书和私钥：

```bash
./target/release/qadbrs local-store \
  --address 0.0.0.0:443 \
  --data-dir /path/to/data \
  --tls-cert-path /path/to/cert.pem \
  --tls-key-path /path/to/key.pem
```

## 4. 完整配置示例

下面是一个包含所有主要选项的完整配置示例：

```bash
./target/release/qadbrs local-store \
  --address 0.0.0.0:18000 \
  --data-dir /path/to/data \
  --staging-dir /path/to/staging \
  --mode all \
  --username admin_user \
  --password secure_password \
  --grpc-port 18001 \
  --flight-port 18002 \
  --max-disk-usage 90.0 \
  --row-group-size 262144 \
  --execution-batch-size 20000 \
  --compression-algo zstd \
  --websocket-enabled \
  --websocket-port 8765 \
  --websocket-path /ws \
  --websocket-stream market_data
```

## 5. 使用环境变量配置

所有参数都可以使用对应的环境变量进行配置：

```bash
export P_ADDR=0.0.0.0:18000
export P_DATA_DIR=/path/to/data
export P_STAGING_DIR=/path/to/staging
export P_MODE=all
export P_USERNAME=admin_user
export P_PASSWORD=secure_password
export P_GRPC_PORT=18001
export P_FLIGHT_PORT=18002

# WebSocket 客户端配置
export QADB_WS_CLIENT_ENABLED=true
export QADB_WS_CLIENT_HOST=192.168.2.115
export QADB_WS_CLIENT_PORT=8014
export QADB_WS_CLIENT_PATH=/ws/market
export QADB_WS_CLIENT_STREAM=futures_market

# 然后启动服务
./target/release/qadbrs local-store
```

## 6. 启动脚本

为简化配置，您可以创建一个启动脚本：

```bash
#!/bin/bash
# start-qadbrs.sh

# 基本配置
export P_ADDR=0.0.0.0:18000
export P_DATA_DIR=/path/to/data
export P_STAGING_DIR=/path/to/staging
export P_MODE=all

# WebSocket 客户端配置
export QADB_WS_CLIENT_ENABLED=true
export QADB_WS_CLIENT_HOST=192.168.2.115
export QADB_WS_CLIENT_PORT=8014
export QADB_WS_CLIENT_PATH=/ws/market
export QADB_WS_CLIENT_STREAM=futures_market
export QADB_WS_CLIENT_INSTRUMENTS=au2412,rb2412,IF2406

# WebSocket 服务器配置
WEBSOCKET_SERVER_ARGS="--websocket-enabled --websocket-port 8765 --websocket-path /ws --websocket-stream market_data"

# 启动 QADB-RS
./target/release/qadbrs local-store $WEBSOCKET_SERVER_ARGS "$@"
```

使用方法：

```bash
chmod +x start-qadbrs.sh
./start-qadbrs.sh --additional-args
```

## 7. 参数参考

### 7.1 基础服务配置

| 参数 | 环境变量 | 默认值 | 描述 |
|------|---------|-------|------|
| `--address` | `P_ADDR` | `0.0.0.0:18000` | 服务器监听地址和端口 |
| `--mode` | `P_MODE` | `all` | 运行模式 (all/ingest/query) |
| `--data-dir` | `P_DATA_DIR` | `./data` | 数据存储根目录 |
| `--staging-dir` | `P_STAGING_DIR` | `./staging` | 传入事件的暂存目录 |

### 7.2 认证配置

| 参数 | 环境变量 | 默认值 | 描述 |
|------|---------|-------|------|
| `--username` | `P_USERNAME` | `admin` | 管理员用户名 |
| `--password` | `P_PASSWORD` | `admin` | 管理员密码 |

### 7.3 服务端口配置

| 参数 | 环境变量 | 默认值 | 描述 |
|------|---------|-------|------|
| `--grpc-port` | `P_GRPC_PORT` | `18001` | gRPC 服务端口 |
| `--flight-port` | `P_FLIGHT_PORT` | `18002` | Arrow Flight 查询引擎端口 |

### 7.4 性能配置

| 参数 | 环境变量 | 默认值 | 描述 |
|------|---------|-------|------|
| `--max-disk-usage` | `P_MAX_DISK_USAGE_PERCENT` | `80.0` | 最大磁盘使用率百分比 |
| `--row-group-size` | `P_PARQUET_ROW_GROUP_SIZE` | `262144` | Parquet 行组大小 |
| `--execution-batch-size` | `P_EXECUTION_BATCH_SIZE` | `20000` | 查询执行批处理大小 |
| `--compression-algo` | `P_PARQUET_COMPRESSION_ALGO` | `zstd` | Parquet 压缩算法 |
| `--livetail-capacity` | `P_LIVETAIL_CAPACITY` | `1000` | 实时跟踪通道中的行数 |
| `--query-mempool-size` | `P_QUERY_MEMORY_LIMIT` | - | 查询的固定内存限制 (GiB) |

### 7.5 TLS/安全配置

| 参数 | 环境变量 | 默认值 | 描述 |
|------|---------|-------|------|
| `--tls-cert-path` | `P_TLS_CERT_PATH` | - | 证书文件路径 |
| `--tls-key-path` | `P_TLS_KEY_PATH` | - | 私钥文件路径 |
| `--trusted-ca-certs-path` | `P_TRUSTED_CA_CERTS_DIR` | - | 受信任证书目录 |

### 7.6 WebSocket 服务器配置 (需要 websocket feature)

| 参数 | 环境变量 | 默认值 | 描述 |
|------|---------|-------|------|
| `--websocket-enabled` | `QADB_WEBSOCKET_ENABLED` | `false` | 启用 WebSocket 服务器 |
| `--websocket-address` | `QADB_WEBSOCKET_ADDRESS` | `0.0.0.0` | WebSocket 服务器地址 |
| `--websocket-port` | `QADB_WEBSOCKET_PORT` | `18003` | WebSocket 服务器端口 |
| `--websocket-path` | `QADB_WEBSOCKET_PATH` | `/ws` | WebSocket 服务器路径 |
| `--websocket-stream` | `QADB_WEBSOCKET_STREAM` | - | WebSocket 数据流名称 |

### 7.7 WebSocket 客户端配置 (需要 websocket feature)

| 环境变量 | 默认值 | 描述 |
|---------|-------|------|
| `QADB_WS_CLIENT_ENABLED` | `false` | 启用 WebSocket 客户端 |
| `QADB_WS_CLIENT_HOST` | - | WebSocket 服务器主机 |
| `QADB_WS_CLIENT_PORT` | - | WebSocket 服务器端口 |
| `QADB_WS_CLIENT_PATH` | `/ws` | WebSocket 服务器路径 |
| `QADB_WS_CLIENT_STREAM` | - | 存储数据的流名称 |
| `QADB_WS_CLIENT_INSTRUMENTS` | - | 订阅的金融工具列表 (逗号分隔) |

## 8. 常见问题排查

### 8.1 无法启动服务

* **问题**: 启动时出现 `unexpected argument '--xxx' found` 错误
* **解决方案**: 确认使用了正确的参数名称，使用 `--help` 查看可用参数

### 8.2 端口绑定失败

* **问题**: 启动时出现 `Address already in use` 错误
* **解决方案**: 更改监听端口或停止使用该端口的其他服务

### 8.3 WebSocket 连接失败

* **问题**: WebSocket 客户端无法连接到数据源
* **解决方案**: 
  - 确认目标服务器地址和端口是否正确
  - 检查网络连接和防火墙设置
  - 验证 WebSocket 路径是否正确

### 8.4 无法写入数据

* **问题**: 出现权限或磁盘空间相关错误
* **解决方案**: 
  - 确保数据目录存在且有写入权限
  - 检查磁盘空间是否充足
  - 确保数据目录路径正确，使用绝对路径避免歧义 