/// Database connection management for Fiscus
///
/// This module provides a connection manager that handles database connections
/// with proper pooling, configuration, and error handling for the Tauri SQL plugin.
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tracing::{debug, error, info, warn};

use super::config::{DatabaseConfig, DatabaseType};
use crate::error::{FiscusError, FiscusResult};
use crate::logging::DatabaseLogger;

/// Database connection handle for Tauri SQL plugin
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    /// Database URL for the connection
    pub url: String,
    /// Database type
    pub db_type: DatabaseType,
    /// Connection creation timestamp
    pub created_at: Instant,
    /// Last used timestamp
    pub last_used: Instant,
    /// Connection ID for tracking
    pub connection_id: String,
}

impl DatabaseConnection {
    /// Create a new database connection
    pub fn new(url: String, db_type: DatabaseType) -> Self {
        let now = Instant::now();
        let connection_id = uuid::Uuid::new_v4().to_string();

        Self {
            url,
            db_type,
            created_at: now,
            last_used: now,
            connection_id,
        }
    }

    /// Update the last used timestamp
    pub fn touch(&mut self) {
        self.last_used = Instant::now();
    }

    /// Get connection age
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Get time since last use
    pub fn idle_time(&self) -> std::time::Duration {
        self.last_used.elapsed()
    }
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_requests: u64,
    pub failed_requests: u64,
}

/// Database connection manager
#[derive(Debug)]
pub struct ConnectionManager {
    /// Database configuration
    config: DatabaseConfig,
    /// Connection pool
    pool: Arc<RwLock<Vec<DatabaseConnection>>>,
    /// Connection usage tracking
    #[allow(dead_code)]
    usage_stats: Arc<RwLock<HashMap<String, u64>>>,
    /// Pool statistics
    stats: Arc<RwLock<PoolStats>>,
    /// Database logger
    #[allow(dead_code)]
    logger: DatabaseLogger,
}

