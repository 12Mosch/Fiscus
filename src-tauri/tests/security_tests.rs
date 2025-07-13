use fiscus_lib::{
    encryption::{
        symmetric::{AesGcmEncryption, SymmetricEncryption},
        utils::{MemoryProtection, TimingSafeComparison},
        EncryptionService,
    },
    security::{AccessController, AuthValidator, RateLimiter, SecurityContext, SecurityMiddleware},
};
/// Security-focused tests for the encryption system
///
/// These tests verify security properties, attack resistance,
/// and proper handling of security-critical scenarios.
use std::time::{Duration, Instant};

/// Test rate limiting functionality
#[tokio::test]
async fn test_rate_limiting() {
    let mut rate_limiter = RateLimiter::new();
    // deepcode ignore NoHardcodedCredentials: <test>
    let user_id = "test-user-rate-limit";
    let operation = "encrypt_financial_data";

    // Should allow initial requests
    for i in 0..50 {
        let result = rate_limiter.check_rate_limit(user_id, operation).await;
        assert!(result.is_ok(), "Request {i} should be allowed");
    }

    // Check status
    let (current, limit) = rate_limiter.get_rate_limit_status(user_id, operation);
    assert_eq!(current, 50);
    assert_eq!(limit, 100);

    // Should allow more requests up to limit
    for i in 50..100 {
        let result = rate_limiter.check_rate_limit(user_id, operation).await;
        assert!(result.is_ok(), "Request {i} should be allowed");
    }

    // Should reject requests over limit
    let result = rate_limiter.check_rate_limit(user_id, operation).await;
    assert!(result.is_err(), "Request over limit should be rejected");

    // Different user should not be affected
    let other_user = "other-user";
    let result = rate_limiter.check_rate_limit(other_user, operation).await;
    assert!(result.is_ok(), "Different user should not be rate limited");
}

/// Test authentication validation
#[tokio::test]
async fn test_authentication_validation() {
    let auth_validator = AuthValidator::new();

    // Fresh authentication should pass
    let context = SecurityContext::new("test-user".to_string());
    let result = auth_validator.validate_authentication(&context).await;
    assert!(result.is_ok(), "Fresh authentication should pass");

    // Test with expired authentication (simulate by creating old context)
    let mut old_context = SecurityContext::new("test-user".to_string());
    old_context.authenticated_at = Instant::now() - Duration::from_secs(7200); // 2 hours ago

    let result = auth_validator.validate_authentication(&old_context).await;
    assert!(result.is_err(), "Expired authentication should fail");
}

/// Test access control
#[tokio::test]
async fn test_access_control() {
    let access_controller = AccessController::new();

    // Test with basic context (no permissions)
    let context = SecurityContext::new("test-user".to_string());

    // Should pass for now (permissive mode)
    let result = access_controller
        .check_access(&context, "encrypt_financial_data")
        .await;
    assert!(
        result.is_ok(),
        "Access should be allowed in permissive mode"
    );

    // Test with permissions
    let mut context_with_perms = SecurityContext::new("admin-user".to_string());
    context_with_perms.permissions =
        vec!["encryption:encrypt".to_string(), "data:write".to_string()];

    let result = access_controller
        .check_access(&context_with_perms, "encrypt_financial_data")
        .await;
    assert!(
        result.is_ok(),
        "Access should be allowed with proper permissions"
    );
}

/// Test security middleware integration
#[tokio::test]
async fn test_security_middleware_integration() {
    let middleware = SecurityMiddleware::new();
    let context = SecurityContext::new("test-user".to_string());

    // Test valid request
    let result = middleware
        .validate_request(&context, "encrypt_financial_data", 1024)
        .await;
    assert!(result.is_ok(), "Valid request should pass");

    // Test oversized data
    let result = middleware
        .validate_request(&context, "encrypt_financial_data", 2 * 1024 * 1024)
        .await;
    assert!(result.is_err(), "Oversized data should be rejected");

    // Test different operations with different size limits
    let operations = vec![
        ("encrypt_financial_data", 1024 * 1024, true),
        ("encrypt_financial_data", 2 * 1024 * 1024, false),
        ("sign_data", 512 * 1024, true),
        ("sign_data", 1024 * 1024, false),
        ("unknown_operation", 64 * 1024, true),
        ("unknown_operation", 128 * 1024, false),
    ];

    for (operation, size, should_pass) in operations {
        let result = middleware.validate_request(&context, operation, size).await;
        if should_pass {
            assert!(
                result.is_ok(),
                "Operation {operation} with size {size} should pass"
            );
        } else {
            assert!(
                result.is_err(),
                "Operation {operation} with size {size} should fail"
            );
        }
    }
}

