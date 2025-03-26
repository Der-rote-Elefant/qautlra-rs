use thiserror::Error;

/// Custom error types for the QAMD Gateway
#[derive(Error, Debug)]
pub enum GatewayError {
    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// CTP errors
    #[error("CTP error: {0}")]
    CtpError(String),

    /// QAMD errors
    #[error("QAMD error: {0}")]
    QamdError(#[from] qamd_rs::QAMDError),

    /// Market data conversion errors
    #[error("Market data conversion error: {0}")]
    ConversionError(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// WebSocket errors
    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    /// Invalid instrument error
    #[error("Invalid instrument: {0}")]
    InvalidInstrument(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for the QAMD Gateway
pub type GatewayResult<T> = Result<T, GatewayError>; 