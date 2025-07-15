/// Example demonstrating nonce reuse prevention in high-volume encryption scenarios
///
/// This example shows how to configure and use the NonceManager to prevent nonce reuse
/// in high-volume encryption scenarios, which is critical for maintaining security
/// when performing many encryption operations with the same key.
use fiscus_lib::encryption::{
    nonce_manager::{NonceConfig, NonceManager, NonceStrategy},
    symmetric::{AesGcmEncryption, SymmetricEncryption},
};
use std::collections::HashSet;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting nonce reuse prevention demonstration");

    // Example 1: Default random nonce strategy (current behavior)
    demonstrate_random_nonces().await?;

    // Example 2: Counter-based nonce strategy for high-volume scenarios
    demonstrate_counter_based_nonces().await?;

    // Example 3: Key rotation enforcement
    demonstrate_key_rotation().await?;

    // Example 4: Hybrid strategy with fallback
    demonstrate_hybrid_strategy().await?;

    info!("Nonce reuse prevention demonstration completed successfully");
    Ok(())
}

/// Demonstrate traditional random nonce generation
async fn demonstrate_random_nonces() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Random Nonce Strategy Demo ===");

    let encryption = AesGcmEncryption::new()?;
    let key = encryption.generate_key().await?;
    let data = b"Financial transaction: $1,234.56";

    let mut nonces = HashSet::new();

    // Perform 1000 encryptions with random nonces
    for i in 0..1000 {
        let encrypted = encryption.encrypt(data, &key).await?;

        // Check for nonce collision (very unlikely but possible)
        if !nonces.insert(encrypted.nonce.clone()) {
            warn!("Nonce collision detected at iteration {}", i);
        }

        // Verify decryption works
        let decrypted = encryption.decrypt(&encrypted, &key).await?;
        assert_eq!(data, decrypted.as_slice());
    }

    info!(
        "Random nonces: Generated {} unique nonces out of 1000 encryptions",
        nonces.len()
    );

    Ok(())
}

/// Demonstrate counter-based nonce generation for guaranteed uniqueness
async fn demonstrate_counter_based_nonces() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Counter-Based Nonce Strategy Demo ===");

    // Configure counter-based nonce generation
    let config = NonceConfig {
        default_strategy: NonceStrategy::CounterBased,
        rotation_threshold: 1_000_000, // 1 million encryptions
        warning_threshold: 750_000,    // Warning at 75%
        persist_counters: true,
    };

    let nonce_manager = NonceManager::with_config(config)?;
    let encryption = AesGcmEncryption::with_nonce_manager(nonce_manager)?;
    let key = encryption.generate_key().await?;
    let data = b"High-volume financial data processing";

    let mut nonces = HashSet::new();

    // Perform 10,000 encryptions with counter-based nonces
    for i in 0..10_000 {
        let encrypted = encryption.encrypt(data, &key).await?;

        // Counter-based nonces guarantee uniqueness
        assert!(
            nonces.insert(encrypted.nonce.clone()),
            "Duplicate nonce detected at iteration {i}"
        );

        // Verify decryption works
        let decrypted = encryption.decrypt(&encrypted, &key).await?;
        assert_eq!(data, decrypted.as_slice());

        if i % 1000 == 0 {
            info!("Processed {} encryptions with unique nonces", i + 1);
        }
    }

    info!(
        "Counter-based nonces: Successfully generated {} unique nonces",
        nonces.len()
    );

    Ok(())
}

/// Demonstrate key rotation enforcement
async fn demonstrate_key_rotation() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Key Rotation Enforcement Demo ===");

    // Configure with very low threshold for demonstration
    let config = NonceConfig {
        default_strategy: NonceStrategy::CounterBased,
        rotation_threshold: 5, // Rotate after 5 encryptions
        warning_threshold: 3,  // Warning after 3 encryptions
        persist_counters: false,
    };

    let nonce_manager = NonceManager::with_config(config)?;
    let encryption = AesGcmEncryption::with_nonce_manager(nonce_manager)?;
    let key = encryption.generate_key().await?;
    let data = b"Test data for rotation demo";

    // Perform encryptions until rotation threshold
    for i in 0..5 {
        match encryption.encrypt(data, &key).await {
            Ok(encrypted) => {
                info!("Encryption {} succeeded", i + 1);
                let decrypted = encryption.decrypt(&encrypted, &key).await?;
                assert_eq!(data, decrypted.as_slice());
            }
            Err(e) => {
                warn!("Encryption {} failed: {}", i + 1, e);
                break;
            }
        }
    }

    // Next encryption should fail due to rotation threshold
    match encryption.encrypt(data, &key).await {
        Ok(_) => panic!("Expected encryption to fail after rotation threshold"),
        Err(e) => {
            info!("Key rotation enforcement working: {}", e);
            assert!(e.to_string().contains("rotation threshold"));
        }
    }

    Ok(())
}

