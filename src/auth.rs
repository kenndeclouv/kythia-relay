use crate::errors::{NexusError, NexusResult};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID or identifier)
    pub sub: String,

    /// Issued at (Unix timestamp)
    pub iat: u64,

    /// Expiration time (Unix timestamp)
    pub exp: u64,

    /// Optional: Room access permissions
    pub rooms: Option<Vec<String>>,
}

impl Claims {
    /// Create new claims with default expiration (24 hours)
    pub fn new(subject: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Claims {
            sub: subject,
            iat: now,
            exp: now + 86400, // 24 hours
            rooms: None,
        }
    }

    /// Create claims with custom expiration
    pub fn with_expiration(subject: String, expires_in_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Claims {
            sub: subject,
            iat: now,
            exp: now + expires_in_seconds,
            rooms: None,
        }
    }

    /// Create claims with room permissions
    pub fn with_rooms(subject: String, rooms: Vec<String>) -> Self {
        let mut claims = Self::new(subject);
        claims.rooms = Some(rooms);
        claims
    }
}

/// Authentication manager
pub struct AuthManager {
    secret: String,
    enabled: bool,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new(secret: String, enabled: bool) -> Self {
        AuthManager { secret, enabled }
    }

    /// Check if authentication is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Generate a JWT token
    pub fn generate_token(&self, claims: &Claims) -> NexusResult<String> {
        if !self.enabled {
            return Err(NexusError::Auth(
                "Authentication is not enabled".to_string(),
            ));
        }

        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| NexusError::Auth(format!("Failed to generate token: {}", e)))
    }

    /// Validate a JWT token and return claims
    pub fn validate_token(&self, token: &str) -> NexusResult<Claims> {
        if !self.enabled {
            // If auth is disabled, create a default claim
            return Ok(Claims::new("anonymous".to_string()));
        }

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| NexusError::Auth(format!("Invalid token: {}", e)))
    }

    /// Validate API key (simple string comparison)
    pub fn validate_api_key(&self, api_key: &str) -> NexusResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // In a production system, you'd check against a database
        // For now, we'll just check if it matches the secret
        if api_key == self.secret {
            Ok(())
        } else {
            Err(NexusError::Auth("Invalid API key".to_string()))
        }
    }

    /// Extract token from WebSocket URL query parameters
    /// Example: ws://localhost:8080/?token=eyJ...
    pub fn extract_token_from_url(url: &str) -> Option<String> {
        url.split('?')
            .nth(1)
            .and_then(|query| query.split('&').find(|param| param.starts_with("token=")))
            .map(|param| param.trim_start_matches("token=").to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_and_validation() {
        let auth = AuthManager::new("test-secret-key-at-least-32-chars".to_string(), true);
        let claims = Claims::new("test-user".to_string());

        let token = auth.generate_token(&claims).unwrap();
        let validated = auth.validate_token(&token).unwrap();

        assert_eq!(validated.sub, "test-user");
    }

    #[test]
    fn test_token_extraction() {
        let url = "ws://localhost:8080/?token=abc123";
        let token = AuthManager::extract_token_from_url(url);
        assert_eq!(token, Some("abc123".to_string()));
    }

    #[test]
    fn test_disabled_auth() {
        let auth = AuthManager::new("secret".to_string(), false);
        let result = auth.validate_token("invalid");
        assert!(result.is_ok());
    }
}
