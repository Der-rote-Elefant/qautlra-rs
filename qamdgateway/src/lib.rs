//! QAMD Gateway
//! 
//! 这是一个市场数据网关，支持多种数据源，并提供统一的WebSocket接口。
//! 
//! 主要功能：
//! 1. 支持CTP、QQ、Sina等市场数据源
//! 2. 提供统一的WebSocket接口
//! 3. 支持TradingView格式的消息

pub mod actors;
pub mod config;
pub mod converter;
pub mod error;
pub mod ws_server;

/// 重新导出qamd_rs中的类型
pub use qamd_rs::MDSnapshot;

/// 预导入模块，提供常用类型
pub mod prelude {
    pub use crate::actors::messages::*;
    pub use crate::actors::md_actor::MarketDataActor;
    pub use crate::actors::md_distributor::MarketDataDistributor;
    pub use crate::config::BrokerConfig;
    pub use crate::ws_server::ws_handler;

    #[cfg(feature = "ctp")]
    pub use crate::actors::messages::{RegisterCTPMdActor, MarketDataSource};

    #[cfg(feature = "qq")]
    pub use crate::actors::messages::{RegisterQQMdActor, MarketDataSource};

    #[cfg(feature = "sina")]
    pub use crate::actors::messages::{RegisterSinaMdActor, MarketDataSource};
} 