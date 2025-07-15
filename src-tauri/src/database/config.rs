/// Database configuration management for Fiscus
///
/// This module provides configuration management for database connections,
/// including connection pooling settings, database URLs, and connection timeouts.
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tracing::{debug, info};

use crate::error::{FiscusError, FiscusResult};

/// Database configuration for the Fiscus application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub database_url: String,
    /// Database type (sqlite, mysql, postgres)
    pub database_type: DatabaseType,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: Duration,
    /// Query timeout in seconds
    pub query_timeout: Duration,
    /// Enable connection pooling
    pub enable_pooling: bool,
    /// Enable query logging
    pub enable_query_logging: bool,
    /// Enable slow query detection
    pub enable_slow_query_detection: bool,
    /// Slow query threshold in milliseconds
    pub slow_query_threshold_ms: u64,
}

/// Supported database types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseType {
    SQLite,
    MySQL,
    PostgreSQL,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite:fiscus.db".to_string(),
            database_type: DatabaseType::SQLite,
            max_connections: 5, // Lower for local SQLite
            min_connections: 1,
            connection_timeout: Duration::from_secs(10), // Faster for local
            query_timeout: Duration::from_secs(30),      // Faster for local
            enable_pooling: true,
            enable_query_logging: true,
            enable_slow_query_detection: true,
            slow_query_threshold_ms: 50, // Lower threshold for local
        }
    }
}

impl DatabaseConfig {
    /// Create database configuration from environment variables
    pub fn from_env() -> FiscusResult<Self> {
        let mut config = Self::default();

        // Database URL - required
        if let Ok(url) = env::var("FISCUS_DATABASE_URL") {
            config.database_url = url;
            config.database_type = Self::detect_database_type(&config.database_url)?;
        }

        // Connection pool settings
        if let Ok(max_conn) = env::var("FISCUS_DB_MAX_CONNECTIONS") {
            config.max_connections = max_conn
                .parse()
                .map_err(|e| FiscusError::InvalidInput(format!("Invalid max connections: {e}")))?;
        }

        if let Ok(min_conn) = env::var("FISCUS_DB_MIN_CONNECTIONS") {
            config.min_connections = min_conn
                .parse()
                .map_err(|e| FiscusError::InvalidInput(format!("Invalid min connections: {e}")))?;
        }

        // Timeout settings
        if let Ok(conn_timeout) = env::var("FISCUS_DB_CONNECTION_TIMEOUT") {
            let timeout_secs: u64 = conn_timeout.parse().map_err(|e| {
                FiscusError::InvalidInput(format!("Invalid connection timeout: {e}"))
            })?;
            config.connection_timeout = Duration::from_secs(timeout_secs);
        }

        if let Ok(query_timeout) = env::var("FISCUS_DB_QUERY_TIMEOUT") {
            let timeout_secs: u64 = query_timeout
                .parse()
                .map_err(|e| FiscusError::InvalidInput(format!("Invalid query timeout: {e}")))?;
            config.query_timeout = Duration::from_secs(timeout_secs);
        }

        // Feature flags
        if let Ok(enable_pooling) = env::var("FISCUS_DB_ENABLE_POOLING") {
            config.enable_pooling = enable_pooling.parse().map_err(|e| {
                FiscusError::InvalidInput(format!("Invalid enable pooling setting: {e}"))
            })?;
        }

        if let Ok(enable_logging) = env::var("FISCUS_DB_ENABLE_QUERY_LOGGING") {
            config.enable_query_logging = enable_logging.parse().map_err(|e| {
                FiscusError::InvalidInput(format!("Invalid query logging setting: {e}"))
            })?;
        }

        if let Ok(enable_slow_detection) = env::var("FISCUS_DB_ENABLE_SLOW_QUERY_DETECTION") {
            config.enable_slow_query_detection = enable_slow_detection.parse().map_err(|e| {
                FiscusError::InvalidInput(format!("Invalid slow query detection setting: {e}"))
            })?;
        }

        if let Ok(threshold) = env::var("FISCUS_DB_SLOW_QUERY_THRESHOLD_MS") {
            config.slow_query_threshold_ms = threshold.parse().map_err(|e| {
                FiscusError::InvalidInput(format!("Invalid slow query threshold: {e}"))
            })?;
        }

        config.validate()?;

        debug!("Loaded database configuration from environment");
        Ok(config)
    }

