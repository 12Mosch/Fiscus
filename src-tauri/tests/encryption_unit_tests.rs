/// Unit tests for individual encryption components
///
/// These tests focus on testing individual components in isolation
/// to ensure each part of the encryption system works correctly.
use fiscus_lib::encryption::{
    asymmetric::{AsymmetricEncryption, Ed25519Encryption},
    key_derivation::{Argon2Kdf, KeyDerivation, Pbkdf2Kdf, ScryptKdf},
    symmetric::{AesGcmEncryption, SymmetricEncryption},
    types::{
        EncryptedData, EncryptionAlgorithm, EncryptionKey, EncryptionMetadata,
        KeyDerivationAlgorithm, KeyDerivationParams, KeyType, SecureBytes,
    },
    utils::{EncodingUtils, MemoryProtection, SecureRandom, TimingSafeComparison, ValidationUtils},
};

/// Test secure random number generation
#[tokio::test]
async fn test_secure_random() {
    let mut rng = SecureRandom::new().expect("Failed to create secure random");

    // Test byte generation
    let bytes1 = rng.generate_bytes(32).expect("Failed to generate bytes");
    let bytes2 = rng.generate_bytes(32).expect("Failed to generate bytes");

    assert_eq!(bytes1.len(), 32);
    assert_eq!(bytes2.len(), 32);
    assert_ne!(bytes1, bytes2, "Random bytes should be different");

    // Test salt generation
    let salt = rng.generate_salt().expect("Failed to generate salt");
    assert_eq!(salt.len(), 32);

    // Test nonce generation
    let nonce = rng
        .generate_nonce(EncryptionAlgorithm::Aes256Gcm)
        .expect("Failed to generate nonce");
    assert_eq!(nonce.len(), 12); // AES-GCM uses 96-bit nonces

    // Test error cases
    let result = rng.generate_bytes(0);
    assert!(result.is_err(), "Should fail for zero-length bytes");

    let result = rng.generate_bytes(2 * 1024 * 1024); // 2MB
    assert!(result.is_err(), "Should fail for oversized requests");
}

/// Test memory protection utilities
#[test]
fn test_memory_protection() {
    let mut data = vec![0x42u8; 16];

    // Test secure clearing
    MemoryProtection::secure_clear(&mut data);
    assert!(
        MemoryProtection::is_cleared(&data),
        "Data should be cleared"
    );

    // Test secure buffer
    let mut buffer = MemoryProtection::secure_buffer(32);
    assert_eq!(buffer.len(), 32);
    assert!(MemoryProtection::is_cleared(buffer.as_slice()));

    // Fill buffer with test data
    buffer.as_mut_slice().copy_from_slice(&[0x55u8; 32]);
    assert!(!MemoryProtection::is_cleared(buffer.as_slice()));

    // Clear buffer
    buffer.clear();
    assert!(MemoryProtection::is_cleared(buffer.as_slice()));
}

/// Test encoding utilities
#[test]
fn test_encoding_utils() {
    let test_data = b"Hello, World!";

    // Test base64 encoding/decoding
    let base64_encoded = EncodingUtils::encode_base64(test_data);
    let base64_decoded =
        EncodingUtils::decode_base64(&base64_encoded).expect("Base64 decode failed");
    assert_eq!(base64_decoded, test_data);

    // Test hex encoding/decoding
    let hex_encoded = EncodingUtils::encode_hex(test_data);
    let hex_decoded = EncodingUtils::decode_hex(&hex_encoded).expect("Hex decode failed");
    assert_eq!(hex_decoded, test_data);

    // Test safe display
    let safe_display = EncodingUtils::safe_display(test_data, 10);
    assert!(safe_display.contains("48656c6c")); // "Hell" in hex

    // Test error cases
    let result = EncodingUtils::decode_base64("invalid base64!");
    assert!(result.is_err(), "Should fail for invalid base64");

    let result = EncodingUtils::decode_hex("invalid hex!");
    assert!(result.is_err(), "Should fail for invalid hex");
}

