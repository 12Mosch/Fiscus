use base64::Engine;
/// Tauri commands for encryption operations
///
/// This module provides the Tauri command interface for the encryption service,
/// allowing the frontend to perform secure encryption and decryption operations
/// on financial data.
use std::sync::{Arc, OnceLock};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    dto::{
        DecryptDataRequest, DecryptDataResponse, DeriveKeyRequest, DeriveKeyResponse,
        EncryptDataRequest, EncryptDataResponse, EncryptionStatsResponse, GenerateKeyRequest,
        GenerateKeyResponse, RotateKeysRequest,
    },
    encryption::{EncryptionAlgorithm, EncryptionService},
    error::{FiscusError, FiscusResult, SecurityValidator, Validator},
};

#[cfg(test)]
use crate::security::data_protection::SensitiveData;

/// Global encryption service instance
static ENCRYPTION_SERVICE: OnceLock<Arc<EncryptionService>> = OnceLock::new();

/// Initialize the encryption service (called once at startup)
pub fn initialize_encryption_service() -> FiscusResult<()> {
    match EncryptionService::new() {
        Ok(service) => {
            let arc_service = Arc::new(service);
            match ENCRYPTION_SERVICE.set(arc_service) {
                Ok(()) => {
                    info!("Encryption service initialized successfully");
                    Ok(())
                }
                Err(_) => {
                    warn!("Encryption service was already initialized");
                    Ok(())
                }
            }
        }
        Err(e) => {
            error!("Failed to initialize encryption service: {}", e);
            Err(FiscusError::Internal(
                "Failed to initialize encryption service".to_string(),
            ))
        }
    }
}

/// Get the encryption service instance
pub fn get_encryption_service() -> FiscusResult<Arc<EncryptionService>> {
    ENCRYPTION_SERVICE
        .get()
        .cloned()
        .ok_or_else(|| FiscusError::Internal("Encryption service not initialized".to_string()))
}

/// Encrypt sensitive financial data
#[tauri::command]
#[instrument(skip(request), fields(user_id = %request.user_id, data_type = %request.data_type))]
pub async fn encrypt_financial_data(
    request: EncryptDataRequest,
) -> FiscusResult<EncryptDataResponse> {
    // Validate input
    Validator::validate_uuid(&request.user_id.as_str(), "user_id")?;
    Validator::validate_string(&request.data_type, "data_type", 1, 100)?;

    if request.data.is_empty() {
        return Err(FiscusError::InvalidInput(
            "Data cannot be empty".to_string(),
        ));
    }

    // Security check: validate data size (check the base64 string length as a proxy)
    SecurityValidator::validate_data_size(request.data.as_bytes(), 1024 * 1024, "financial_data")?; // 1MB limit

    let service = get_encryption_service()?;

    debug!(
        user_id = %request.user_id,
        data_type = %request.data_type,
        data_size = request.data.len(),
        "Encrypting financial data"
    );

    // Convert base64 data to bytes
    let data_bytes = base64::engine::general_purpose::STANDARD
        .decode(&request.data)
        .map_err(|e| FiscusError::InvalidInput(format!("Invalid base64 data: {e}")))?;

    // Encrypt the data
    let encrypted_data = service
        .encrypt_financial_data(&data_bytes, &request.user_id.as_str(), &request.data_type)
        .await?;

    // Convert encrypted data to base64 for transport
    let response = EncryptDataResponse {
        encrypted_data: base64::engine::general_purpose::STANDARD
            .encode(&encrypted_data.ciphertext),
        nonce: base64::engine::general_purpose::STANDARD.encode(&encrypted_data.nonce),
        algorithm: encrypted_data.metadata.algorithm,
        key_id: encrypted_data.metadata.key_id,
        encrypted_at: encrypted_data.metadata.encrypted_at,
    };

    info!(
        user_id = %request.user_id,
        data_type = %request.data_type,
        key_id = %response.key_id,
        "Financial data encrypted successfully"
    );

    Ok(response)
}