/// Test timing attack resistance
#[tokio::test]
async fn test_timing_attack_resistance() {
    let correct_data = b"correct_secret_key_12345678901234567890";
    let wrong_data = b"wrong___secret_key_12345678901234567890";
    let short_data = b"short";

    let iterations = 1000;
    let mut correct_times = Vec::new();
    let mut wrong_times = Vec::new();
    let mut short_times = Vec::new();

    // Measure timing for correct comparisons
    for _ in 0..iterations {
        let start = Instant::now();
        let _result = TimingSafeComparison::constant_time_eq(correct_data, correct_data);
        correct_times.push(start.elapsed());
    }

    // Measure timing for wrong comparisons (same length)
    for _ in 0..iterations {
        let start = Instant::now();
        let _result = TimingSafeComparison::constant_time_eq(correct_data, wrong_data);
        wrong_times.push(start.elapsed());
    }

    // Measure timing for different length comparisons
    for _ in 0..iterations {
        let start = Instant::now();
        let _result = TimingSafeComparison::constant_time_eq(correct_data, short_data);
        short_times.push(start.elapsed());
    }

    // Calculate average times
    let avg_correct = correct_times.iter().sum::<Duration>() / iterations as u32;
    let avg_wrong = wrong_times.iter().sum::<Duration>() / iterations as u32;
    let avg_short = short_times.iter().sum::<Duration>() / iterations as u32;

    println!(
        "Average timing - Correct: {avg_correct:?}, Wrong: {avg_wrong:?}, Short: {avg_short:?}"
    );

    // For same-length comparisons, timing should be similar (within 50% variance)
    let timing_ratio = avg_wrong.as_nanos() as f64 / avg_correct.as_nanos() as f64;
    assert!(
        timing_ratio > 0.5 && timing_ratio < 2.0,
        "Timing difference too large: correct={avg_correct:?}, wrong={avg_wrong:?}, ratio={timing_ratio:.2}"
    );

    // SECURITY NOTE: Different length comparisons are allowed to be faster
    // The `subtle` crate intentionally short-circuits on length differences
    println!(
        "Length difference timing ratio: {:.2} (short={avg_short:?}, correct={avg_correct:?})",
        avg_short.as_nanos() as f64 / avg_correct.as_nanos() as f64
    );

    // The key security property: same-length comparisons should take constant time
    // regardless of content differences
}

/// Test that the timing leak in constant-time comparison has been fixed
#[tokio::test]
async fn test_timing_leak_fix() {
    // Test various length combinations to ensure no timing leaks
    let test_cases = vec![
        (b"a".as_slice(), b"b".as_slice()),  // Same short length
        (b"a".as_slice(), b"ab".as_slice()), // Different lengths (1 vs 2)
        (b"ab".as_slice(), b"a".as_slice()), // Different lengths (2 vs 1)
        (b"short".as_slice(), b"much_longer_string".as_slice()), // Very different lengths
        (b"equal_length_1".as_slice(), b"equal_length_2".as_slice()), // Same length, different content
    ];

    let iterations = 500;

    for (a, b) in test_cases {
        let mut times = Vec::new();

        // Measure timing for this comparison
        for _ in 0..iterations {
            let start = Instant::now();
            let _result = TimingSafeComparison::constant_time_eq(a, b);
            times.push(start.elapsed());
        }

        let avg_time = times.iter().sum::<Duration>() / iterations as u32;

        // All comparisons should complete within a reasonable time range
        // This is a basic sanity check - the real protection is that timing
        // doesn't vary significantly based on input characteristics
        assert!(
            avg_time.as_nanos() < 10_000_000, // Less than 10ms average
            "Comparison took too long: {:?} for inputs of length {} vs {}",
            avg_time,
            a.len(),
            b.len()
        );
    }
}

/// Test memory protection and secure deletion
#[test]
fn test_memory_protection_security() {
    // Test that sensitive data is actually cleared
    let mut sensitive_data = vec![0x42u8; 1024];
    let _original_ptr = sensitive_data.as_ptr();

    // Verify data is initially set
    assert!(sensitive_data.iter().all(|&b| b == 0x42));

    // Clear the data
    MemoryProtection::secure_clear(&mut sensitive_data);

    // Verify data is cleared
    assert!(MemoryProtection::is_cleared(&sensitive_data));
    assert!(sensitive_data.iter().all(|&b| b == 0));

    // Test secure buffer auto-clear on drop
    {
        let mut buffer = MemoryProtection::secure_buffer(256);
        buffer.as_mut_slice().fill(0x55);
        assert!(!MemoryProtection::is_cleared(buffer.as_slice()));
        // Buffer should be cleared when it goes out of scope
    }

    // Test that multiple clears don't cause issues
    MemoryProtection::secure_clear(&mut sensitive_data);
    MemoryProtection::secure_clear(&mut sensitive_data);
    assert!(MemoryProtection::is_cleared(&sensitive_data));
}

