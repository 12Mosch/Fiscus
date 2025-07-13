# Secure Storage Migration Guide

## Overview

This document describes the migration from in-memory secure storage to a production-grade database-backed solution in the Fiscus application.

## Problem Statement

The original implementation used an in-memory HashMap for storing encrypted data, which had several critical limitations:

- **Data Loss**: All data was lost when the process restarted
- **No Persistence**: No data survived across application sessions
- **Limited Scalability**: Memory constraints limited storage capacity
- **Security Concerns**: No audit trail or access logging

## Solution Architecture

### Database Schema

The new implementation uses a dedicated `secure_storage` table with the following features:

```sql
CREATE TABLE secure_storage (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    data_type TEXT NOT NULL,
    storage_key TEXT NOT NULL UNIQUE,
    encrypted_data TEXT NOT NULL,
    nonce TEXT NOT NULL,
    algorithm TEXT NOT NULL,
    key_id TEXT NOT NULL,
    stored_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME,
    access_count INTEGER NOT NULL DEFAULT 0,
    last_accessed_at DATETIME,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, data_type)
);
```

### Key Features

1. **Persistent Storage**: Data survives application restarts
2. **Automatic Expiration**: Optional TTL for sensitive data
3. **Access Tracking**: Monitor data access patterns
4. **Audit Trail**: Complete history of storage operations
5. **Performance Optimization**: Proper indexing for fast queries
6. **Security**: Foreign key constraints and data validation

## Migration Components

### 1. Database Repository (`SecureStorageRepository`)

- **Location**: `src-tauri/src/database/secure_storage_repository.rs`
- **Purpose**: Provides database operations for secure storage
- **Key Methods**:
  - `store()`: Store encrypted data with optional expiration
  - `retrieve()`: Retrieve non-expired data with access tracking
  - `delete()`: Securely delete data
  - `cleanup_expired()`: Remove expired entries

### 2. Service Layer (`SecureStorageService`)

- **Location**: `src-tauri/src/services/secure_storage_service.rs`
- **Purpose**: Provides automatic cleanup and monitoring
- **Features**:
  - Automatic cleanup of expired data
  - Configurable cleanup intervals
  - Storage statistics and monitoring
  - Service lifecycle management

### 3. Updated Commands

- **Location**: `src-tauri/src/commands/secure_storage.rs`
- **Changes**: Modified to use database repository instead of HashMap
- **New Commands**:
  - `secure_cleanup_expired`: Manual cleanup trigger
  - `secure_get_statistics`: Storage usage statistics

## API Compatibility

The migration maintains full API compatibility with the existing frontend code:

### Existing Commands (Updated)

```rust
// Store encrypted data
secure_store(request: SecureStoreRequest) -> SecureStoreResponse

// Retrieve encrypted data
secure_retrieve(request: SecureRetrieveRequest) -> SecureRetrieveResponse

// Delete encrypted data
secure_delete(request: SecureDeleteRequest) -> SecureDeleteResponse
```

### New Commands

```rust
// Clean up expired entries
secure_cleanup_expired() -> u64

// Get storage statistics
secure_get_statistics(user_id: Option<String>) -> Vec<HashMap<String, Value>>
```

## Security Enhancements

### 1. Data Expiration

```rust
// Store data with 24-hour expiration
let expires_at = Some(Utc::now() + Duration::hours(24));
repository.store(user_id, data_type, encrypted_data, nonce, algorithm, key_id, expires_at).await?;
```

### 2. Access Tracking

- Automatic tracking of access count and last access time
- Useful for detecting unusual access patterns
- Helps with compliance and audit requirements

### 3. Automatic Cleanup

```rust
// Configure automatic cleanup
let config = SecureStorageConfig {
    cleanup_interval_minutes: 60,  // Run every hour
    default_expiration_hours: 168, // 7 days default
    auto_cleanup_enabled: true,
};
```

## Performance Considerations

### Indexing Strategy

```sql
-- Primary indexes for fast lookups
CREATE INDEX idx_secure_storage_user_id ON secure_storage(user_id);
CREATE INDEX idx_secure_storage_data_type ON secure_storage(data_type);
CREATE INDEX idx_secure_storage_storage_key ON secure_storage(storage_key);

-- Cleanup optimization
CREATE INDEX idx_secure_storage_expires_at ON secure_storage(expires_at) WHERE expires_at IS NOT NULL;
```

### Query Optimization

- Uses UPSERT for efficient insert/update operations
- Filtered queries to exclude expired data
- Batch operations for cleanup

## Testing

### Unit Tests

- **Location**: `src-tauri/src/database/secure_storage_repository.rs` (tests module)
- **Coverage**: Store, retrieve, delete, expiration, cleanup, validation
- **Test Database**: Uses in-memory SQLite for isolation

### Integration Tests

- End-to-end testing of Tauri commands
- Service lifecycle testing
- Cleanup automation testing

## Deployment Considerations

### Migration Steps

1. **Database Migration**: Run migration `002_secure_storage.sql`
2. **Service Initialization**: Initialize secure storage service on startup
3. **Cleanup Configuration**: Configure automatic cleanup intervals
4. **Monitoring**: Set up logging and monitoring for storage operations

### Configuration

```rust
// Example service configuration
let config = SecureStorageConfig {
    cleanup_interval_minutes: 60,
    default_expiration_hours: 24 * 7, // 7 days
    max_access_attempts: 1000,
    auto_cleanup_enabled: true,
};
```

### Monitoring

- Storage usage statistics via `secure_get_statistics`
- Cleanup operation logging
- Access pattern monitoring
- Performance metrics

## Security Best Practices

1. **Data Encryption**: All data is encrypted before storage
2. **Key Management**: Proper key rotation and management
3. **Access Control**: User-based data isolation
4. **Audit Logging**: Complete operation history
5. **Secure Deletion**: Proper cleanup of expired data
6. **Validation**: Input validation and sanitization

## Troubleshooting

### Common Issues

1. **Migration Failures**: Check database permissions and schema
2. **Cleanup Not Running**: Verify service initialization
3. **Performance Issues**: Check index usage and query plans
4. **Data Not Found**: Verify expiration settings and cleanup

### Debugging

```rust
// Enable debug logging
RUST_LOG=debug cargo run

// Check storage statistics
let stats = secure_get_statistics(Some(user_id)).await?;
println!("Storage stats: {:?}", stats);
```

## Future Enhancements

1. **Encryption at Rest**: Database-level encryption
2. **Backup and Recovery**: Automated backup strategies
3. **Replication**: Multi-node storage for high availability
4. **Compression**: Data compression for large payloads
5. **Caching**: In-memory caching for frequently accessed data
