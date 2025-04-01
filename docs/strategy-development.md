# 量化策略开发指南

## 简介

本指南旨在帮助开发者使用QAUTLRA-RS生态系统开发高效的量化交易策略。QAUTLRA-RS提供了一套完整的工具链，支持从数据获取、策略回测到实盘交易的全流程量化交易开发。

## 技术栈概述

QAUTLRA-RS策略开发涉及以下核心组件：

1. **QADB-RS**：高性能时序数据库，用于存储和查询市场数据
2. **QARS**：量化分析和交易引擎，提供策略开发和执行的核心功能
3. **QIFI-RS**：金融交易接口标准，提供账户数据结构和交易接口
4. **QAMDGATEWAY**：市场数据网关，提供实时市场数据

## 策略开发流程

量化策略开发通常包括以下步骤：

1. **数据准备**：收集和处理历史与实时市场数据
2. **策略设计**：制定交易逻辑和规则
3. **回测验证**：在历史数据上验证策略表现
4. **参数优化**：调整策略参数以获得最佳性能
5. **实盘部署**：将策略部署到实盘环境中运行

## 数据准备

### 使用QADB-RS存储和查询数据

QADB-RS提供了高性能的时序数据存储和查询功能，是策略开发的数据基础。

#### 数据导入

可以通过以下方式将数据导入QADB-RS：

```rust
use qadb_rs::stream::Stream;
use qadb_rs::event::Event;

async fn import_historical_data() -> Result<()> {
    // 创建或获取流
    let stream = Stream::new("futures_data", None)?;
    
    // 准备事件数据
    let events = prepare_events_from_csv("path/to/data.csv")?;
    
    // 批量导入事件
    for event in events {
        stream.push_event(event)?;
    }
    
    println!("Imported {} events", events.len());
    Ok(())
}

fn prepare_events_from_csv(path: &str) -> Result<Vec<Event>> {
    // 从CSV文件读取数据并转换为Event结构
    // ...
}
```

#### 数据查询

在策略中查询历史数据：

```rust
use qadb_rs::query::{Query, TimeRange};
use chrono::{Duration, Utc};

async fn fetch_historical_data() -> Result<Vec<Value>> {
    // 创建查询
    let query = Query::new()
        .stream("futures_data")
        .time_range(TimeRange::new(
            Utc::now() - Duration::days(30), // 起始时间
            Utc::now(),                     // 结束时间
        ))
        .filter("instrument='SHFE.au2412'")
        .fields(vec!["time", "open", "high", "low", "close", "volume"])
        .limit(1000);
    
    // 执行查询
    let results = qadb_rs::execute_query(query).await?;
    
    Ok(results)
}
```

### 实时数据订阅

通过WebSocket接收实时市场数据：

```rust
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

async fn subscribe_to_market_data() -> Result<()> {
    // 连接到市场数据网关
    let (mut ws_stream, _) = connect_async("ws://localhost:8014/ws/market").await?;
    
    // 发送订阅请求
    let subscribe_msg = json!({
        "aid": "subscribe_quote",
        "ins_list": "SHFE.au2412,DCE.m2405"
    });
    
    ws_stream.send(Message::Text(subscribe_msg.to_string())).await?;
    
    // 处理接收到的数据
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let data: Value = serde_json::from_str(&text)?;
                process_market_data(data);
            }
            Ok(Message::Binary(bin)) => {
                // 处理二进制数据
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

fn process_market_data(data: Value) {
    // 解析和处理市场数据
    // ...
}
```

## 策略设计

### 策略架构

QAUTLRA-RS中的策略通常包含以下组件：

1. **策略核心**：实现交易逻辑
2. **数据管理器**：处理市场数据
3. **信号生成器**：生成交易信号
4. **仓位管理器**：管理交易头寸
5. **风险控制器**：实施风险控制措施
6. **执行引擎**：执行交易指令

### 基本策略框架