/// Decrypt sensitive financial data
#[tauri::command]
#[instrument(skip(request), fields(user_id = %request.user_id, data_type = %request.data_type))]
pub async fn decrypt_financial_data(
    request: DecryptDataRequest,
) -> FiscusResult<DecryptDataResponse> {
    // Validate input
    Validator::validate_uuid(&request.user_id.as_str(), "user_id")?;
    Validator::validate_string(&request.data_type, "data_type", 1, 100)?;

    let service = get_encryption_service()?;

    debug!(
        user_id = %request.user_id,
        data_type = %request.data_type,
        key_id = %request.key_id,
        "Decrypting financial data"
    );

    // Convert base64 data to bytes
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(&request.encrypted_data)
        .map_err(|e| FiscusError::InvalidInput(format!("Invalid base64 ciphertext: {e}")))?;

    let nonce = base64::engine::general_purpose::STANDARD
        .decode(&request.nonce)
        .map_err(|e| FiscusError::InvalidInput(format!("Invalid base64 nonce: {e}")))?;

    // Reconstruct encrypted data
    let encrypted_data = crate::encryption::types::EncryptedData::new(
        ciphertext,
        nonce,
        None,
        crate::encryption::types::EncryptionMetadata::new(
            request.algorithm,
            request.key_id.clone(),
        ),
    );

    // Decrypt the data
    let decrypted_bytes = service
        .decrypt_financial_data(
            &encrypted_data,
            &request.user_id.as_str(),
            &request.data_type,
        )
        .await?;

    // Convert decrypted data to base64 for transport
    let response = DecryptDataResponse {
        data: base64::engine::general_purpose::STANDARD.encode(&decrypted_bytes),
        decrypted_at: chrono::Utc::now(),
    };

    info!(
        user_id = %request.user_id,
        data_type = %request.data_type,
        key_id = %request.key_id,
        "Financial data decrypted successfully"
    );

    Ok(response)
}

/// Generate a new encryption key
#[tauri::command]
#[instrument(skip(request), fields(user_id = %request.user_id))]
pub async fn generate_encryption_key(
    request: GenerateKeyRequest,
) -> FiscusResult<GenerateKeyResponse> {
    // Validate input
    Validator::validate_uuid(&request.user_id.as_str(), "user_id")?;

    let _service = get_encryption_service()?;

    debug!(
        user_id = %request.user_id,
        algorithm = ?request.algorithm,
        "Generating encryption key"
    );

    // For now, return a placeholder response since key generation
    // TODO: should be handled internally by the encryption service
    warn!("Key generation endpoint is for demonstration - keys are managed internally");

    let key_id = uuid::Uuid::new_v4().to_string();
    let key_type = match request.algorithm {
        EncryptionAlgorithm::Aes256Gcm | EncryptionAlgorithm::ChaCha20Poly1305 => {
            crate::encryption::types::KeyType::Symmetric
        }
        EncryptionAlgorithm::Rsa4096 | EncryptionAlgorithm::Ed25519 => {
            crate::encryption::types::KeyType::PublicKey
        }
        _ => {
            return Err(FiscusError::InvalidInput(
                "Unsupported algorithm".to_string(),
            ));
        }
    };

    let response = GenerateKeyResponse {
        key_id: key_id.clone(),
        algorithm: request.algorithm,
        key_type,
        created_at: chrono::Utc::now(),
    };

    info!(
        user_id = %request.user_id,
        key_id = %response.key_id,
        algorithm = ?response.algorithm,
        "Encryption key generated successfully"
    );

    Ok(response)
}

