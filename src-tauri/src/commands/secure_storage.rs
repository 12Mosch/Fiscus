use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    dto::{
        SecureDeleteRequest, SecureDeleteResponse, SecureRetrieveRequest, SecureRetrieveResponse,
        SecureStoreRequest, SecureStoreResponse,
    },
    error::{FiscusError, FiscusResult, SecurityValidator, Validator},
};

/// In-memory secure storage for encrypted data
/// In production, this should be replaced with a proper database or secure file storage
static SECURE_STORAGE: Mutex<Option<Arc<Mutex<HashMap<String, StoredData>>>>> = Mutex::new(None);

#[derive(Debug, Clone)]
struct StoredData {
    encrypted_data: String,
    nonce: String,
    algorithm: crate::encryption::EncryptionAlgorithm,
    key_id: String,
    stored_at: chrono::DateTime<Utc>,
}

/// Initialize the secure storage
fn get_secure_storage() -> Arc<Mutex<HashMap<String, StoredData>>> {
    let mut storage_guard = SECURE_STORAGE.lock().unwrap();
    if storage_guard.is_none() {
        *storage_guard = Some(Arc::new(Mutex::new(HashMap::new())));
    }
    storage_guard.as_ref().unwrap().clone()
}

/// Generate a storage key for the given user and data type
fn generate_storage_key(user_id: &str, data_type: &str) -> String {
    format!("secure_{}_{}", data_type, user_id)
}

/// Store encrypted data securely
#[tauri::command]
#[instrument(skip(request), fields(user_id = %request.user_id, data_type = %request.data_type))]
pub async fn secure_store(request: SecureStoreRequest) -> FiscusResult<SecureStoreResponse> {
    // Validate input
    Validator::validate_uuid(&request.user_id, "user_id")?;
    Validator::validate_string(&request.data_type, "data_type", 1, 100)?;
    
    if request.encrypted_data.is_empty() {
        return Err(FiscusError::InvalidInput(
            "Encrypted data cannot be empty".to_string(),
        ));
    }
    
    if request.nonce.is_empty() {
        return Err(FiscusError::InvalidInput(
            "Nonce cannot be empty".to_string(),
        ));
    }
    
    if request.key_id.is_empty() {
        return Err(FiscusError::InvalidInput(
            "Key ID cannot be empty".to_string(),
        ));
    }

    let storage_key = generate_storage_key(&request.user_id, &request.data_type);
    let stored_at = Utc::now();

    let stored_data = StoredData {
        encrypted_data: request.encrypted_data,
        nonce: request.nonce,
        algorithm: request.algorithm,
        key_id: request.key_id,
        stored_at,
    };

    // Store the data
    let storage = get_secure_storage();
    {
        let mut storage_map = storage.lock().unwrap();
        storage_map.insert(storage_key.clone(), stored_data);
    }

    info!(
        user_id = %request.user_id,
        data_type = %request.data_type,
        storage_key = %storage_key,
        "Data stored securely"
    );

    Ok(SecureStoreResponse {
        stored: true,
        storage_key,
        stored_at,
    })
}

/// Retrieve encrypted data securely
#[tauri::command]
#[instrument(skip(request), fields(user_id = %request.user_id, data_type = %request.data_type))]
pub async fn secure_retrieve(
    request: SecureRetrieveRequest,
) -> FiscusResult<SecureRetrieveResponse> {
    // Validate input
    Validator::validate_uuid(&request.user_id, "user_id")?;
    Validator::validate_string(&request.data_type, "data_type", 1, 100)?;

    let storage_key = generate_storage_key(&request.user_id, &request.data_type);

    // Retrieve the data
    let storage = get_secure_storage();
    let stored_data = {
        let storage_map = storage.lock().unwrap();
        storage_map.get(&storage_key).cloned()
    };

    match stored_data {
        Some(data) => {
            debug!(
                user_id = %request.user_id,
                data_type = %request.data_type,
                storage_key = %storage_key,
                "Data retrieved securely"
            );

            Ok(SecureRetrieveResponse {
                encrypted_data: data.encrypted_data,
                nonce: data.nonce,
                algorithm: data.algorithm,
                key_id: data.key_id,
                stored_at: data.stored_at,
            })
        }
        None => {
            warn!(
                user_id = %request.user_id,
                data_type = %request.data_type,
                storage_key = %storage_key,
                "No data found for retrieval"
            );
            
            Err(FiscusError::NotFound(format!(
                "No secure data found for user {} and data type {}",
                request.user_id, request.data_type
            )))
        }
    }
}

/// Delete encrypted data securely
#[tauri::command]
#[instrument(skip(request), fields(user_id = %request.user_id, data_type = %request.data_type))]
pub async fn secure_delete(request: SecureDeleteRequest) -> FiscusResult<SecureDeleteResponse> {
    // Validate input
    Validator::validate_uuid(&request.user_id, "user_id")?;
    Validator::validate_string(&request.data_type, "data_type", 1, 100)?;

    let storage_key = generate_storage_key(&request.user_id, &request.data_type);
    let deleted_at = Utc::now();

    // Delete the data
    let storage = get_secure_storage();
    let was_present = {
        let mut storage_map = storage.lock().unwrap();
        storage_map.remove(&storage_key).is_some()
    };

    if was_present {
        info!(
            user_id = %request.user_id,
            data_type = %request.data_type,
            storage_key = %storage_key,
            "Data deleted securely"
        );
    } else {
        warn!(
            user_id = %request.user_id,
            data_type = %request.data_type,
            storage_key = %storage_key,
            "No data found to delete"
        );
    }

    Ok(SecureDeleteResponse {
        deleted: was_present,
        deleted_at,
    })
}
