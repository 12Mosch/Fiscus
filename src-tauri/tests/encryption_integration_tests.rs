/// Integration tests for the encryption service
///
/// These tests verify the complete encryption workflow including
/// key management, data encryption/decryption, and security controls.
use std::sync::Arc;

use fiscus_lib::{
    encryption::{
        asymmetric::{AsymmetricEncryption, Ed25519Encryption},
        key_derivation::{Argon2Kdf, KeyDerivation},
        key_management::KeyManager,
        symmetric::{AesGcmEncryption, SymmetricEncryption},
        types::{KeyDerivationAlgorithm, KeyType},
        EncryptionAlgorithm, EncryptionService,
    },
    security::{SecurityContext, SecurityMiddleware},
};

/// Test the complete encryption service workflow
#[tokio::test]
async fn test_encryption_service_workflow() {
    let service = EncryptionService::new().expect("Failed to create encryption service");

    // file deepcode ignore NoHardcodedCredentials: <test>
    let user_id = "test-user-123";
    let data_type = "transaction_amount";
    let test_data = b"$12,345.67 - Salary payment from ACME Corp";

    // Test encryption
    let encrypted_data = service
        .encrypt_financial_data(test_data, user_id, data_type)
        .await
        .expect("Failed to encrypt data");

    assert!(!encrypted_data.ciphertext.is_empty());
    assert!(!encrypted_data.nonce.is_empty());
    assert_eq!(
        encrypted_data.metadata.algorithm,
        EncryptionAlgorithm::Aes256Gcm
    );

    // Test decryption
    let decrypted_data = service
        .decrypt_financial_data(&encrypted_data, user_id, data_type)
        .await
        .expect("Failed to decrypt data");

    assert_eq!(decrypted_data, test_data);
}

/// Test key rotation functionality
#[tokio::test]
async fn test_key_rotation() {
    let service = EncryptionService::new().expect("Failed to create encryption service");
    let user_id = "test-user-rotation";
    let data_type = "account_balance";
    let test_data = b"$50,000.00";

    // Encrypt data with initial key
    let encrypted_1 = service
        .encrypt_financial_data(test_data, user_id, data_type)
        .await
        .expect("Failed to encrypt with initial key");

    // Rotate keys
    service
        .rotate_user_keys(user_id)
        .await
        .expect("Failed to rotate keys");

    // Should still be able to decrypt old data
    let decrypted_1 = service
        .decrypt_financial_data(&encrypted_1, user_id, data_type)
        .await
        .expect("Failed to decrypt after key rotation");

    assert_eq!(decrypted_1, test_data);

    // New encryption should use new key
    let encrypted_2 = service
        .encrypt_financial_data(test_data, user_id, data_type)
        .await
        .expect("Failed to encrypt with new key");

    // Both should decrypt to the same data
    let decrypted_2 = service
        .decrypt_financial_data(&encrypted_2, user_id, data_type)
        .await
        .expect("Failed to decrypt with new key");

    assert_eq!(decrypted_2, test_data);
}

/// Test encryption with different data types
#[tokio::test]
async fn test_multiple_data_types() {
    let service = EncryptionService::new().expect("Failed to create encryption service");
    let user_id = "test-user-multi";

    let test_cases: Vec<(&str, &[u8])> = vec![
        ("transaction_amount", b"$1,234.56"),
        ("account_number", b"1234567890123456"),
        ("personal_note", b"Remember to pay rent on the 1st"),
        ("category_description", b"Groceries and household items"),
    ];

    for (data_type, test_data) in test_cases {
        // Encrypt
        let encrypted = service
            .encrypt_financial_data(test_data, user_id, data_type)
            .await
            .unwrap_or_else(|_| panic!("Failed to encrypt {data_type}"));

        // Decrypt
        let decrypted = service
            .decrypt_financial_data(&encrypted, user_id, data_type)
            .await
            .unwrap_or_else(|_| panic!("Failed to decrypt {data_type}"));

        assert_eq!(decrypted, test_data, "Data mismatch for type: {data_type}");
    }
}

