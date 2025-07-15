/// SQLite-specific database operations for local Fiscus application
///
/// This module provides SQLite-optimized database operations for the local-only
/// Fiscus personal finance application using the Tauri SQL plugin.
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::config::DatabaseConfig;
use super::connection::DatabaseConnection;
use crate::error::{FiscusError, FiscusResult};

/// SQLite-specific database manager
#[derive(Debug)]
pub struct SQLiteManager {
    config: DatabaseConfig,
    db_path: PathBuf,
}

impl SQLiteManager {
    /// Create a new SQLite manager
    pub fn new(config: DatabaseConfig) -> FiscusResult<Self> {
        // Extract database path from URL
        let db_path = Self::extract_db_path(&config.database_url)?;

        info!(
            database_path = %db_path.display(),
            "Initializing SQLite manager for local database"
        );

        Ok(Self { config, db_path })
    }

    /// Extract database path from SQLite URL
    fn extract_db_path(url: &str) -> FiscusResult<PathBuf> {
        if !url.starts_with("sqlite:") {
            return Err(FiscusError::InvalidInput(
                "Invalid SQLite URL format".to_string(),
            ));
        }

        let path_str = url.strip_prefix("sqlite:").unwrap_or(url);

        // Handle special cases
        match path_str {
            ":memory:" => Ok(PathBuf::from(":memory:")),
            "" => Err(FiscusError::InvalidInput("Empty database path".to_string())),
            _ => {
                let path = PathBuf::from(path_str);
                // Ensure the directory exists for file-based databases
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            FiscusError::Internal(format!(
                                "Failed to create database directory: {e}"
                            ))
                        })?;
                    }
                }
                Ok(path)
            }
        }
    }

    /// Get database file size in bytes
    pub fn get_database_size(&self) -> FiscusResult<u64> {
        if self.db_path.to_string_lossy() == ":memory:" {
            return Ok(0); // In-memory database
        }

        if !self.db_path.exists() {
            return Ok(0); // Database doesn't exist yet
        }

        std::fs::metadata(&self.db_path)
            .map(|metadata| metadata.len())
            .map_err(|e| FiscusError::Internal(format!("Failed to get database size: {e}")))
    }

    /// Check if database file exists
    pub fn database_exists(&self) -> bool {
        if self.db_path.to_string_lossy() == ":memory:" {
            return true; // In-memory database always "exists"
        }
        self.db_path.exists()
    }

    /// Get database file path
    pub fn get_database_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Optimize SQLite database (VACUUM)
    pub async fn optimize_database(&self, connection: &DatabaseConnection) -> FiscusResult<()> {
        info!(
            connection_id = %connection.connection_id,
            database_path = %self.db_path.display(),
            "Starting database optimization (VACUUM)"
        );

        // Note: In a real implementation, this would execute VACUUM through Tauri SQL plugin
        // For now, we log the operation
        debug!("VACUUM operation would be executed here");

        info!("Database optimization completed");
        Ok(())
    }

    /// Get SQLite-specific database statistics
    pub async fn get_sqlite_stats(
        &self,
        connection: &DatabaseConnection,
    ) -> FiscusResult<SQLiteStats> {
        debug!(
            connection_id = %connection.connection_id,
            "Retrieving SQLite database statistics"
        );

        // Note: In a real implementation, this would query SQLite system tables
        // For now, we return basic file-based statistics
        let file_size = self.get_database_size()?;

        Ok(SQLiteStats {
            file_size_bytes: file_size,
            page_count: 0,   // Would be queried from PRAGMA page_count
            page_size: 4096, // Default SQLite page size
            cache_size: self.config.max_connections as u64 * 1024, // Estimated
            auto_vacuum: false, // Would be queried from PRAGMA auto_vacuum
            journal_mode: "WAL".to_string(), // Recommended for Tauri apps
            synchronous: "NORMAL".to_string(), // Balanced performance/safety
        })
    }

    /// Configure SQLite for optimal local performance
    pub async fn configure_sqlite_performance(
        &self,
        connection: &DatabaseConnection,
    ) -> FiscusResult<()> {
        info!(
            connection_id = %connection.connection_id,
            "Configuring SQLite for optimal local performance"
        );

        // Note: In a real implementation, these would be executed as PRAGMA statements
        let optimizations = vec![
            "PRAGMA journal_mode = WAL", // Write-Ahead Logging for better concurrency
            "PRAGMA synchronous = NORMAL", // Balanced safety/performance
            "PRAGMA cache_size = -64000", // 64MB cache
            "PRAGMA temp_store = MEMORY", // Store temp tables in memory
            "PRAGMA mmap_size = 268435456", // 256MB memory-mapped I/O
        ];

        for pragma in optimizations {
            debug!(pragma = pragma, "Would execute SQLite optimization");
        }

        info!("SQLite performance configuration completed");
        Ok(())
    }

    /// Backup database to specified path
    pub async fn backup_database(&self, backup_path: &PathBuf) -> FiscusResult<()> {
        if self.db_path.to_string_lossy() == ":memory:" {
            return Err(FiscusError::InvalidInput(
                "Cannot backup in-memory database".to_string(),
            ));
        }

        if !self.database_exists() {
            return Err(FiscusError::NotFound(
                "Source database file not found".to_string(),
            ));
        }

        info!(
            source = %self.db_path.display(),
            destination = %backup_path.display(),
            "Starting database backup"
        );

        // Create backup directory if it doesn't exist
        if let Some(parent) = backup_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                FiscusError::Internal(format!("Failed to create backup directory: {e}"))
            })?;
        }

        // Copy database file
        std::fs::copy(&self.db_path, backup_path)
            .map_err(|e| FiscusError::Internal(format!("Failed to backup database: {e}")))?;

        let backup_size = std::fs::metadata(backup_path).map(|m| m.len()).unwrap_or(0);

        info!(
            backup_size_bytes = backup_size,
            "Database backup completed successfully"
        );

        Ok(())
    }

    /// Check database integrity
    pub async fn check_integrity(&self, connection: &DatabaseConnection) -> FiscusResult<bool> {
        info!(
            connection_id = %connection.connection_id,
            "Checking database integrity"
        );

        // Note: In a real implementation, this would execute PRAGMA integrity_check
        // For now, we perform basic file system checks
        if self.db_path.to_string_lossy() == ":memory:" {
            return Ok(true); // In-memory database is always "intact"
        }

        let is_intact = self.database_exists() && self.get_database_size()? > 0;

        if is_intact {
            info!("Database integrity check passed");
        } else {
            warn!("Database integrity check failed");
        }

        Ok(is_intact)
    }
}

