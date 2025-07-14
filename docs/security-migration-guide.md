# Security Migration Guide: Database Service to API Service

## Overview

This guide documents the migration from direct database access to the secure API service layer. This migration addresses a critical security vulnerability where the frontend was bypassing the Tauri security layer.

## Security Issue

**Problem**: The frontend TypeScript code had direct database access via `databaseService` that bypassed the Tauri API layer, potentially storing sensitive financial data in plaintext and circumventing security measures.

**Solution**: All database operations now go through secure Tauri API commands with proper encryption, authentication, and authorization.

## Migration Summary

| Old (Insecure) | New (Secure) |
|----------------|--------------|
| `databaseService` | `apiService` |
| `useDatabaseInitialization` | `useApiInitialization` |
| `useDatabaseStatus` | `useApiStatus` |
| Direct SQL queries | Encrypted Tauri commands |
| Plaintext data storage | Encrypted data with proper key management |

## Code Migration Examples

### Service Initialization

**Before (Insecure):**
```typescript
import { databaseService } from '@/lib/database';

// Initialize database service
await databaseService.initialize();
```

**After (Secure):**
```typescript
import { apiService } from '@/lib/api-service';

// Initialize API service
await apiService.initialize();
```

### React Hooks

**Before (Insecure):**
```typescript
import { 
  useDatabaseInitialization, 
  useDatabaseStatus,
  useAccounts 
} from '@/lib/database/hooks';

function MyComponent() {
  const { initialized } = useDatabaseInitialization();
  const { connected, version } = useDatabaseStatus();
  const { accounts } = useAccounts(userId);
  
  // Component logic...
}
```

**After (Secure):**
```typescript
import { 
  useApiInitialization, 
  useApiStatus,
  useAccounts 
} from '@/lib/database/hooks';

function MyComponent() {
  const { initialized } = useApiInitialization();
  const { connected } = useApiStatus();
  const { accounts } = useAccounts(userId);
  
  // Component logic...
}
```

### Account Operations

**Before (Insecure):**
```typescript
import { databaseService } from '@/lib/database';

// Create account
const account = await databaseService.accounts.create({
  user_id: 'user-123',
  account_type_id: 'checking',
  name: 'My Account',
  initial_balance: 1000,
  current_balance: 1000,
  currency: 'USD'
});

// Get accounts with type info
const accounts = await databaseService.accounts.findWithType('user-123');
```

**After (Secure):**
```typescript
import { apiService } from '@/lib/api-service';

// Create account
const account = await apiService.accounts.create({
  user_id: 'user-123',
  account_type_id: 'checking',
  name: 'My Account',
  initial_balance: 1000,
  currency: 'USD'
});

// Get accounts with type info
const accounts = await apiService.accounts.findWithType('user-123');
```

### Transaction Operations

**Before (Insecure):**
```typescript
import { databaseService } from '@/lib/database';

// Create transaction with balance update
const transaction = await databaseService.transactions.createWithBalanceUpdate({
  user_id: 'user-123',
  account_id: 'account-456',
  amount: -50.00,
  description: 'Grocery shopping',
  transaction_date: '2024-01-15',
  transaction_type: 'expense'
});

// Get transactions with details
const result = await databaseService.transactions.findWithDetails('user-123');
```

**After (Secure):**
```typescript
import { apiService } from '@/lib/api-service';

// Create transaction with balance update
const transaction = await apiService.transactions.createWithBalanceUpdate({
  user_id: 'user-123',
  account_id: 'account-456',
  amount: -50.00,
  description: 'Grocery shopping',
  transaction_date: '2024-01-15',
  transaction_type: 'expense'
});

// Get transactions with details
const result = await apiService.transactions.findWithDetails('user-123');
```

### Dashboard Data

**Before (Insecure):**
```typescript
import { useDashboard } from '@/lib/database/hooks';

function Dashboard() {
  const { dashboard, loading, error } = useDashboard('user-123');
  
  // Dashboard rendering...
}
```

**After (Secure):**
```typescript
import { useDashboard } from '@/lib/database/hooks';

function Dashboard() {
  const { dashboard, loading, error } = useDashboard('user-123');
  
  // Dashboard rendering... (no change needed - hooks updated internally)
}
```

## Security Improvements

### 1. Data Encryption
- All sensitive data is now encrypted using AES-256-GCM
- Encryption keys are properly managed and rotated
- No plaintext financial data in storage

### 2. Authentication & Authorization
- All operations require proper user authentication
- Ownership verification for all data access
- Rate limiting and security validation

### 3. Input Validation
- Comprehensive input validation on all API endpoints
- SQL injection protection through parameterized queries
- Field whitelisting for database operations

### 4. Audit Trail
- All operations are logged with sanitized data
- Performance monitoring and error tracking
- Security event logging

## Breaking Changes

### Type Changes
- `CreateAccountInput` → `CreateAccountRequest`
- `CreateTransactionInput` → `CreateTransactionRequest`
- `UpdateAccountInput` → `UpdateAccountRequest`
- `UpdateTransactionInput` → `UpdateTransactionRequest`

### Method Signature Changes
- Account operations now require `userId` parameter for security
- Transaction operations now require `userId` parameter for security
- Some utility functions moved from `dbUtils` to `apiUtils`

### Removed Features
- Direct SQL query execution
- Database connection management from frontend
- Unencrypted data access methods

## Testing the Migration

### 1. Development Testing
Use the updated `DatabaseTest` component (now `ApiServiceTest`) to verify functionality:

```typescript
// Available at /dev route in development mode
// Tests all API service operations with proper security
```

### 2. Integration Testing
```bash
# Run the test suite
npm run test

# Run specific API service tests
npm run test -- --grep "API Service"
```

### 3. Security Validation
- Verify no direct database imports in production code
- Confirm all operations go through Tauri commands
- Test encryption/decryption of sensitive data
- Validate authentication and authorization

## Rollback Plan

If issues are discovered, the old database service is still available but deprecated:

1. The old `databaseService` still exists with deprecation warnings
2. All old hooks have legacy aliases for backward compatibility
3. Direct database access can be temporarily re-enabled if needed

**Note**: The rollback should only be used in emergencies as it reintroduces the security vulnerability.

## Next Steps

1. **Monitor**: Watch for any issues in production
2. **Clean Up**: Remove deprecated code after migration is stable
3. **Documentation**: Update all documentation to reflect new patterns
4. **Training**: Ensure team understands new security patterns

## Support

For questions or issues with the migration:
1. Check the console for deprecation warnings
2. Review the API service documentation
3. Test with the development tools at `/dev`
4. Refer to the comprehensive examples in this guide
