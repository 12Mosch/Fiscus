/// Key derivation implementations for the Fiscus encryption service
///
/// This module provides secure key derivation functions (KDFs) for generating
/// encryption keys from passwords or other key material. It supports multiple
/// algorithms including Argon2, PBKDF2, and Scrypt.
use argon2::{Algorithm, Argon2, Params, Version};
use async_trait::async_trait;
use pbkdf2::pbkdf2_hmac;
use scrypt::Params as ScryptParams;
use sha2::Sha256;
use tracing::{debug, error, instrument, warn};

use super::types::{
    EncryptionAlgorithm, EncryptionKey, EncryptionResult, KeyDerivationAlgorithm,
    KeyDerivationParams, KeyType,
};
use super::utils::SecureRandom;
use crate::error::FiscusError;

/// Trait for key derivation operations
#[async_trait]
pub trait KeyDerivation {
    /// Derive a key from a password and salt
    async fn derive_key(
        &self,
        password: &[u8],
        params: &KeyDerivationParams,
    ) -> EncryptionResult<EncryptionKey>;

    /// Verify a password against a derived key
    async fn verify_password(
        &self,
        password: &[u8],
        key: &EncryptionKey,
        params: &KeyDerivationParams,
    ) -> EncryptionResult<bool>;

    /// Get the algorithm identifier
    fn algorithm(&self) -> KeyDerivationAlgorithm;

    /// Generate recommended parameters for this algorithm
    fn generate_params(&self, key_length: usize) -> EncryptionResult<KeyDerivationParams>;
}

/// Argon2id key derivation implementation
///
/// Argon2id is the recommended password hashing algorithm, providing
/// resistance against both side-channel and GPU-based attacks.
#[derive(Debug)]
pub struct Argon2Kdf {
    #[allow(dead_code)]
    secure_random: SecureRandom,
}

impl Argon2Kdf {
    /// Create a new Argon2 key derivation instance
    pub fn new() -> EncryptionResult<Self> {
        debug!("Initializing Argon2id key derivation");
        Ok(Self {
            secure_random: SecureRandom::new()?,
        })
    }

    /// Derive key with custom Argon2 parameters
    #[instrument(skip(self, password), fields(salt_len = params.salt.len(), key_len = params.key_length))]
    pub async fn derive_key_with_params(
        &self,
        password: &[u8],
        params: &KeyDerivationParams,
    ) -> EncryptionResult<Vec<u8>> {
        if params.algorithm != KeyDerivationAlgorithm::Argon2id {
            return Err(FiscusError::InvalidInput(
                "Algorithm mismatch for Argon2 key derivation".to_string(),
            ));
        }

        // Extract Argon2 parameters
        let memory_cost = params.memory_cost.unwrap_or(65536); // 64 MB default
        let time_cost = params.time_cost.unwrap_or(3);
        let parallelism = params.parallelism.unwrap_or(1);

        // Validate parameters
        if memory_cost < 8 {
            return Err(FiscusError::InvalidInput(
                "Argon2 memory cost too low (minimum 8 KB)".to_string(),
            ));
        }

        if time_cost < 1 {
            return Err(FiscusError::InvalidInput(
                "Argon2 time cost too low (minimum 1)".to_string(),
            ));
        }

        if !(1..=16).contains(&parallelism) {
            return Err(FiscusError::InvalidInput(
                "Argon2 parallelism must be between 1 and 16".to_string(),
            ));
        }

        // Create Argon2 parameters
        let argon2_params =
            Params::new(memory_cost, time_cost, parallelism, Some(params.key_length)).map_err(
                |e| {
                    error!("Invalid Argon2 parameters: {}", e);
                    FiscusError::InvalidInput(format!("Invalid Argon2 parameters: {e}"))
                },
            )?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

        // Derive the key
        let mut output = vec![0u8; params.key_length];
        argon2
            .hash_password_into(password, &params.salt, &mut output)
            .map_err(|e| {
                error!("Argon2 key derivation failed: {}", e);
                FiscusError::Internal(format!("Key derivation failed: {e}"))
            })?;

        debug!(
            memory_cost = memory_cost,
            time_cost = time_cost,
            parallelism = parallelism,
            key_length = params.key_length,
            "Argon2 key derivation completed successfully"
        );

        Ok(output)
    }
}

