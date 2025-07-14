/// Nonce management system for preventing nonce reuse in high-volume scenarios
///
/// This module provides counter-based nonce generation to eliminate the possibility
/// of nonce reuse, which is critical for maintaining security in high-volume
/// encryption scenarios where the birthday paradox could lead to nonce collisions.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use super::types::{EncryptionAlgorithm, EncryptionResult};
use super::utils::SecureRandom;
use crate::error::FiscusError;

/// Strategy for nonce generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NonceStrategy {
    /// Pure random nonces (current behavior, suitable for low-volume)
    #[default]
    Random,
    /// Counter-based nonces (8-byte counter + 4-byte random)
    CounterBased,
    /// Hybrid approach with fallback to random if counter fails
    Hybrid,
}

/// Configuration for nonce generation and key rotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceConfig {
    /// Default nonce generation strategy
    pub default_strategy: NonceStrategy,
    /// Threshold for key rotation (number of encryptions)
    pub rotation_threshold: u64,
    /// Warning threshold (warn when approaching rotation)
    pub warning_threshold: u64,
    /// Enable persistence of counter state
    pub persist_counters: bool,
}

impl Default for NonceConfig {
    fn default() -> Self {
        Self {
            default_strategy: NonceStrategy::Random,
            rotation_threshold: 1u64 << 32, // 2^32 encryptions
            warning_threshold: 1u64 << 30,  // 2^30 encryptions (warning at 25%)
            persist_counters: true,
        }
    }
}

/// Thread-safe counter for a specific key
#[derive(Debug)]
struct KeyCounter {
    counter: AtomicU64,
    #[allow(dead_code)]
    created_at: std::time::Instant,
}

impl KeyCounter {
    fn new(initial_value: u64) -> Self {
        Self {
            counter: AtomicU64::new(initial_value),
            created_at: std::time::Instant::now(),
        }
    }