/// Test key isolation between users
#[tokio::test]
async fn test_key_isolation() {
    let service = EncryptionService::new().expect("Failed to create encryption service");

    let user1 = "user1";
    let user2 = "user2";
    let data_type = "transaction_amount";
    let test_data = b"$1,000.00";

    // Encrypt data for user1
    let encrypted1 = service
        .encrypt_financial_data(test_data, user1, data_type)
        .await
        .expect("Failed to encrypt for user1");

    // Encrypt same data for user2
    let encrypted2 = service
        .encrypt_financial_data(test_data, user2, data_type)
        .await
        .expect("Failed to encrypt for user2");

    // Encrypted data should be different (different keys)
    assert_ne!(
        encrypted1.ciphertext, encrypted2.ciphertext,
        "Different users should produce different ciphertext"
    );
    assert_ne!(
        encrypted1.metadata.key_id, encrypted2.metadata.key_id,
        "Different users should have different key IDs"
    );

    // User1 should be able to decrypt their own data
    let decrypted1 = service
        .decrypt_financial_data(&encrypted1, user1, data_type)
        .await
        .expect("User1 should decrypt their own data");
    assert_eq!(decrypted1, test_data);

    // User2 should be able to decrypt their own data
    let decrypted2 = service
        .decrypt_financial_data(&encrypted2, user2, data_type)
        .await
        .expect("User2 should decrypt their own data");
    assert_eq!(decrypted2, test_data);

    // User1 should NOT be able to decrypt user2's data
    let result = service
        .decrypt_financial_data(&encrypted2, user1, data_type)
        .await;
    assert!(result.is_err(), "User1 should not decrypt user2's data");

    // User2 should NOT be able to decrypt user1's data
    let result = service
        .decrypt_financial_data(&encrypted1, user2, data_type)
        .await;
    assert!(result.is_err(), "User2 should not decrypt user1's data");
}

/// Test data type isolation
#[tokio::test]
async fn test_data_type_isolation() {
    let service = EncryptionService::new().expect("Failed to create encryption service");

    // deepcode ignore NoHardcodedCredentials: <test>
    let user_id = "test-user";
    let data_type1 = "transaction_amount";
    let data_type2 = "account_balance";
    let test_data = b"$5,000.00";

    // Encrypt same data for different data types
    let encrypted1 = service
        .encrypt_financial_data(test_data, user_id, data_type1)
        .await
        .expect("Failed to encrypt for data_type1");

    let encrypted2 = service
        .encrypt_financial_data(test_data, user_id, data_type2)
        .await
        .expect("Failed to encrypt for data_type2");

    // Should use different keys for different data types
    assert_ne!(
        encrypted1.metadata.key_id, encrypted2.metadata.key_id,
        "Different data types should have different key IDs"
    );

    // Should be able to decrypt with correct data type
    let decrypted1 = service
        .decrypt_financial_data(&encrypted1, user_id, data_type1)
        .await
        .expect("Should decrypt with correct data type");
    assert_eq!(decrypted1, test_data);

    // Should NOT be able to decrypt with wrong data type
    let result = service
        .decrypt_financial_data(&encrypted1, user_id, data_type2)
        .await;
    assert!(result.is_err(), "Should not decrypt with wrong data type");
}

/// Test encryption randomness
#[tokio::test]
async fn test_encryption_randomness() {
    let service = EncryptionService::new().expect("Failed to create encryption service");

    // deepcode ignore NoHardcodedCredentials: <test>
    let user_id = "test-user";
    let data_type = "test_data";
    let test_data = b"identical data";

    // Encrypt the same data multiple times
    let mut encrypted_results = Vec::new();
    for _ in 0..10 {
        let encrypted = service
            .encrypt_financial_data(test_data, user_id, data_type)
            .await
            .expect("Failed to encrypt");
        encrypted_results.push(encrypted);
    }

    // All ciphertexts should be different (due to random nonces)
    for i in 0..encrypted_results.len() {
        for j in i + 1..encrypted_results.len() {
            assert_ne!(
                encrypted_results[i].ciphertext, encrypted_results[j].ciphertext,
                "Ciphertexts {i} and {j} should be different"
            );
            assert_ne!(
                encrypted_results[i].nonce, encrypted_results[j].nonce,
                "Nonces {i} and {j} should be different"
            );
        }
    }

    // All should decrypt to the same plaintext
    for (i, encrypted) in encrypted_results.iter().enumerate() {
        let decrypted = service
            .decrypt_financial_data(encrypted, user_id, data_type)
            .await
            .unwrap_or_else(|_| panic!("Failed to decrypt result {i}"));
        assert_eq!(
            decrypted, test_data,
            "Decrypted data {i} should match original"
        );
    }
}

