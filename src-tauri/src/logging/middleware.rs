use crate::error::FiscusError;
use crate::logging::performance::get_performance_monitor;
use crate::logging::sanitizer::DataSanitizer;
use serde_json::Value;
use std::future::Future;
use std::time::{Duration, Instant};
use tracing::{error, info, warn, Instrument};
use uuid::Uuid;

/// Request context for logging
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub command_name: String,
    pub user_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub start_time: Instant,
}

impl RequestContext {
    pub fn new(command_name: &str, user_id: Option<String>) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            command_name: command_name.to_string(),
            user_id,
            timestamp: chrono::Utc::now(),
            start_time: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Logging middleware for Tauri commands
pub struct LoggingMiddleware {
    sanitizer: DataSanitizer,
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self {
            sanitizer: DataSanitizer::new(),
        }
    }

    /// Log a command request
    pub fn log_request<T>(&self, ctx: &RequestContext, params: &T)
    where
        T: serde::Serialize,
    {
        let sanitized_params = self.sanitizer.sanitize_serializable(params);

        info!(
            request_id = %ctx.request_id,
            command = %ctx.command_name,
            user_id = ctx.user_id.as_deref(),
            timestamp = %ctx.timestamp,
            params = %sanitized_params,
            "Command request started"
        );
    }

    /// Log a successful command response
    pub fn log_success<T>(&self, ctx: &RequestContext, response: &T)
    where
        T: serde::Serialize,
    {
        let sanitized_response = self.sanitizer.sanitize_serializable(response);
        let duration = ctx.elapsed();

        // Record performance metrics
        let monitor = get_performance_monitor();
        monitor.record_command(&ctx.command_name, duration, true);

        info!(
            request_id = %ctx.request_id,
            command = %ctx.command_name,
            user_id = ctx.user_id.as_deref(),
            duration_ms = duration.as_millis(),
            response = %sanitized_response,
            "Command completed successfully"
        );
    }

    /// Log a command error
    pub fn log_error(&self, ctx: &RequestContext, error: &FiscusError) {
        let duration = ctx.elapsed();
        let sanitized_error = self.sanitizer.sanitize_error_message(&error.to_string());

        // Record performance metrics
        let monitor = get_performance_monitor();
        monitor.record_command(&ctx.command_name, duration, false);

        error!(
            request_id = %ctx.request_id,
            command = %ctx.command_name,
            user_id = ctx.user_id.as_deref(),
            duration_ms = duration.as_millis(),
            error = %sanitized_error,
            error_type = ?error,
            "Command failed with error"
        );
    }

    /// Log performance warning for slow commands
    pub fn log_performance_warning(&self, ctx: &RequestContext, threshold_ms: u64) {
        let duration = ctx.elapsed();
        if duration.as_millis() > threshold_ms as u128 {
            warn!(
                request_id = %ctx.request_id,
                command = %ctx.command_name,
                user_id = ctx.user_id.as_deref(),
                duration_ms = duration.as_millis(),
                threshold_ms = threshold_ms,
                "Command execution exceeded performance threshold"
            );
        }
    }
}

/// Macro to wrap Tauri commands with logging
#[macro_export]
macro_rules! logged_command {
    ($command_name:expr, $user_id:expr, $params:expr, $handler:expr) => {{
        use $crate::logging::middleware::{LoggingMiddleware, RequestContext};

        let middleware = LoggingMiddleware::new();
        let ctx = RequestContext::new($command_name, $user_id);

        // Create a span for this command
        let span = tracing::info_span!(
            "tauri_command",
            command = $command_name,
            request_id = %ctx.request_id,
            user_id = ctx.user_id.as_deref()
        );

        async move {
            // Log the request
            middleware.log_request(&ctx, &$params);

            // Execute the handler
            let result = $handler.await;

            // Log the result
            match &result {
                Ok(response) => {
                    middleware.log_success(&ctx, response);
                    middleware.log_performance_warning(&ctx, 1000); // 1 second threshold
                }
                Err(error) => {
                    middleware.log_error(&ctx, error);
                }
            }

            result
        }
        .instrument(span)
    }};
}

/// Trait for extracting user ID from command parameters
pub trait ExtractUserId {
    fn extract_user_id(&self) -> Option<String>;
}

