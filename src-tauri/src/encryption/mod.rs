pub mod asymmetric;
pub mod key_derivation;
pub mod key_management;
/// Comprehensive encryption service for Fiscus personal finance application
///
/// This module provides secure data protection for sensitive financial information
/// using industry-standard encryption algorithms and best practices.
///
/// Features:
/// - AES-256-GCM symmetric encryption for data at rest
/// - RSA-4096 and Ed25519 asymmetric encryption for key exchange
/// - Secure key derivation using Argon2, PBKDF2, and Scrypt
/// - Key management with rotation capabilities
/// - Memory-safe operations with secure deletion
/// - Comprehensive error handling and logging
pub mod symmetric;
pub mod types;
pub mod utils;

// Re-export main types and functions for easier access
pub use asymmetric::{AsymmetricEncryption, Ed25519Encryption, RsaEncryption};
pub use key_management::KeyManager;
pub use symmetric::{AesGcmEncryption, SymmetricEncryption};
pub use types::{EncryptedData, EncryptionAlgorithm, EncryptionResult};

use crate::error::FiscusError;
use tracing::{debug, info};

/// Main encryption service that coordinates all encryption operations
///
/// This service provides a high-level interface for encryption operations
/// while maintaining security best practices and proper error handling.
pub struct EncryptionService {
    symmetric: Box<dyn SymmetricEncryption + Send + Sync>,
    asymmetric_rsa: Box<dyn AsymmetricEncryption + Send + Sync>,
    asymmetric_ed25519: Box<dyn AsymmetricEncryption + Send + Sync>,
    key_manager: KeyManager,
}

impl EncryptionService {
    /// Create a new encryption service with default algorithms
    pub fn new() -> Result<Self, FiscusError> {
        info!("Initializing encryption service");

        let symmetric = Box::new(AesGcmEncryption::new()?);
        let asymmetric_rsa = Box::new(RsaEncryption::new()?);
        let asymmetric_ed25519 = Box::new(Ed25519Encryption::new()?);
        let key_manager = KeyManager::new()?;

        debug!("Encryption service initialized successfully");

        Ok(Self {
            symmetric,
            asymmetric_rsa,
            asymmetric_ed25519,
            key_manager,
        })
    }

    /// Encrypt sensitive financial data using symmetric encryption
    ///
    /// This method is optimized for encrypting financial data like transaction amounts,
    /// account balances, and personal information that needs to be stored securely.
    pub async fn encrypt_financial_data(
        &self,
        data: &[u8],
        user_id: &str,
        data_type: &str,
    ) -> EncryptionResult<EncryptedData> {
        debug!(
            user_id = user_id,
            data_type = data_type,
            data_size = data.len(),
            "Encrypting financial data"
        );

        // Get or derive encryption key for this user and data type
        let key = self
            .key_manager
            .get_or_create_key(user_id, data_type)
            .await?;

        // Encrypt using AES-256-GCM
        let encrypted = self.symmetric.encrypt(data, &key).await?;

        debug!(
            user_id = user_id,
            data_type = data_type,
            encrypted_size = encrypted.ciphertext.len(),
            "Financial data encrypted successfully"
        );

        Ok(encrypted)
    }

    /// Decrypt sensitive financial data
    pub async fn decrypt_financial_data(
        &self,
        encrypted_data: &EncryptedData,
        user_id: &str,
        data_type: &str,
    ) -> EncryptionResult<Vec<u8>> {
        debug!(
            user_id = user_id,
            data_type = data_type,
            encrypted_size = encrypted_data.ciphertext.len(),
            key_id = %encrypted_data.metadata.key_id,
            "Decrypting financial data"
        );

        // Validate that the user has access to this key
        // This ensures security even when using key_id directly and prevents
        // users from accessing data encrypted with keys they don't own
        self.key_manager
            .validate_user_key_access(user_id, data_type, &encrypted_data.metadata.key_id)
            .await?;

        // Get the encryption key using the key_id from the encrypted data's metadata
        // This ensures we use the correct key even after key rotation, as old keys
        // are kept available for decrypting existing data while new keys are used
        // for new encryptions
        let key = self
            .key_manager
            .get_key_by_id(&encrypted_data.metadata.key_id)
            .await?;

        // Decrypt using AES-256-GCM
        let decrypted = self.symmetric.decrypt(encrypted_data, &key).await?;

        debug!(
            user_id = user_id,
            data_type = data_type,
            decrypted_size = decrypted.len(),
            key_id = %encrypted_data.metadata.key_id,
            "Financial data decrypted successfully"
        );

        Ok(decrypted)
    }

    /// Encrypt data for transmission (using asymmetric encryption)
    pub async fn encrypt_for_transmission(
        &self,
        data: &[u8],
        recipient_public_key: &[u8],
        algorithm: EncryptionAlgorithm,
    ) -> EncryptionResult<EncryptedData> {
        debug!(
            data_size = data.len(),
            algorithm = ?algorithm,
            "Encrypting data for transmission"
        );

        let encrypted = match algorithm {
            EncryptionAlgorithm::Rsa4096 => {
                self.asymmetric_rsa
                    .encrypt_with_public_key(data, recipient_public_key)
                    .await?
            }
            EncryptionAlgorithm::Ed25519 => {
                self.asymmetric_ed25519
                    .encrypt_with_public_key(data, recipient_public_key)
                    .await?
            }
            _ => {
                return Err(FiscusError::InvalidInput(
                    "Invalid algorithm for transmission encryption".to_string(),
                ));
            }
        };

        debug!(
            encrypted_size = encrypted.ciphertext.len(),
            algorithm = ?algorithm,
            "Data encrypted for transmission successfully"
        );

        Ok(encrypted)
    }

    /// Rotate encryption keys for a user
    pub async fn rotate_user_keys(&self, user_id: &str) -> EncryptionResult<()> {
        info!(user_id = user_id, "Starting key rotation");

        self.key_manager.rotate_user_keys(user_id).await?;

        info!(user_id = user_id, "Key rotation completed successfully");
        Ok(())
    }

    /// Get encryption statistics for monitoring
    pub async fn get_encryption_stats(&self) -> EncryptionResult<EncryptionStats> {
        self.key_manager.get_stats().await
    }
}

/// Statistics about encryption operations for monitoring and auditing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptionStats {
    pub total_keys: usize,
    pub active_keys: usize,
    pub rotated_keys: usize,
    pub encryption_operations: u64,
    pub decryption_operations: u64,
    pub key_derivation_operations: u64,
    pub last_key_rotation: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for EncryptionService {
    fn default() -> Self {
        Self::new().expect("Failed to create default encryption service")
    }
}

// Ensure sensitive data is properly cleared from memory
impl Drop for EncryptionService {
    fn drop(&mut self) {
        debug!("Cleaning up encryption service");
        // The individual components will handle their own cleanup
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encryption_service_creation() {
        let service = EncryptionService::new();
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_financial_data_encryption_roundtrip() {
        let service = EncryptionService::new().unwrap();
        let test_data = b"sensitive financial data: $12,345.67";
        let user_id = "test-user-123";
        let data_type = "transaction_amount";

        // Encrypt
        let encrypted = service
            .encrypt_financial_data(test_data, user_id, data_type)
            .await
            .unwrap();

        // Decrypt
        let decrypted = service
            .decrypt_financial_data(&encrypted, user_id, data_type)
            .await
            .unwrap();

        assert_eq!(test_data, decrypted.as_slice());
    }
}
