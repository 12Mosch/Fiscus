use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{error, info, instrument};

use crate::{
    database::secure_storage_repository::SecureStorageRepository,
    error::{FiscusError, FiscusResult},
};

/// Parameters for storing data with expiration
#[derive(Debug)]
pub struct StoreWithExpirationParams<'a> {
    pub user_id: &'a str,
    pub data_type: &'a str,
    pub encrypted_data: &'a str,
    pub nonce: &'a str,
    pub algorithm: crate::encryption::types::EncryptionAlgorithm,
    pub key_id: &'a str,
    pub custom_expiration_hours: Option<i64>,
}

/// Configuration for the secure storage service
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - fields will be used by consumers
pub struct SecureStorageConfig {
    /// How often to run cleanup (in minutes)
    pub cleanup_interval_minutes: u64,
    /// Default expiration time for data (in hours)
    pub default_expiration_hours: i64,
    /// Maximum number of access attempts before logging warning
    pub max_access_attempts: i64,
    /// Enable automatic cleanup
    pub auto_cleanup_enabled: bool,
}

impl Default for SecureStorageConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_minutes: 60,     // Run cleanup every hour
            default_expiration_hours: 24 * 7, // 7 days default expiration
            max_access_attempts: 1000,        // Log warning after 1000 accesses
            auto_cleanup_enabled: true,
        }
    }
}

/// Secure storage service with automatic cleanup and monitoring
#[allow(dead_code)] // Service fields are used internally
pub struct SecureStorageService {
    repository: Arc<SecureStorageRepository>,
    config: Arc<RwLock<SecureStorageConfig>>,
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl SecureStorageService {
    /// Create a new secure storage service
    #[allow(dead_code)] // Public API method
    pub fn new(db: String, config: Option<SecureStorageConfig>) -> Self {
        let repository = Arc::new(SecureStorageRepository::new(db));
        let config = Arc::new(RwLock::new(config.unwrap_or_default()));

        Self {
            repository,
            config,
            cleanup_handle: None,
        }
    }

    /// Start the automatic cleanup service
    #[instrument(skip(self))]
    #[allow(dead_code)] // Public API method
    pub async fn start_cleanup_service(&mut self) -> FiscusResult<()> {
        let config = self.config.read().await;

        if !config.auto_cleanup_enabled {
            info!("Automatic cleanup is disabled");
            return Ok(());
        }

        let cleanup_interval = config.cleanup_interval_minutes;
        drop(config); // Release the lock

        let repository = Arc::clone(&self.repository);
        let config_arc = Arc::clone(&self.config);

        let handle = tokio::spawn(async move {
            let mut interval = interval(TokioDuration::from_secs(cleanup_interval * 60));

            loop {
                interval.tick().await;

                let config = config_arc.read().await;
                if !config.auto_cleanup_enabled {
                    info!("Cleanup service stopping - auto cleanup disabled");
                    break;
                }
                drop(config);

                match repository.cleanup_expired().await {
                    Ok(deleted_count) => {
                        if deleted_count > 0 {
                            info!(
                                deleted_count = deleted_count,
                                "Automatic cleanup completed successfully"
                            );
                        }
                    }
                    Err(e) => {
                        error!(
                            error = %e,
                            "Failed to run automatic cleanup"
                        );
                    }
                }
            }
        });

        self.cleanup_handle = Some(handle);
        info!(
            interval_minutes = cleanup_interval,
            "Started automatic cleanup service"
        );

        Ok(())
    }

    /// Stop the automatic cleanup service
    #[instrument(skip(self))]
    #[allow(dead_code)] // Public API method
    pub async fn stop_cleanup_service(&mut self) {
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
            info!("Stopped automatic cleanup service");
        }
    }

