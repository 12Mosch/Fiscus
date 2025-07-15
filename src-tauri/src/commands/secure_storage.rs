use chrono::Utc;
use once_cell::sync::Lazy;
use tracing::{info, instrument};

use crate::{
    database::{
        secure_storage_repository::SecureStorageRepository, ConnectionManager, DatabaseConnection,
        PoolStats, SQLiteManager, SQLiteStats,
    },
    dto::{
        SecureDeleteRequest, SecureDeleteResponse, SecureRetrieveRequest, SecureRetrieveResponse,
        SecureStoreRequest, SecureStoreResponse,
    },
    error::{FiscusError, FiscusResult},
    services::get_secure_storage_service,
};

/// Global connection manager instance
static CONNECTION_MANAGER: Lazy<ConnectionManager> = Lazy::new(|| {
    ConnectionManager::from_env().unwrap_or_else(|e| {
        tracing::error!(
            "Failed to initialize connection manager from environment: {}",
            e
        );
        tracing::info!("Falling back to default configuration");
        ConnectionManager::new(crate::database::DatabaseConfig::default())
            .expect("Failed to create connection manager with default config")
    })
});

/// Get database connection for secure storage operations
/// Uses proper connection pooling and configuration management
fn get_database() -> FiscusResult<DatabaseConnection> {
    CONNECTION_MANAGER.get_connection()
}

/// Store encrypted data securely
#[tauri::command]
#[instrument(skip(request), fields(user_id = %request.user_id, data_type = %request.data_type))]
pub async fn secure_store(request: SecureStoreRequest) -> FiscusResult<SecureStoreResponse> {
    // Get database connection from pool
    let db = get_database()?;
    let repository = SecureStorageRepository::new(db);

    // Store the data using the repository
    let record = repository
        .store(
            &request.user_id.as_str(),
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
    // Get database connection from pool
    let db = get_database()?;
    let repository = SecureStorageRepository::new(db);

    // Retrieve the data using the repository
    match repository
        .retrieve(&request.user_id.as_str(), &request.data_type)
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
    // Get database connection from pool
    let db = get_database()?;
    let repository = SecureStorageRepository::new(db);

    // Delete the data using the repository
    let was_deleted = repository
        .delete(&request.user_id.as_str(), &request.data_type)
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
            let db = get_database()?;
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
            let db = get_database()?;
            let repository = SecureStorageRepository::new(db);
            repository.get_storage_stats(user_id.as_deref()).await
        }
    }
}

/// Get database connection pool statistics
#[allow(dead_code)]
#[tauri::command]
#[instrument]
pub async fn get_connection_stats() -> FiscusResult<PoolStats> {
    CONNECTION_MANAGER.get_stats()
}

/// Perform database connection health check
#[allow(dead_code)]
#[tauri::command]
#[instrument]
pub async fn database_health_check() -> FiscusResult<bool> {
    CONNECTION_MANAGER.health_check()
}

/// Clean up idle database connections
#[allow(dead_code)]
#[tauri::command]
#[instrument]
pub async fn cleanup_idle_connections() -> FiscusResult<usize> {
    CONNECTION_MANAGER.cleanup_idle_connections()
}

/// Get SQLite-specific database statistics
#[allow(dead_code)]
#[tauri::command]
#[instrument]
pub async fn get_sqlite_stats() -> FiscusResult<SQLiteStats> {
    let db = get_database()?;
    let sqlite_manager = SQLiteManager::new(CONNECTION_MANAGER.config().clone())?;
    sqlite_manager.get_sqlite_stats(&db).await
}

/// Optimize SQLite database (VACUUM)
#[allow(dead_code)]
#[tauri::command]
#[instrument]
pub async fn optimize_sqlite_database() -> FiscusResult<()> {
    let db = get_database()?;
    let sqlite_manager = SQLiteManager::new(CONNECTION_MANAGER.config().clone())?;
    sqlite_manager.optimize_database(&db).await
}

/// Configure SQLite for optimal local performance
#[allow(dead_code)]
#[tauri::command]
#[instrument]
pub async fn configure_sqlite_performance() -> FiscusResult<()> {
    let db = get_database()?;
    let sqlite_manager = SQLiteManager::new(CONNECTION_MANAGER.config().clone())?;
    sqlite_manager.configure_sqlite_performance(&db).await
}

/// Check SQLite database integrity
#[allow(dead_code)]
#[tauri::command]
#[instrument]
pub async fn check_sqlite_integrity() -> FiscusResult<bool> {
    let db = get_database()?;
    let sqlite_manager = SQLiteManager::new(CONNECTION_MANAGER.config().clone())?;
    sqlite_manager.check_integrity(&db).await
}

/// Backup SQLite database to specified path
#[allow(dead_code)]
#[tauri::command]
#[instrument]
pub async fn backup_sqlite_database(backup_path: String) -> FiscusResult<()> {
    let sqlite_manager = SQLiteManager::new(CONNECTION_MANAGER.config().clone())?;
    let backup_path = std::path::PathBuf::from(backup_path);
    sqlite_manager.backup_database(&backup_path).await
}

/// Get SQLite database file size
#[allow(dead_code)]
#[tauri::command]
#[instrument]
pub async fn get_sqlite_database_size() -> FiscusResult<u64> {
    let sqlite_manager = SQLiteManager::new(CONNECTION_MANAGER.config().clone())?;
    sqlite_manager.get_database_size()
}
