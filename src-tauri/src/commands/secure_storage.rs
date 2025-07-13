use chrono::Utc;
use tracing::{info, instrument};

use crate::{
    database::secure_storage_repository::SecureStorageRepository,
    dto::{
        SecureDeleteRequest, SecureDeleteResponse, SecureRetrieveRequest, SecureRetrieveResponse,
        SecureStoreRequest, SecureStoreResponse,
    },
    error::{FiscusError, FiscusResult},
    services::get_secure_storage_service,
};

/// Get database connection for secure storage operations
/// In production, this should use a proper connection pool
fn get_database() -> String {
    // TODO: Replace with actual database connection
    // For now, using the default database name from tauri.conf.json
    "sqlite:fiscus.db".to_string()
}

/// Store encrypted data securely
#[tauri::command]
#[instrument(skip(request), fields(user_id = %request.user_id, data_type = %request.data_type))]
pub async fn secure_store(request: SecureStoreRequest) -> FiscusResult<SecureStoreResponse> {
    // Create repository instance
    let db = get_database();
    let repository = SecureStorageRepository::new(db);

    // Store the data using the repository
    let record = repository
        .store(
            &request.user_id,
            &request.data_type,
            &request.encrypted_data,
            &request.nonce,
            request.algorithm,
            &request.key_id,
            None, // No expiration by default
        )
        .await?;

    Ok(SecureStoreResponse {
        stored: true,
        storage_key: record.storage_key,
        stored_at: record.stored_at,
    })
}

/// Retrieve encrypted data securely
#[tauri::command]
#[instrument(skip(request), fields(user_id = %request.user_id, data_type = %request.data_type))]
pub async fn secure_retrieve(
    request: SecureRetrieveRequest,
) -> FiscusResult<SecureRetrieveResponse> {
    // Create repository instance
    let db = get_database();
    let repository = SecureStorageRepository::new(db);

    // Retrieve the data using the repository
    match repository
        .retrieve(&request.user_id, &request.data_type)
        .await?
    {
        Some(record) => Ok(SecureRetrieveResponse {
            encrypted_data: record.encrypted_data,
            nonce: record.nonce,
            algorithm: record.algorithm,
            key_id: record.key_id,
            stored_at: record.stored_at,
        }),
        None => Err(FiscusError::NotFound(format!(
            "No secure data found for user {} and data type {}",
            request.user_id, request.data_type
        ))),
    }
}

/// Delete encrypted data securely
#[tauri::command]
#[instrument(skip(request), fields(user_id = %request.user_id, data_type = %request.data_type))]
pub async fn secure_delete(request: SecureDeleteRequest) -> FiscusResult<SecureDeleteResponse> {
    // Create repository instance
    let db = get_database();
    let repository = SecureStorageRepository::new(db);

    // Delete the data using the repository
    let was_deleted = repository
        .delete(&request.user_id, &request.data_type)
        .await?;
    let deleted_at = Utc::now();

    Ok(SecureDeleteResponse {
        deleted: was_deleted,
        deleted_at,
    })
}

/// Clean up expired secure storage entries
#[tauri::command]
#[instrument]
pub async fn secure_cleanup_expired() -> FiscusResult<u64> {
    // Try to use the service first, fall back to direct repository access
    match get_secure_storage_service().await {
        Ok(service_arc) => {
            let service = service_arc.lock().await;
            let report = service.manual_cleanup().await?;
            Ok(report.deleted_count)
        }
        Err(_) => {
            // Fallback to direct repository access
            let db = get_database();
            let repository = SecureStorageRepository::new(db);
            let deleted_count = repository.cleanup_expired().await?;

            info!(
                deleted_count = deleted_count,
                "Cleaned up expired secure storage entries (fallback mode)"
            );

            Ok(deleted_count)
        }
    }
}

/// Get secure storage statistics
#[tauri::command]
#[instrument(skip(user_id))]
pub async fn secure_get_statistics(
    user_id: Option<String>,
) -> FiscusResult<Vec<std::collections::HashMap<String, serde_json::Value>>> {
    // Try to use the service first, fall back to direct repository access
    match get_secure_storage_service().await {
        Ok(service_arc) => {
            let service = service_arc.lock().await;
            service.get_statistics(user_id.as_deref()).await
        }
        Err(_) => {
            // Fallback to direct repository access
            let db = get_database();
            let repository = SecureStorageRepository::new(db);
            repository.get_storage_stats(user_id.as_deref()).await
        }
    }
}
