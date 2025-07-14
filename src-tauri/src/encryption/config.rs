/// Configuration system for encryption service
///
/// This module provides configuration management for encryption operations,
/// including nonce generation strategies, key rotation policies, and security settings.
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

use super::nonce_manager::{NonceConfig, NonceStrategy};
use super::types::{EncryptionAlgorithm, EncryptionResult};
use crate::error::FiscusError;

/// Global encryption service configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncryptionConfig {
    /// Nonce generation configuration
    pub nonce: NonceConfig,
    /// Key rotation policies
    pub rotation: RotationConfig,
    /// Security settings
    pub security: SecurityConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
}

/// Key rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    /// Enable automatic key rotation
    pub auto_rotation_enabled: bool,
    /// Rotation policies per algorithm
    pub policies: Vec<RotationPolicy>,
    /// Grace period before old keys are deactivated
    pub grace_period: Duration,
    /// Maximum number of active keys per algorithm
    pub max_active_keys: usize,
}

/// Key rotation policy for specific algorithms or key types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    /// Algorithm this policy applies to
    pub algorithm: Option<EncryptionAlgorithm>,
    /// Maximum number of encryptions before rotation
    pub max_encryptions: Option<u64>,
    /// Maximum age before rotation
    #[serde(with = "humantime_serde")]
    pub max_age: Option<Duration>,
    /// Maximum data volume before rotation (in bytes)
    pub max_data_volume: Option<u64>,
    /// Priority level (higher = more important)
    pub priority: u8,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable nonce reuse detection
    pub nonce_reuse_detection: bool,
    /// Enable key usage auditing
    pub key_usage_auditing: bool,
    /// Minimum key strength requirements
    pub min_key_strength: KeyStrengthConfig,
    /// Enable secure memory clearing
    pub secure_memory_clearing: bool,
    /// Enable timing attack protection
    pub timing_attack_protection: bool,
}

/// Key strength requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStrengthConfig {
    /// Minimum symmetric key size (bits)
    pub min_symmetric_key_bits: u32,
    /// Minimum asymmetric key size (bits)
    pub min_asymmetric_key_bits: u32,
    /// Required key derivation iterations
    pub min_kdf_iterations: u32,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable parallel encryption for large data
    pub parallel_encryption: bool,
    /// Chunk size for parallel processing (bytes)
    pub parallel_chunk_size: usize,
    /// Maximum memory usage for encryption operations (bytes)
    pub max_memory_usage: usize,
    /// Enable encryption operation caching
    pub enable_caching: bool,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            auto_rotation_enabled: true,
            policies: vec![
                // Conservative policy for AES-256-GCM
                RotationPolicy {
                    algorithm: Some(EncryptionAlgorithm::Aes256Gcm),
                    max_encryptions: Some(1u64 << 32), // 2^32
                    max_age: Some(Duration::from_secs(30 * 24 * 3600)), // 30 days
                    max_data_volume: Some(1u64 << 40), // 1TB
                    priority: 10,
                },
                // Conservative policy for ChaCha20-Poly1305
                RotationPolicy {
                    algorithm: Some(EncryptionAlgorithm::ChaCha20Poly1305),
                    max_encryptions: Some(1u64 << 32), // 2^32
                    max_age: Some(Duration::from_secs(30 * 24 * 3600)), // 30 days
                    max_data_volume: Some(1u64 << 40), // 1TB
                    priority: 10,
                },
            ],
            grace_period: Duration::from_secs(24 * 3600), // 24 hours
            max_active_keys: 5,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            nonce_reuse_detection: true,
            key_usage_auditing: true,
            min_key_strength: KeyStrengthConfig::default(),
            secure_memory_clearing: true,
            timing_attack_protection: true,
        }
    }
}

impl Default for KeyStrengthConfig {
    fn default() -> Self {
        Self {
            min_symmetric_key_bits: 256,
            min_asymmetric_key_bits: 2048,
            min_kdf_iterations: 100_000,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            parallel_encryption: true,
            parallel_chunk_size: 64 * 1024,      // 64KB
            max_memory_usage: 256 * 1024 * 1024, // 256MB
            enable_caching: false,               // Disabled by default for security
        }
    }
}

/// Configuration manager for the encryption service
#[derive(Debug)]
pub struct ConfigManager {
    config: EncryptionConfig,
}

impl ConfigManager {
    /// Create a new configuration manager with default settings
    pub fn new() -> Self {
        Self {
            config: EncryptionConfig::default(),
        }
    }

