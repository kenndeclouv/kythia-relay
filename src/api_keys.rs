use crate::db::Database;
use crate::errors::{KythiaError, NexusResult};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;

/// API Key representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i32,
    pub name: String,
    pub is_active: bool,
    pub is_master: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_used_at: Option<i64>,
    pub metadata: Option<String>,
}

/// API Key with the actual key (only shown on creation)
#[derive(Debug, Serialize)]
pub struct ApiKeyWithSecret {
    #[serde(flatten)]
    pub key_info: ApiKey,
    pub key: String,
}

/// Request to create a new API key
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub metadata: Option<String>,
}

/// API Key Manager
#[derive(Clone)]
pub struct ApiKeyManager {
    db: Database,
}

impl ApiKeyManager {
    /// Create a new API key manager
    pub fn new(db: Database) -> Self {
        ApiKeyManager { db }
    }

    /// Generate a new API key
    pub fn generate_key() -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.r#gen();
        format!("kythia-{}", hex::encode(random_bytes))
    }

    /// Hash an API key for storage
    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify an API key
    pub async fn verify_key(&self, key: &str) -> NexusResult<bool> {
        let key_hash = Self::hash_key(key);
        let valid = self.db.verify_api_key(&key_hash).await?;

        if valid {
            // Update last_used_at asynchronously
            let _ = self.db.update_last_used(&key_hash).await;
        }

        Ok(valid)
    }

    /// Create a new API key
    pub async fn create_key(
        &self,
        name: String,
        is_master: bool,
        metadata: Option<String>,
    ) -> NexusResult<ApiKeyWithSecret> {
        let key = Self::generate_key();
        let key_hash = Self::hash_key(&key);
        let now = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            INSERT INTO api_keys (key_hash, name, is_active, is_master, created_at, updated_at, metadata)
            VALUES (?, ?, TRUE, ?, ?, ?, ?)
            "#,
        )
        .bind(&key_hash)
        .bind(&name)
        .bind(is_master)
        .bind(now)
        .bind(now)
        .bind(&metadata)
        .execute(self.db.pool())
        .await
        .map_err(|e| KythiaError::Other(format!("Failed to create API key: {}", e)))?;

        let id = result.last_insert_id() as i32;

        Ok(ApiKeyWithSecret {
            key_info: ApiKey {
                id,
                name,
                is_active: true,
                is_master,
                created_at: now,
                updated_at: now,
                last_used_at: None,
                metadata,
            },
            key,
        })
    }

    /// List all API keys
    pub async fn list_keys(&self) -> NexusResult<Vec<ApiKey>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, is_active, is_master, created_at, updated_at, last_used_at, metadata
            FROM api_keys
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| KythiaError::Other(format!("Failed to list keys: {}", e)))?;

        let keys = rows
            .iter()
            .map(|row| ApiKey {
                id: row.get("id"),
                name: row.get("name"),
                is_active: row.get("is_active"),
                is_master: row.get("is_master"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_used_at: row.get("last_used_at"),
                metadata: row.get("metadata"),
            })
            .collect();

        Ok(keys)
    }

    /// Get a specific API key by ID
    pub async fn get_key(&self, id: i32) -> NexusResult<Option<ApiKey>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, is_active, is_master, created_at, updated_at, last_used_at, metadata
            FROM api_keys
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| KythiaError::Other(format!("Failed to get key: {}", e)))?;

        Ok(row.map(|r| ApiKey {
            id: r.get("id"),
            name: r.get("name"),
            is_active: r.get("is_active"),
            is_master: r.get("is_master"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            last_used_at: r.get("last_used_at"),
            metadata: r.get("metadata"),
        }))
    }

    /// Activate an API key
    pub async fn activate_key(&self, id: i32) -> NexusResult<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query("UPDATE api_keys SET is_active = TRUE, updated_at = ? WHERE id = ?")
            .bind(now)
            .bind(id)
            .execute(self.db.pool())
            .await
            .map_err(|e| KythiaError::Other(format!("Failed to activate key: {}", e)))?;

        Ok(())
    }

    /// Deactivate an API key
    pub async fn deactivate_key(&self, id: i32) -> NexusResult<()> {
        // Don't allow deactivating master key
        let key = self.get_key(id).await?;
        if let Some(k) = key {
            if k.is_master {
                return Err(KythiaError::Other(
                    "Cannot deactivate master key".to_string(),
                ));
            }
        }

        let now = chrono::Utc::now().timestamp();

        sqlx::query("UPDATE api_keys SET is_active = FALSE, updated_at = ? WHERE id = ? AND is_master = FALSE")
            .bind(now)
            .bind(id)
            .execute(self.db.pool())
            .await
            .map_err(|e| KythiaError::Other(format!("Failed to deactivate key: {}", e)))?;

        Ok(())
    }

    /// Delete an API key
    pub async fn delete_key(&self, id: i32) -> NexusResult<()> {
        // Don't allow deleting master key
        let key = self.get_key(id).await?;
        if let Some(k) = key {
            if k.is_master {
                return Err(KythiaError::Other("Cannot delete master key".to_string()));
            }
        }

        sqlx::query("DELETE FROM api_keys WHERE id = ? AND is_master = FALSE")
            .bind(id)
            .execute(self.db.pool())
            .await
            .map_err(|e| KythiaError::Other(format!("Failed to delete key: {}", e)))?;

        Ok(())
    }

    /// Bootstrap master key if it doesn't exist
    pub async fn bootstrap_master_key(&self) -> NexusResult<Option<String>> {
        // Check if master key already exists
        if self.db.has_master_key().await? {
            return Ok(None);
        }

        // Create master key
        let master_key = self
            .create_key("Master Key".to_string(), true, None)
            .await?;

        Ok(Some(master_key.key))
    }
}