/// Test security middleware integration
#[tokio::test]
async fn test_security_middleware() {
    let middleware = SecurityMiddleware::new();
    let context = SecurityContext::new("test-user-security".to_string());

    // Test valid request
    let result = middleware
        .validate_request(&context, "encrypt_financial_data", 1024)
        .await;
    assert!(
        result.is_ok(),
        "Valid request should pass security validation"
    );

    // Test oversized data
    let result = middleware
        .validate_request(&context, "encrypt_financial_data", 2 * 1024 * 1024) // 2MB
        .await;
    assert!(
        result.is_err(),
        "Oversized data should fail security validation"
    );
}

/// Test symmetric encryption algorithms
#[tokio::test]
async fn test_symmetric_encryption_algorithms() {
    // Test AES-256-GCM
    let aes_encryption = AesGcmEncryption::new().expect("Failed to create AES encryption");
    let aes_key = aes_encryption
        .generate_key()
        .await
        .expect("Failed to generate AES key");

    let test_data = b"Symmetric encryption test data";

    let encrypted = aes_encryption
        .encrypt(test_data, &aes_key)
        .await
        .expect("AES encryption failed");
    let decrypted = aes_encryption
        .decrypt(&encrypted, &aes_key)
        .await
        .expect("AES decryption failed");

    assert_eq!(decrypted, test_data);
    assert_eq!(aes_key.algorithm, EncryptionAlgorithm::Aes256Gcm);
    assert_eq!(aes_key.key_type, KeyType::Symmetric);
}

/// Test asymmetric encryption algorithms
#[tokio::test]
async fn test_asymmetric_encryption_algorithms() {
    // Test Ed25519 signatures
    let ed25519 = Ed25519Encryption::new().expect("Failed to create Ed25519 encryption");
    let (private_key, public_key) = ed25519
        .generate_keypair()
        .await
        .expect("Failed to generate Ed25519 keypair");

    let test_data = b"Digital signature test message";

    // Sign data
    let signature = ed25519
        .sign_data(test_data, &private_key)
        .await
        .expect("Ed25519 signing failed");

    // Verify signature
    let is_valid = ed25519
        .verify_signature(test_data, &signature, public_key.key_bytes())
        .await
        .expect("Ed25519 verification failed");
    assert!(is_valid, "Ed25519 signature should be valid");

    // Test with wrong data
    let wrong_data = b"Wrong message";
    let is_invalid = ed25519
        .verify_signature(wrong_data, &signature, public_key.key_bytes())
        .await
        .expect("Ed25519 verification failed");
    assert!(
        !is_invalid,
        "Ed25519 signature should be invalid for wrong data"
    );
}

/// Test key derivation functions
#[tokio::test]
async fn test_key_derivation() {
    let kdf = Argon2Kdf::new().expect("Failed to create Argon2 KDF");
    // file deepcode ignore HardcodedPassword/test: <test>
    let password = b"test_password_123!@#";

    // Generate parameters
    let params = kdf
        .generate_params(32)
        .expect("Failed to generate KDF params");
    assert_eq!(params.algorithm, KeyDerivationAlgorithm::Argon2id);
    assert_eq!(params.key_length, 32);

    // Derive key
    let key = kdf
        .derive_key(password, &params)
        .await
        .expect("Failed to derive key");
    assert_eq!(key.key_bytes().len(), 32);
    assert_eq!(key.key_type, KeyType::DerivationKey);

    // Verify password
    let is_valid = kdf
        .verify_password(password, &key, &params)
        .await
        .expect("Failed to verify password");
    assert!(is_valid, "Password verification should succeed");

    // Test wrong password
    let wrong_password = b"wrong_password";
    let is_invalid = kdf
        .verify_password(wrong_password, &key, &params)
        .await
        .expect("Failed to verify wrong password");
    assert!(!is_invalid, "Wrong password verification should fail");
}

