use crate::errors::{KythiaError, NexusResult};
use std::env;
use std::time::Duration;

/// Application configuration
pub struct Config {
    /// Host address to bind to
    pub host: String,

    /// Port to listen on for WebSocket connections
    pub port: u16,

    /// Port for HTTP health/metrics endpoints
    pub http_port: u16,

    /// Channel buffer size for message passing
    pub channel_buffer_size: usize,

    /// Maximum number of clients per room (0 = unlimited)
    pub max_room_size: usize,

    /// Maximum message size in bytes
    pub max_message_size: usize,

    /// Connection timeout duration
    pub connection_timeout: Duration,

    /// Enable authentication
    pub auth_enabled: bool,

    /// Secret key for JWT signing (required if auth_enabled)
    pub auth_secret: String,

    /// Rate limit: messages per second per client
    pub rate_limit_per_second: u32,

    /// Enable metrics collection
    pub metrics_enabled: bool,

    /// Database URL for MySQL
    pub database_url: String,

    /// Maximum database connection pool size
    pub db_max_connections: u32,

    /// Path to master key file
    pub master_key_file: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: "0.0.0.0".to_string(),
            port: 8080,
            http_port: 8081,
            channel_buffer_size: 500,
            max_room_size: 0,              // unlimited
            max_message_size: 1024 * 1024, // 1MB
            connection_timeout: Duration::from_secs(60),
            auth_enabled: false,
            auth_secret: String::new(),
            rate_limit_per_second: 100,
            metrics_enabled: true,
            database_url: "mysql://kythia:kythia_password@localhost:3306/kythia".to_string(),
            db_max_connections: 10,
            master_key_file: "./.master_key".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn load() -> NexusResult<Self> {
        let mut config = Config::default();

        // Load host
        if let Ok(host) = env::var("HOST") {
            config.host = host;
        }

        // Load WebSocket port
        if let Ok(port_str) = env::var("PORT") {
            config.port = port_str.parse::<u16>().map_err(|_| {
                KythiaError::Config(format!(
                    "Invalid PORT value '{}': must be a number between 1 and 65535",
                    port_str
                ))
            })?;
        }

        // Load HTTP port
        if let Ok(http_port_str) = env::var("HTTP_PORT") {
            config.http_port = http_port_str.parse::<u16>().map_err(|_| {
                KythiaError::Config(format!(
                    "Invalid HTTP_PORT value '{}': must be a number between 1 and 65535",
                    http_port_str
                ))
            })?;
        }

        // Load channel buffer size
        if let Ok(buffer_str) = env::var("CHANNEL_BUFFER_SIZE") {
            config.channel_buffer_size = buffer_str.parse::<usize>().map_err(|_| {
                KythiaError::Config(format!(
                    "Invalid CHANNEL_BUFFER_SIZE value '{}': must be a positive number",
                    buffer_str
                ))
            })?;

            if config.channel_buffer_size == 0 {
                return Err(KythiaError::Config(
                    "CHANNEL_BUFFER_SIZE must be greater than 0".to_string(),
                ));
            }
        }

        // Load max room size
        if let Ok(max_room_str) = env::var("MAX_ROOM_SIZE") {
            config.max_room_size = max_room_str.parse::<usize>().map_err(|_| {
                KythiaError::Config(format!(
                    "Invalid MAX_ROOM_SIZE value '{}': must be a positive number",
                    max_room_str
                ))
            })?;
        }

        // Load max message size
        if let Ok(max_msg_str) = env::var("MAX_MESSAGE_SIZE") {
            config.max_message_size = max_msg_str.parse::<usize>().map_err(|_| {
                KythiaError::Config(format!(
                    "Invalid MAX_MESSAGE_SIZE value '{}': must be a positive number",
                    max_msg_str
                ))
            })?;

            if config.max_message_size == 0 {
                return Err(KythiaError::Config(
                    "MAX_MESSAGE_SIZE must be greater than 0".to_string(),
                ));
            }
        }

        // Load connection timeout
        if let Ok(timeout_str) = env::var("CONNECTION_TIMEOUT") {
            let timeout_secs = timeout_str.parse::<u64>().map_err(|_| {
                KythiaError::Config(format!(
                    "Invalid CONNECTION_TIMEOUT value '{}': must be a positive number",
                    timeout_str
                ))
            })?;
            config.connection_timeout = Duration::from_secs(timeout_secs);
        }

        // Load authentication settings
        if let Ok(auth_enabled_str) = env::var("AUTH_ENABLED") {
            config.auth_enabled =
                auth_enabled_str.to_lowercase() == "true" || auth_enabled_str == "1";
        }

        if let Ok(secret) = env::var("AUTH_SECRET") {
            config.auth_secret = secret;
        }

        // Validate auth configuration
        if config.auth_enabled && config.auth_secret.is_empty() {
            return Err(KythiaError::Config(
                "AUTH_SECRET must be set when AUTH_ENABLED is true".to_string(),
            ));
        }

        if config.auth_enabled && config.auth_secret.len() < 16 {
            return Err(KythiaError::Config(
                "AUTH_SECRET must be at least 16 characters for security".to_string(),
            ));
        }

        // Load rate limit
        if let Ok(rate_str) = env::var("RATE_LIMIT_PER_SECOND") {
            config.rate_limit_per_second = rate_str.parse::<u32>().map_err(|_| {
                KythiaError::Config(format!(
                    "Invalid RATE_LIMIT_PER_SECOND value '{}': must be a positive number",
                    rate_str
                ))
            })?;
        }

        // Load metrics setting
        if let Ok(metrics_str) = env::var("METRICS_ENABLED") {
            config.metrics_enabled = metrics_str.to_lowercase() == "true" || metrics_str == "1";
        }

        // Load database URL
        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        // Load database pool size
        if let Ok(pool_str) = env::var("DB_MAX_CONNECTIONS") {
            config.db_max_connections = pool_str.parse::<u32>().map_err(|_| {
                KythiaError::Config(format!(
                    "Invalid DB_MAX_CONNECTIONS value '{}': must be a positive number",
                    pool_str
                ))
            })?;
            if config.db_max_connections == 0 {
                return Err(KythiaError::Config(
                    "DB_MAX_CONNECTIONS must be greater than 0".to_string(),
                ));
            }
        }

        // Load master key file path
        if let Ok(key_file) = env::var("MASTER_KEY_FILE") {
            config.master_key_file = key_file;
        }

        Ok(config)
    }

    /// Get the WebSocket server address
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Get the HTTP server address
    pub fn http_addr(&self) -> String {
        format!("{}:{}", self.host, self.http_port)
    }

    /// Validate the configuration
    pub fn validate(&self) -> NexusResult<()> {
        if self.port == 0 {
            return Err(KythiaError::Config("PORT cannot be 0".to_string()));
        }

        if self.http_port == 0 {
            return Err(KythiaError::Config("HTTP_PORT cannot be 0".to_string()));
        }

        if self.port == self.http_port {
            return Err(KythiaError::Config(
                "PORT and HTTP_PORT must be different".to_string(),
            ));
        }

        Ok(())
    }
}
