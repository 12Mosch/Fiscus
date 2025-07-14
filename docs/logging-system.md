# Fiscus Logging System Documentation

## Overview

The Fiscus application includes a comprehensive logging system built on top of the `tracing` ecosystem. This system provides structured logging with automatic request/response tracking, sensitive data sanitization, performance monitoring, and configurable output formats.

## Features

- **Structured Logging**: JSON and console output formats with configurable detail levels
- **Request/Response Tracking**: Automatic logging of Tauri command execution with correlation IDs
- **Sensitive Data Sanitization**: Automatic redaction of financial and personal information
- **Performance Monitoring**: Built-in metrics collection and performance analysis
- **Database Operation Logging**: Detailed logging of all database queries and transactions
- **Error Context**: Enhanced error logging with proper context and categorization
- **Environment-based Configuration**: Different settings for development, testing, and production

## Quick Start

### Basic Setup

The logging system is automatically initialized when the application starts:

```rust
// In main.rs or lib.rs
use fiscus::logging;

fn main() {
    // Initialize logging system
    if let Err(e) = logging::init() {
        eprintln!("Failed to initialize logging: {}", e);
    }
    
    // Your application code here
}
```

### Using Logging in Commands

```rust
use fiscus::logging::{with_simple_logging, with_logging};
use fiscus::error::FiscusError;

// Simple command logging
#[tauri::command]
async fn simple_command() -> Result<String, FiscusError> {
    with_simple_logging("simple_command", None, || async {
        Ok("Command executed successfully".to_string())
    }).await
}

// Command with user context
#[tauri::command]
async fn user_command(user_id: String, data: String) -> Result<(), FiscusError> {
    with_simple_logging("user_command", Some(user_id), || async {
        // Your command logic here
        Ok(())
    }).await
}
```

## Configuration

### Environment Variables

The logging system can be configured using environment variables:

| Variable | Description | Default | Options |
|----------|-------------|---------|---------|
| `FISCUS_LOG_LEVEL` | Log level | `info` | `trace`, `debug`, `info`, `warn`, `error` |
| `FISCUS_LOG_FORMAT` | Output format | `console` | `console`, `json`, `compact` |
| `FISCUS_ENV` | Environment | `development` | `development`, `production`, `test` |
| `FISCUS_LOG_CONSOLE` | Enable console output | `true` | `true`, `false` |
| `FISCUS_LOG_FILE` | Enable file output | `false` | `true`, `false` |
| `FISCUS_LOG_DIR` | Log file directory | `logs` | Any valid path |
| `FISCUS_LOG_LOCATION` | Include source location | `true` (dev) | `true`, `false` |

### Example Configuration

```bash
# Development environment
FISCUS_LOG_LEVEL=debug
FISCUS_LOG_FORMAT=console
FISCUS_ENV=development
FISCUS_LOG_CONSOLE=true
FISCUS_LOG_FILE=false

# Production environment
FISCUS_LOG_LEVEL=info
FISCUS_LOG_FORMAT=json
FISCUS_ENV=production
FISCUS_LOG_CONSOLE=false
FISCUS_LOG_FILE=true
FISCUS_LOG_DIR=/var/log/fiscus
```

## Sensitive Data Sanitization

The logging system automatically sanitizes sensitive information to prevent data leaks:

### Automatically Sanitized Fields

- `password`, `password_hash`
- `token`, `auth_token`, `session_token`, `api_key`
- `account_number`, `routing_number`
- `ssn`, `social_security_number`
- `credit_card`, `card_number`, `cvv`, `cvc`
- `email`, `phone`, `phone_number`
- `address`, `street_address`, `postal_code`

### Pattern-based Sanitization

The system also uses regex patterns to detect and sanitize:

- Credit card numbers: `4532-1234-5678-9012` → `[CARD-****]`
- Social Security Numbers: `123-45-6789` → `[SSN-***]`
- Email addresses: `user@example.com` → `[EMAIL-***]`
- Phone numbers: `555-123-4567` → `[PHONE-***]`

### Custom Sanitization

```rust
use fiscus::logging::DataSanitizer;

// Create a partial sanitizer for specific fields
let sanitizer = DataSanitizer::partial_sanitizer(&["custom_field", "secret_data"]);

// Sanitize data manually
let sanitized = sanitizer.sanitize_json(&your_data);
```

