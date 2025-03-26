pub mod md_actor;
pub mod md_connector;
pub mod md_distributor;
pub mod messages;

#[cfg(feature = "ctp")]
pub use md_actor as ctp_md_actor;

#[cfg(feature = "qq")]
pub use md_actor as qq_md_actor;

#[cfg(feature = "sina")]
pub use md_actor as sina_md_actor;

// 预导入常用类型和消息
pub mod prelude {
    pub use crate::actors::md_actor::*;
    pub use crate::actors::md_connector::*;
    pub use crate::actors::md_distributor::*;
    pub use crate::actors::messages::*;
}