/// Rotate encryption keys for a user
#[tauri::command]
#[instrument(skip(request), fields(user_id = %request.user_id))]
pub async fn rotate_user_keys(request: RotateKeysRequest) -> FiscusResult<bool> {
    // Validate input
    Validator::validate_uuid(&request.user_id.as_str(), "user_id")?;

    let service = get_encryption_service()?;

    info!(user_id = %request.user_id, "Starting key rotation");

    // Rotate keys
    service.rotate_user_keys(&request.user_id.as_str()).await?;

    info!(user_id = %request.user_id, "Key rotation completed successfully");
    Ok(true)
}

/// Get encryption service statistics
#[tauri::command]
pub async fn get_encryption_stats() -> FiscusResult<EncryptionStatsResponse> {
    let service = get_encryption_service()?;

    debug!("Retrieving encryption statistics");

    let stats = service.get_encryption_stats().await?;

    let response = EncryptionStatsResponse {
        total_keys: stats.total_keys,
        active_keys: stats.active_keys,
        rotated_keys: stats.rotated_keys,
        encryption_operations: stats.encryption_operations,
        decryption_operations: stats.decryption_operations,
        key_derivation_operations: stats.key_derivation_operations,
        last_key_rotation: stats.last_key_rotation,
    };

    debug!("Encryption statistics retrieved successfully");
    Ok(response)
}