```rust
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

// 策略参数
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyParams {
    pub fast_period: u32,
    pub slow_period: u32,
    pub risk_limit: f64,
    // 其他参数
}

// 策略状态
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyState {
    pub positions: HashMap<String, f64>,
    pub last_signals: HashMap<String, SignalType>,
    pub equity_curve: Vec<f64>,
    // 其他状态数据
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SignalType {
    Buy,
    Sell,
    Hold,
}

// 策略特质
#[async_trait]
pub trait Strategy {
    async fn initialize(&mut self) -> Result<()>;
    async fn on_bar(&mut self, bar: &Bar) -> Result<Vec<Order>>;
    async fn on_tick(&mut self, tick: &Tick) -> Result<Vec<Order>>;
    async fn on_trade(&mut self, trade: &Trade) -> Result<()>;
    async fn on_order_status(&mut self, order: &OrderStatus) -> Result<()>;
    async fn finalize(&mut self) -> Result<()>;
}

// 示例策略实现
pub struct MovingAverageCrossover {
    params: StrategyParams,
    state: StrategyState,
    data_manager: DataManager,
    position_manager: PositionManager,
    risk_controller: RiskController,
}

#[async_trait]
impl Strategy for MovingAverageCrossover {
    async fn initialize(&mut self) -> Result<()> {
        // 初始化策略，加载历史数据等
        self.data_manager.load_historical_data("SHFE.au2412", 200).await?;
        Ok(())
    }
    
    async fn on_bar(&mut self, bar: &Bar) -> Result<Vec<Order>> {
        // 计算技术指标
        let instrument = &bar.instrument_id;
        let fast_ma = self.data_manager.calculate_ma(instrument, self.params.fast_period)?;
        let slow_ma = self.data_manager.calculate_ma(instrument, self.params.slow_period)?;
        
        // 生成交易信号
        let signal = if fast_ma > slow_ma {
            SignalType::Buy
        } else if fast_ma < slow_ma {
            SignalType::Sell
        } else {
            SignalType::Hold
        };
        
        // 更新策略状态
        self.state.last_signals.insert(instrument.clone(), signal.clone());
        
        // 根据信号生成订单
        let orders = match signal {
            SignalType::Buy => {
                if self.position_manager.get_position(instrument)? <= 0.0 {
                    let quantity = self.calculate_position_size(instrument, bar.close)?;
                    vec![Order::new_buy(instrument, quantity, None)]
                } else {
                    vec![]
                }
            }
            SignalType::Sell => {
                if self.position_manager.get_position(instrument)? >= 0.0 {
                    let quantity = self.calculate_position_size(instrument, bar.close)?;
                    vec![Order::new_sell(instrument, quantity, None)]
                } else {
                    vec![]
                }
            }
            SignalType::Hold => vec![],
        };
        
        // 应用风险控制
        let filtered_orders = self.risk_controller.filter_orders(orders)?;
        
        Ok(filtered_orders)
    }
    
    async fn on_tick(&mut self, tick: &Tick) -> Result<Vec<Order>> {
        // 处理实时行情数据，通常用于高频策略
        // 在此例中简单地忽略tick数据
        Ok(vec![])
    }
    
    async fn on_trade(&mut self, trade: &Trade) -> Result<()> {
        // 处理成交回报
        self.position_manager.update_position(
            &trade.instrument_id,
            trade.direction,
            trade.volume,
            trade.price,
        )?;
        
        Ok(())
    }
    
    async fn on_order_status(&mut self, order: &OrderStatus) -> Result<()> {
        // 处理订单状态更新
        println!("Order status updated: {:?}", order);
        Ok(())
    }
    
    async fn finalize(&mut self) -> Result<()> {
        // 策略结束清理
        println!("Strategy finalized");
        Ok(())
    }
    
    // 辅助方法
    fn calculate_position_size(&self, instrument: &str, price: f64) -> Result<f64> {
        // 计算头寸大小，应用资金管理规则
        let account_value = self.position_manager.get_account_value()?;
        let risk_amount = account_value * self.params.risk_limit;
        
        // 简单的头寸计算，在实际应用中应更复杂
        let quantity = (risk_amount / price).floor();
        
        Ok(quantity)
    }
}
``` 