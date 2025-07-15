# Fiscus Logging System Examples

This document provides practical examples of using the Fiscus logging system in various scenarios.

## Basic Command Logging

### Simple Command with No Parameters

```rust
use fiscus::logging::with_simple_logging;
use fiscus::error::FiscusError;

#[tauri::command]
async fn get_app_version() -> Result<String, FiscusError> {
    with_simple_logging("get_app_version", None, || async {
        Ok("1.0.0".to_string())
    }).await
}
```

### Command with User Context

```rust
#[tauri::command]
async fn get_user_profile(user_id: String) -> Result<UserProfile, FiscusError> {
    with_simple_logging("get_user_profile", Some(user_id.clone()), || async {
        // Your business logic here
        let profile = fetch_user_profile(&user_id).await?;
        Ok(profile)
    }).await
}
```

### Command with Complex Parameters

```rust
use fiscus::logging::{with_logging, ExtractUserId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CreateAccountRequest {
    user_id: String,
    account_name: String,
    account_type: String,
    initial_balance: f64,
    // Sensitive data will be automatically sanitized
    routing_number: String,
    account_number: String,
}

impl ExtractUserId for CreateAccountRequest {
    fn extract_user_id(&self) -> Option<String> {
        Some(self.user_id.clone())
    }
}

#[tauri::command]
async fn create_account(request: CreateAccountRequest) -> Result<Account, FiscusError> {
    with_logging("create_account", request, |req| async move {
        // Business logic here
        let account = create_new_account(req).await?;
        Ok(account)
    }).await
}
```

## Database Operation Logging

### Using the Enhanced DatabaseUtils

```rust
use fiscus::database::DatabaseUtils;
use fiscus::error::FiscusResult;
use tauri_plugin_sql::Database;

async fn get_user_accounts(db: &Database, user_id: &str) -> FiscusResult<Vec<Account>> {
    // This will automatically log the query with sanitized parameters
    let accounts = DatabaseUtils::execute_query::<Account>(
        db,
        "SELECT * FROM accounts WHERE user_id = ? AND deleted = false",
        vec![serde_json::Value::String(user_id.to_string())],
    ).await?;
    
    Ok(accounts)
}

async fn create_transaction_with_logging(
    db: &Database,
    transaction: &Transaction,
) -> FiscusResult<String> {
    // Transaction logging is automatic with the macro
    fiscus::with_transaction!(db, async {
        let transaction_id = DatabaseUtils::execute_non_query(
            db,
            "INSERT INTO transactions (id, user_id, amount, description) VALUES (?, ?, ?, ?)",
            vec![
                serde_json::Value::String(transaction.id.clone()),
                serde_json::Value::String(transaction.user_id.clone()),
                serde_json::Value::Number(serde_json::Number::from_f64(transaction.amount).unwrap()),
                serde_json::Value::String(transaction.description.clone()),
            ],
        ).await?;
        
        Ok(transaction.id.clone())
    })
}
```

## Error Logging Examples

### Automatic Error Logging

```rust
use fiscus::error::FiscusError;

async fn risky_operation() -> Result<String, FiscusError> {
    // Errors are automatically logged when they occur
    match perform_database_operation().await {
        Ok(result) => Ok(result),
        Err(e) => {
            // This will be automatically logged with context
            Err(FiscusError::Database(format!("Operation failed: {}", e)))
        }
    }
}
```

### Manual Error Logging with Context

```rust
async fn authenticate_user(credentials: &UserCredentials) -> Result<User, FiscusError> {
    match validate_credentials(credentials).await {
        Ok(user) => Ok(user),
        Err(e) => {
            let error = FiscusError::Authentication("Invalid credentials".to_string());
            // Log with additional context
            error.log_error(Some("user_authentication"));
            Err(error)
        }
    }
}
```

## Performance Monitoring Examples

### Accessing Performance Metrics

```rust
use fiscus::logging::get_performance_monitor;

#[tauri::command]
async fn get_performance_summary() -> Result<PerformanceSummary, FiscusError> {
    let monitor = get_performance_monitor();
    let summary = monitor.get_summary();
    Ok(summary)
}

// Periodic performance logging
async fn log_performance_periodically() {
    let monitor = get_performance_monitor();
    
    // Log summary every 5 minutes
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            monitor.log_summary();
        }
    });
}
```

## Custom Sanitization Examples

### Creating Custom Sanitizers

```rust
use fiscus::logging::DataSanitizer;
use serde_json::json;

fn sanitize_custom_data() {
    // Create a sanitizer for specific fields only
    let sanitizer = DataSanitizer::partial_sanitizer(&["internal_id", "secret_key"]);
    
    let data = json!({
        "public_info": "visible data",
        "internal_id": "secret_internal_123",
        "secret_key": "sk_live_abcdef123456",
        "user_email": "user@example.com" // Won't be sanitized with partial sanitizer
    });
    
    let sanitized = sanitizer.sanitize_json(&data);
    println!("Sanitized: {}", sanitized);
}
```

