# Fiscus Logging System

A comprehensive, production-ready logging system for the Fiscus personal finance application built on the `tracing` ecosystem.

## Features

✅ **Structured Logging** - JSON and console output formats  
✅ **Request/Response Tracking** - Automatic correlation IDs and timing  
✅ **Sensitive Data Sanitization** - Automatic redaction of financial/personal data  
✅ **Performance Monitoring** - Built-in metrics and performance analysis  
✅ **Database Operation Logging** - Query performance and transaction tracking  
✅ **Error Context** - Enhanced error logging with categorization  
✅ **Environment Configuration** - Different settings per environment  
✅ **Comprehensive Testing** - 25+ unit and integration tests  

## Quick Start

```rust
use fiscus::logging::{init, with_simple_logging};
use fiscus::error::FiscusError;

// Initialize logging (done automatically in main)
let _ = init();

// Use in Tauri commands
#[tauri::command]
async fn my_command() -> Result<String, FiscusError> {
    with_simple_logging("my_command", None, || async {
        Ok("Success".to_string())
    }).await
}
```

## Architecture

```
src/logging/
├── mod.rs              # Main module exports
├── config.rs           # Configuration and initialization
├── middleware.rs       # Request/response logging middleware
├── sanitizer.rs        # Sensitive data sanitization
├── performance.rs      # Performance monitoring
├── tests.rs           # Comprehensive test suite
└── README.md          # This file
```

## Configuration

Configure via environment variables:

```bash
FISCUS_LOG_LEVEL=info          # trace, debug, info, warn, error
FISCUS_LOG_FORMAT=console      # console, json, compact
FISCUS_ENV=development         # development, production, test
FISCUS_LOG_CONSOLE=true        # Enable console output
FISCUS_LOG_FILE=false          # Enable file output
```

## Sensitive Data Protection

Automatically sanitizes:

- Passwords, tokens, API keys
- Account numbers, SSNs, credit cards
- Email addresses, phone numbers
- Personal information

Example:

```json
{
  "username": "john_doe",
  "password": "[REDACTED]",
  "email": "[REDACTED]",
  "account_number": "[REDACTED]"
}
```

## Performance Monitoring

Built-in metrics collection:

```rust
use fiscus::logging::get_performance_monitor;

let monitor = get_performance_monitor();
let summary = monitor.get_summary();

// Command metrics: execution times, error rates
// Database metrics: query performance, slow queries
// System metrics: request counts, uptime
```

## Database Logging

Automatic logging for all database operations:

```rust
// Queries are automatically logged with sanitized parameters
let users = DatabaseUtils::execute_query::<User>(
    db,
    "SELECT * FROM users WHERE email = ?",
    vec![Value::String(email)], // Email will be sanitized in logs
).await?;

// Transactions are tracked with timing
fiscus::with_transaction!(db, async {
    // Your transaction code
    Ok(result)
})
```

## Error Handling

Enhanced error logging with context:

```rust
use fiscus::error::FiscusError;

let error = FiscusError::Database("Connection failed".to_string());
error.log_error(Some("user_authentication")); // Logs with context

// Automatic categorization
if error.is_critical() {
    // Handle critical errors
}
```

## Testing

Run the comprehensive test suite:

```bash
cargo test logging
```

Tests cover:

- Configuration loading
- Data sanitization (field names and regex patterns)
- Request/response middleware
- Performance monitoring
- Database logging
- Error handling
- Integration scenarios

## Log Output Examples

### Development (Console)

``` bash
2024-01-15T10:30:45.123Z  INFO fiscus::commands: Command request started
    with request_id: "550e8400-e29b-41d4-a716-446655440000"
    and command: "create_account"
    and params: {"name": "Savings", "balance": 1000.0, "account_number": "[REDACTED]"}
```

### Production (JSON)

```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "INFO",
  "target": "fiscus::commands",
  "message": "Command request started",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "command": "create_account",
  "duration_ms": 45,
  "params": {
    "name": "Savings",
    "balance": 1000.0,
    "account_number": "[REDACTED]"
  }
}
```

## Security Considerations

- **No sensitive data in logs**: Automatic sanitization prevents data leaks
- **Configurable verbosity**: Production uses minimal logging by default
- **Structured output**: JSON format for secure log aggregation
- **Field validation**: SQL injection protection in database logging

## Performance Impact

- **Minimal overhead**: Async logging with efficient serialization
- **Configurable levels**: Debug logging disabled in production
- **Smart sampling**: Performance monitoring uses lightweight metrics
- **Memory efficient**: Streaming JSON output, no large buffers

## Integration

Works seamlessly with:

- **Tauri commands**: Automatic request/response tracking
- **Database operations**: Built into DatabaseUtils
- **Error handling**: Enhanced FiscusError logging
- **Performance monitoring**: Real-time metrics collection

## Dependencies

- `tracing` - Core logging framework
- `tracing-subscriber` - Log formatting and filtering
- `serde_json` - JSON serialization for structured logs
- `uuid` - Request correlation IDs
- `regex` - Pattern-based data sanitization

## Documentation

- [Full Documentation](../../docs/logging-system.md)
- [Usage Examples](../../docs/logging-examples.md)
- [API Reference](https://docs.rs/tracing)

## Contributing

When adding new features:

1. Add appropriate logging to new commands
2. Update sanitization patterns for new sensitive fields
3. Add tests for new functionality
4. Update documentation

## License

Part of the Fiscus personal finance application.
