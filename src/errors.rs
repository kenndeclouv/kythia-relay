use std::fmt;

/// Custom error types for Kythia RelayCore
#[derive(Debug)]
pub enum KythiaError {
    /// Configuration errors
    Config(String),

    /// WebSocket protocol errors
    #[allow(dead_code)]
    WebSocket(String),

    /// Message protocol errors
    #[allow(dead_code)]
    Protocol(String),

    /// Authentication errors
    #[allow(dead_code)]
    Auth(String),

    /// Rate limiting errors
    #[allow(dead_code)]
    RateLimit(String),

    /// IO errors
    Io(std::io::Error),

    /// JSON serialization errors
    Json(serde_json::Error),

    /// Generic errors
    Other(String),
}

impl fmt::Display for KythiaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KythiaError::Config(msg) => write!(f, "Configuration error: {}", msg),
            KythiaError::WebSocket(msg) => write!(f, "WebSocket error: {}", msg),
            KythiaError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            KythiaError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            KythiaError::RateLimit(msg) => write!(f, "Rate limit error: {}", msg),
            KythiaError::Io(err) => write!(f, "IO error: {}", err),
            KythiaError::Json(err) => write!(f, "JSON error: {}", err),
            KythiaError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for KythiaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            KythiaError::Io(err) => Some(err),
            KythiaError::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for KythiaError {
    fn from(err: std::io::Error) -> Self {
        KythiaError::Io(err)
    }
}

impl From<serde_json::Error> for KythiaError {
    fn from(err: serde_json::Error) -> Self {
        KythiaError::Json(err)
    }
}

impl From<std::num::ParseIntError> for KythiaError {
    fn from(err: std::num::ParseIntError) -> Self {
        KythiaError::Config(format!("Failed to parse integer: {}", err))
    }
}

/// Result type alias for Kythia RelayCore operations
pub type NexusResult<T> = Result<T, KythiaError>;