/// Wrapper function for logging Tauri commands with automatic user ID extraction
pub async fn with_logging<P, R, F, Fut>(
    command_name: &str,
    params: P,
    handler: F,
) -> Result<R, FiscusError>
where
    P: serde::Serialize + ExtractUserId,
    R: serde::Serialize,
    F: FnOnce(P) -> Fut,
    Fut: Future<Output = Result<R, FiscusError>>,
{
    let middleware = LoggingMiddleware::new();
    let user_id = params.extract_user_id();
    let ctx = RequestContext::new(command_name, user_id);

    let span = tracing::info_span!(
        "tauri_command",
        command = command_name,
        request_id = %ctx.request_id,
        user_id = ctx.user_id.as_deref()
    );

    async move {
        middleware.log_request(&ctx, &params);

        let result = handler(params).await;

        match &result {
            Ok(response) => {
                middleware.log_success(&ctx, response);
                middleware.log_performance_warning(&ctx, 1000);
            }
            Err(error) => {
                middleware.log_error(&ctx, error);
            }
        }

        result
    }
    .instrument(span)
    .await
}

/// Simple wrapper for commands without complex parameter extraction
pub async fn with_simple_logging<R, F, Fut>(
    command_name: &str,
    user_id: Option<String>,
    handler: F,
) -> Result<R, FiscusError>
where
    R: serde::Serialize,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<R, FiscusError>>,
{
    let middleware = LoggingMiddleware::new();
    let ctx = RequestContext::new(command_name, user_id);

    let span = tracing::info_span!(
        "tauri_command",
        command = command_name,
        request_id = %ctx.request_id,
        user_id = ctx.user_id.as_deref()
    );

    async move {
        info!(
            request_id = %ctx.request_id,
            command = %ctx.command_name,
            user_id = ctx.user_id.as_deref(),
            "Command started"
        );

        let result = handler().await;

        match &result {
            Ok(response) => {
                middleware.log_success(&ctx, response);
                middleware.log_performance_warning(&ctx, 1000);
            }
            Err(error) => {
                middleware.log_error(&ctx, error);
            }
        }

        result
    }
    .instrument(span)
    .await
}

/// Database operation logging helper
pub struct DatabaseLogger {
    sanitizer: DataSanitizer,
}

impl DatabaseLogger {
    pub fn new() -> Self {
        Self {
            sanitizer: DataSanitizer::new(),
        }
    }

    pub fn log_query(&self, query: &str, params: &[Value], duration: Duration) {
        let sanitized_params = self.sanitizer.sanitize_sql_params(params);
        let is_slow = duration > Duration::from_millis(100); // 100ms threshold

        // Record performance metrics
        let monitor = get_performance_monitor();
        monitor.record_database_query(duration, true, is_slow);

        info!(
            query = query,
            params = ?sanitized_params,
            duration_ms = duration.as_millis(),
            is_slow = is_slow,
            "Database query executed"
        );
    }

    pub fn log_query_error(&self, query: &str, params: &[Value], error: &str) {
        let sanitized_params = self.sanitizer.sanitize_sql_params(params);
        let sanitized_error = self.sanitizer.sanitize_error_message(error);

        // Record performance metrics
        let monitor = get_performance_monitor();
        monitor.record_database_query(Duration::ZERO, false, false);

        error!(
            query = query,
            params = ?sanitized_params,
            error = %sanitized_error,
            "Database query failed"
        );
    }

    pub fn log_transaction_start(&self) {
        info!("Database transaction started");
    }

    pub fn log_transaction_commit(&self, duration: Duration) {
        // Record performance metrics
        let monitor = get_performance_monitor();
        monitor.record_transaction(true);

        info!(
            duration_ms = duration.as_millis(),
            "Database transaction committed"
        );
    }

    pub fn log_transaction_rollback(&self, reason: &str) {
        // Record performance metrics
        let monitor = get_performance_monitor();
        monitor.record_transaction(false);

        warn!(reason = reason, "Database transaction rolled back");
    }
}

impl Default for DatabaseLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::FiscusError;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct TestParams {
        user_id: String,
        password: String,
        amount: f64,
    }

    impl ExtractUserId for TestParams {
        fn extract_user_id(&self) -> Option<String> {
            Some(self.user_id.clone())
        }
    }

    #[derive(Serialize)]
    struct TestResponse {
        id: String,
        success: bool,
    }

    #[tokio::test]
    async fn test_logging_middleware() {
        let params = TestParams {
            user_id: "user123".to_string(),
            password: "secret".to_string(),
            amount: 100.0,
        };

        let result = with_logging("test_command", params, |p| async move {
            Ok(TestResponse {
                id: p.user_id,
                success: true,
            })
        })
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_error_logging() {
        let params = TestParams {
            user_id: "user123".to_string(),
            password: "secret".to_string(),
            amount: -100.0,
        };

        let result: Result<(), FiscusError> =
            with_logging("test_command", params, |_| async move {
                Err(FiscusError::Validation("Invalid amount".to_string()))
            })
            .await;

        assert!(result.is_err());
    }
}
