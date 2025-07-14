/// Symmetric encryption implementations for the Fiscus encryption service
///
/// This module provides symmetric encryption capabilities using industry-standard
/// algorithms like AES-256-GCM and ChaCha20-Poly1305 for encrypting financial data.
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use async_trait::async_trait;
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce};
use tracing::{debug, error, instrument};

use super::nonce_manager::NonceManager;
use super::types::{
    EncryptedData, EncryptionAlgorithm, EncryptionKey, EncryptionMetadata, EncryptionResult,
};
use super::utils::SecureRandom;
use crate::error::FiscusError;

/// Trait for symmetric encryption operations
#[async_trait]
pub trait SymmetricEncryption {
    /// Encrypt data using the provided key
    async fn encrypt(&self, data: &[u8], key: &EncryptionKey) -> EncryptionResult<EncryptedData>;

    /// Decrypt data using the provided key
    async fn decrypt(
        &self,
        encrypted_data: &EncryptedData,
        key: &EncryptionKey,
    ) -> EncryptionResult<Vec<u8>>;

    /// Generate a new symmetric key
    async fn generate_key(&self) -> EncryptionResult<EncryptionKey>;

    /// Get the algorithm identifier
    fn algorithm(&self) -> EncryptionAlgorithm;
}

/// AES-256-GCM symmetric encryption implementation
///
/// This is the primary symmetric encryption algorithm used for encrypting
/// financial data at rest. AES-256-GCM provides both confidentiality and
/// authenticity through authenticated encryption.
#[derive(Debug)]
pub struct AesGcmEncryption {
    secure_random: std::sync::Mutex<SecureRandom>,
    nonce_manager: NonceManager,
}

impl AesGcmEncryption {
    /// Create a new AES-GCM encryption instance
    pub fn new() -> EncryptionResult<Self> {
        debug!("Initializing AES-256-GCM encryption");
        Ok(Self {
            secure_random: std::sync::Mutex::new(SecureRandom::new()?),
            nonce_manager: NonceManager::new()?,
        })
    }

    /// Create a new AES-GCM encryption instance with custom nonce manager
    pub fn with_nonce_manager(nonce_manager: NonceManager) -> EncryptionResult<Self> {
        debug!("Initializing AES-256-GCM encryption with custom nonce manager");
        Ok(Self {
            secure_random: std::sync::Mutex::new(SecureRandom::new()?),
            nonce_manager,
        })
    }

    /// Encrypt with additional authenticated data (AAD)
    #[instrument(skip(self, data, key, aad), fields(data_len = data.len(), aad_len = aad.as_ref().map_or(0, |a| a.len())))]
    pub async fn encrypt_with_aad(
        &self,
        data: &[u8],
        key: &EncryptionKey,
        aad: Option<&[u8]>,
    ) -> EncryptionResult<EncryptedData> {
        // Validate key type and algorithm
        if key.algorithm != EncryptionAlgorithm::Aes256Gcm {
            return Err(FiscusError::InvalidInput(
                "Key algorithm mismatch for AES-256-GCM".to_string(),
            ));
        }

        if key.key_bytes().len() != 32 {
            return Err(FiscusError::InvalidInput(
                "Invalid key length for AES-256-GCM (expected 32 bytes)".to_string(),
            ));
        }

        // Create cipher instance
        let key_array = Key::<Aes256Gcm>::from_slice(key.key_bytes());
        let cipher = Aes256Gcm::new(key_array);

        // Generate nonce using nonce manager (supports both random and counter-based)
        let nonce_bytes = self
            .nonce_manager
            .generate_nonce(&key.key_id, EncryptionAlgorithm::Aes256Gcm, None)
            .await?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Perform encryption
        let ciphertext = if let Some(aad_data) = aad {
            cipher.encrypt(
                nonce,
                aes_gcm::aead::Payload {
                    msg: data,
                    aad: aad_data,
                },
            )
        } else {
            cipher.encrypt(nonce, data)
        }
        .map_err(|e| {
            error!("AES-GCM encryption failed: {}", e);
            FiscusError::Internal("Encryption operation failed".to_string())
        })?;

        // Create metadata
        let mut metadata =
            EncryptionMetadata::new(EncryptionAlgorithm::Aes256Gcm, key.key_id.clone());

        if let Some(aad_data) = aad {
            metadata = metadata.with_aad(aad_data.to_vec());
        }

        debug!(
            ciphertext_len = ciphertext.len(),
            nonce_len = nonce_bytes.len(),
            "AES-GCM encryption completed successfully"
        );

        Ok(EncryptedData::new(
            ciphertext,
            nonce_bytes,
            None, // GCM includes auth tag in ciphertext
            metadata,
        ))
    }

