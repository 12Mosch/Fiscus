//! Comprehensive logging system for the Fiscus application
//!
//! This module provides structured logging capabilities with:
//! - Request/response logging for Tauri commands
//! - Database operation logging
//! - Sensitive data sanitization
//! - Configurable output formats (console, file, JSON)
//! - Performance monitoring
//! - Error tracking with context

pub mod config;
pub mod middleware;
pub mod performance;
pub mod sanitizer;

#[cfg(test)]
mod tests;

// Re-export main types for easier access
pub use config::{init_logging, Environment, LogFormat, LoggingConfig};
pub use middleware::{DatabaseLogger, ExtractUserId, LoggingMiddleware, RequestContext};
pub use performance::{init_performance_monitoring, PerformanceMonitor, PerformanceSummary};
pub use sanitizer::{DataSanitizer, Sanitizable};

/// Initialize the complete logging system
pub fn init() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging first
    init_logging()?;

    // Initialize performance monitoring
    init_performance_monitoring();

    Ok(())
}

/// Create a new data sanitizer with default settings
pub fn create_sanitizer() -> DataSanitizer {
    DataSanitizer::new()
}

/// Create a new logging middleware instance
pub fn create_middleware() -> LoggingMiddleware {
    LoggingMiddleware::new()
}

/// Create a new database logger instance
pub fn create_db_logger() -> DatabaseLogger {
    DatabaseLogger::new()
}
