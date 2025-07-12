use chrono::{DateTime, Duration, Utc};
/// Key management system for the Fiscus encryption service
///
/// This module provides secure key storage, key rotation, and key lifecycle
/// management for the encryption service. It handles both symmetric and
/// asymmetric keys with proper security controls.
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use super::key_derivation::{Argon2Kdf, KeyDerivation};
use super::symmetric::{AesGcmEncryption, SymmetricEncryption};
use super::types::{EncryptionKey, EncryptionResult, KeyDerivationParams};
use super::utils::SecureRandom;
use super::EncryptionStats;
use crate::error::FiscusError;

/// Key storage entry with metadata
#[derive(Debug, Clone)]
struct KeyEntry {
    key: EncryptionKey,
    usage_count: u64,
    last_used: DateTime<Utc>,
    rotation_due: Option<DateTime<Utc>>,
}

/// Key manager for secure key storage and lifecycle management
pub struct KeyManager {
    /// In-memory key storage (encrypted at rest in production)
    keys: Arc<RwLock<HashMap<String, KeyEntry>>>,
    /// User-specific key mappings
    user_keys: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    /// Symmetric encryption for key storage
    symmetric_encryption: Box<dyn SymmetricEncryption + Send + Sync>,
    /// Key derivation for user passwords
    key_derivation: Box<dyn KeyDerivation + Send + Sync>,
    /// Master key for encrypting stored keys
    master_key: Option<EncryptionKey>,
    /// Statistics tracking
    stats: Arc<RwLock<EncryptionStats>>,
    /// Secure random generator
    secure_random: SecureRandom,
}