    /// Create configuration manager from file
    pub fn from_file(path: &str) -> EncryptionResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| FiscusError::Internal(format!("Failed to read config file: {e}")))?;

        let config: EncryptionConfig = toml::from_str(&content)
            .map_err(|e| FiscusError::Internal(format!("Failed to parse config: {e}")))?;

        info!("Loaded encryption configuration from {}", path);
        Ok(Self { config })
    }

    /// Create configuration manager from environment variables
    pub fn from_env() -> EncryptionResult<Self> {
        let mut config = EncryptionConfig::default();

        // Override with environment variables
        if let Ok(strategy) = std::env::var("FISCUS_NONCE_STRATEGY") {
            config.nonce.default_strategy = match strategy.to_lowercase().as_str() {
                "random" => NonceStrategy::Random,
                "counter_based" => NonceStrategy::CounterBased,
                "hybrid" => NonceStrategy::Hybrid,
                _ => {
                    return Err(FiscusError::InvalidInput(format!(
                        "Invalid nonce strategy: {strategy}"
                    )))
                }
            };
        }

        if let Ok(threshold) = std::env::var("FISCUS_ROTATION_THRESHOLD") {
            config.nonce.rotation_threshold = threshold.parse().map_err(|e| {
                FiscusError::InvalidInput(format!("Invalid rotation threshold: {e}"))
            })?;
        }

        if let Ok(auto_rotation) = std::env::var("FISCUS_AUTO_ROTATION") {
            config.rotation.auto_rotation_enabled = auto_rotation.parse().map_err(|e| {
                FiscusError::InvalidInput(format!("Invalid auto rotation setting: {e}"))
            })?;
        }

        debug!("Loaded encryption configuration from environment");
        Ok(Self { config })
    }

    /// Get current configuration
    pub fn config(&self) -> &EncryptionConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: EncryptionConfig) {
        debug!("Updating encryption configuration");
        self.config = config;
    }

    /// Get rotation policy for algorithm
    pub fn get_rotation_policy(&self, algorithm: EncryptionAlgorithm) -> Option<&RotationPolicy> {
        self.config
            .rotation
            .policies
            .iter()
            .filter(|p| p.algorithm == Some(algorithm))
            .max_by_key(|p| p.priority)
    }

    /// Check if key needs rotation based on policy
    pub fn should_rotate_key(
        &self,
        algorithm: EncryptionAlgorithm,
        encryption_count: u64,
        key_age: Duration,
        data_volume: u64,
    ) -> bool {
        if !self.config.rotation.auto_rotation_enabled {
            return false;
        }

        if let Some(policy) = self.get_rotation_policy(algorithm) {
            if let Some(max_encryptions) = policy.max_encryptions {
                if encryption_count >= max_encryptions {
                    return true;
                }
            }

            if let Some(max_age) = policy.max_age {
                if key_age >= max_age {
                    return true;
                }
            }

            if let Some(max_volume) = policy.max_data_volume {
                if data_volume >= max_volume {
                    return true;
                }
            }
        }

        false
    }

    /// Validate configuration
    pub fn validate(&self) -> EncryptionResult<()> {
        // Validate nonce configuration
        if self.config.nonce.rotation_threshold == 0 {
            return Err(FiscusError::InvalidInput(
                "Rotation threshold must be greater than 0".to_string(),
            ));
        }

        if self.config.nonce.warning_threshold >= self.config.nonce.rotation_threshold {
            return Err(FiscusError::InvalidInput(
                "Warning threshold must be less than rotation threshold".to_string(),
            ));
        }

        // Validate rotation policies
        for policy in &self.config.rotation.policies {
            if policy.max_encryptions == Some(0) {
                return Err(FiscusError::InvalidInput(
                    "Max encryptions must be greater than 0".to_string(),
                ));
            }
        }

        // Validate security settings
        if self.config.security.min_key_strength.min_symmetric_key_bits < 128 {
            return Err(FiscusError::InvalidInput(
                "Minimum symmetric key strength must be at least 128 bits".to_string(),
            ));
        }

        if self
            .config
            .security
            .min_key_strength
            .min_asymmetric_key_bits
            < 1024
        {
            return Err(FiscusError::InvalidInput(
                "Minimum asymmetric key strength must be at least 1024 bits".to_string(),
            ));
        }

        // Validate performance settings
        if self.config.performance.parallel_chunk_size == 0 {
            return Err(FiscusError::InvalidInput(
                "Parallel chunk size must be greater than 0".to_string(),
            ));
        }

        if self.config.performance.max_memory_usage == 0 {
            return Err(FiscusError::InvalidInput(
                "Max memory usage must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> EncryptionResult<()> {
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| FiscusError::Internal(format!("Failed to serialize config: {e}")))?;

        std::fs::write(path, content)
            .map_err(|e| FiscusError::Internal(format!("Failed to write config file: {e}")))?;

        info!("Saved encryption configuration to {}", path);
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let manager = ConfigManager::new();
        assert!(manager.validate().is_ok());
    }

    #[test]
    fn test_rotation_policy_selection() {
        let manager = ConfigManager::new();

        let policy = manager.get_rotation_policy(EncryptionAlgorithm::Aes256Gcm);
        assert!(policy.is_some());
        assert_eq!(
            policy.unwrap().algorithm,
            Some(EncryptionAlgorithm::Aes256Gcm)
        );
    }

    #[test]
    fn test_should_rotate_key() {
        let manager = ConfigManager::new();

        // Should not rotate with low usage
        assert!(!manager.should_rotate_key(
            EncryptionAlgorithm::Aes256Gcm,
            1000,
            Duration::from_secs(3600),
            1024 * 1024,
        ));

        // Should rotate with high encryption count
        assert!(manager.should_rotate_key(
            EncryptionAlgorithm::Aes256Gcm,
            1u64 << 33, // Exceeds 2^32
            Duration::from_secs(3600),
            1024 * 1024,
        ));

        // Should rotate with old age
        assert!(manager.should_rotate_key(
            EncryptionAlgorithm::Aes256Gcm,
            1000,
            Duration::from_secs(31 * 24 * 3600), // 31 days
            1024 * 1024,
        ));
    }

    #[test]
    fn test_config_validation_errors() {
        let mut config = EncryptionConfig::default();

        // Test invalid rotation threshold
        config.nonce.rotation_threshold = 0;
        let manager = ConfigManager {
            config: config.clone(),
        };
        assert!(manager.validate().is_err());

        // Test invalid warning threshold
        config.nonce.rotation_threshold = 100;
        config.nonce.warning_threshold = 200;
        let manager = ConfigManager { config };
        assert!(manager.validate().is_err());
    }
}