## Performance Monitoring

The logging system includes built-in performance monitoring:

### Metrics Collected

- **Command Metrics**: Execution times, error rates, slow command detection
- **Database Metrics**: Query performance, slow queries, transaction success rates
- **System Metrics**: Request counts, active requests, uptime

### Accessing Performance Data

```rust
use fiscus::logging::get_performance_monitor;

// Get performance summary
let monitor = get_performance_monitor();
let summary = monitor.get_summary();

// Log performance summary
monitor.log_summary();
```

## Database Logging

Database operations are automatically logged with detailed information:

```rust
use fiscus::logging::DatabaseLogger;

let logger = DatabaseLogger::new();

// Query logging is automatic when using DatabaseUtils
// But you can also log manually:
logger.log_query("SELECT * FROM users WHERE id = ?", &params, duration);
logger.log_query_error("SELECT * FROM invalid", &params, "Table not found");

// Transaction logging
logger.log_transaction_start();
logger.log_transaction_commit(duration);
logger.log_transaction_rollback("Constraint violation");
```

## Error Logging

Enhanced error logging with context and categorization:

```rust
use fiscus::error::FiscusError;

// Errors are automatically categorized and logged
let error = FiscusError::Database("Connection failed".to_string());
error.log_error(Some("user_authentication"));

// Check error criticality
if error.is_critical() {
    // Handle critical errors
}
```

## Log Output Formats

### Console Format (Development)

``` console
2024-01-15T10:30:45.123Z  INFO fiscus::commands::auth: Command request started
    with request_id: "550e8400-e29b-41d4-a716-446655440000"
    and command: "authenticate_user"
    and user_id: "user123"
    and params: {"username": "john_doe", "password": "[REDACTED]"}
```

### JSON Format (Production)

```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "INFO",
  "target": "fiscus::commands::auth",
  "message": "Command request started",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "command": "authenticate_user",
  "user_id": "user123",
  "params": {
    "username": "john_doe",
    "password": "[REDACTED]"
  }
}
```

## Best Practices

### 1. Use Structured Logging

```rust
// Good: Structured logging with context
tracing::info!(
    user_id = %user_id,
    action = "account_created",
    account_type = %account_type,
    "User account created successfully"
);

// Avoid: Plain string logging
tracing::info!("User {} created account of type {}", user_id, account_type);
```

### 2. Include Request Context

Always use the logging middleware for Tauri commands to ensure proper request tracking and correlation.

### 3. Log at Appropriate Levels

- `ERROR`: System errors, critical failures
- `WARN`: Recoverable errors, performance issues
- `INFO`: Important business events, request/response
- `DEBUG`: Detailed execution flow
- `TRACE`: Very detailed debugging information

### 4. Sanitize Sensitive Data

The system automatically sanitizes known sensitive fields, but be aware of custom sensitive data that might need manual sanitization.

## Troubleshooting

### Common Issues

1. **Logs not appearing**: Check `FISCUS_LOG_LEVEL` and ensure it's set appropriately
2. **Sensitive data in logs**: Verify field names match the sanitization patterns
3. **Performance impact**: Use appropriate log levels in production
4. **File logging not working**: Check directory permissions and `FISCUS_LOG_DIR`

### Debug Mode

Enable debug logging to troubleshoot issues:

```bash
FISCUS_LOG_LEVEL=debug cargo run
```

### Log Analysis

For production environments, consider using log aggregation tools like:

- ELK Stack (Elasticsearch, Logstash, Kibana)
- Grafana Loki
- Fluentd

## Testing

The logging system includes comprehensive tests:

```bash
# Run logging tests
cargo test logging

# Run all tests with logging output
RUST_LOG=debug cargo test
```

## Migration Guide

If upgrading from a previous logging implementation:

1. Remove old logging dependencies from `Cargo.toml`
2. Replace manual logging calls with the new middleware
3. Update environment variables to use the new naming convention
4. Test sensitive data sanitization in your specific use cases

## Support

For issues or questions about the logging system:

1. Check the test files for usage examples
2. Review the source code in `src/logging/`
3. Create an issue in the project repository
