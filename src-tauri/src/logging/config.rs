use std::env;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::EnvFilter;

/// Logging configuration for the Fiscus application
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: Level,
    /// Output format (console, json, file)
    pub format: LogFormat,
    /// Whether to log to console
    pub console_enabled: bool,
    /// Whether to log to file
    pub file_enabled: bool,
    /// Log file directory
    pub log_dir: PathBuf,
    /// Log file prefix
    pub file_prefix: String,
    /// Whether to include spans in logs
    pub include_spans: bool,
    /// Whether to include source location
    pub include_location: bool,
    /// Environment (development, production)
    pub environment: Environment,
    /// Fields to sanitize in logs
    pub sensitive_fields: Vec<String>,
}

/// Log output format
#[derive(Debug, Clone, PartialEq)]
pub enum LogFormat {
    /// Human-readable console format
    Console,
    /// Structured JSON format
    Json,
    /// Compact format for production
    Compact,
}

/// Application environment
#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            format: LogFormat::Console,
            console_enabled: true,
            file_enabled: false,
            log_dir: PathBuf::from("logs"),
            file_prefix: "fiscus".to_string(),
            include_spans: true,
            include_location: false,
            environment: Environment::Development,
            sensitive_fields: vec![
                "password".to_string(),
                "password_hash".to_string(),
                "token".to_string(),
                "secret".to_string(),
                "key".to_string(),
                "account_number".to_string(),
                "ssn".to_string(),
                "social_security_number".to_string(),
                "credit_card".to_string(),
                "card_number".to_string(),
                "pin".to_string(),
                "auth_token".to_string(),
                "session_token".to_string(),
                "api_key".to_string(),
            ],
        }
    }
}

impl LoggingConfig {
    /// Create logging configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Set log level from environment
        if let Ok(level_str) = env::var("FISCUS_LOG_LEVEL") {
            config.level = match level_str.to_lowercase().as_str() {
                "trace" => Level::TRACE,
                "debug" => Level::DEBUG,
                "info" => Level::INFO,
                "warn" => Level::WARN,
                "error" => Level::ERROR,
                _ => Level::INFO,
            };
        }

        // Set log format from environment
        if let Ok(format_str) = env::var("FISCUS_LOG_FORMAT") {
            config.format = match format_str.to_lowercase().as_str() {
                "json" => LogFormat::Json,
                "compact" => LogFormat::Compact,
                "console" => LogFormat::Console,
                _ => LogFormat::Console,
            };
        }

        // Set environment
        if let Ok(env_str) = env::var("FISCUS_ENV") {
            config.environment = match env_str.to_lowercase().as_str() {
                "production" | "prod" => Environment::Production,
                "test" => Environment::Test,
                _ => Environment::Development,
            };
        }

        // Enable/disable console logging
        if let Ok(console_str) = env::var("FISCUS_LOG_CONSOLE") {
            config.console_enabled = console_str.to_lowercase() == "true";
        }

        // Enable/disable file logging
        if let Ok(file_str) = env::var("FISCUS_LOG_FILE") {
            config.file_enabled = file_str.to_lowercase() == "true";
        }

        // Set log directory
        if let Ok(log_dir) = env::var("FISCUS_LOG_DIR") {
            config.log_dir = PathBuf::from(log_dir);
        }

        // Include source location in production for debugging
        if config.environment == Environment::Production {
            if let Ok(location_str) = env::var("FISCUS_LOG_LOCATION") {
                config.include_location = location_str.to_lowercase() == "true";
            }
        } else {
            config.include_location = true;
        }

        // Adjust defaults for production
        if config.environment == Environment::Production {
            config.format = LogFormat::Json;
            config.file_enabled = true;
            config.console_enabled = false;
            config.include_spans = false;
        }

        config
    }

    /// Get the environment filter string
    pub fn env_filter(&self) -> String {
        let base_level = match self.level {
            Level::TRACE => "trace",
            Level::DEBUG => "debug",
            Level::INFO => "info",
            Level::WARN => "warn",
            Level::ERROR => "error",
        };

        // Create filter that includes our app and reduces noise from dependencies
        format!(
            "fiscus={base_level},tauri={base_level},sqlx=warn,hyper=warn,reqwest=warn,rustls=warn,{base_level}"
        )
    }
}

/// Initialize the logging system
pub fn init_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = LoggingConfig::from_env();
    init_logging_with_config(config)
}

/// Initialize logging with a specific configuration
pub fn init_logging_with_config(
    config: LoggingConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let env_filter =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(config.env_filter()))?;

    // Simplified approach - just use console logging for now
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_file(config.include_location)
        .with_line_number(config.include_location);

    match config.format {
        LogFormat::Json => subscriber.json().init(),
        LogFormat::Compact => subscriber.compact().init(),
        LogFormat::Console => subscriber.pretty().init(),
    }

    tracing::info!(
        level = ?config.level,
        format = ?config.format,
        environment = ?config.environment,
        console_enabled = config.console_enabled,
        file_enabled = config.file_enabled,
        "Logging system initialized"
    );

    Ok(())
}

/// Get the current logging configuration
pub fn get_config() -> LoggingConfig {
    LoggingConfig::from_env()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert_eq!(config.format, LogFormat::Console);
        assert!(config.console_enabled);
        assert!(!config.file_enabled);
    }

    #[test]
    fn test_config_from_env() {
        env::set_var("FISCUS_LOG_LEVEL", "debug");
        env::set_var("FISCUS_LOG_FORMAT", "json");
        env::set_var("FISCUS_ENV", "production");

        let config = LoggingConfig::from_env();
        assert_eq!(config.level, Level::DEBUG);
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.environment, Environment::Production);

        // Clean up
        env::remove_var("FISCUS_LOG_LEVEL");
        env::remove_var("FISCUS_LOG_FORMAT");
        env::remove_var("FISCUS_ENV");
    }

    #[test]
    fn test_env_filter() {
        let config = LoggingConfig {
            level: Level::DEBUG,
            ..Default::default()
        };
        let filter = config.env_filter();
        assert!(filter.contains("fiscus=debug"));
        assert!(filter.contains("tauri=debug"));
    }
}
