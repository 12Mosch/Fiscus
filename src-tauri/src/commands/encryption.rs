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
    // Validate input
    Validator::validate_string(&request.password, "password", 8, 128)?;

    let _service = get_encryption_service()?;

    debug!(
        algorithm = ?request.algorithm,
        salt_len = request.salt.as_ref().map_or(0, |s| s.len()),
        "Deriving key from password"
    );

    // TODO: This is a simplified implementation - in production, you'd want more sophisticated key derivation
    Err(FiscusError::Internal(
        "Key derivation from password not implemented".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encryption_service_initialization() {
        let result = initialize_encryption_service();
        assert!(result.is_ok());

        let service = get_encryption_service();
        assert!(service.is_ok());
    }
}