#[async_trait]
impl KeyDerivation for Argon2Kdf {
    #[instrument(skip(self, password), fields(salt_len = params.salt.len()))]
    async fn derive_key(
        &self,
        password: &[u8],
        params: &KeyDerivationParams,
    ) -> EncryptionResult<EncryptionKey> {
        let key_bytes = self.derive_key_with_params(password, params).await?;
        let key_id = uuid::Uuid::new_v4().to_string();

        let key = EncryptionKey::new(
            key_bytes,
            KeyType::DerivationKey,
            EncryptionAlgorithm::Aes256Gcm, // Default to AES for derived keys
            key_id,
        );

        debug!(key_id = %key.key_id, "Argon2 key derived successfully");
        Ok(key)
    }

    async fn verify_password(
        &self,
        password: &[u8],
        key: &EncryptionKey,
        params: &KeyDerivationParams,
    ) -> EncryptionResult<bool> {
        let derived_key = self.derive_key_with_params(password, params).await?;

        // Use constant-time comparison
        let is_valid =
            super::utils::TimingSafeComparison::constant_time_eq(&derived_key, key.key_bytes());

        debug!(is_valid = is_valid, "Password verification completed");
        Ok(is_valid)
    }

    fn algorithm(&self) -> KeyDerivationAlgorithm {
        KeyDerivationAlgorithm::Argon2id
    }

    fn generate_params(&self, _key_length: usize) -> EncryptionResult<KeyDerivationParams> {
        let mut rng = SecureRandom::new()?;
        let salt = rng.generate_salt()?;

        Ok(KeyDerivationParams::argon2id_default(salt))
    }
}

/// PBKDF2 key derivation implementation
///
/// PBKDF2 is a widely supported key derivation function, though not as
/// resistant to specialized attacks as Argon2.
#[derive(Debug)]
pub struct Pbkdf2Kdf {
    #[allow(dead_code)]
    secure_random: SecureRandom,
}

impl Pbkdf2Kdf {
    /// Create a new PBKDF2 key derivation instance
    pub fn new() -> EncryptionResult<Self> {
        debug!("Initializing PBKDF2 key derivation");
        Ok(Self {
            secure_random: SecureRandom::new()?,
        })
    }
}

#[async_trait]
impl KeyDerivation for Pbkdf2Kdf {
    #[instrument(skip(self, password), fields(salt_len = params.salt.len(), iterations = params.iterations))]
    async fn derive_key(
        &self,
        password: &[u8],
        params: &KeyDerivationParams,
    ) -> EncryptionResult<EncryptionKey> {
        if params.algorithm != KeyDerivationAlgorithm::Pbkdf2Sha256 {
            return Err(FiscusError::InvalidInput(
                "Algorithm mismatch for PBKDF2 key derivation".to_string(),
            ));
        }

        let iterations = params.iterations.unwrap_or(120_000);

        if iterations < 120_000 {
            warn!(
                "PBKDF2 iteration count is below NIST recommendation (120,000): {}",
                iterations
            );
        }

        let mut output = vec![0u8; params.key_length];
        pbkdf2_hmac::<Sha256>(password, &params.salt, iterations, &mut output);

        let key_id = uuid::Uuid::new_v4().to_string();
        let key = EncryptionKey::new(
            output,
            KeyType::DerivationKey,
            EncryptionAlgorithm::Aes256Gcm,
            key_id,
        );

        debug!(
            iterations = iterations,
            key_id = %key.key_id,
            "PBKDF2 key derived successfully"
        );

        Ok(key)
    }

    async fn verify_password(
        &self,
        password: &[u8],
        key: &EncryptionKey,
        params: &KeyDerivationParams,
    ) -> EncryptionResult<bool> {
        let derived_key = self.derive_key(password, params).await?;

        let is_valid = super::utils::TimingSafeComparison::constant_time_eq(
            derived_key.key_bytes(),
            key.key_bytes(),
        );

        debug!(
            is_valid = is_valid,
            "PBKDF2 password verification completed"
        );
        Ok(is_valid)
    }

    fn algorithm(&self) -> KeyDerivationAlgorithm {
        KeyDerivationAlgorithm::Pbkdf2Sha256
    }

    fn generate_params(&self, _key_length: usize) -> EncryptionResult<KeyDerivationParams> {
        let mut rng = SecureRandom::new()?;
        let salt = rng.generate_salt()?;

        Ok(KeyDerivationParams::pbkdf2_default(salt))
    }
}

/// Scrypt key derivation implementation
///
/// Scrypt is designed to be memory-hard, making it resistant to
/// hardware-based attacks.
#[derive(Debug)]
pub struct ScryptKdf {
    #[allow(dead_code)]
    secure_random: SecureRandom,
}