/// Test key manager functionality
#[tokio::test]
async fn test_key_manager() {
    let key_manager = KeyManager::new().expect("Failed to create key manager");
    let user_id = "test-user-km";
    let data_type = "test_data";

    // Get or create key
    let key1 = key_manager
        .get_or_create_key(user_id, data_type)
        .await
        .expect("Failed to get/create key");

    // Get same key again
    let key2 = key_manager
        .get_key(user_id, data_type)
        .await
        .expect("Failed to get existing key");

    assert_eq!(key1.key_id, key2.key_id, "Should return the same key");

    // List user keys
    let user_keys = key_manager
        .list_user_keys(user_id)
        .await
        .expect("Failed to list user keys");
    assert!(
        user_keys.contains(&data_type.to_string()),
        "Should contain the created key type"
    );

    // Get stats
    let stats = key_manager.get_stats().await.expect("Failed to get stats");
    assert!(stats.total_keys > 0, "Should have at least one key");
}

/// Test error handling
#[tokio::test]
async fn test_error_handling() {
    let service = EncryptionService::new().expect("Failed to create encryption service");

    // Test decryption with wrong key
    let user_id = "test-user-error";
    let data_type = "test_data";
    let test_data = b"test data";

    // Encrypt with one user
    let encrypted = service
        .encrypt_financial_data(test_data, user_id, data_type)
        .await
        .expect("Failed to encrypt");

    // Try to decrypt with different user (should fail)
    let wrong_user = "wrong-user";
    let result = service
        .decrypt_financial_data(&encrypted, wrong_user, data_type)
        .await;

    assert!(result.is_err(), "Decryption with wrong user should fail");

    // Test with invalid data type
    let result = service
        .decrypt_financial_data(&encrypted, user_id, "wrong_data_type")
        .await;

    assert!(
        result.is_err(),
        "Decryption with wrong data type should fail"
    );
}

/// Test concurrent operations
#[tokio::test]
async fn test_concurrent_operations() {
    let service = Arc::new(EncryptionService::new().expect("Failed to create encryption service"));
    let user_id = "test-user-concurrent";

    let mut handles = Vec::new();

    // Spawn multiple concurrent encryption operations
    for i in 0..10 {
        let service_clone = Arc::clone(&service);
        let user_id_clone = user_id.to_string();
        let data_type = format!("data_type_{i}");
        let test_data = format!("Test data {i}").into_bytes();

        let handle = tokio::spawn(async move {
            // Encrypt
            let encrypted = service_clone
                .encrypt_financial_data(&test_data, &user_id_clone, &data_type)
                .await
                .expect("Concurrent encryption failed");

            // Decrypt
            let decrypted = service_clone
                .decrypt_financial_data(&encrypted, &user_id_clone, &data_type)
                .await
                .expect("Concurrent decryption failed");

            assert_eq!(decrypted, test_data);
            i
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await.expect("Concurrent operation panicked");
        assert!(result < 10, "All operations should complete successfully");
    }
}

/// Test large data encryption
#[tokio::test]
async fn test_large_data_encryption() {
    let service = EncryptionService::new().expect("Failed to create encryption service");
    let user_id = "test-user-large";
    let data_type = "large_data";

    // Create 100KB of test data
    let large_data = vec![0x42u8; 100 * 1024];

    let start = std::time::Instant::now();

    // Encrypt
    let encrypted = service
        .encrypt_financial_data(&large_data, user_id, data_type)
        .await
        .expect("Failed to encrypt large data");

    // Decrypt
    let decrypted = service
        .decrypt_financial_data(&encrypted, user_id, data_type)
        .await
        .expect("Failed to decrypt large data");

    let duration = start.elapsed();

    assert_eq!(decrypted, large_data);
    assert!(
        duration.as_millis() < 1000,
        "Large data encryption should complete within 1 second"
    );

    println!("Large data encryption/decryption took: {duration:?}");
}