    /// Decrypt with additional authenticated data (AAD)
    #[instrument(skip(self, encrypted_data, key), fields(ciphertext_len = encrypted_data.ciphertext.len()))]
    pub async fn decrypt_with_aad(
        &self,
        encrypted_data: &EncryptedData,
        key: &EncryptionKey,
    ) -> EncryptionResult<Vec<u8>> {
        // Validate algorithm
        if encrypted_data.metadata.algorithm != EncryptionAlgorithm::Aes256Gcm {
            return Err(FiscusError::InvalidInput(
                "Algorithm mismatch for AES-256-GCM decryption".to_string(),
            ));
        }

        // Validate key
        if key.algorithm != EncryptionAlgorithm::Aes256Gcm {
            return Err(FiscusError::InvalidInput(
                "Key algorithm mismatch for AES-256-GCM".to_string(),
            ));
        }

        if encrypted_data.nonce.len() != 12 {
            return Err(FiscusError::InvalidInput(
                "Invalid nonce length for AES-256-GCM (expected 12 bytes)".to_string(),
            ));
        }

        // Create cipher instance
        let key_array = Key::<Aes256Gcm>::from_slice(key.key_bytes());
        let cipher = Aes256Gcm::new(key_array);
        let nonce = Nonce::from_slice(&encrypted_data.nonce);

        // Perform decryption
        let plaintext = if let Some(ref aad) = encrypted_data.metadata.aad {
            cipher.decrypt(
                nonce,
                aes_gcm::aead::Payload {
                    msg: &encrypted_data.ciphertext,
                    aad,
                },
            )
        } else {
            cipher.decrypt(nonce, encrypted_data.ciphertext.as_slice())
        }
        .map_err(|e| {
            error!("AES-GCM decryption failed: {}", e);
            FiscusError::Authentication(
                "Decryption failed - invalid key or corrupted data".to_string(),
            )
        })?;

        debug!(
            plaintext_len = plaintext.len(),
            "AES-GCM decryption completed successfully"
        );

        Ok(plaintext)
    }
}

#[async_trait]
impl SymmetricEncryption for AesGcmEncryption {
    #[instrument(skip(self, data, key), fields(data_len = data.len()))]
    async fn encrypt(&self, data: &[u8], key: &EncryptionKey) -> EncryptionResult<EncryptedData> {
        self.encrypt_with_aad(data, key, None).await
    }

    #[instrument(skip(self, encrypted_data, key), fields(ciphertext_len = encrypted_data.ciphertext.len()))]
    async fn decrypt(
        &self,
        encrypted_data: &EncryptedData,
        key: &EncryptionKey,
    ) -> EncryptionResult<Vec<u8>> {
        self.decrypt_with_aad(encrypted_data, key).await
    }

    async fn generate_key(&self) -> EncryptionResult<EncryptionKey> {
        debug!("Generating new AES-256-GCM key");

        let key_bytes = self.secure_random.lock().unwrap().generate_bytes(32)?; // 256-bit key
        let key_id = uuid::Uuid::new_v4().to_string();

        let key = EncryptionKey::new(
            key_bytes,
            super::types::KeyType::Symmetric,
            EncryptionAlgorithm::Aes256Gcm,
            key_id,
        );

        debug!(key_id = %key.key_id, "AES-256-GCM key generated successfully");
        Ok(key)
    }

    fn algorithm(&self) -> EncryptionAlgorithm {
        EncryptionAlgorithm::Aes256Gcm
    }
}

/// ChaCha20-Poly1305 symmetric encryption implementation
///
/// Alternative to AES-256-GCM, particularly useful on systems without
/// hardware AES acceleration.
#[derive(Debug)]
pub struct ChaCha20Poly1305Encryption {
    secure_random: std::sync::Mutex<SecureRandom>,
    nonce_manager: NonceManager,
}

