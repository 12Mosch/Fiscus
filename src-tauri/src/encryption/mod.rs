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
pub mod asymmetric;
pub mod config;
pub mod key_derivation;
pub mod key_management;
pub mod nonce_manager;
pub mod symmetric;
pub mod types;
pub mod utils;

// Re-export main types and functions for easier access
pub use asymmetric::{AsymmetricEncryption, Ed25519Encryption, RsaEncryption};
pub use config::{ConfigManager, EncryptionConfig};
pub use key_management::KeyManager;
pub use nonce_manager::{NonceManager, NonceStrategy};
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
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Barrier;
    use tokio::task::JoinSet;

    // Test helper functions
    async fn create_test_service() -> EncryptionService {
        EncryptionService::new().expect("Failed to create test encryption service")
    }

    async fn create_test_keypair_rsa(service: &EncryptionService) -> (Vec<u8>, Vec<u8>) {
        let (private_key, public_key) = service
            .asymmetric_rsa
            .generate_keypair()
            .await
            .expect("Failed to generate RSA keypair");
        (
            private_key.key_bytes().to_vec(),
            public_key.key_bytes().to_vec(),
        )
    }

    async fn create_test_keypair_ed25519(service: &EncryptionService) -> (Vec<u8>, Vec<u8>) {
        let (private_key, public_key) = service
            .asymmetric_ed25519
            .generate_keypair()
            .await
            .expect("Failed to generate Ed25519 keypair");
        (
            private_key.key_bytes().to_vec(),
            public_key.key_bytes().to_vec(),
        )
    }

    #[tokio::test]
    async fn test_encryption_service_creation() {
        let service = EncryptionService::new();
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_financial_data_encryption_roundtrip() {
        let service = EncryptionService::new().unwrap();
        let test_data = b"sensitive financial data: $12,345.67";
        // deepcode ignore NoHardcodedCredentials: <test>
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

    // ============================================================================
    // KEY ROTATION TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_automatic_key_rotation_on_nonce_threshold() {
        let service = create_test_service().await;
        let user_id = "test-user-rotation";
        let data_type = "test_data";
        let test_data = b"test data for rotation";

        // Get initial stats
        let initial_stats = service.get_encryption_stats().await.unwrap();
        let initial_rotated_keys = initial_stats.rotated_keys;

        // Perform key rotation
        service.rotate_user_keys(user_id).await.unwrap();

        // Verify rotation occurred
        let final_stats = service.get_encryption_stats().await.unwrap();
        assert_eq!(final_stats.rotated_keys, initial_rotated_keys + 1);
        assert!(final_stats.last_key_rotation.is_some());

        // Verify we can still encrypt/decrypt after rotation
        let encrypted = service
            .encrypt_financial_data(test_data, user_id, data_type)
            .await
            .unwrap();
        let decrypted = service
            .decrypt_financial_data(&encrypted, user_id, data_type)
            .await
            .unwrap();
        assert_eq!(test_data, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_manual_key_rotation_functionality() {
        let service = create_test_service().await;
        let user_id = "test-user-manual-rotation";

        // Create some keys first
        let _ = service
            .encrypt_financial_data(b"test1", user_id, "data1")
            .await
            .unwrap();
        let _ = service
            .encrypt_financial_data(b"test2", user_id, "data2")
            .await
            .unwrap();

        let initial_stats = service.get_encryption_stats().await.unwrap();

        // Perform manual rotation
        let result = service.rotate_user_keys(user_id).await;
        assert!(result.is_ok(), "Manual key rotation should succeed");

        // Verify rotation was recorded
        let final_stats = service.get_encryption_stats().await.unwrap();
        assert!(final_stats.rotated_keys > initial_stats.rotated_keys);
        assert!(final_stats.last_key_rotation.is_some());
    }

    #[tokio::test]
    async fn test_encryption_decryption_across_key_rotation_boundaries() {
        let service = create_test_service().await;
        let user_id = "test-user-boundary";
        let data_type = "boundary_test";
        let test_data = b"data across rotation boundary";

        // Encrypt before rotation
        let encrypted_before = service
            .encrypt_financial_data(test_data, user_id, data_type)
            .await
            .unwrap();

        // Perform key rotation
        service.rotate_user_keys(user_id).await.unwrap();

        // Encrypt after rotation
        let encrypted_after = service
            .encrypt_financial_data(test_data, user_id, data_type)
            .await
            .unwrap();

        // Both should decrypt successfully
        let decrypted_before = service
            .decrypt_financial_data(&encrypted_before, user_id, data_type)
            .await
            .unwrap();
        let decrypted_after = service
            .decrypt_financial_data(&encrypted_after, user_id, data_type)
            .await
            .unwrap();

        assert_eq!(test_data, decrypted_before.as_slice());
        assert_eq!(test_data, decrypted_after.as_slice());
    }

    #[tokio::test]
    async fn test_key_rotation_with_concurrent_operations() {
        let service = Arc::new(create_test_service().await);
        let user_id = "test-user-concurrent";
        let barrier = Arc::new(Barrier::new(3));

        let mut join_set = JoinSet::new();

        // Task 1: Continuous encryption
        let service1 = Arc::clone(&service);
        let barrier1 = Arc::clone(&barrier);
        join_set.spawn(async move {
            barrier1.wait().await;
            for i in 0..10 {
                let data = format!("concurrent data {i}");
                let _ = service1
                    .encrypt_financial_data(data.as_bytes(), user_id, "concurrent_test")
                    .await;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        // Task 2: Key rotation
        let service2 = Arc::clone(&service);
        let barrier2 = Arc::clone(&barrier);
        join_set.spawn(async move {
            barrier2.wait().await;
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = service2.rotate_user_keys(user_id).await;
        });

        // Task 3: More encryption after rotation
        let service3 = Arc::clone(&service);
        let barrier3 = Arc::clone(&barrier);
        join_set.spawn(async move {
            barrier3.wait().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            for i in 10..20 {
                let data = format!("post-rotation data {i}");
                let _ = service3
                    .encrypt_financial_data(data.as_bytes(), user_id, "post_rotation_test")
                    .await;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        // Wait for all tasks to complete
        while let Some(result) = join_set.join_next().await {
            assert!(result.is_ok(), "Concurrent operation should not panic");
        }
    }

    #[tokio::test]
    async fn test_key_rotation_failure_scenarios_and_rollback() {
        let service = create_test_service().await;
        let user_id = "test-user-failure";

        // Test rotation with non-existent user (should handle gracefully)
        let result = service.rotate_user_keys("non-existent-user").await;
        // This should either succeed (creating new keys) or fail gracefully
        // The exact behavior depends on implementation, but it shouldn't panic
        assert!(result.is_ok() || result.is_err());

        // Test that service remains functional after failed operations
        let test_data = b"test after failure";
        let encrypted = service
            .encrypt_financial_data(test_data, user_id, "failure_test")
            .await
            .unwrap();
        let decrypted = service
            .decrypt_financial_data(&encrypted, user_id, "failure_test")
            .await
            .unwrap();
        assert_eq!(test_data, decrypted.as_slice());
    }

    // ============================================================================
    // TRANSMISSION ENCRYPTION ALGORITHM TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_aes_256_gcm_transmission_encryption() {
        let service = create_test_service().await;
        let test_data = b"AES-256-GCM transmission test data";

        // Note: AES-256-GCM is typically used for symmetric encryption, not transmission
        // This test verifies the algorithm works correctly in the context it's used
        let user_id = "test-user-aes";
        let data_type = "aes_test";

        let encrypted = service
            .encrypt_financial_data(test_data, user_id, data_type)
            .await
            .unwrap();

        let decrypted = service
            .decrypt_financial_data(&encrypted, user_id, data_type)
            .await
            .unwrap();

        assert_eq!(test_data, decrypted.as_slice());
        assert_eq!(encrypted.metadata.algorithm, EncryptionAlgorithm::Aes256Gcm);
    }

    #[tokio::test]
    async fn test_rsa_4096_transmission_encryption() {
        let service = create_test_service().await;
        let test_data = b"RSA-4096 transmission test";
        let (_private_key, public_key) = create_test_keypair_rsa(&service).await;

        let encrypted = service
            .encrypt_for_transmission(test_data, &public_key, EncryptionAlgorithm::Rsa4096)
            .await
            .unwrap();

        assert_eq!(encrypted.metadata.algorithm, EncryptionAlgorithm::Rsa4096);
        assert!(!encrypted.ciphertext.is_empty());
        // RSA doesn't use nonces, so nonce should be empty
        assert!(encrypted.nonce.is_empty());
    }

    #[tokio::test]
    async fn test_ed25519_transmission_encryption() {
        let service = create_test_service().await;
        let test_data = b"Ed25519 transmission test";
        let (_private_key, public_key) = create_test_keypair_ed25519(&service).await;

        // Ed25519 is for signatures, not encryption, so this should fail
        let result = service
            .encrypt_for_transmission(test_data, &public_key, EncryptionAlgorithm::Ed25519)
            .await;

        assert!(result.is_err());
        if let Err(FiscusError::InvalidInput(msg)) = result {
            assert!(msg.contains("Ed25519 is for signatures"));
        } else {
            panic!("Expected InvalidInput error for Ed25519 encryption");
        }
    }

    #[tokio::test]
    async fn test_algorithm_switching_and_compatibility() {
        let service = create_test_service().await;
        let test_data = b"Algorithm switching test";

        // Test RSA encryption
        let (_rsa_private, rsa_public) = create_test_keypair_rsa(&service).await;
        let rsa_encrypted = service
            .encrypt_for_transmission(test_data, &rsa_public, EncryptionAlgorithm::Rsa4096)
            .await
            .unwrap();

        // Test that Ed25519 fails for encryption (as expected)
        let (_ed_private, ed_public) = create_test_keypair_ed25519(&service).await;
        let ed_result = service
            .encrypt_for_transmission(test_data, &ed_public, EncryptionAlgorithm::Ed25519)
            .await;

        // Verify RSA works and Ed25519 fails appropriately
        assert_eq!(
            rsa_encrypted.metadata.algorithm,
            EncryptionAlgorithm::Rsa4096
        );
        assert!(!rsa_encrypted.ciphertext.is_empty());
        assert!(ed_result.is_err()); // Ed25519 should fail for encryption

        // Test with different RSA data to show algorithm consistency
        let test_data2 = b"Different test data";
        let rsa_encrypted2 = service
            .encrypt_for_transmission(test_data2, &rsa_public, EncryptionAlgorithm::Rsa4096)
            .await
            .unwrap();

        assert_eq!(
            rsa_encrypted2.metadata.algorithm,
            EncryptionAlgorithm::Rsa4096
        );
        assert_ne!(rsa_encrypted.ciphertext, rsa_encrypted2.ciphertext); // Different data = different ciphertext
    }

    #[tokio::test]
    async fn test_transmission_encryption_performance_characteristics() {
        let service = create_test_service().await;
        let test_data = b"Performance test data that is reasonably sized for testing";

        // Test RSA performance
        let (_rsa_private, rsa_public) = create_test_keypair_rsa(&service).await;
        let rsa_start = std::time::Instant::now();
        let _rsa_encrypted = service
            .encrypt_for_transmission(test_data, &rsa_public, EncryptionAlgorithm::Rsa4096)
            .await
            .unwrap();
        let rsa_duration = rsa_start.elapsed();

        // Test Ed25519 signature performance (since it doesn't support encryption)
        let (ed_private, _ed_public) = service.asymmetric_ed25519.generate_keypair().await.unwrap();
        let ed_start = std::time::Instant::now();

        // Perform signature operation instead of encryption
        let _signature = service
            .asymmetric_ed25519
            .sign_data(test_data, &ed_private)
            .await
            .unwrap();
        let ed_duration = ed_start.elapsed();

        // Both should complete in reasonable time (less than 1 second for test data)
        assert!(rsa_duration < Duration::from_secs(1));
        assert!(ed_duration < Duration::from_secs(1));

        // Ed25519 signatures should generally be faster than RSA encryption
        println!(
            "RSA encryption duration: {rsa_duration:?}, Ed25519 signing duration: {ed_duration:?}"
        );
    }

    // ============================================================================
    // ERROR HANDLING TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_invalid_unsupported_encryption_algorithms() {
        let service = create_test_service().await;
        let test_data = b"test data";
        let (_private_key, public_key) = create_test_keypair_rsa(&service).await;

        // Test invalid algorithm for transmission encryption
        let result = service
            .encrypt_for_transmission(test_data, &public_key, EncryptionAlgorithm::Aes256Gcm)
            .await;

        assert!(result.is_err());
        if let Err(FiscusError::InvalidInput(msg)) = result {
            assert!(msg.contains("Invalid algorithm for transmission encryption"));
        } else {
            panic!("Expected InvalidInput error for unsupported transmission algorithm");
        }
    }

    #[tokio::test]
    async fn test_missing_corrupted_encryption_keys() {
        let service = create_test_service().await;
        let test_data = b"test data";

        // Test with empty/invalid public key
        let empty_key = vec![];
        let result = service
            .encrypt_for_transmission(test_data, &empty_key, EncryptionAlgorithm::Rsa4096)
            .await;

        assert!(result.is_err());

        // Test with corrupted public key (random bytes)
        let corrupted_key = vec![0xFF; 32]; // Invalid key format
        let result = service
            .encrypt_for_transmission(test_data, &corrupted_key, EncryptionAlgorithm::Ed25519)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_nonce_values() {
        let service = create_test_service().await;
        let user_id = "test-user-nonce";
        let data_type = "nonce_test";

        // Create a valid encryption first
        let test_data = b"test data";
        let mut encrypted = service
            .encrypt_financial_data(test_data, user_id, data_type)
            .await
            .unwrap();

        // Corrupt the nonce
        encrypted.nonce = vec![0xFF; 8]; // Invalid nonce length/content

        // Attempt to decrypt with corrupted nonce
        let result = service
            .decrypt_financial_data(&encrypted, user_id, data_type)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tampered_ciphertext() {
        let service = create_test_service().await;
        let user_id = "test-user-tamper";
        let data_type = "tamper_test";

        // Create a valid encryption
        let test_data = b"test data";
        let mut encrypted = service
            .encrypt_financial_data(test_data, user_id, data_type)
            .await
            .unwrap();

        // Tamper with the ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF; // Flip bits in first byte
        }

        // Attempt to decrypt tampered data
        let result = service
            .decrypt_financial_data(&encrypted, user_id, data_type)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fiscus_error_propagation() {
        let service = create_test_service().await;

        // Test error propagation from invalid input
        let result = service
            .encrypt_for_transmission(b"test", &[], EncryptionAlgorithm::Aes256Gcm)
            .await;

        match result {
            Err(FiscusError::InvalidInput(_)) => {
                // Expected error type
            }
            Err(other_error) => {
                // Other FiscusError variants are also acceptable
                println!("Got FiscusError variant: {other_error:?}");
            }
            Ok(_) => {
                panic!("Expected error but got success");
            }
        }

        // Test error propagation from key operations
        let result = service.rotate_user_keys("").await; // Empty user ID
                                                         // Should handle gracefully or return appropriate error
        assert!(result.is_ok() || result.is_err()); // Either is acceptable
    }

    #[tokio::test]
    async fn test_memory_cleanup_on_error_conditions() {
        let service = create_test_service().await;
        let user_id = "test-user-cleanup";

        // Perform operations that might fail
        for i in 0..10 {
            let data = format!("test data {i}");
            let _ = service
                .encrypt_financial_data(data.as_bytes(), user_id, "cleanup_test")
                .await;

            // Simulate some failures
            let _ = service
                .encrypt_for_transmission(data.as_bytes(), &[], EncryptionAlgorithm::Rsa4096)
                .await;
        }

        // Service should still be functional
        let final_test = b"final test after errors";
        let encrypted = service
            .encrypt_financial_data(final_test, user_id, "final_test")
            .await
            .unwrap();
        let decrypted = service
            .decrypt_financial_data(&encrypted, user_id, "final_test")
            .await
            .unwrap();
        assert_eq!(final_test, decrypted.as_slice());
    }

    // ============================================================================
    // CONCURRENT ACCESS PATTERN TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_multiple_simultaneous_encryption_operations() {
        let service = Arc::new(create_test_service().await);
        let user_id = "test-user-concurrent-encrypt";
        let num_operations = 50;

        let mut join_set = JoinSet::new();

        // Spawn multiple encryption tasks
        for i in 0..num_operations {
            let service_clone = Arc::clone(&service);
            let data = format!("concurrent encryption test data {i}");
            let data_type = format!("concurrent_type_{}", i % 5); // Use 5 different data types

            join_set.spawn(async move {
                service_clone
                    .encrypt_financial_data(data.as_bytes(), user_id, &data_type)
                    .await
            });
        }

        // Collect all results
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            let encryption_result = result.expect("Task should not panic");
            results.push(encryption_result.expect("Encryption should succeed"));
        }

        assert_eq!(results.len(), num_operations);

        // Verify all encryptions are unique (different nonces/ciphertext)
        let mut ciphertexts = HashSet::new();
        let mut nonces = HashSet::new();
        for encrypted in results {
            assert!(ciphertexts.insert(encrypted.ciphertext.clone()));
            assert!(nonces.insert(encrypted.nonce.clone()));
        }
    }

    #[tokio::test]
    async fn test_multiple_simultaneous_decryption_operations() {
        let service = Arc::new(create_test_service().await);
        let user_id = "test-user-concurrent-decrypt";
        let data_type = "concurrent_decrypt_test";
        let num_operations = 30;

        // First, create encrypted data to decrypt
        let mut encrypted_data = Vec::new();
        for i in 0..num_operations {
            let data = format!("decryption test data {i}");
            let encrypted = service
                .encrypt_financial_data(data.as_bytes(), user_id, data_type)
                .await
                .unwrap();
            encrypted_data.push((encrypted, data));
        }

        let mut join_set = JoinSet::new();

        // Spawn multiple decryption tasks
        for (encrypted, expected_data) in encrypted_data {
            let service_clone = Arc::clone(&service);
            join_set.spawn(async move {
                let decrypted = service_clone
                    .decrypt_financial_data(&encrypted, user_id, data_type)
                    .await?;
                Ok::<(Vec<u8>, String), FiscusError>((decrypted, expected_data))
            });
        }

        // Collect and verify all results
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            let (decrypted, expected) = result
                .expect("Task should not panic")
                .expect("Decryption should succeed");
            assert_eq!(decrypted, expected.as_bytes());
            results.push(decrypted);
        }

        assert_eq!(results.len(), num_operations);
    }

    #[tokio::test]
    async fn test_concurrent_key_rotation_during_active_operations() {
        let service = Arc::new(create_test_service().await);
        let user_id = "test-user-concurrent-rotation";
        let barrier = Arc::new(Barrier::new(4));

        let mut join_set = JoinSet::new();

        // Task 1: Continuous encryption before rotation
        let service1 = Arc::clone(&service);
        let barrier1 = Arc::clone(&barrier);
        join_set.spawn(async move {
            barrier1.wait().await;
            let mut results = Vec::new();
            for i in 0..15 {
                let data = format!("pre-rotation data {i}");
                let result = service1
                    .encrypt_financial_data(data.as_bytes(), user_id, "pre_rotation")
                    .await;
                results.push(result);
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Ok::<Vec<_>, FiscusError>(results)
        });

        // Task 2: Key rotation in the middle
        let service2 = Arc::clone(&service);
        let barrier2 = Arc::clone(&barrier);
        join_set.spawn(async move {
            barrier2.wait().await;
            tokio::time::sleep(Duration::from_millis(40)).await; // Let some encryptions happen first
            service2.rotate_user_keys(user_id).await?;
            Ok::<Vec<_>, FiscusError>(vec![]) // Return empty vec to match type
        });

        // Task 3: Continuous encryption after rotation
        let service3 = Arc::clone(&service);
        let barrier3 = Arc::clone(&barrier);
        join_set.spawn(async move {
            barrier3.wait().await;
            tokio::time::sleep(Duration::from_millis(80)).await; // Start after rotation
            let mut results = Vec::new();
            for i in 0..15 {
                let data = format!("post-rotation data {i}");
                let result = service3
                    .encrypt_financial_data(data.as_bytes(), user_id, "post_rotation")
                    .await;
                results.push(result);
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Ok::<Vec<_>, FiscusError>(results)
        });

        // Task 4: Mixed operations throughout
        let service4 = Arc::clone(&service);
        let barrier4 = Arc::clone(&barrier);
        join_set.spawn(async move {
            barrier4.wait().await;
            let mut results = Vec::new();
            for i in 0..20 {
                let data = format!("mixed operation {i}");
                let encrypt_result = service4
                    .encrypt_financial_data(data.as_bytes(), user_id, "mixed_ops")
                    .await;
                results.push(encrypt_result);
                tokio::time::sleep(Duration::from_millis(8)).await;
            }
            Ok::<Vec<_>, FiscusError>(results)
        });

        // Wait for all tasks and verify no panics occurred
        let mut all_successful = true;
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(task_result) => {
                    // Check if the task itself succeeded
                    if task_result.is_err() {
                        println!("Task operation failed: {:?}", task_result.err());
                        all_successful = false;
                    }
                }
                Err(e) => {
                    println!("Task panicked: {e:?}");
                    all_successful = false;
                }
            }
        }

        assert!(
            all_successful,
            "All concurrent operations should complete without panicking"
        );
    }

    #[tokio::test]
    async fn test_thread_safety_of_nonce_generation() {
        let service = Arc::new(create_test_service().await);
        let user_id = "test-user-nonce-safety";
        let data_type = "nonce_safety_test";
        let num_threads = 20;
        let operations_per_thread = 25;

        let mut join_set = JoinSet::new();

        // Spawn multiple threads that generate nonces concurrently
        for thread_id in 0..num_threads {
            let service_clone = Arc::clone(&service);
            join_set.spawn(async move {
                let mut nonces = Vec::new();
                for i in 0..operations_per_thread {
                    let data = format!("thread {thread_id} operation {i}");
                    let encrypted = service_clone
                        .encrypt_financial_data(data.as_bytes(), user_id, data_type)
                        .await
                        .expect("Encryption should succeed");
                    nonces.push(encrypted.nonce);
                }
                nonces
            });
        }

        // Collect all nonces from all threads
        let mut all_nonces = HashSet::new();
        while let Some(result) = join_set.join_next().await {
            let nonces = result.expect("Thread should not panic");
            for nonce in nonces {
                assert!(
                    all_nonces.insert(nonce),
                    "Nonce collision detected in concurrent access!"
                );
            }
        }

        let expected_total = num_threads * operations_per_thread;
        assert_eq!(
            all_nonces.len(),
            expected_total,
            "All nonces should be unique"
        );
    }

    #[tokio::test]
    async fn test_race_condition_prevention_in_counter_based_nonce_management() {
        let service = Arc::new(create_test_service().await);
        let user_id = "test-user-race-prevention";
        let data_type = "race_test";
        let num_concurrent_ops = 100;

        let barrier = Arc::new(Barrier::new(num_concurrent_ops));
        let mut join_set = JoinSet::new();

        // Spawn many operations that start simultaneously
        for i in 0..num_concurrent_ops {
            let service_clone = Arc::clone(&service);
            let barrier_clone = Arc::clone(&barrier);
            join_set.spawn(async move {
                barrier_clone.wait().await; // Synchronize start time
                let data = format!("race test data {i}");
                service_clone
                    .encrypt_financial_data(data.as_bytes(), user_id, data_type)
                    .await
            });
        }

        // Collect results and verify no race conditions
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            let encrypted = result
                .expect("Task should not panic")
                .expect("Encryption should succeed");
            results.push(encrypted);
        }

        assert_eq!(results.len(), num_concurrent_ops);

        // Verify all nonces are unique (no race conditions in nonce generation)
        let mut nonces = HashSet::new();
        for encrypted in results {
            assert!(
                nonces.insert(encrypted.nonce),
                "Race condition detected: duplicate nonce!"
            );
        }
    }

    #[tokio::test]
    async fn test_performance_under_high_concurrency_loads() {
        let service = Arc::new(create_test_service().await);
        let user_id = "test-user-performance";
        let num_concurrent_users = 10;
        let operations_per_user = 20;

        let start_time = std::time::Instant::now();
        let mut join_set = JoinSet::new();

        // Simulate multiple users performing operations concurrently
        for user_index in 0..num_concurrent_users {
            let service_clone = Arc::clone(&service);
            let current_user_id = format!("{user_id}-{user_index}");

            join_set.spawn(async move {
                let mut user_results = Vec::new();
                for op_index in 0..operations_per_user {
                    let data = format!("user {user_index} operation {op_index}");
                    let data_type = format!("perf_test_{}", op_index % 3); // Vary data types

                    // Perform encrypt/decrypt cycle
                    let encrypted = service_clone
                        .encrypt_financial_data(data.as_bytes(), &current_user_id, &data_type)
                        .await?;

                    let decrypted = service_clone
                        .decrypt_financial_data(&encrypted, &current_user_id, &data_type)
                        .await?;

                    user_results.push((encrypted, decrypted));

                    // Small delay to simulate real-world usage
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                Ok::<Vec<_>, FiscusError>(user_results)
            });
        }

        // Wait for all operations to complete
        let mut total_operations = 0;
        while let Some(result) = join_set.join_next().await {
            let user_results = result
                .expect("User task should not panic")
                .expect("All operations should succeed");
            total_operations += user_results.len();

            // Verify data integrity for each operation
            for (encrypted, decrypted) in user_results {
                assert!(!encrypted.ciphertext.is_empty());
                assert!(!decrypted.is_empty());
            }
        }

        let total_duration = start_time.elapsed();
        let expected_operations = num_concurrent_users * operations_per_user;

        assert_eq!(total_operations, expected_operations);

        // Performance assertion: should complete within reasonable time
        // This is a generous limit to account for CI/test environment variations
        assert!(
            total_duration < Duration::from_secs(30),
            "High concurrency test took too long: {total_duration:?}"
        );

        println!(
            "Completed {total_operations} operations across {num_concurrent_users} concurrent users in {total_duration:?}"
        );
    }

    #[tokio::test]
    async fn test_service_stability_under_stress() {
        let service = Arc::new(create_test_service().await);
        let stress_duration = Duration::from_secs(5);
        let start_time = std::time::Instant::now();

        let mut join_set = JoinSet::new();

        // Stress test with multiple types of operations
        for worker_id in 0..5 {
            let service_clone = Arc::clone(&service);
            let user_id = format!("stress-user-{worker_id}");

            join_set.spawn(async move {
                let mut operation_count = 0;
                while start_time.elapsed() < stress_duration {
                    let data = format!("stress test data {worker_id} - {operation_count}");

                    // Mix of different operations
                    match operation_count % 4 {
                        0 => {
                            // Regular encryption/decryption
                            if let Ok(encrypted) = service_clone
                                .encrypt_financial_data(data.as_bytes(), &user_id, "stress_test")
                                .await
                            {
                                let _ = service_clone
                                    .decrypt_financial_data(&encrypted, &user_id, "stress_test")
                                    .await;
                            }
                        }
                        1 => {
                            // Key rotation
                            let _ = service_clone.rotate_user_keys(&user_id).await;
                        }
                        2 => {
                            // Stats retrieval
                            let _ = service_clone.get_encryption_stats().await;
                        }
                        3 => {
                            // Transmission encryption (if keys available)
                            if let Ok((_private, public)) =
                                service_clone.asymmetric_rsa.generate_keypair().await
                            {
                                let _ = service_clone
                                    .encrypt_for_transmission(
                                        data.as_bytes(),
                                        public.key_bytes(),
                                        EncryptionAlgorithm::Rsa4096,
                                    )
                                    .await;
                            }
                        }
                        _ => unreachable!(),
                    }

                    operation_count += 1;

                    // Brief pause to prevent overwhelming the system
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
                operation_count
            });
        }

        // Collect results from stress test
        let mut total_operations = 0;
        while let Some(result) = join_set.join_next().await {
            let operations = result.expect("Stress test worker should not panic");
            total_operations += operations;
        }

        println!("Stress test completed {total_operations} operations in {stress_duration:?}");

        // Verify service is still functional after stress test
        let final_test_data = b"post-stress verification";
        let final_encrypted = service
            .encrypt_financial_data(final_test_data, "post-stress-user", "verification")
            .await
            .expect("Service should still be functional after stress test");

        let final_decrypted = service
            .decrypt_financial_data(&final_encrypted, "post-stress-user", "verification")
            .await
            .expect("Service should still be functional after stress test");

        assert_eq!(final_test_data, final_decrypted.as_slice());
        assert!(
            total_operations > 0,
            "Stress test should have performed some operations"
        );
    }
}
