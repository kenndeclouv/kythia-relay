use crate::errors::{KythiaError, NexusResult};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

/// Database connection pool
#[derive(Clone)]
pub struct Database {
    pool: MySqlPool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_url: &str, max_connections: u32) -> NexusResult<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .map_err(|e| KythiaError::Other(format!("Failed to connect to database: {}", e)))?;

        Ok(Database { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> NexusResult<()> {
        // Create api_keys table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id INT AUTO_INCREMENT PRIMARY KEY,
                key_hash VARCHAR(64) NOT NULL UNIQUE,
                name VARCHAR(255) NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT TRUE,
                is_master BOOLEAN NOT NULL DEFAULT FALSE,
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL,
                last_used_at BIGINT,
                metadata TEXT,
                INDEX idx_key_hash (key_hash),
                INDEX idx_is_active (is_active)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| KythiaError::Other(format!("Migration failed: {}", e)))?;

        log::info!("Database migrations completed successfully");
        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }

    /// Check if a key hash exists and is active
    pub async fn verify_api_key(&self, key_hash: &str) -> NexusResult<bool> {
        let result =
            sqlx::query("SELECT is_active FROM api_keys WHERE key_hash = ? AND is_active = TRUE")
                .bind(key_hash)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| KythiaError::Other(format!("Database query failed: {}", e)))?;

        Ok(result.is_some())
    }

    /// Update last_used_at for an API key
    pub async fn update_last_used(&self, key_hash: &str) -> NexusResult<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query("UPDATE api_keys SET last_used_at = ? WHERE key_hash = ?")
            .bind(now)
            .bind(key_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| KythiaError::Other(format!("Failed to update last_used: {}", e)))?;

        Ok(())
    }

    /// Check if master key exists
    pub async fn has_master_key(&self) -> NexusResult<bool> {
        let result = sqlx::query("SELECT id FROM api_keys WHERE is_master = TRUE LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| KythiaError::Other(format!("Database query failed: {}", e)))?;

        Ok(result.is_some())
    }
}