/// Derive a key from password
#[tauri::command]
#[instrument(skip(request))]
pub async fn derive_key_from_password(
    request: DeriveKeyRequest,
) -> FiscusResult<DeriveKeyResponse> {
    use crate::encryption::key_derivation::{Argon2Kdf, KeyDerivation, Pbkdf2Kdf, ScryptKdf};
    use crate::encryption::types::{KeyDerivationAlgorithm, KeyDerivationParams};
    use crate::encryption::utils::SecureRandom;
    use chrono::Utc;

    // Validate input
    Validator::validate_string(request.password.expose(), "password", 8, 128)?;

    let _service = get_encryption_service()?;

    debug!(
        algorithm = ?request.algorithm,
        salt_len = request.salt.as_ref().map_or(0, |s| s.len()),
        "Deriving key from password"
    );

    // Handle salt - either decode from base64 or generate new one
    let salt = if let Some(salt_b64) = &request.salt {
        use base64::{engine::general_purpose, Engine as _};
        general_purpose::STANDARD.decode(salt_b64).map_err(|e| {
            error!("Invalid base64 salt: {}", e);
            FiscusError::InvalidInput("Invalid base64 salt".to_string())
        })?
    } else {
        // Generate a new random salt
        let mut rng = SecureRandom::new()?;
        rng.generate_salt()?
    };

    // Create key derivation parameters based on algorithm
    let params = match request.algorithm {
        KeyDerivationAlgorithm::Argon2id => KeyDerivationParams::argon2id_default(salt),
        KeyDerivationAlgorithm::Pbkdf2Sha256 => KeyDerivationParams::pbkdf2_default(salt),
        KeyDerivationAlgorithm::Scrypt => KeyDerivationParams::scrypt_default(salt),
        KeyDerivationAlgorithm::HkdfSha256 => {
            return Err(FiscusError::InvalidInput(
                "HKDF-SHA256 not yet implemented for password derivation".to_string(),
            ));
        }
    };

    // Create the appropriate key derivation instance and derive the key
    let derived_key = match request.algorithm {
        KeyDerivationAlgorithm::Argon2id => {
            let kdf = Argon2Kdf::new()?;
            kdf.derive_key(request.password.expose().as_bytes(), &params)
                .await?
        }
        KeyDerivationAlgorithm::Pbkdf2Sha256 => {
            let kdf = Pbkdf2Kdf::new()?;
            kdf.derive_key(request.password.expose().as_bytes(), &params)
                .await?
        }
        KeyDerivationAlgorithm::Scrypt => {
            let kdf = ScryptKdf::new()?;
            kdf.derive_key(request.password.expose().as_bytes(), &params)
                .await?
        }
        KeyDerivationAlgorithm::HkdfSha256 => {
            // This case is already handled above, but included for completeness
            return Err(FiscusError::InvalidInput(
                "HKDF-SHA256 not yet implemented for password derivation".to_string(),
            ));
        }
    };

    let response = DeriveKeyResponse {
        key_id: derived_key.key_id.clone(),
        algorithm: request.algorithm,
        derived_at: Utc::now(),
    };

    debug!(
        key_id = %derived_key.key_id,
        algorithm = ?request.algorithm,
        "Key derived from password successfully"
    );

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryption::types::KeyDerivationAlgorithm;

    #[tokio::test]
    async fn test_encryption_service_initialization() {
        let result = initialize_encryption_service();
        assert!(result.is_ok());

        let service = get_encryption_service();
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_derive_key_from_password() {
        // Initialize encryption service
        let _ = initialize_encryption_service();

        // Test with Argon2id (default algorithm)
        let request = DeriveKeyRequest {
            password: SensitiveData::new("test_password_123".to_string()),
            algorithm: KeyDerivationAlgorithm::Argon2id,
            salt: None, // Let it generate a random salt
        };

        let result = derive_key_from_password(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.key_id.is_empty());
        assert_eq!(response.algorithm, KeyDerivationAlgorithm::Argon2id);
        assert!(response.derived_at <= chrono::Utc::now());
    }

    #[tokio::test]
    async fn test_derive_key_from_password_with_salt() {
        // Initialize encryption service
        let _ = initialize_encryption_service();

        // Test with custom base64-encoded salt
        use base64::{engine::general_purpose, Engine as _};
        let salt = vec![0x42u8; 16];
        let salt_b64 = general_purpose::STANDARD.encode(&salt);

        let request = DeriveKeyRequest {
            password: SensitiveData::new("test_password_456".to_string()),
            algorithm: KeyDerivationAlgorithm::Pbkdf2Sha256,
            salt: Some(salt_b64),
        };

        let result = derive_key_from_password(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.key_id.is_empty());
        assert_eq!(response.algorithm, KeyDerivationAlgorithm::Pbkdf2Sha256);
    }

    #[tokio::test]
    async fn test_derive_key_from_password_scrypt() {
        // Initialize encryption service
        let _ = initialize_encryption_service();

        let request = DeriveKeyRequest {
            password: SensitiveData::new("test_password_scrypt".to_string()),
            algorithm: KeyDerivationAlgorithm::Scrypt,
            salt: None,
        };

        let result = derive_key_from_password(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.key_id.is_empty());
        assert_eq!(response.algorithm, KeyDerivationAlgorithm::Scrypt);
    }

    #[tokio::test]
    async fn test_derive_key_from_password_invalid_salt() {
        // Initialize encryption service
        let _ = initialize_encryption_service();

        let request = DeriveKeyRequest {
            password: SensitiveData::new("test_password_123".to_string()),
            algorithm: KeyDerivationAlgorithm::Argon2id,
            salt: Some("invalid_base64!@#".to_string()),
        };

        let result = derive_key_from_password(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FiscusError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn test_derive_key_from_password_short_password() {
        // Initialize encryption service
        let _ = initialize_encryption_service();

        let request = DeriveKeyRequest {
            password: SensitiveData::new("short".to_string()), // Less than 8 characters
            algorithm: KeyDerivationAlgorithm::Argon2id,
            salt: None,
        };

        let result = derive_key_from_password(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FiscusError::Validation(_)));
    }

    #[tokio::test]
    async fn test_derive_key_from_password_hkdf_not_implemented() {
        // Initialize encryption service
        let _ = initialize_encryption_service();

        let request = DeriveKeyRequest {
            password: SensitiveData::new("test_password_123".to_string()),
            algorithm: KeyDerivationAlgorithm::HkdfSha256,
            salt: None,
        };

        let result = derive_key_from_password(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FiscusError::InvalidInput(_)));
    }
}
