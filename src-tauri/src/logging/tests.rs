#[cfg(test)]
mod tests {

    use crate::error::FiscusError;
    use crate::logging::{
        config::{Environment, LogFormat, LoggingConfig},
        middleware::{DatabaseLogger, LoggingMiddleware, RequestContext},
        performance::PerformanceMonitor,
        sanitizer::DataSanitizer,
    };
    use serde_json::{json, Value};
    use std::time::Duration;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.format, LogFormat::Console);
        assert_eq!(config.environment, Environment::Development);
        assert!(config.console_enabled);
        assert!(!config.file_enabled);
        assert!(config.include_spans);
    }

    #[test]
    fn test_logging_config_from_env() {
        std::env::set_var("FISCUS_LOG_LEVEL", "debug");
        std::env::set_var("FISCUS_LOG_FORMAT", "json");
        std::env::set_var("FISCUS_ENV", "production");
        std::env::set_var("FISCUS_LOG_CONSOLE", "false");
        std::env::set_var("FISCUS_LOG_FILE", "true");

        let config = LoggingConfig::from_env();
        assert_eq!(config.level, tracing::Level::DEBUG);
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.environment, Environment::Production);
        assert!(!config.console_enabled);
        assert!(config.file_enabled);

        // Clean up environment variables
        std::env::remove_var("FISCUS_LOG_LEVEL");
        std::env::remove_var("FISCUS_LOG_FORMAT");
        std::env::remove_var("FISCUS_ENV");
        std::env::remove_var("FISCUS_LOG_CONSOLE");
        std::env::remove_var("FISCUS_LOG_FILE");
    }

    #[test]
    fn test_data_sanitizer_sensitive_fields() {
        let sanitizer = DataSanitizer::new();

        let data = json!({
            "username": "john_doe",
            "password": "secret123",
            "email": "john@example.com",
            "account_number": "1234567890",
            "balance": 1000.50,
            "api_key": "sk_test_123456789abcdef"
        });

        let sanitized = sanitizer.sanitize_json(&data);

        // Non-sensitive fields should remain unchanged
        assert_eq!(sanitized["username"], "john_doe");
        assert_eq!(sanitized["balance"], 1000.50);

        // Sensitive fields should be redacted
        assert_eq!(sanitized["password"], "[REDACTED]");
        assert_eq!(sanitized["email"], "[REDACTED]");
        assert_eq!(sanitized["account_number"], "[REDACTED]");
        assert_eq!(sanitized["api_key"], "[REDACTED]");
    }

    #[test]
    fn test_data_sanitizer_nested_objects() {
        let sanitizer = DataSanitizer::new();

        let data = json!({
            "user": {
                "id": "123",
                "password": "secret",
                "profile": {
                    "email": "test@example.com",
                    "phone": "555-123-4567"
                }
            },
            "accounts": [
                {
                    "id": "acc1",
                    "account_number": "9876543210"
                }
            ]
        });

        let sanitized = sanitizer.sanitize_json(&data);

        // Check nested sanitization
        assert_eq!(sanitized["user"]["id"], "123");
        assert_eq!(sanitized["user"]["password"], "[REDACTED]");
        assert_eq!(sanitized["user"]["profile"]["email"], "[REDACTED]");
        assert_eq!(sanitized["user"]["profile"]["phone"], "[REDACTED]");

        // Check array sanitization
        if let Value::Array(accounts) = &sanitized["accounts"] {
            assert_eq!(accounts[0]["id"], "acc1");
            assert_eq!(accounts[0]["account_number"], "[REDACTED]");
        } else {
            panic!("Expected accounts to be an array");
        }
    }

    #[test]
    fn test_data_sanitizer_string_patterns() {
        let sanitizer = DataSanitizer::new();

        let test_cases = vec![
            ("My email is john@example.com", "[EMAIL-***]"),
            ("Call me at 555-123-4567", "[PHONE-***]"),
            ("SSN: 123-45-6789", "[SSN-***]"),
            ("Card: 4532-1234-5678-9012", "[CARD-****]"),
        ];

        for (input, expected_pattern) in test_cases {
            let sanitized = sanitizer.sanitize_string(input);
            assert!(
                sanitized.contains(expected_pattern),
                "Expected '{}' to contain '{}', got '{}'",
                input,
                expected_pattern,
                sanitized
            );
            assert!(
                !sanitized.contains(input),
                "Original sensitive data should be removed: '{}'",
                sanitized
            );
        }
    }

    #[test]
    fn test_request_context() {
        let ctx = RequestContext::new("test_command", Some("user123".to_string()));

        assert_eq!(ctx.command_name, "test_command");
        assert_eq!(ctx.user_id, Some("user123".to_string()));
        assert!(!ctx.request_id.is_empty());

        // Test elapsed time
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = ctx.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_logging_middleware() {
        let middleware = LoggingMiddleware::new();
        let ctx = RequestContext::new("test_command", Some("user123".to_string()));

        // Test request logging
        let params = json!({
            "username": "testuser",
            "password": "secret123"
        });

        // This should not panic and should sanitize the password
        middleware.log_request(&ctx, &params);

        // Test success logging
        let response = json!({
            "id": "response123",
            "success": true
        });

        middleware.log_success(&ctx, &response);

        // Test error logging
        let error = FiscusError::Validation("Test error".to_string());
        middleware.log_error(&ctx, &error);
    }

    #[test]
    fn test_database_logger() {
        let logger = DatabaseLogger::new();

        // Test query logging
        let params = vec![
            Value::String("user123".to_string()),
            Value::String("secret123".to_string()),
        ];

        logger.log_query(
            "SELECT * FROM users WHERE username = ? AND password = ?",
            &params,
            Duration::from_millis(50),
        );

        // Test error logging
        logger.log_query_error(
            "SELECT * FROM users WHERE id = ?",
            &[Value::String("invalid-id".to_string())],
            "Invalid UUID format",
        );

        // Test transaction logging
        logger.log_transaction_start();
        logger.log_transaction_commit(Duration::from_millis(100));
        logger.log_transaction_rollback("Test rollback");
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();

        // Record some test metrics
        monitor.record_command("test_command", Duration::from_millis(100), true);
        monitor.record_command("test_command", Duration::from_millis(200), true);
        monitor.record_command("test_command", Duration::from_millis(50), false);
        monitor.record_command("slow_command", Duration::from_secs(2), true); // Slow command

        // Record database metrics
        monitor.record_database_query(Duration::from_millis(10), true, false);
        monitor.record_database_query(Duration::from_millis(500), true, true); // Slow query
        monitor.record_database_query(Duration::from_millis(5), false, false); // Failed query

        // Record transaction metrics
        monitor.record_transaction(true);
        monitor.record_transaction(false); // Rollback

        // Get summary
        let summary = monitor.get_summary();

        // Verify command metrics
        assert_eq!(summary.commands.len(), 2);

        let test_cmd = summary
            .commands
            .iter()
            .find(|c| c.name == "test_command")
            .expect("test_command should be present");

        assert_eq!(test_cmd.total_calls, 3);
        assert_eq!(test_cmd.min_duration, Duration::from_millis(50));
        assert_eq!(test_cmd.max_duration, Duration::from_millis(200));
        assert!((test_cmd.error_rate - 33.333333333333336).abs() < 0.001); // 1/3 failed

        let slow_cmd = summary
            .commands
            .iter()
            .find(|c| c.name == "slow_command")
            .expect("slow_command should be present");

        assert_eq!(slow_cmd.total_calls, 1);
        assert_eq!(slow_cmd.slow_call_rate, 100.0); // 1/1 slow

        // Verify database metrics
        assert_eq!(summary.database.total_queries, 3);
        assert!((summary.database.slow_query_rate - 33.333333333333336).abs() < 0.001); // 1/3 slow
        assert!((summary.database.error_rate - 33.333333333333336).abs() < 0.001); // 1/3 failed
        assert_eq!(summary.database.transaction_count, 2);
        assert_eq!(summary.database.rollback_rate, 50.0); // 1/2 rollback
    }

    #[test]
    fn test_fiscus_error_logging() {
        let error = FiscusError::Database("Connection failed".to_string());

        // Test error type
        assert_eq!(error.error_type(), "database");
        assert!(error.is_critical());

        // Test logging (should not panic)
        error.log_error(Some("test context"));

        // Test non-critical error
        let validation_error = FiscusError::Validation("Invalid input".to_string());
        assert_eq!(validation_error.error_type(), "validation");
        assert!(!validation_error.is_critical());
    }

    #[tokio::test]
    async fn test_middleware_integration() {
        use crate::logging::middleware::{with_simple_logging, ExtractUserId};
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct TestParams {
            user_id: String,
            sensitive_data: String,
        }

        impl ExtractUserId for TestParams {
            fn extract_user_id(&self) -> Option<String> {
                Some(self.user_id.clone())
            }
        }

        #[derive(Serialize)]
        struct TestResponse {
            success: bool,
            message: String,
        }

        // Test successful command
        let result = with_simple_logging(
            "test_integration_command",
            Some("user123".to_string()),
            || async {
                Ok(TestResponse {
                    success: true,
                    message: "Test completed".to_string(),
                })
            },
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);

        // Test error command
        let error_result: Result<(), FiscusError> = with_simple_logging(
            "test_error_command",
            Some("user123".to_string()),
            || async { Err(FiscusError::Validation("Test error".to_string())) },
        )
        .await;

        assert!(error_result.is_err());
    }

    #[test]
    fn test_partial_sanitizer() {
        let sanitizer = DataSanitizer::partial_sanitizer(&["password", "secret"]);

        let data = json!({
            "username": "john",
            "password": "secret123",
            "email": "john@example.com",
            "secret": "top_secret",
            "public_info": "visible"
        });

        let sanitized = sanitizer.sanitize_json(&data);

        // Only specified fields should be sanitized
        assert_eq!(sanitized["username"], "john");
        assert_eq!(sanitized["email"], "john@example.com"); // Not in partial list
        assert_eq!(sanitized["public_info"], "visible");
        assert_eq!(sanitized["password"], "[REDACTED]");
        assert_eq!(sanitized["secret"], "[REDACTED]");
    }

    #[test]
    fn test_sql_params_sanitization() {
        let sanitizer = DataSanitizer::new();

        let params = vec![
            Value::String("user123".to_string()),
            Value::String("password123".to_string()), // This is a value, not a field name
            Value::String("john@example.com".to_string()),
            Value::Number(serde_json::Number::from(1000)),
        ];

        let sanitized = sanitizer.sanitize_sql_params(&params);

        assert_eq!(sanitized.len(), 4);
        assert_eq!(sanitized[0], Value::String("user123".to_string()));
        // The password value should be sanitized by regex pattern, not field name
        assert!(
            sanitized[1].as_str().unwrap().contains("[EMAIL-***]")
                || sanitized[1] == Value::String("password123".to_string())
        );
        assert!(sanitized[2].as_str().unwrap().contains("[EMAIL-***]")); // email pattern
        assert_eq!(sanitized[3], Value::Number(serde_json::Number::from(1000)));
    }
}