impl ChaCha20Poly1305Encryption {
    /// Create a new ChaCha20-Poly1305 encryption instance
    pub fn new() -> EncryptionResult<Self> {
        debug!("Initializing ChaCha20-Poly1305 encryption");
        Ok(Self {
            secure_random: std::sync::Mutex::new(SecureRandom::new()?),
            nonce_manager: NonceManager::new()?,
        })
    }

    /// Create a new ChaCha20-Poly1305 encryption instance with custom nonce manager
    pub fn with_nonce_manager(nonce_manager: NonceManager) -> EncryptionResult<Self> {
        debug!("Initializing ChaCha20-Poly1305 encryption with custom nonce manager");
        Ok(Self {
            secure_random: std::sync::Mutex::new(SecureRandom::new()?),
            nonce_manager,
        })
    }
}

#[async_trait]
impl SymmetricEncryption for ChaCha20Poly1305Encryption {
    #[instrument(skip(self, data, key), fields(data_len = data.len()))]
    async fn encrypt(&self, data: &[u8], key: &EncryptionKey) -> EncryptionResult<EncryptedData> {
        // Validate key
        if key.algorithm != EncryptionAlgorithm::ChaCha20Poly1305 {
            return Err(FiscusError::InvalidInput(
                "Key algorithm mismatch for ChaCha20-Poly1305".to_string(),
            ));
        }

        if key.key_bytes().len() != 32 {
            return Err(FiscusError::InvalidInput(
                "Invalid key length for ChaCha20-Poly1305 (expected 32 bytes)".to_string(),
            ));
        }

        // Create cipher instance
        let key_array = ChaChaKey::from_slice(key.key_bytes());
        let cipher = ChaCha20Poly1305::new(key_array);

        // Generate nonce using nonce manager (supports both random and counter-based)
        let nonce_bytes = self
            .nonce_manager
            .generate_nonce(&key.key_id, EncryptionAlgorithm::ChaCha20Poly1305, None)
            .await?;
        let nonce = ChaChaNonce::from_slice(&nonce_bytes);

        // Perform encryption
        let ciphertext = cipher.encrypt(nonce, data).map_err(|e| {
            error!("ChaCha20-Poly1305 encryption failed: {}", e);
            FiscusError::Internal("Encryption operation failed".to_string())
        })?;

        let metadata =
            EncryptionMetadata::new(EncryptionAlgorithm::ChaCha20Poly1305, key.key_id.clone());

        debug!(
            ciphertext_len = ciphertext.len(),
            "ChaCha20-Poly1305 encryption completed successfully"
        );

        Ok(EncryptedData::new(
            ciphertext,
            nonce_bytes,
            None, // Poly1305 includes auth tag in ciphertext
            metadata,
        ))
    }

    #[instrument(skip(self, encrypted_data, key), fields(ciphertext_len = encrypted_data.ciphertext.len()))]
    async fn decrypt(
        &self,
        encrypted_data: &EncryptedData,
        key: &EncryptionKey,
    ) -> EncryptionResult<Vec<u8>> {
        // Validate algorithm
        if encrypted_data.metadata.algorithm != EncryptionAlgorithm::ChaCha20Poly1305 {
            return Err(FiscusError::InvalidInput(
                "Algorithm mismatch for ChaCha20-Poly1305 decryption".to_string(),
            ));
        }

        // Create cipher instance
        let key_array = ChaChaKey::from_slice(key.key_bytes());
        let cipher = ChaCha20Poly1305::new(key_array);
        let nonce = ChaChaNonce::from_slice(&encrypted_data.nonce);

        // Perform decryption
        let plaintext = cipher
            .decrypt(nonce, encrypted_data.ciphertext.as_slice())
            .map_err(|e| {
                error!("ChaCha20-Poly1305 decryption failed: {}", e);
                FiscusError::Authentication(
                    "Decryption failed - invalid key or corrupted data".to_string(),
                )
            })?;

        debug!(
            plaintext_len = plaintext.len(),
            "ChaCha20-Poly1305 decryption completed successfully"
        );

        Ok(plaintext)
    }