/// Test timing-safe comparison
#[test]
fn test_timing_safe_comparison() {
    let data1 = b"secret_key_123";
    let data2 = b"secret_key_123";
    let data3 = b"secret_key_456";

    assert!(TimingSafeComparison::constant_time_eq(data1, data2));
    assert!(!TimingSafeComparison::constant_time_eq(data1, data3));

    // Test different lengths
    let short_data = b"short";
    assert!(!TimingSafeComparison::constant_time_eq(data1, short_data));

    // Test string comparison
    assert!(TimingSafeComparison::verify_strings("test", "test"));
    assert!(!TimingSafeComparison::verify_strings("test", "different"));
}

/// Test validation utilities
#[test]
fn test_validation_utils() {
    // Test key length validation
    assert!(ValidationUtils::validate_key_length(32, EncryptionAlgorithm::Aes256Gcm).is_ok());
    assert!(ValidationUtils::validate_key_length(16, EncryptionAlgorithm::Aes256Gcm).is_err());

    // Test nonce length validation
    assert!(ValidationUtils::validate_nonce_length(12, EncryptionAlgorithm::Aes256Gcm).is_ok());
    assert!(ValidationUtils::validate_nonce_length(16, EncryptionAlgorithm::Aes256Gcm).is_err());

    // Test non-empty validation
    assert!(ValidationUtils::validate_non_empty(b"data", "test_field").is_ok());
    assert!(ValidationUtils::validate_non_empty(b"", "test_field").is_err());

    // Test data size validation
    assert!(ValidationUtils::validate_data_size(b"small data", 1024, "test_field").is_ok());
    assert!(ValidationUtils::validate_data_size(&vec![0u8; 2048], 1024, "test_field").is_err());
}

/// Test encryption key functionality
#[test]
fn test_encryption_key() {
    let key_data = vec![0x42u8; 32];
    let key_id = "test-key-123".to_string();

    let key = EncryptionKey::new(
        key_data.clone(),
        KeyType::Symmetric,
        EncryptionAlgorithm::Aes256Gcm,
        key_id.clone(),
    );

    assert_eq!(key.key_bytes(), &key_data);
    assert_eq!(key.key_type, KeyType::Symmetric);
    assert_eq!(key.algorithm, EncryptionAlgorithm::Aes256Gcm);
    assert_eq!(key.key_id, key_id);
    assert!(key.is_valid());
    assert!(!key.is_expired());
}

/// Test secure bytes container
#[test]
fn test_secure_bytes() {
    let data = vec![1, 2, 3, 4, 5];
    let secure_bytes = SecureBytes::from(data.clone());

    assert_eq!(secure_bytes.as_slice(), &data);
    assert_eq!(secure_bytes.len(), 5);
    assert!(!secure_bytes.is_empty());

    let recovered_data = secure_bytes.into_vec();
    assert_eq!(recovered_data, data);
}

/// Test encrypted data container
#[test]
fn test_encrypted_data() {
    let ciphertext = vec![1, 2, 3, 4];
    let nonce = vec![5, 6, 7, 8];
    let tag = Some(vec![9, 10, 11, 12]);
    let metadata = EncryptionMetadata::new(EncryptionAlgorithm::Aes256Gcm, "test-key".to_string());

    let encrypted_data = EncryptedData::new(
        ciphertext.clone(),
        nonce.clone(),
        tag.clone(),
        metadata.clone(),
    );

    assert_eq!(encrypted_data.ciphertext, ciphertext);
    assert_eq!(encrypted_data.nonce, nonce);
    assert_eq!(encrypted_data.tag, tag);
    assert_eq!(
        encrypted_data.metadata.algorithm,
        EncryptionAlgorithm::Aes256Gcm
    );

    let total_size = encrypted_data.total_size();
    assert_eq!(total_size, 12); // 4 (ciphertext) + 4 (nonce) + 4 (tag)
}