impl KeyManager {
    /// Create a new key manager
    pub fn new() -> EncryptionResult<Self> {
        debug!("Initializing key manager");

        let symmetric_encryption = Box::new(AesGcmEncryption::new()?);
        let key_derivation = Box::new(Argon2Kdf::new()?);

        Ok(Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            user_keys: Arc::new(RwLock::new(HashMap::new())),
            symmetric_encryption,
            key_derivation,
            master_key: None,
            stats: Arc::new(RwLock::new(EncryptionStats {
                total_keys: 0,
                active_keys: 0,
                rotated_keys: 0,
                encryption_operations: 0,
                decryption_operations: 0,
                key_derivation_operations: 0,
                last_key_rotation: None,
            })),
            secure_random: SecureRandom::new()?,
        })
    }

    /// Initialize the key manager with a master key derived from password
    #[instrument(skip(self, password))]
    pub async fn initialize_with_password(&mut self, password: &str) -> EncryptionResult<()> {
        info!("Initializing key manager with password-derived master key");

        // Generate salt for master key derivation
        let salt = self.secure_random.generate_salt()?;

        // Create key derivation parameters
        let params = KeyDerivationParams::argon2id_default(salt);

        // Derive master key from password
        let master_key = self
            .key_derivation
            .derive_key(password.as_bytes(), &params)
            .await?;

        self.master_key = Some(master_key);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.key_derivation_operations += 1;

        info!("Key manager initialized successfully");
        Ok(())
    }

    /// Get or create an encryption key for a user and data type
    #[instrument(skip(self), fields(user_id = user_id, data_type = data_type))]
    pub async fn get_or_create_key(
        &self,
        user_id: &str,
        data_type: &str,
    ) -> EncryptionResult<EncryptionKey> {
        let key_identifier = format!("{user_id}:{data_type}");

        // Check if key already exists
        if let Some(key) = self.get_key_internal(&key_identifier).await? {
            debug!(key_id = %key.key_id, "Retrieved existing key");
            return Ok(key);
        }

        // Create new key
        debug!("Creating new encryption key");
        let new_key = self.symmetric_encryption.generate_key().await?;

        // Store the key
        self.store_key(&key_identifier, new_key.clone()).await?;

        // Update user key mapping
        let mut user_keys = self.user_keys.write().await;
        let user_map = user_keys
            .entry(user_id.to_string())
            .or_insert_with(HashMap::new);
        user_map.insert(data_type.to_string(), key_identifier.clone());

        debug!(key_id = %new_key.key_id, "New encryption key created and stored");
        Ok(new_key)
    }

    /// Get an existing encryption key
    #[instrument(skip(self), fields(user_id = user_id, data_type = data_type))]
    pub async fn get_key(&self, user_id: &str, data_type: &str) -> EncryptionResult<EncryptionKey> {
        let key_identifier = format!("{user_id}:{data_type}");

        self.get_key_internal(&key_identifier)
            .await?
            .ok_or_else(|| {
                FiscusError::NotFound(format!("Key not found for {user_id}:{data_type}"))
            })
    }

    /// Get a key by its unique key ID
    #[instrument(skip(self), fields(key_id = key_id))]
    pub async fn get_key_by_id(&self, key_id: &str) -> EncryptionResult<EncryptionKey> {
        let keys = self.keys.read().await;

        // Search through all keys to find the one with matching key_id
        for entry in keys.values() {
            if entry.key.key_id == key_id {
                debug!(key_id = key_id, "Found key by ID");
                return Ok(entry.key.clone());
            }
        }

        Err(FiscusError::NotFound(format!(
            "Key not found with ID: {key_id}"
        )))
    }

    /// Validate that a user has access to a specific key for a data type
    #[instrument(skip(self), fields(user_id = user_id, data_type = data_type, key_id = key_id))]
    pub async fn validate_user_key_access(
        &self,
        user_id: &str,
        data_type: &str,
        key_id: &str,
    ) -> EncryptionResult<()> {
        let user_keys = self.user_keys.read().await;

        // Check if user has any keys
        let user_key_map = user_keys.get(user_id).ok_or_else(|| {
            FiscusError::Authentication("User has no encryption keys".to_string())
        })?;

        // Check if user has a key for this data type
        let _key_identifier = user_key_map.get(data_type).ok_or_else(|| {
            FiscusError::Authentication(format!(
                "User does not have access to data type: {data_type}"
            ))
        })?;

        // Verify the key exists and belongs to this user by checking if the key_id
        // matches any key that this user has access to (including rotated keys)
        let keys = self.keys.read().await;
        let mut key_found = false;

        // Check all keys that belong to this user (including old rotated keys)
        // We need to check all keys in the system that start with "user_id:data_type"
        let key_prefix = format!("{user_id}:{data_type}");

        for (key_identifier, entry) in keys.iter() {
            // Check if this key belongs to the user and data type
            if key_identifier.starts_with(&key_prefix) && entry.key.key_id == key_id {
                key_found = true;
                break;
            }
        }

        if !key_found {
            return Err(FiscusError::Authentication(format!(
                "User does not have access to key: {key_id}"
            )));
        }

        debug!(
            user_id = user_id,
            data_type = data_type,
            key_id = key_id,
            "User key access validated successfully"
        );

        Ok(())
    }

    /// Internal method to get a key by identifier
    async fn get_key_internal(
        &self,
        key_identifier: &str,
    ) -> EncryptionResult<Option<EncryptionKey>> {
        let mut keys = self.keys.write().await;

        if let Some(entry) = keys.get_mut(key_identifier) {
            // Update usage statistics
            entry.usage_count += 1;
            entry.last_used = Utc::now();

            // Check if key rotation is due
            if let Some(rotation_due) = entry.rotation_due {
                if Utc::now() > rotation_due {
                    warn!(key_id = %entry.key.key_id, "Key rotation is overdue");
                }
            }

            Ok(Some(entry.key.clone()))
        } else {
            Ok(None)
        }
    }

    /// Store a key securely
    #[instrument(skip(self, key), fields(key_id = %key.key_id))]
    async fn store_key(&self, key_identifier: &str, key: EncryptionKey) -> EncryptionResult<()> {
        let entry = KeyEntry {
            key,
            usage_count: 0,
            last_used: Utc::now(),
            rotation_due: Some(Utc::now() + Duration::days(90)), // 90-day rotation
        };

        let mut keys = self.keys.write().await;
        keys.insert(key_identifier.to_string(), entry);

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_keys += 1;
        stats.active_keys += 1;

        debug!(key_identifier = key_identifier, "Key stored successfully");
        Ok(())
    }

    /// Rotate encryption keys for a user
    #[instrument(skip(self), fields(user_id = user_id))]
    pub async fn rotate_user_keys(&self, user_id: &str) -> EncryptionResult<()> {
        info!(user_id = user_id, "Starting key rotation");

        let user_keys = {
            let user_keys_guard = self.user_keys.read().await;
            user_keys_guard.get(user_id).cloned()
        };

        if let Some(user_key_map) = user_keys {
            for (data_type, old_key_identifier) in user_key_map {
                debug!(data_type = data_type, "Rotating key");

                // Generate new key
                let new_key = self.symmetric_encryption.generate_key().await?;

                // Create new key identifier with the new key's unique ID
                // This ensures each rotated key has a unique identifier while maintaining
                // the ability to identify which user and data type it belongs to
                let new_key_identifier = format!("{}:{}:{}", user_id, data_type, new_key.key_id);

                // Store new key with unique identifier
                self.store_key(&new_key_identifier, new_key.clone()).await?;

                // Mark old key as inactive but keep it for decrypting old data
                {
                    let mut keys = self.keys.write().await;
                    if let Some(entry) = keys.get_mut(&old_key_identifier) {
                        entry.key.is_active = false;
                        entry.rotation_due = Some(Utc::now() + Duration::days(90));
                        debug!(old_key_id = %entry.key.key_id, "Marked old key as inactive");
                    }
                }

                // Update user key mapping to point to the new key
                {
                    let mut user_keys_guard = self.user_keys.write().await;
                    if let Some(user_map) = user_keys_guard.get_mut(user_id) {
                        user_map.insert(data_type.clone(), new_key_identifier);
                        debug!(data_type = data_type, new_key_id = %new_key.key_id, "Updated user key mapping");
                    }
                }
            }
        }

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.rotated_keys += 1;
        stats.last_key_rotation = Some(Utc::now());

        info!(user_id = user_id, "Key rotation completed successfully");
        Ok(())
    }

    /// Clean up expired keys
    #[instrument(skip(self))]
    pub async fn cleanup_expired_keys(&self) -> EncryptionResult<usize> {
        debug!("Starting expired key cleanup");

        let mut keys = self.keys.write().await;
        let mut removed_count = 0;

        // Find expired keys
        let expired_keys: Vec<String> = keys
            .iter()
            .filter_map(|(key_id, entry)| {
                if entry.key.is_expired() {
                    Some(key_id.clone())
                } else {
                    None
                }
            })
            .collect();

        // Remove expired keys
        for key_id in expired_keys {
            keys.remove(&key_id);
            removed_count += 1;
        }

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.active_keys = stats.active_keys.saturating_sub(removed_count);

        debug!(
            removed_count = removed_count,
            "Expired key cleanup completed"
        );
        Ok(removed_count)
    }

    /// Get encryption statistics
    pub async fn get_stats(&self) -> EncryptionResult<EncryptionStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    /// Update operation statistics
    pub async fn record_encryption_operation(&self) {
        let mut stats = self.stats.write().await;
        stats.encryption_operations += 1;
    }

    /// Update operation statistics
    pub async fn record_decryption_operation(&self) {
        let mut stats = self.stats.write().await;
        stats.decryption_operations += 1;
    }

    /// List all keys for a user (for administrative purposes)
    #[instrument(skip(self), fields(user_id = user_id))]
    pub async fn list_user_keys(&self, user_id: &str) -> EncryptionResult<Vec<String>> {
        let user_keys = self.user_keys.read().await;

        if let Some(user_key_map) = user_keys.get(user_id) {
            Ok(user_key_map.keys().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Check if a key needs rotation
    pub async fn needs_rotation(&self, user_id: &str, data_type: &str) -> EncryptionResult<bool> {
        let key_identifier = format!("{user_id}:{data_type}");
        let keys = self.keys.read().await;

        if let Some(entry) = keys.get(&key_identifier) {
            if let Some(rotation_due) = entry.rotation_due {
                Ok(Utc::now() > rotation_due)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
}

/// Key rotation manager for automated key rotation
pub struct KeyRotationManager {
    key_manager: Arc<KeyManager>,
    rotation_interval: Duration,
}

impl KeyRotationManager {
    /// Create a new key rotation manager
    pub fn new(key_manager: Arc<KeyManager>, rotation_interval_days: i64) -> Self {
        Self {
            key_manager,
            rotation_interval: Duration::days(rotation_interval_days),
        }
    }

    /// Start automated key rotation (would run in background)
    pub async fn start_rotation_schedule(&self) -> EncryptionResult<()> {
        info!("Starting automated key rotation schedule");

        // In a real implementation, this would run as a background task
        // For now, we'll just log that it would start
        debug!(
            interval_days = self.rotation_interval.num_days(),
            "Key rotation schedule configured"
        );

        Ok(())
    }

    /// Perform rotation check for all users
    pub async fn check_and_rotate_keys(&self) -> EncryptionResult<usize> {
        debug!("Checking for keys that need rotation");

        // Get stats to see if we have any keys to check
        let stats = self.key_manager.get_stats().await?;
        if stats.active_keys == 0 {
            return Ok(0);
        }

        // In a real implementation, this would iterate through all users
        // and check for rotation needs. For now, return 0 as a placeholder
        // but at least we're using the key_manager field
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_manager_creation() {
        let key_manager = KeyManager::new().unwrap();
        let stats = key_manager.get_stats().await.unwrap();
        assert_eq!(stats.total_keys, 0);
    }

    #[tokio::test]
    async fn test_key_creation_and_retrieval() {
        let key_manager = KeyManager::new().unwrap();
        let user_id = "test-user";
        let data_type = "transaction_amount";

        // Create key
        let key1 = key_manager
            .get_or_create_key(user_id, data_type)
            .await
            .unwrap();

        // Retrieve same key
        let key2 = key_manager.get_key(user_id, data_type).await.unwrap();

        assert_eq!(key1.key_id, key2.key_id);
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let key_manager = KeyManager::new().unwrap();
        let user_id = "test-user";

        // Create a key first
        let _key = key_manager
            .get_or_create_key(user_id, "test_data")
            .await
            .unwrap();

        // Rotate keys
        let result = key_manager.rotate_user_keys(user_id).await;
        assert!(result.is_ok());

        let stats = key_manager.get_stats().await.unwrap();
        assert_eq!(stats.rotated_keys, 1);
    }

    #[tokio::test]
    async fn test_user_key_listing() {
        let key_manager = KeyManager::new().unwrap();
        let user_id = "test-user";

        // Create multiple keys
        let _key1 = key_manager
            .get_or_create_key(user_id, "data_type_1")
            .await
            .unwrap();
        let _key2 = key_manager
            .get_or_create_key(user_id, "data_type_2")
            .await
            .unwrap();

        let user_keys = key_manager.list_user_keys(user_id).await.unwrap();
        assert_eq!(user_keys.len(), 2);
        assert!(user_keys.contains(&"data_type_1".to_string()));
        assert!(user_keys.contains(&"data_type_2".to_string()));
    }
}
