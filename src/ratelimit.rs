use crate::errors::{KythiaError, NexusResult};
use dashmap::DashMap;
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

/// Rate limiter for individual clients
#[derive(Clone)]
pub struct ClientRateLimiter {
    limiters: Arc<DashMap<SocketAddr, DefaultDirectRateLimiter>>,
    quota: Quota,
}

impl ClientRateLimiter {
    /// Create a new rate limiter with messages per second limit
    pub fn new(messages_per_second: u32) -> Self {
        let quota = Quota::per_second(
            NonZeroU32::new(messages_per_second).unwrap_or(NonZeroU32::new(100).unwrap()),
        );

        ClientRateLimiter {
            limiters: Arc::new(DashMap::new()),
            quota,
        }
    }

    /// Check if a client is allowed to send a message
    /// Returns Ok(()) if allowed, Err if rate limited
    pub fn check(&self, addr: SocketAddr) -> NexusResult<()> {
        let limiter = self
            .limiters
            .entry(addr)
            .or_insert_with(|| RateLimiter::direct(self.quota));

        match limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(KythiaError::RateLimit(format!(
                "Client {} is rate limited",
                addr
            ))),
        }
    }

    /// Remove rate limiter for a disconnected client
    pub fn remove(&self, addr: &SocketAddr) {
        self.limiters.remove(addr);
    }

    /// Get the number of active limiters
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.limiters.len()
    }
}

/// Message size validator
#[allow(dead_code)]
pub struct MessageValidator {
    max_size: usize,
}

#[allow(dead_code)]
impl MessageValidator {
    /// Create a new message validator
    pub fn new(max_size: usize) -> Self {
        MessageValidator { max_size }
    }

    /// Validate message size
    pub fn validate_size(&self, data: &[u8]) -> NexusResult<()> {
        if data.len() > self.max_size {
            Err(KythiaError::Protocol(format!(
                "Message size {} exceeds maximum allowed size {}",
                data.len(),
                self.max_size
            )))
        } else {
            Ok(())
        }
    }

    /// Validate text message size
    pub fn validate_text_size(&self, text: &str) -> NexusResult<()> {
        self.validate_size(text.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::str::FromStr;

    #[test]
    fn test_rate_limiter() {
        let limiter = ClientRateLimiter::new(2); // 2 messages per second
        let addr = SocketAddr::from_str("127.0.0.1:8080").unwrap();

        // First two should succeed
        assert!(limiter.check(addr).is_ok());
        assert!(limiter.check(addr).is_ok());

        // Third should be rate limited
        assert!(limiter.check(addr).is_err());
    }

    #[test]
    fn test_message_validator() {
        let validator = MessageValidator::new(100);

        // Should pass
        assert!(validator.validate_text_size("short message").is_ok());

        // Should fail
        let long_message = "a".repeat(101);
        assert!(validator.validate_text_size(&long_message).is_err());
    }
}