/// Demonstrate hybrid strategy with fallback
async fn demonstrate_hybrid_strategy() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Hybrid Strategy Demo ===");

    let config = NonceConfig {
        default_strategy: NonceStrategy::Hybrid, // Try counter-based, fallback to random
        rotation_threshold: 1_000_000,
        warning_threshold: 750_000,
        persist_counters: true,
    };

    let nonce_manager = NonceManager::with_config(config)?;
    let encryption = AesGcmEncryption::with_nonce_manager(nonce_manager)?;
    let key = encryption.generate_key().await?;
    let data = b"Hybrid strategy test data";

    let mut nonces = HashSet::new();

    // Perform encryptions with hybrid strategy
    for i in 0..1000 {
        let encrypted = encryption.encrypt(data, &key).await?;

        // Should still get unique nonces (counter-based should work)
        assert!(
            nonces.insert(encrypted.nonce.clone()),
            "Duplicate nonce detected at iteration {i}"
        );

        let decrypted = encryption.decrypt(&encrypted, &key).await?;
        assert_eq!(data, decrypted.as_slice());
    }

    info!(
        "Hybrid strategy: Successfully generated {} unique nonces",
        nonces.len()
    );

    Ok(())
}

/// Demonstrate nonce format analysis
#[allow(dead_code)]
async fn analyze_nonce_formats() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Nonce Format Analysis ===");

    // Counter-based nonces
    let config = NonceConfig {
        default_strategy: NonceStrategy::CounterBased,
        rotation_threshold: 1000,
        warning_threshold: 800,
        persist_counters: false,
    };

    let nonce_manager = NonceManager::with_config(config)?;
    let encryption = AesGcmEncryption::with_nonce_manager(nonce_manager)?;
    let key = encryption.generate_key().await?;
    let data = b"Nonce format analysis";

    for i in 0..5 {
        let encrypted = encryption.encrypt(data, &key).await?;
        let nonce = &encrypted.nonce;

        // Counter-based nonce format: 8-byte counter + 4-byte random
        assert_eq!(nonce.len(), 12, "Nonce should be 12 bytes");

        // Extract counter (first 8 bytes, big-endian)
        let counter_bytes: [u8; 8] = nonce[0..8].try_into().unwrap();
        let counter = u64::from_be_bytes(counter_bytes);

        info!(
            "Encryption {}: Counter = {}, Full nonce = {:02x?}",
            i + 1,
            counter,
            nonce
        );

        // Counter should increment
        assert_eq!(counter, i as u64, "Counter should match iteration");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nonce_uniqueness_guarantee() {
        let config = NonceConfig {
            default_strategy: NonceStrategy::CounterBased,
            rotation_threshold: 10_000,
            warning_threshold: 8_000,
            persist_counters: false,
        };

        let nonce_manager = NonceManager::with_config(config).unwrap();
        let encryption = AesGcmEncryption::with_nonce_manager(nonce_manager).unwrap();
        let key = encryption.generate_key().await.unwrap();
        let data = b"Test data";

        let mut nonces = HashSet::new();

        // Test with 5000 encryptions to ensure no collisions
        for _ in 0..5000 {
            let encrypted = encryption.encrypt(data, &key).await.unwrap();
            assert!(
                nonces.insert(encrypted.nonce.clone()),
                "Nonce collision detected!"
            );
        }

        assert_eq!(nonces.len(), 5000);
    }

    #[tokio::test]
    async fn test_concurrent_nonce_generation() {
        use std::sync::Arc;
        use tokio::task::JoinSet;

        let config = NonceConfig {
            default_strategy: NonceStrategy::CounterBased,
            rotation_threshold: 100_000,
            warning_threshold: 80_000,
            persist_counters: false,
        };

        let nonce_manager = NonceManager::with_config(config).unwrap();
        let encryption = Arc::new(AesGcmEncryption::with_nonce_manager(nonce_manager).unwrap());
        let key = Arc::new(encryption.generate_key().await.unwrap());
        let data = b"Concurrent test data";

        let nonces = Arc::new(std::sync::Mutex::new(HashSet::new()));
        let mut tasks = JoinSet::new();

        // Spawn 10 concurrent tasks, each performing 100 encryptions
        for _ in 0..10 {
            let encryption_clone = encryption.clone();
            let key_clone = key.clone();
            let nonces_clone = nonces.clone();

            tasks.spawn(async move {
                for _ in 0..100 {
                    let encrypted = encryption_clone.encrypt(data, &key_clone).await.unwrap();
                    let mut nonces_set = nonces_clone.lock().unwrap();
                    assert!(
                        nonces_set.insert(encrypted.nonce.clone()),
                        "Concurrent nonce collision detected!"
                    );
                }
            });
        }

        // Wait for all tasks to complete
        while let Some(result) = tasks.join_next().await {
            result.unwrap();
        }

        let final_nonces = nonces.lock().unwrap();
        assert_eq!(final_nonces.len(), 1000);
    }
}