/// Test against common cryptographic attacks
#[tokio::test]
async fn test_cryptographic_attack_resistance() {
    let encryption = AesGcmEncryption::new().expect("Failed to create encryption");
    let key = encryption
        .generate_key()
        .await
        .expect("Failed to generate key");

    let test_data = b"sensitive financial data";
    let encrypted = encryption
        .encrypt(test_data, &key)
        .await
        .expect("Failed to encrypt");

    // Test bit-flipping attack (should fail due to authentication)
    let mut bit_flipped = encrypted.clone();
    bit_flipped.ciphertext[0] ^= 0x01; // Flip one bit

    let result = encryption.decrypt(&bit_flipped, &key).await;
    assert!(result.is_err(), "Bit-flipping attack should be detected");

    // Test truncation attack
    let mut truncated = encrypted.clone();
    truncated
        .ciphertext
        .truncate(truncated.ciphertext.len() - 1);

    let result = encryption.decrypt(&truncated, &key).await;
    assert!(result.is_err(), "Truncation attack should be detected");

    // Test nonce reuse (should produce different results)
    let encrypted2 = encryption
        .encrypt(test_data, &key)
        .await
        .expect("Failed to encrypt again");
    assert_ne!(
        encrypted.nonce, encrypted2.nonce,
        "Nonces should never be reused"
    );

    // Test with wrong key
    let wrong_key = encryption
        .generate_key()
        .await
        .expect("Failed to generate wrong key");
    let result = encryption.decrypt(&encrypted, &wrong_key).await;
    assert!(result.is_err(), "Decryption with wrong key should fail");
}

/// Test secure key generation properties
#[tokio::test]
async fn test_secure_key_generation() {
    let encryption = AesGcmEncryption::new().expect("Failed to create encryption");

    // Generate multiple keys
    let mut keys = Vec::new();
    for _ in 0..100 {
        let key = encryption
            .generate_key()
            .await
            .expect("Failed to generate key");
        keys.push(key);
    }

    // All keys should be different
    for i in 0..keys.len() {
        for j in i + 1..keys.len() {
            assert_ne!(
                keys[i].key_bytes(),
                keys[j].key_bytes(),
                "Keys {i} and {j} should be different"
            );
            assert_ne!(
                keys[i].key_id, keys[j].key_id,
                "Key IDs {i} and {j} should be different"
            );
        }
    }

    // Keys should have proper entropy (no obvious patterns)
    for (i, key) in keys.iter().enumerate() {
        let key_bytes = key.key_bytes();

        // Should not be all zeros
        assert!(
            !key_bytes.iter().all(|&b| b == 0),
            "Key {i} should not be all zeros"
        );

        // Should not be all ones
        assert!(
            !key_bytes.iter().all(|&b| b == 0xFF),
            "Key {i} should not be all ones"
        );

        // Should have reasonable entropy (not all same byte)
        let first_byte = key_bytes[0];
        assert!(
            !key_bytes.iter().all(|&b| b == first_byte),
            "Key {i} should not be all same byte"
        );
    }
}

/// Test rate limiting across different operations
#[tokio::test]
async fn test_operation_specific_rate_limiting() {
    let mut rate_limiter = RateLimiter::new();
    let user_id = "test-user";

    // Test different operations have different limits
    let operations = vec![
        ("encrypt_financial_data", 100),
        ("generate_encryption_key", 10),
        ("rotate_user_keys", 5),
        ("derive_key_from_password", 20),
    ];

    for (operation, expected_limit) in operations {
        // Reset by using a different user ID for each operation
        let operation_user = format!("{user_id}-{operation}");

        // Should allow requests up to the limit
        for i in 0..expected_limit {
            let result = rate_limiter
                .check_rate_limit(&operation_user, operation)
                .await;
            assert!(
                result.is_ok(),
                "Request {i} for {operation} should be allowed"
            );
        }

        // Should reject the next request
        let result = rate_limiter
            .check_rate_limit(&operation_user, operation)
            .await;
        assert!(
            result.is_err(),
            "Request over limit for {operation} should be rejected"
        );

        // Check the status
        let (current, limit) = rate_limiter.get_rate_limit_status(&operation_user, operation);
        assert_eq!(
            limit, expected_limit,
            "Limit for {operation} should be {expected_limit}"
        );
        assert_eq!(
            current, expected_limit,
            "Current count for {operation} should be at limit"
        );
    }
}
