use std::fmt;

/// Custom error types for Kythia Nexus Core
#[derive(Debug)]
pub enum NexusError {
    /// Configuration errors
    Config(String),

    /// WebSocket protocol errors
    WebSocket(String),

    /// Message protocol errors
    Protocol(String),

    /// Authentication errors
    Auth(String),

    /// Rate limiting errors
    RateLimit(String),

    /// IO errors
    Io(std::io::Error),

    /// JSON serialization errors
    Json(serde_json::Error),

    /// Generic errors
    Other(String),
}

impl fmt::Display for NexusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NexusError::Config(msg) => write!(f, "Configuration error: {}", msg),
            NexusError::WebSocket(msg) => write!(f, "WebSocket error: {}", msg),
            NexusError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            NexusError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            NexusError::RateLimit(msg) => write!(f, "Rate limit error: {}", msg),
            NexusError::Io(err) => write!(f, "IO error: {}", err),
            NexusError::Json(err) => write!(f, "JSON error: {}", err),
            NexusError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for NexusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NexusError::Io(err) => Some(err),
            NexusError::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for NexusError {
    fn from(err: std::io::Error) -> Self {
        NexusError::Io(err)
    }
}

impl From<serde_json::Error> for NexusError {
    fn from(err: serde_json::Error) -> Self {
        NexusError::Json(err)
    }
}

impl From<std::num::ParseIntError> for NexusError {
    fn from(err: std::num::ParseIntError) -> Self {
        NexusError::Config(format!("Failed to parse integer: {}", err))
    }
}

/// Result type alias for Kythia Nexus Core operations
pub type NexusResult<T> = Result<T, NexusError>;