/// Test key derivation parameters
#[test]
fn test_key_derivation_params() {
    let salt = vec![0x42u8; 16];

    // Test Argon2 parameters
    let argon2_params = KeyDerivationParams::argon2id_default(salt.clone());
    assert_eq!(argon2_params.algorithm, KeyDerivationAlgorithm::Argon2id);
    assert_eq!(argon2_params.salt, salt);
    assert_eq!(argon2_params.key_length, 32);
    assert_eq!(argon2_params.memory_cost, Some(65536));
    assert_eq!(argon2_params.time_cost, Some(3));
    assert_eq!(argon2_params.parallelism, Some(1));

    // Test PBKDF2 parameters
    let pbkdf2_params = KeyDerivationParams::pbkdf2_default(salt.clone());
    assert_eq!(
        pbkdf2_params.algorithm,
        KeyDerivationAlgorithm::Pbkdf2Sha256
    );
    assert_eq!(pbkdf2_params.iterations, Some(100_000));

    // Test Scrypt parameters
    let scrypt_params = KeyDerivationParams::scrypt_default(salt.clone());
    assert_eq!(scrypt_params.algorithm, KeyDerivationAlgorithm::Scrypt);
    assert_eq!(scrypt_params.memory_cost, Some(8)); // r parameter (block size)
    assert_eq!(scrypt_params.time_cost, Some(15)); // log_n parameter (2^15 = 32768)
    assert_eq!(scrypt_params.parallelism, Some(1)); // p parameter
}

/// Test AES-GCM encryption edge cases
#[tokio::test]
async fn test_aes_gcm_edge_cases() {
    let encryption = AesGcmEncryption::new().expect("Failed to create AES-GCM encryption");
    let key = encryption
        .generate_key()
        .await
        .expect("Failed to generate key");

    // Test empty data
    let empty_data = b"";
    let encrypted = encryption
        .encrypt(empty_data, &key)
        .await
        .expect("Failed to encrypt empty data");
    let decrypted = encryption
        .decrypt(&encrypted, &key)
        .await
        .expect("Failed to decrypt empty data");
    assert_eq!(decrypted, empty_data);

    // Test single byte
    let single_byte = b"A";
    let encrypted = encryption
        .encrypt(single_byte, &key)
        .await
        .expect("Failed to encrypt single byte");
    let decrypted = encryption
        .decrypt(&encrypted, &key)
        .await
        .expect("Failed to decrypt single byte");
    assert_eq!(decrypted, single_byte);

    // Test with AAD
    let data = b"test data";
    let aad = b"additional authenticated data";
    let encrypted = encryption
        .encrypt_with_aad(data, &key, Some(aad))
        .await
        .expect("Failed to encrypt with AAD");
    let decrypted = encryption
        .decrypt_with_aad(&encrypted, &key)
        .await
        .expect("Failed to decrypt with AAD");
    assert_eq!(decrypted, data);
}

/// Test Ed25519 signature edge cases
#[tokio::test]
async fn test_ed25519_edge_cases() {
    let ed25519 = Ed25519Encryption::new().expect("Failed to create Ed25519");
    let (private_key, public_key) = ed25519
        .generate_keypair()
        .await
        .expect("Failed to generate keypair");

    // Test empty message
    let empty_message = b"";
    let signature = ed25519
        .sign_data(empty_message, &private_key)
        .await
        .expect("Failed to sign empty message");
    let is_valid = ed25519
        .verify_signature(empty_message, &signature, public_key.key_bytes())
        .await
        .expect("Failed to verify empty message");
    assert!(is_valid);

    // Test large message
    let large_message = vec![0x42u8; 10000];
    let signature = ed25519
        .sign_data(&large_message, &private_key)
        .await
        .expect("Failed to sign large message");
    let is_valid = ed25519
        .verify_signature(&large_message, &signature, public_key.key_bytes())
        .await
        .expect("Failed to verify large message");
    assert!(is_valid);

    // Test invalid signature length
    let test_message = b"test message";
    let invalid_signature = vec![0u8; 32]; // Wrong length (should be 64)
    let result = ed25519
        .verify_signature(test_message, &invalid_signature, public_key.key_bytes())
        .await;
    assert!(result.is_err(), "Should fail with invalid signature length");

    // Test invalid public key length
    let signature = ed25519
        .sign_data(test_message, &private_key)
        .await
        .expect("Failed to sign");
    let invalid_public_key = vec![0u8; 16]; // Wrong length (should be 32)
    let result = ed25519
        .verify_signature(test_message, &signature, &invalid_public_key)
        .await;
    assert!(
        result.is_err(),
        "Should fail with invalid public key length"
    );
}