### Manual Data Sanitization

```rust
use fiscus::logging::{DataSanitizer, sanitize_serializable};

async fn log_user_action(user: &User, action: &str) {
    let sanitizer = DataSanitizer::new();
    let sanitized_user = sanitize_serializable(user, &sanitizer);
    
    tracing::info!(
        user = %sanitized_user,
        action = action,
        "User performed action"
    );
}
```

## Environment-Specific Configuration Examples

### Development Environment

```bash
# .env.development
FISCUS_LOG_LEVEL=debug
FISCUS_LOG_FORMAT=console
FISCUS_ENV=development
FISCUS_LOG_CONSOLE=true
FISCUS_LOG_FILE=false
FISCUS_LOG_LOCATION=true
```

### Production Environment

```bash
# .env.production
FISCUS_LOG_LEVEL=info
FISCUS_LOG_FORMAT=json
FISCUS_ENV=production
FISCUS_LOG_CONSOLE=false
FISCUS_LOG_FILE=true
FISCUS_LOG_DIR=/var/log/fiscus
FISCUS_LOG_LOCATION=false
```

### Testing Environment

```bash
# .env.test
FISCUS_LOG_LEVEL=warn
FISCUS_LOG_FORMAT=compact
FISCUS_ENV=test
FISCUS_LOG_CONSOLE=true
FISCUS_LOG_FILE=false
```

## Integration with External Systems

### Structured Logging for Monitoring

```rust
use tracing::{info, error};

async fn process_payment(payment: &Payment) -> Result<PaymentResult, FiscusError> {
    // Structured logging for monitoring systems
    info!(
        payment_id = %payment.id,
        user_id = %payment.user_id,
        amount = payment.amount,
        currency = %payment.currency,
        payment_method = %payment.method,
        "Payment processing started"
    );
    
    match process_payment_internal(payment).await {
        Ok(result) => {
            info!(
                payment_id = %payment.id,
                transaction_id = %result.transaction_id,
                status = %result.status,
                processing_time_ms = result.processing_time.as_millis(),
                "Payment processed successfully"
            );
            Ok(result)
        }
        Err(e) => {
            error!(
                payment_id = %payment.id,
                error = %e,
                error_code = %e.error_code(),
                "Payment processing failed"
            );
            Err(e)
        }
    }
}
```

### Health Check Logging

```rust
#[tauri::command]
async fn health_check() -> Result<HealthStatus, FiscusError> {
    with_simple_logging("health_check", None, || async {
        let mut status = HealthStatus::new();
        
        // Check database connectivity
        match check_database_health().await {
            Ok(_) => {
                status.database = "healthy".to_string();
                tracing::debug!("Database health check passed");
            }
            Err(e) => {
                status.database = "unhealthy".to_string();
                tracing::error!(error = %e, "Database health check failed");
            }
        }
        
        // Check external services
        match check_external_services().await {
            Ok(_) => {
                status.external_services = "healthy".to_string();
                tracing::debug!("External services health check passed");
            }
            Err(e) => {
                status.external_services = "unhealthy".to_string();
                tracing::warn!(error = %e, "External services health check failed");
            }
        }
        
        tracing::info!(
            database_status = %status.database,
            external_services_status = %status.external_services,
            overall_status = %status.overall_status(),
            "Health check completed"
        );
        
        Ok(status)
    }).await
}
```

## Testing with Logging

### Unit Tests with Logging

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use fiscus::logging;
    
    #[tokio::test]
    async fn test_command_with_logging() {
        // Initialize logging for tests
        let _ = logging::init();
        
        let result = create_account(CreateAccountRequest {
            user_id: "test_user".to_string(),
            account_name: "Test Account".to_string(),
            account_type: "checking".to_string(),
            initial_balance: 1000.0,
            routing_number: "123456789".to_string(),
            account_number: "9876543210".to_string(),
        }).await;
        
        assert!(result.is_ok());
        // Check logs for proper sanitization
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_transaction_flow() {
    // Set test environment
    std::env::set_var("FISCUS_ENV", "test");
    std::env::set_var("FISCUS_LOG_LEVEL", "debug");
    
    let _ = logging::init();
    
    // Test the full flow with logging
    let result = process_transaction_flow().await;
    
    assert!(result.is_ok());
    
    // Verify performance metrics were recorded
    let monitor = get_performance_monitor();
    let summary = monitor.get_summary();
    assert!(summary.commands.len() > 0);
}
```

This comprehensive set of examples should help developers understand how to effectively use the Fiscus logging system in various scenarios.