impl ScryptKdf {
    /// Create a new Scrypt key derivation instance
    pub fn new() -> EncryptionResult<Self> {
        debug!("Initializing Scrypt key derivation");
        Ok(Self {
            secure_random: SecureRandom::new()?,
        })
    }
}

#[async_trait]
impl KeyDerivation for ScryptKdf {
    #[instrument(skip(self, password), fields(salt_len = params.salt.len()))]
    async fn derive_key(
        &self,
        password: &[u8],
        params: &KeyDerivationParams,
    ) -> EncryptionResult<EncryptionKey> {
        if params.algorithm != KeyDerivationAlgorithm::Scrypt {
            return Err(FiscusError::InvalidInput(
                "Algorithm mismatch for Scrypt key derivation".to_string(),
            ));
        }

        let log_n = params.time_cost.unwrap_or(15) as u8;
        let r = params.memory_cost.unwrap_or(8);
        let p = params.parallelism.unwrap_or(1);

        let scrypt_params = ScryptParams::new(log_n, r, p, params.key_length).map_err(|e| {
            error!("Invalid Scrypt parameters: {}", e);
            FiscusError::InvalidInput(format!("Invalid Scrypt parameters: {e}"))
        })?;

        let mut output = vec![0u8; params.key_length];
        scrypt::scrypt(password, &params.salt, &scrypt_params, &mut output).map_err(|e| {
            error!("Scrypt key derivation failed: {}", e);
            FiscusError::Internal(format!("Key derivation failed: {e}"))
        })?;

        let key_id = uuid::Uuid::new_v4().to_string();
        let key = EncryptionKey::new(
            output,
            KeyType::DerivationKey,
            EncryptionAlgorithm::Aes256Gcm,
            key_id,
        );

        debug!(
            log_n = log_n,
            r = r,
            p = p,
            key_id = %key.key_id,
            "Scrypt key derived successfully"
        );

        Ok(key)
    }

    async fn verify_password(
        &self,
        password: &[u8],
        key: &EncryptionKey,
        params: &KeyDerivationParams,
    ) -> EncryptionResult<bool> {
        let derived_key = self.derive_key(password, params).await?;

        let is_valid = super::utils::TimingSafeComparison::constant_time_eq(
            derived_key.key_bytes(),
            key.key_bytes(),
        );

        debug!(
            is_valid = is_valid,
            "Scrypt password verification completed"
        );
        Ok(is_valid)
    }

    fn algorithm(&self) -> KeyDerivationAlgorithm {
        KeyDerivationAlgorithm::Scrypt
    }

    fn generate_params(&self, _key_length: usize) -> EncryptionResult<KeyDerivationParams> {
        let mut rng = SecureRandom::new()?;
        let salt = rng.generate_salt()?;

        Ok(KeyDerivationParams::scrypt_default(salt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_argon2_key_derivation() {
        let kdf = Argon2Kdf::new().unwrap();
        // deepcode ignore HardcodedPassword: <test>
        let password = b"test_password_123";
        let params = kdf.generate_params(32).unwrap();

        let key = kdf.derive_key(password, &params).await.unwrap();
        assert_eq!(key.key_bytes().len(), 32);

        // Verify password
        let is_valid = kdf.verify_password(password, &key, &params).await.unwrap();
        assert!(is_valid);

        // Verify wrong password
        // deepcode ignore HardcodedPassword: <test>
        let wrong_password = b"wrong_password";
        let is_invalid = kdf
            .verify_password(wrong_password, &key, &params)
            .await
            .unwrap();
        assert!(!is_invalid);
    }

    #[tokio::test]
    async fn test_pbkdf2_key_derivation() {
        let kdf = Pbkdf2Kdf::new().unwrap();
        // deepcode ignore HardcodedPassword: <test>
        let password = b"test_password_123";
        let params = kdf.generate_params(32).unwrap();

        let key = kdf.derive_key(password, &params).await.unwrap();
        assert_eq!(key.key_bytes().len(), 32);

        let is_valid = kdf.verify_password(password, &key, &params).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_scrypt_key_derivation() {
        let kdf = ScryptKdf::new().unwrap();
        // deepcode ignore HardcodedPassword: <test>
        let password = b"test_password_123";
        let params = kdf.generate_params(32).unwrap();

        let key = kdf.derive_key(password, &params).await.unwrap();
        assert_eq!(key.key_bytes().len(), 32);

        let is_valid = kdf.verify_password(password, &key, &params).await.unwrap();
        assert!(is_valid);
    }
}