/// Test key derivation with different parameters
#[tokio::test]
async fn test_key_derivation_parameters() {
    // deepcode ignore HardcodedPassword: <test>
    let password = b"test_password_123";
    let salt = vec![0x42u8; 32];

    // Test Argon2 with different parameters
    let argon2_kdf = Argon2Kdf::new().expect("Failed to create Argon2 KDF");

    let mut params = KeyDerivationParams::argon2id_default(salt.clone());
    params.memory_cost = Some(32768); // 32 MB
    params.time_cost = Some(2);

    let key = argon2_kdf
        .derive_key(password, &params)
        .await
        .expect("Failed to derive key with custom params");
    assert_eq!(key.key_bytes().len(), 32);

    // Test PBKDF2 with different iterations
    let pbkdf2_kdf = Pbkdf2Kdf::new().expect("Failed to create PBKDF2 KDF");

    let mut params = KeyDerivationParams::pbkdf2_default(salt.clone());
    params.iterations = Some(50_000);

    let key = pbkdf2_kdf
        .derive_key(password, &params)
        .await
        .expect("Failed to derive key with custom iterations");
    assert_eq!(key.key_bytes().len(), 32);

    // Test Scrypt with different parameters
    let scrypt_kdf = ScryptKdf::new().expect("Failed to create Scrypt KDF");

    let mut params = KeyDerivationParams::scrypt_default(salt.clone());
    params.time_cost = Some(10); // log_n = 10 (N = 2^10 = 1024)
    params.memory_cost = Some(8); // r = 8 (block size)

    let key = scrypt_kdf
        .derive_key(password, &params)
        .await
        .expect("Failed to derive key with custom Scrypt params");
    assert_eq!(key.key_bytes().len(), 32);
}

/// Test error conditions
#[tokio::test]
async fn test_error_conditions() {
    // Test invalid key for AES-GCM
    let encryption = AesGcmEncryption::new().expect("Failed to create AES-GCM");
    let invalid_key = EncryptionKey::new(
        vec![0u8; 16], // Wrong length (should be 32)
        KeyType::Symmetric,
        EncryptionAlgorithm::Aes256Gcm,
        "invalid-key".to_string(),
    );

    let result = encryption.encrypt(b"test", &invalid_key).await;
    assert!(result.is_err(), "Should fail with invalid key length");

    // Test algorithm mismatch
    let valid_key = encryption
        .generate_key()
        .await
        .expect("Failed to generate key");
    let _mismatched_key = valid_key.clone();
    // Can't directly modify the algorithm field, so we'll test with wrong key type instead

    // Test with corrupted encrypted data
    let test_data = b"test data";
    let encrypted = encryption
        .encrypt(test_data, &valid_key)
        .await
        .expect("Failed to encrypt");

    let mut corrupted = encrypted.clone();
    corrupted.ciphertext[0] ^= 0xFF; // Flip bits to corrupt

    let result = encryption.decrypt(&corrupted, &valid_key).await;
    assert!(result.is_err(), "Should fail with corrupted ciphertext");
}

/// Test performance characteristics
#[tokio::test]
async fn test_performance_characteristics() {
    let encryption = AesGcmEncryption::new().expect("Failed to create encryption");
    let key = encryption
        .generate_key()
        .await
        .expect("Failed to generate key");

    let data_sizes = vec![1024, 10240, 102400]; // 1KB, 10KB, 100KB

    for size in data_sizes {
        let test_data = vec![0x42u8; size];

        let start = std::time::Instant::now();
        let encrypted = encryption
            .encrypt(&test_data, &key)
            .await
            .expect("Encryption failed");
        let encrypt_time = start.elapsed();

        let start = std::time::Instant::now();
        let decrypted = encryption
            .decrypt(&encrypted, &key)
            .await
            .expect("Decryption failed");
        let decrypt_time = start.elapsed();

        assert_eq!(decrypted, test_data);

        // Performance should be reasonable (less than 100 ms for these sizes)
        assert!(
            encrypt_time.as_millis() < 100,
            "Encryption too slow for {size} bytes: {encrypt_time:?}"
        );
        assert!(
            decrypt_time.as_millis() < 100,
            "Decryption too slow for {size} bytes: {decrypt_time:?}"
        );

        println!("Size: {size} bytes, Encrypt: {encrypt_time:?}, Decrypt: {decrypt_time:?}");
    }
}