impl ConnectionManager {
    /// Create a new connection manager with configuration
    pub fn new(config: DatabaseConfig) -> FiscusResult<Self> {
        config.validate()?;

        let manager = Self {
            config,
            pool: Arc::new(RwLock::new(Vec::new())),
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PoolStats {
                total_connections: 0,
                active_connections: 0,
                idle_connections: 0,
                total_requests: 0,
                failed_requests: 0,
            })),
            logger: DatabaseLogger::new(),
        };

        info!(
            database_url = %manager.config.database_url(),
            database_type = ?manager.config.database_type(),
            max_connections = manager.config.max_connections,
            pooling_enabled = manager.config.is_pooling_enabled(),
            "Database connection manager initialized"
        );

        Ok(manager)
    }

    /// Create connection manager from environment variables
    pub fn from_env() -> FiscusResult<Self> {
        let config = DatabaseConfig::from_env()?;
        Self::new(config)
    }

    /// Create connection manager from configuration file
    pub fn from_file(path: &str) -> FiscusResult<Self> {
        let config = DatabaseConfig::from_file(path)?;
        Self::new(config)
    }

    /// Get a database connection
    pub fn get_connection(&self) -> FiscusResult<DatabaseConnection> {
        let start_time = Instant::now();

        // Update request statistics
        {
            let mut stats = self
                .stats
                .write()
                .map_err(|e| FiscusError::Internal(format!("Failed to acquire stats lock: {e}")))?;
            stats.total_requests += 1;
        }

        let result = if self.config.is_pooling_enabled() {
            self.get_pooled_connection()
        } else {
            self.create_new_connection()
        };

        match &result {
            Ok(conn) => {
                debug!(
                    connection_id = %conn.connection_id,
                    database_url = %conn.url,
                    duration_ms = start_time.elapsed().as_millis(),
                    "Database connection acquired"
                );
            }
            Err(error) => {
                // Update failure statistics
                if let Ok(mut stats) = self.stats.write() {
                    stats.failed_requests += 1;
                }

                error!(
                    error = %error,
                    duration_ms = start_time.elapsed().as_millis(),
                    "Failed to acquire database connection"
                );
            }
        }

        result
    }

    /// Get a pooled connection
    fn get_pooled_connection(&self) -> FiscusResult<DatabaseConnection> {
        // Try to get an existing connection from the pool
        {
            let mut pool = self
                .pool
                .write()
                .map_err(|e| FiscusError::Internal(format!("Failed to acquire pool lock: {e}")))?;

            if let Some(mut conn) = pool.pop() {
                conn.touch();
                debug!(
                    connection_id = %conn.connection_id,
                    age_ms = conn.age().as_millis(),
                    "Reusing pooled connection"
                );
                return Ok(conn);
            }
        }

        // No available connections, create a new one
        self.create_new_connection()
    }

    /// Create a new database connection
    fn create_new_connection(&self) -> FiscusResult<DatabaseConnection> {
        let current_count = {
            let pool = self
                .pool
                .read()
                .map_err(|e| FiscusError::Internal(format!("Failed to acquire pool lock: {e}")))?;
            pool.len()
        };

        if current_count >= self.config.max_connections as usize {
            return Err(FiscusError::Internal(
                "Maximum number of database connections reached".to_string(),
            ));
        }

        // Ensure we're only working with SQLite for local operations
        if self.config.database_type() != &DatabaseType::SQLite {
            return Err(FiscusError::InvalidInput(
                "Only SQLite is supported for local operations".to_string(),
            ));
        }

        let connection = DatabaseConnection::new(
            self.config.database_url().to_string(),
            self.config.database_type().clone(),
        );

        debug!(
            connection_id = %connection.connection_id,
            database_url = %connection.url,
            database_type = "SQLite",
            "Created new local SQLite database connection"
        );

        Ok(connection)
    }

    /// Return a connection to the pool
    pub fn return_connection(&self, mut connection: DatabaseConnection) -> FiscusResult<()> {
        if !self.config.is_pooling_enabled() {
            debug!(
                connection_id = %connection.connection_id,
                "Connection pooling disabled, discarding connection"
            );
            return Ok(());
        }

        connection.touch();

        let mut pool = self
            .pool
            .write()
            .map_err(|e| FiscusError::Internal(format!("Failed to acquire pool lock: {e}")))?;

        if pool.len() < self.config.max_connections as usize {
            pool.push(connection);
            debug!("Connection returned to pool");
        } else {
            debug!("Pool full, discarding connection");
        }

        Ok(())
    }

    /// Get connection pool statistics
    pub fn get_stats(&self) -> FiscusResult<PoolStats> {
        let stats = self
            .stats
            .read()
            .map_err(|e| FiscusError::Internal(format!("Failed to acquire stats lock: {e}")))?;

        let pool = self
            .pool
            .read()
            .map_err(|e| FiscusError::Internal(format!("Failed to acquire pool lock: {e}")))?;

        Ok(PoolStats {
            total_connections: pool.len(),
            active_connections: pool.len(), // Simplified for now
            idle_connections: pool.len(),
            total_requests: stats.total_requests,
            failed_requests: stats.failed_requests,
        })
    }

    /// Clean up idle connections
    pub fn cleanup_idle_connections(&self) -> FiscusResult<usize> {
        let mut pool = self
            .pool
            .write()
            .map_err(|e| FiscusError::Internal(format!("Failed to acquire pool lock: {e}")))?;

        let initial_count = pool.len();
        let idle_threshold = std::time::Duration::from_secs(300); // 5 minutes

        pool.retain(|conn| conn.idle_time() < idle_threshold);

        let cleaned_count = initial_count - pool.len();

        if cleaned_count > 0 {
            info!(
                cleaned_connections = cleaned_count,
                remaining_connections = pool.len(),
                "Cleaned up idle database connections"
            );
        }

        Ok(cleaned_count)
    }

    /// Get the database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Check if the connection manager is healthy
    pub fn health_check(&self) -> FiscusResult<bool> {
        let stats = self.get_stats()?;

        // Basic health checks
        let is_healthy = stats.total_connections <= self.config.max_connections as usize
            && (stats.failed_requests == 0
                || (stats.total_requests > 0
                    && (stats.failed_requests as f64 / stats.total_requests as f64) < 0.1));

        if !is_healthy {
            warn!(
                total_connections = stats.total_connections,
                max_connections = self.config.max_connections,
                failed_requests = stats.failed_requests,
                total_requests = stats.total_requests,
                "Database connection manager health check failed"
            );
        }

        Ok(is_healthy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_creation() {
        let conn = DatabaseConnection::new("sqlite:test.db".to_string(), DatabaseType::SQLite);

        assert_eq!(conn.url, "sqlite:test.db");
        assert_eq!(conn.db_type, DatabaseType::SQLite);
        assert!(!conn.connection_id.is_empty());
    }

    #[test]
    fn test_connection_manager_creation() {
        let config = DatabaseConfig::default();
        let manager = ConnectionManager::new(config).unwrap();

        assert_eq!(manager.config.database_url(), "sqlite:fiscus.db");
        assert!(manager.config.is_pooling_enabled());
    }

    #[test]
    fn test_get_connection() {
        let config = DatabaseConfig::default();
        let manager = ConnectionManager::new(config).unwrap();

        let conn = manager.get_connection().unwrap();
        assert_eq!(conn.url, "sqlite:fiscus.db");
        assert_eq!(conn.db_type, DatabaseType::SQLite);
    }

    #[test]
    fn test_connection_pooling() {
        let config = DatabaseConfig::default();
        let manager = ConnectionManager::new(config).unwrap();

        let conn1 = manager.get_connection().unwrap();
        let conn_id = conn1.connection_id.clone();

        // Return connection to pool
        manager.return_connection(conn1).unwrap();

        // Get another connection - should reuse the pooled one
        let conn2 = manager.get_connection().unwrap();
        assert_eq!(conn2.connection_id, conn_id);
    }
}