    fn next(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    fn current(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
}

/// Manages nonce generation with reuse prevention
#[derive(Debug)]
pub struct NonceManager {
    config: NonceConfig,
    /// Per-key counters for deterministic nonce generation
    counters: Arc<RwLock<HashMap<String, Arc<KeyCounter>>>>,
    /// Secure random number generator for random components
    secure_random: std::sync::Mutex<SecureRandom>,
}

impl NonceManager {
    /// Create a new nonce manager with default configuration
    pub fn new() -> EncryptionResult<Self> {
        Self::with_config(NonceConfig::default())
    }

    /// Create a new nonce manager with custom configuration
    pub fn with_config(config: NonceConfig) -> EncryptionResult<Self> {
        debug!(
            "Initializing NonceManager with strategy: {:?}",
            config.default_strategy
        );

        Ok(Self {
            config,
            counters: Arc::new(RwLock::new(HashMap::new())),
            secure_random: std::sync::Mutex::new(SecureRandom::new()?),
        })
    }

    /// Generate a nonce for the given key and algorithm
    #[instrument(skip(self), fields(key_id = %key_id, algorithm = ?algorithm, strategy = ?strategy))]
    pub async fn generate_nonce(
        &self,
        key_id: &str,
        algorithm: EncryptionAlgorithm,
        strategy: Option<NonceStrategy>,
    ) -> EncryptionResult<Vec<u8>> {
        let nonce_strategy = strategy.unwrap_or(self.config.default_strategy);
        let nonce_length = self.get_nonce_length(algorithm)?;

        match nonce_strategy {
            NonceStrategy::Random => self.generate_random_nonce(nonce_length).await,
            NonceStrategy::CounterBased => {
                self.generate_counter_based_nonce(key_id, nonce_length)
                    .await
            }
            NonceStrategy::Hybrid => {
                // Try counter-based first, fallback to random
                match self
                    .generate_counter_based_nonce(key_id, nonce_length)
                    .await
                {
                    Ok(nonce) => Ok(nonce),
                    Err(e) => {
                        warn!(
                            "Counter-based nonce generation failed, falling back to random: {}",
                            e
                        );
                        self.generate_random_nonce(nonce_length).await
                    }
                }
            }
        }
    }

    /// Generate a purely random nonce
    async fn generate_random_nonce(&self, length: usize) -> EncryptionResult<Vec<u8>> {
        let nonce = self.secure_random.lock().unwrap().generate_bytes(length)?;

        debug!(length = length, "Generated random nonce");
        Ok(nonce)
    }

    /// Generate a counter-based nonce (8-byte counter + 4-byte random)
    async fn generate_counter_based_nonce(
        &self,
        key_id: &str,
        length: usize,
    ) -> EncryptionResult<Vec<u8>> {
        if length != 12 {
            return Err(FiscusError::InvalidInput(
                "Counter-based nonces only support 12-byte length".to_string(),
            ));
        }

        // Get or create counter for this key
        let counter_value = {
            let counters = self.counters.read().await;
            if let Some(counter) = counters.get(key_id) {
                counter.next()
            } else {
                drop(counters);
                // Need to create new counter
                let mut counters = self.counters.write().await;
                // Double-check in case another thread created it
                if let Some(counter) = counters.get(key_id) {
                    counter.next()
                } else {
                    let new_counter = Arc::new(KeyCounter::new(0));
                    let value = new_counter.next();
                    counters.insert(key_id.to_string(), new_counter);
                    debug!(key_id = %key_id, "Created new counter for key");
                    value
                }
            }
        };

        // Check if we're approaching rotation threshold
        if counter_value >= self.config.warning_threshold {
            if counter_value >= self.config.rotation_threshold {
                return Err(FiscusError::Internal(format!(
                    "Key {} has exceeded rotation threshold ({} encryptions). Key rotation required.",
                    key_id, self.config.rotation_threshold
                )));
            } else {
                warn!(
                    key_id = %key_id,
                    count = counter_value,
                    threshold = self.config.rotation_threshold,
                    "Key approaching rotation threshold"
                );
            }
        }

        // Generate nonce: 8-byte counter (big-endian) + 4-byte random
        let mut nonce = Vec::with_capacity(12);
        nonce.extend_from_slice(&counter_value.to_be_bytes());

        let random_suffix = self.secure_random.lock().unwrap().generate_bytes(4)?;
        nonce.extend_from_slice(&random_suffix);

        debug!(
            key_id = %key_id,
            counter = counter_value,
            "Generated counter-based nonce"
        );

        Ok(nonce)
    }

    /// Get the current encryption count for a key
    pub async fn get_encryption_count(&self, key_id: &str) -> u64 {
        let counters = self.counters.read().await;
        counters
            .get(key_id)
            .map(|counter| counter.current())
            .unwrap_or(0)
    }

    /// Check if a key needs rotation based on encryption count
    pub async fn needs_rotation(&self, key_id: &str) -> bool {
        self.get_encryption_count(key_id).await >= self.config.rotation_threshold
    }

    /// Reset counter for a key (used after key rotation)
    pub async fn reset_counter(&self, key_id: &str) -> EncryptionResult<()> {
        let mut counters = self.counters.write().await;
        counters.remove(key_id);
        info!(key_id = %key_id, "Reset counter for key");
        Ok(())
    }

    /// Get nonce length for algorithm
    fn get_nonce_length(&self, algorithm: EncryptionAlgorithm) -> EncryptionResult<usize> {
        match algorithm {
            EncryptionAlgorithm::Aes256Gcm => Ok(12),
            EncryptionAlgorithm::ChaCha20Poly1305 => Ok(12),
            _ => Err(FiscusError::InvalidInput(
                "Unsupported algorithm for nonce generation".to_string(),
            )),
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &NonceConfig {
        &self.config
    }

    /// Update configuration (affects future operations)
    pub fn update_config(&mut self, config: NonceConfig) {
        debug!("Updating NonceManager configuration");
        self.config = config;
    }
}

impl Default for NonceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default NonceManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn test_random_nonce_generation() {
        let manager = NonceManager::new().unwrap();

        let nonce1 = manager
            .generate_nonce(
                "test-key",
                EncryptionAlgorithm::Aes256Gcm,
                Some(NonceStrategy::Random),
            )
            .await
            .unwrap();

        let nonce2 = manager
            .generate_nonce(
                "test-key",
                EncryptionAlgorithm::Aes256Gcm,
                Some(NonceStrategy::Random),
            )
            .await
            .unwrap();

        assert_eq!(nonce1.len(), 12);
        assert_eq!(nonce2.len(), 12);
        assert_ne!(nonce1, nonce2);
    }

    #[tokio::test]
    async fn test_counter_based_nonce_uniqueness() {
        let manager = NonceManager::new().unwrap();
        let mut nonces = HashSet::new();

        // Generate 1000 nonces and ensure they're all unique
        for _ in 0..1000 {
            let nonce = manager
                .generate_nonce(
                    "test-key",
                    EncryptionAlgorithm::Aes256Gcm,
                    Some(NonceStrategy::CounterBased),
                )
                .await
                .unwrap();

            assert_eq!(nonce.len(), 12);
            assert!(nonces.insert(nonce), "Duplicate nonce detected");
        }
    }

    #[tokio::test]
    async fn test_concurrent_counter_based_nonces() {
        let manager = Arc::new(NonceManager::new().unwrap());
        let mut tasks = JoinSet::new();
        let nonces = Arc::new(std::sync::Mutex::new(HashSet::new()));

        // Spawn 10 concurrent tasks, each generating 100 nonces
        for _ in 0..10 {
            let manager_clone = manager.clone();
            let nonces_clone = nonces.clone();

            tasks.spawn(async move {
                for _ in 0..100 {
                    let nonce = manager_clone
                        .generate_nonce(
                            "test-key",
                            EncryptionAlgorithm::Aes256Gcm,
                            Some(NonceStrategy::CounterBased),
                        )
                        .await
                        .unwrap();

                    let mut nonces_set = nonces_clone.lock().unwrap();
                    assert!(
                        nonces_set.insert(nonce),
                        "Duplicate nonce in concurrent test"
                    );
                }
            });
        }

        // Wait for all tasks to complete
        while let Some(result) = tasks.join_next().await {
            result.unwrap();
        }

        // Verify we have 1000 unique nonces
        let final_nonces = nonces.lock().unwrap();
        assert_eq!(final_nonces.len(), 1000);
    }

    #[tokio::test]
    async fn test_rotation_threshold() {
        let config = NonceConfig {
            rotation_threshold: 5,
            warning_threshold: 3,
            ..Default::default()
        };

        let manager = NonceManager::with_config(config).unwrap();

        // Generate nonces up to warning threshold
        for _ in 0..3 {
            manager
                .generate_nonce(
                    "test-key",
                    EncryptionAlgorithm::Aes256Gcm,
                    Some(NonceStrategy::CounterBased),
                )
                .await
                .unwrap();
        }

        // Next nonces should trigger warnings but still work
        for _ in 0..2 {
            manager
                .generate_nonce(
                    "test-key",
                    EncryptionAlgorithm::Aes256Gcm,
                    Some(NonceStrategy::CounterBased),
                )
                .await
                .unwrap();
        }

        // Next nonce should fail due to rotation threshold
        let result = manager
            .generate_nonce(
                "test-key",
                EncryptionAlgorithm::Aes256Gcm,
                Some(NonceStrategy::CounterBased),
            )
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("rotation threshold"));
    }
}