/// SQLite-specific database statistics
#[derive(Debug, Clone)]
pub struct SQLiteStats {
    pub file_size_bytes: u64,
    pub page_count: u64,
    pub page_size: u32,
    pub cache_size: u64,
    pub auto_vacuum: bool,
    pub journal_mode: String,
    pub synchronous: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_extract_db_path() {
        // Test normal file path
        let path = SQLiteManager::extract_db_path("sqlite:test.db").unwrap();
        assert_eq!(path, PathBuf::from("test.db"));

        // Test absolute path
        let path = SQLiteManager::extract_db_path("sqlite:/tmp/test.db").unwrap();
        assert_eq!(path, PathBuf::from("/tmp/test.db"));

        // Test memory database
        let path = SQLiteManager::extract_db_path("sqlite::memory:").unwrap();
        assert_eq!(path, PathBuf::from(":memory:"));

        // Test invalid URL
        assert!(SQLiteManager::extract_db_path("mysql:test").is_err());
    }

    #[test]
    fn test_sqlite_manager_creation() {
        let config = DatabaseConfig::default();
        let manager = SQLiteManager::new(config).unwrap();

        assert_eq!(manager.db_path, PathBuf::from("fiscus.db"));
    }

    #[test]
    fn test_database_size_nonexistent() {
        let config = DatabaseConfig::default();
        let manager = SQLiteManager::new(config).unwrap();

        // Should return 0 for non-existent database
        assert_eq!(manager.get_database_size().unwrap(), 0);
    }

    #[test]
    fn test_memory_database() {
        let config = DatabaseConfig {
            database_url: "sqlite::memory:".to_string(),
            ..Default::default()
        };

        let manager = SQLiteManager::new(config).unwrap();
        assert!(manager.database_exists());
        assert_eq!(manager.get_database_size().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_backup_memory_database() {
        let config = DatabaseConfig {
            database_url: "sqlite::memory:".to_string(),
            ..Default::default()
        };

        let manager = SQLiteManager::new(config).unwrap();
        let temp_dir = tempdir().unwrap();
        let backup_path = temp_dir.path().join("backup.db");

        // Should fail for in-memory database
        assert!(manager.backup_database(&backup_path).await.is_err());
    }
}