    /// Create database configuration from file
    pub fn from_file(path: &str) -> FiscusResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| FiscusError::Internal(format!("Failed to read config file: {e}")))?;

        let config: DatabaseConfig = toml::from_str(&content)
            .map_err(|e| FiscusError::Internal(format!("Failed to parse config: {e}")))?;

        config.validate()?;

        info!("Loaded database configuration from {}", path);
        Ok(config)
    }

    /// Detect database type from URL
    fn detect_database_type(url: &str) -> FiscusResult<DatabaseType> {
        if url.starts_with("sqlite:") {
            Ok(DatabaseType::SQLite)
        } else if url.starts_with("mysql:") {
            Ok(DatabaseType::MySQL)
        } else if url.starts_with("postgres:") || url.starts_with("postgresql:") {
            Ok(DatabaseType::PostgreSQL)
        } else {
            Err(FiscusError::InvalidInput(format!(
                "Unsupported database URL format: {url}"
            )))
        }
    }

    /// Validate configuration settings
    pub fn validate(&self) -> FiscusResult<()> {
        if self.database_url.is_empty() {
            return Err(FiscusError::InvalidInput(
                "Database URL cannot be empty".to_string(),
            ));
        }

        if self.max_connections == 0 {
            return Err(FiscusError::InvalidInput(
                "Max connections must be greater than 0".to_string(),
            ));
        }

        if self.min_connections > self.max_connections {
            return Err(FiscusError::InvalidInput(
                "Min connections cannot be greater than max connections".to_string(),
            ));
        }

        if self.connection_timeout.is_zero() {
            return Err(FiscusError::InvalidInput(
                "Connection timeout must be greater than 0".to_string(),
            ));
        }

        if self.query_timeout.is_zero() {
            return Err(FiscusError::InvalidInput(
                "Query timeout must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the database URL
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    /// Get the database type
    pub fn database_type(&self) -> &DatabaseType {
        &self.database_type
    }

    /// Check if pooling is enabled
    pub fn is_pooling_enabled(&self) -> bool {
        self.enable_pooling
    }

    /// Check if query logging is enabled
    pub fn is_query_logging_enabled(&self) -> bool {
        self.enable_query_logging
    }

    /// Check if slow query detection is enabled
    pub fn is_slow_query_detection_enabled(&self) -> bool {
        self.enable_slow_query_detection
    }

    /// Get slow query threshold
    pub fn slow_query_threshold(&self) -> Duration {
        Duration::from_millis(self.slow_query_threshold_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.database_url, "sqlite:fiscus.db");
        assert_eq!(config.database_type, DatabaseType::SQLite);
        assert_eq!(config.max_connections, 5); // Updated for local SQLite optimization
        assert_eq!(config.min_connections, 1);
        assert!(config.enable_pooling);
    }

    #[test]
    fn test_detect_database_type() {
        assert_eq!(
            DatabaseConfig::detect_database_type("sqlite:test.db").unwrap(),
            DatabaseType::SQLite
        );
        assert_eq!(
            DatabaseConfig::detect_database_type("mysql://user:pass@host/db").unwrap(),
            DatabaseType::MySQL
        );
        assert_eq!(
            DatabaseConfig::detect_database_type("postgres://user:pass@host/db").unwrap(),
            DatabaseType::PostgreSQL
        );
    }

    #[test]
    fn test_config_validation() {
        let mut config = DatabaseConfig::default();
        assert!(config.validate().is_ok());

        config.database_url = String::new();
        assert!(config.validate().is_err());

        config = DatabaseConfig::default();
        config.max_connections = 0;
        assert!(config.validate().is_err());

        config = DatabaseConfig::default();
        config.min_connections = 20;
        config.max_connections = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_from_env() {
        env::set_var("FISCUS_DATABASE_URL", "sqlite:test.db");
        env::set_var("FISCUS_DB_MAX_CONNECTIONS", "20");
        env::set_var("FISCUS_DB_ENABLE_POOLING", "false");

        let config = DatabaseConfig::from_env().unwrap();
        assert_eq!(config.database_url, "sqlite:test.db");
        assert_eq!(config.max_connections, 20);
        assert!(!config.enable_pooling);

        // Clean up
        env::remove_var("FISCUS_DATABASE_URL");
        env::remove_var("FISCUS_DB_MAX_CONNECTIONS");
        env::remove_var("FISCUS_DB_ENABLE_POOLING");
    }
}