    /// Store data with automatic expiration
    #[instrument(skip(self, params), fields(user_id = %params.user_id, data_type = %params.data_type))]
    #[allow(dead_code)] // Public API method
    pub async fn store_with_expiration(
        &self,
        params: StoreWithExpirationParams<'_>,
    ) -> FiscusResult<crate::database::secure_storage_repository::SecureStorageRecord> {
        let config = self.config.read().await;
        let expiration_hours = params
            .custom_expiration_hours
            .unwrap_or(config.default_expiration_hours);
        drop(config);

        let expires_at = if expiration_hours > 0 {
            Some(Utc::now() + Duration::hours(expiration_hours))
        } else {
            None // No expiration
        };

        self.repository
            .store(
                params.user_id,
                params.data_type,
                params.encrypted_data,
                params.nonce,
                params.algorithm,
                params.key_id,
                expires_at,
            )
            .await
    }

    /// Get repository for direct access
    #[allow(dead_code)] // Public API method
    pub fn repository(&self) -> &SecureStorageRepository {
        &self.repository
    }

    /// Update configuration
    #[instrument(skip(self, new_config))]
    #[allow(dead_code)] // Public API method
    pub async fn update_config(&self, new_config: SecureStorageConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
        info!("Updated secure storage service configuration");
    }

    /// Get current configuration
    #[allow(dead_code)] // Public API method
    pub async fn get_config(&self) -> SecureStorageConfig {
        self.config.read().await.clone()
    }

    /// Manual cleanup with detailed reporting
    #[instrument(skip(self))]
    pub async fn manual_cleanup(&self) -> FiscusResult<CleanupReport> {
        let start_time = Utc::now();

        let deleted_count = self.repository.cleanup_expired().await?;

        let duration = Utc::now().signed_duration_since(start_time);

        let report = CleanupReport {
            deleted_count,
            duration_ms: duration.num_milliseconds(),
            timestamp: start_time,
        };

        info!(
            deleted_count = deleted_count,
            duration_ms = duration.num_milliseconds(),
            "Manual cleanup completed"
        );

        Ok(report)
    }

    /// Get storage statistics
    #[instrument(skip(self))]
    pub async fn get_statistics(
        &self,
        user_id: Option<&str>,
    ) -> FiscusResult<Vec<std::collections::HashMap<String, serde_json::Value>>> {
        self.repository.get_storage_stats(user_id).await
    }
}

impl Drop for SecureStorageService {
    fn drop(&mut self) {
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
        }
    }
}

/// Report from cleanup operations
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - fields will be used by consumers
pub struct CleanupReport {
    pub deleted_count: u64,
    pub duration_ms: i64,
    pub timestamp: DateTime<Utc>,
}

/// Global secure storage service instance
static SECURE_STORAGE_SERVICE: tokio::sync::OnceCell<
    Arc<tokio::sync::Mutex<SecureStorageService>>,
> = tokio::sync::OnceCell::const_new();

/// Initialize the global secure storage service
#[allow(dead_code)] // Public API function
pub async fn initialize_secure_storage_service(
    db: String,
    config: Option<SecureStorageConfig>,
) -> FiscusResult<()> {
    let mut service = SecureStorageService::new(db, config);
    service.start_cleanup_service().await?;

    SECURE_STORAGE_SERVICE
        .set(Arc::new(tokio::sync::Mutex::new(service)))
        .map_err(|_| {
            FiscusError::Internal("Failed to initialize secure storage service".to_string())
        })?;

    info!("Secure storage service initialized successfully");
    Ok(())
}

/// Get the global secure storage service
pub async fn get_secure_storage_service(
) -> FiscusResult<Arc<tokio::sync::Mutex<SecureStorageService>>> {
    SECURE_STORAGE_SERVICE
        .get()
        .cloned()
        .ok_or_else(|| FiscusError::Internal("Secure storage service not initialized".to_string()))
}

/// Shutdown the secure storage service
#[allow(dead_code)] // Public API function
pub async fn shutdown_secure_storage_service() -> FiscusResult<()> {
    if let Some(service_arc) = SECURE_STORAGE_SERVICE.get() {
        let mut service = service_arc.lock().await;
        service.stop_cleanup_service().await;
        info!("Secure storage service shutdown completed");
    }
    Ok(())
}
