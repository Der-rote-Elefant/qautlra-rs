use thiserror::Error;

/// Errors that can occur in the QAMD library
#[derive(Error, Debug)]
pub enum QAMDError {
    /// Error during serialization or deserialization
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// Error parsing a date or time
    #[error("Date/time parse error: {0}")]
    DateTimeParseError(#[from] chrono::ParseError),
    
    /// Market data is invalid or missing required fields
    #[error("Invalid market data: {0}")]
    InvalidMarketData(String),
    
    /// General error
    #[error("{0}")]
    General(String),
}

/// Result type for QAMD operations
pub type Result<T> = std::result::Result<T, QAMDError>; 