    async fn generate_key(&self) -> EncryptionResult<EncryptionKey> {
        debug!("Generating new ChaCha20-Poly1305 key");

        let key_bytes = self.secure_random.lock().unwrap().generate_bytes(32)?; // 256-bit key
        let key_id = uuid::Uuid::new_v4().to_string();

        let key = EncryptionKey::new(
            key_bytes,
            super::types::KeyType::Symmetric,
            EncryptionAlgorithm::ChaCha20Poly1305,
            key_id,
        );

        debug!(key_id = %key.key_id, "ChaCha20-Poly1305 key generated successfully");
        Ok(key)
    }

    fn algorithm(&self) -> EncryptionAlgorithm {
        EncryptionAlgorithm::ChaCha20Poly1305
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aes_gcm_encryption_roundtrip() {
        let encryption = AesGcmEncryption::new().unwrap();
        let key = encryption.generate_key().await.unwrap();
        let data = b"sensitive financial data";

        let encrypted = encryption.encrypt(data, &key).await.unwrap();
        let decrypted = encryption.decrypt(&encrypted, &key).await.unwrap();

        assert_eq!(data, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_chacha20_encryption_roundtrip() {
        let encryption = ChaCha20Poly1305Encryption::new().unwrap();
        let key = encryption.generate_key().await.unwrap();
        let data = b"sensitive financial data";

        let encrypted = encryption.encrypt(data, &key).await.unwrap();
        let decrypted = encryption.decrypt(&encrypted, &key).await.unwrap();

        assert_eq!(data, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_aes_gcm_with_aad() {
        let encryption = AesGcmEncryption::new().unwrap();
        let key = encryption.generate_key().await.unwrap();
        let data = b"sensitive data";
        let aad = b"additional authenticated data";

        let encrypted = encryption
            .encrypt_with_aad(data, &key, Some(aad))
            .await
            .unwrap();
        let decrypted = encryption.decrypt_with_aad(&encrypted, &key).await.unwrap();

        assert_eq!(data, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_nonce_reuse_prevention() {
        use crate::encryption::nonce_manager::{NonceConfig, NonceManager, NonceStrategy};
        use std::collections::HashSet;

        // Create encryption with counter-based nonce strategy
        let config = NonceConfig {
            default_strategy: NonceStrategy::CounterBased,
            rotation_threshold: 1000,
            warning_threshold: 800,
            persist_counters: false,
        };
        let nonce_manager = NonceManager::with_config(config).unwrap();
        let encryption = AesGcmEncryption::with_nonce_manager(nonce_manager).unwrap();
        let key = encryption.generate_key().await.unwrap();

        let mut nonces = HashSet::new();
        let data = b"test data for nonce uniqueness";

        // Encrypt 100 times and verify all nonces are unique
        for _ in 0..100 {
            let encrypted = encryption.encrypt(data, &key).await.unwrap();
            assert!(
                nonces.insert(encrypted.nonce.clone()),
                "Duplicate nonce detected!"
            );

            // Verify we can decrypt
            let decrypted = encryption.decrypt(&encrypted, &key).await.unwrap();
            assert_eq!(data, decrypted.as_slice());
        }

        // Verify we have 100 unique nonces
        assert_eq!(nonces.len(), 100);
    }

    #[tokio::test]
    async fn test_rotation_threshold_enforcement() {
        use crate::encryption::nonce_manager::{NonceConfig, NonceManager, NonceStrategy};

        // Create encryption with very low rotation threshold for testing
        let config = NonceConfig {
            default_strategy: NonceStrategy::CounterBased,
            rotation_threshold: 3,
            warning_threshold: 2,
            persist_counters: false,
        };
        let nonce_manager = NonceManager::with_config(config).unwrap();
        let encryption = AesGcmEncryption::with_nonce_manager(nonce_manager).unwrap();
        let key = encryption.generate_key().await.unwrap();

        let data = b"test data";

        // First 3 encryptions should work
        for _ in 0..3 {
            let result = encryption.encrypt(data, &key).await;
            assert!(result.is_ok(), "Encryption should succeed before threshold");
        }

        // 4th encryption should fail due to rotation threshold
        let result = encryption.encrypt(data, &key).await;
        assert!(
            result.is_err(),
            "Encryption should fail after rotation threshold"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("rotation threshold"),
            "Error should mention rotation threshold"
        );
    }
}
