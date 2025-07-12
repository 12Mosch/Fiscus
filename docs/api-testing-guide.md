# API Testing Guide

## Overview

This guide covers testing strategies for the Fiscus API, including unit tests for Rust commands, integration tests for the TypeScript client, and end-to-end testing patterns.

## Testing Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Unit Tests    │    │ Integration Tests│    │  E2E Tests      │
│   (Rust)        │    │  (TypeScript)    │    │  (Full Stack)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  Mock Database  │    │   Test Database  │    │  Real Database  │
│  Fast Execution │    │  Isolated Tests  │    │  User Scenarios │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Rust Command Testing

### Unit Test Setup

```rust
// src-tauri/src/commands/auth.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_database, create_test_user};
    use tauri::State;

    #[tokio::test]
    async fn test_create_user_success() {
        let db = create_test_database().await;
        let request = CreateUserRequest {
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            password: "password123".to_string(),
        };

        let result = create_user(request, State::from(&db)).await;
        
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert!(user.id.len() > 0);
    }

    #[tokio::test]
    async fn test_create_user_duplicate_username() {
        let db = create_test_database().await;
        
        // Create first user
        let request1 = CreateUserRequest {
            username: "duplicate".to_string(),
            email: None,
            password: "password123".to_string(),
        };
        create_user(request1, State::from(&db)).await.unwrap();

        // Try to create second user with same username
        let request2 = CreateUserRequest {
            username: "duplicate".to_string(),
            email: Some("different@example.com".to_string()),
            password: "password456".to_string(),
        };

        let result = create_user(request2, State::from(&db)).await;
        assert!(result.is_err());
        
        if let Err(FiscusError::Conflict(msg)) = result {
            assert!(msg.contains("Username already exists"));
        } else {
            panic!("Expected Conflict error");
        }
    }

    #[tokio::test]
    async fn test_create_user_validation_errors() {
        let db = create_test_database().await;

        // Test short username
        let request = CreateUserRequest {
            username: "ab".to_string(), // Too short
            email: None,
            password: "password123".to_string(),
        };

        let result = create_user(request, State::from(&db)).await;
        assert!(result.is_err());
        
        if let Err(FiscusError::Validation(msg)) = result {
            assert!(msg.contains("username"));
        } else {
            panic!("Expected Validation error");
        }
    }
}
```

### Account Command Tests

```rust
// src-tauri/src/commands/accounts.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_database, create_test_user, create_test_account_type};

    #[tokio::test]
    async fn test_create_account_success() {
        let db = create_test_database().await;
        let user = create_test_user(&db).await;
        let account_type = create_test_account_type(&db).await;

        let request = CreateAccountRequest {
            user_id: user.id.clone(),
            account_type_id: account_type.id.clone(),
            name: "Test Checking".to_string(),
            balance: Some(1000.00),
            currency: "USD".to_string(),
            account_number: Some("****1234".to_string()),
        };

        let result = create_account(request, State::from(&db)).await;
        
        assert!(result.is_ok());
        let account = result.unwrap();
        assert_eq!(account.name, "Test Checking");
        assert_eq!(account.balance, 1000.00);
        assert_eq!(account.currency, "USD");
        assert_eq!(account.user_id, user.id);
    }

    #[tokio::test]
    async fn test_get_accounts_with_filters() {
        let db = create_test_database().await;
        let user = create_test_user(&db).await;
        let account_type = create_test_account_type(&db).await;

        // Create multiple accounts
        for i in 1..=3 {
            let request = CreateAccountRequest {
                user_id: user.id.clone(),
                account_type_id: account_type.id.clone(),
                name: format!("Account {}", i),
                balance: Some(i as f64 * 100.0),
                currency: "USD".to_string(),
                account_number: None,
            };
            create_account(request, State::from(&db)).await.unwrap();
        }

        let filters = AccountFilters {
            user_id: user.id.clone(),
            account_type_id: None,
            is_active: Some(true),
            sort_by: Some("name".to_string()),
            sort_direction: Some(SortDirection::ASC),
            limit: Some(10),
            offset: Some(0),
        };

        let result = get_accounts(filters, State::from(&db)).await;
        assert!(result.is_ok());
        
        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 3);
        assert_eq!(accounts[0].name, "Account 1");
        assert_eq!(accounts[2].name, "Account 3");
    }

    #[tokio::test]
    async fn test_delete_account_with_transactions() {
        let db = create_test_database().await;
        let user = create_test_user(&db).await;
        let account = create_test_account(&db, &user.id).await;
        
        // Create a transaction for this account
        create_test_transaction(&db, &user.id, &account.id).await;

        let result = delete_account(account.id.clone(), user.id.clone(), State::from(&db)).await;
        
        // Should fail because account has transactions
        assert!(result.is_err());
        if let Err(FiscusError::Conflict(msg)) = result {
            assert!(msg.contains("transactions"));
        } else {
            panic!("Expected Conflict error");
        }
    }
}
```

### Transaction Command Tests

```rust
// src-tauri/src/commands/transactions.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[tokio::test]
    async fn test_create_transaction_updates_balance() {
        let db = create_test_database().await;
        let user = create_test_user(&db).await;
        let account = create_test_account(&db, &user.id).await;
        let category = create_test_category(&db, &user.id).await;

        let initial_balance = account.balance;

        let request = CreateTransactionRequest {
            user_id: user.id.clone(),
            account_id: account.id.clone(),
            category_id: Some(category.id.clone()),
            amount: -50.00,
            description: "Test expense".to_string(),
            notes: None,
            transaction_date: "2024-01-15".to_string(),
            transaction_type: TransactionType::Expense,
            status: Some(TransactionStatus::Completed),
            reference_number: None,
            payee: Some("Test Store".to_string()),
            tags: Some(vec!["test".to_string()]),
        };

        let result = create_transaction(request, State::from(&db)).await;
        assert!(result.is_ok());

        let transaction = result.unwrap();
        assert_eq!(transaction.amount, -50.00);
        assert_eq!(transaction.description, "Test expense");

        // Verify account balance was updated
        let updated_account = get_account_by_id(account.id, State::from(&db)).await.unwrap();
        assert_eq!(updated_account.balance, initial_balance - 50.00);
    }

    #[tokio::test]
    async fn test_create_transfer() {
        let db = create_test_database().await;
        let user = create_test_user(&db).await;
        let from_account = create_test_account(&db, &user.id).await;
        let to_account = create_test_account(&db, &user.id).await;

        let request = CreateTransferRequest {
            user_id: user.id.clone(),
            from_account_id: from_account.id.clone(),
            to_account_id: to_account.id.clone(),
            amount: 200.00,
            description: "Test transfer".to_string(),
            transfer_date: "2024-01-15".to_string(),
        };

        let result = create_transfer(request, State::from(&db)).await;
        assert!(result.is_ok());

        let transfer = result.unwrap();
        assert_eq!(transfer.amount, 200.00);
        assert!(transfer.from_transaction_id.len() > 0);
        assert!(transfer.to_transaction_id.len() > 0);

        // Verify both transactions were created
        let from_tx = get_transaction_by_id(transfer.from_transaction_id, State::from(&db)).await.unwrap();
        let to_tx = get_transaction_by_id(transfer.to_transaction_id, State::from(&db)).await.unwrap();

        assert_eq!(from_tx.amount, -200.00);
        assert_eq!(to_tx.amount, 200.00);
    }
}
```

### Test Utilities

```rust
// src-tauri/src/test_utils.rs
use crate::database::Database;
use crate::models::*;
use uuid::Uuid;

pub async fn create_test_database() -> Database {
    let db = Database::new(":memory:").await.expect("Failed to create test database");
    db.run_migrations().await.expect("Failed to run migrations");
    db
}

pub async fn create_test_user(db: &Database) -> User {
    let user_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    let query = r#"
        INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
    "#;
    
    db.execute(query, &[
        &user_id,
        &format!("testuser_{}", &user_id[0..8]),
        &format!("test_{}@example.com", &user_id[0..8]),
        "$2b$12$test_hash",
        &now,
        &now,
    ]).await.expect("Failed to create test user");

    User {
        id: user_id,
        username: format!("testuser_{}", &user_id[0..8]),
        email: Some(format!("test_{}@example.com", &user_id[0..8])),
        created_at: now.clone(),
        updated_at: now,
    }
}

pub async fn create_test_account_type(db: &Database) -> AccountType {
    let type_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    let query = r#"
        INSERT INTO account_types (id, name, description, is_asset, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
    "#;
    
    db.execute(query, &[
        &type_id,
        "Checking",
        "Standard checking account",
        &true,
        &now,
    ]).await.expect("Failed to create test account type");

    AccountType {
        id: type_id,
        name: "Checking".to_string(),
        description: Some("Standard checking account".to_string()),
        is_asset: true,
        created_at: now,
    }
}

pub async fn create_test_account(db: &Database, user_id: &str) -> Account {
    let account_type = create_test_account_type(db).await;
    let account_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    let query = r#"
        INSERT INTO accounts (id, user_id, account_type_id, name, balance, currency, is_active, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
    "#;
    
    db.execute(query, &[
        &account_id,
        user_id,
        &account_type.id,
        "Test Account",
        &1000.00,
        "USD",
        &true,
        &now,
        &now,
    ]).await.expect("Failed to create test account");

    Account {
        id: account_id,
        user_id: user_id.to_string(),
        account_type_id: account_type.id,
        name: "Test Account".to_string(),
        balance: 1000.00,
        currency: "USD".to_string(),
        account_number: None,
        is_active: true,
        created_at: now.clone(),
        updated_at: now,
    }
}
```

## TypeScript Client Testing

### Unit Tests with Jest/Vitest

```typescript
// src/api/__tests__/client.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { FiscusApiClient, FiscusApiError } from '../client';

// Mock Tauri invoke function
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke
}));

describe('FiscusApiClient', () => {
  let client: FiscusApiClient;

  beforeEach(() => {
    client = new FiscusApiClient();
    mockInvoke.mockClear();
  });

  describe('Authentication', () => {
    it('should create user successfully', async () => {
      const mockUser = {
        id: 'user-123',
        username: 'testuser',
        email: 'test@example.com',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z'
      };

      mockInvoke.mockResolvedValue(mockUser);

      const request = {
        username: 'testuser',
        email: 'test@example.com',
        password: 'password123'
      };

      const result = await client.createUser(request);

      expect(mockInvoke).toHaveBeenCalledWith('create_user', { request });
      expect(result).toEqual(mockUser);
    });

    it('should handle authentication errors', async () => {
      const mockError = {
        type: 'Authentication',
        message: 'Invalid credentials'
      };

      mockInvoke.mockRejectedValue(mockError);

      const request = {
        username: 'invalid',
        password: 'wrong'
      };

      await expect(client.loginUser(request)).rejects.toThrow(FiscusApiError);

      try {
        await client.loginUser(request);
      } catch (error) {
        expect(error).toBeInstanceOf(FiscusApiError);
        expect(error.code).toBe('Authentication');
        expect(error.message).toBe('Invalid credentials');
      }
    });
  });

  describe('Account Management', () => {
    it('should create account with proper parameters', async () => {
      const mockAccount = {
        id: 'account-123',
        user_id: 'user-123',
        account_type_id: 'type-123',
        name: 'Test Account',
        balance: 1000.00,
        currency: 'USD',
        is_active: true,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z'
      };

      mockInvoke.mockResolvedValue(mockAccount);

      const request = {
        user_id: 'user-123',
        account_type_id: 'type-123',
        name: 'Test Account',
        balance: 1000.00,
        currency: 'USD'
      };

      const result = await client.createAccount(request);

      expect(mockInvoke).toHaveBeenCalledWith('create_account', { request });
      expect(result).toEqual(mockAccount);
    });
  });
});
```

## React Hook Testing

### Testing Custom Hooks

```typescript
// src/hooks/__tests__/useAccounts.test.ts
import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useAccounts } from '../useAccounts';
import { apiClient } from '@/api/client';

// Mock the API client
vi.mock('@/api/client');

describe('useAccounts', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should fetch accounts successfully', async () => {
    const mockAccounts = [
      { id: 'account-1', name: 'Checking', balance: 1000 },
      { id: 'account-2', name: 'Savings', balance: 5000 }
    ];

    vi.mocked(apiClient.getAccounts).mockResolvedValue(mockAccounts);

    const { result } = renderHook(() => useAccounts('user-123'));

    expect(result.current.loading).toBe(true);
    expect(result.current.accounts).toEqual([]);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.accounts).toEqual(mockAccounts);
    expect(result.current.error).toBeNull();
    expect(apiClient.getAccounts).toHaveBeenCalledWith({
      user_id: 'user-123'
    });
  });

  it('should handle errors', async () => {
    const mockError = new Error('Failed to fetch accounts');
    vi.mocked(apiClient.getAccounts).mockRejectedValue(mockError);

    const { result } = renderHook(() => useAccounts('user-123'));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.accounts).toEqual([]);
    expect(result.current.error).toBe('Failed to fetch accounts');
  });
});
```

## End-to-End Testing

### E2E Test Setup with Playwright

```typescript
// tests/e2e/api-flows.spec.ts
import { test, expect } from '@playwright/test';

test.describe('API Integration Flows', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('/');

    // Wait for app to load
    await page.waitForSelector('[data-testid="app-loaded"]');
  });

  test('complete user registration and account creation flow', async ({ page }) => {
    // Register new user
    await page.click('[data-testid="register-button"]');
    await page.fill('[data-testid="username-input"]', 'e2euser');
    await page.fill('[data-testid="email-input"]', 'e2e@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="submit-registration"]');

    // Wait for registration success
    await expect(page.locator('[data-testid="registration-success"]')).toBeVisible();

    // Login with new credentials
    await page.fill('[data-testid="login-username"]', 'e2euser');
    await page.fill('[data-testid="login-password"]', 'password123');
    await page.click('[data-testid="login-submit"]');

    // Wait for dashboard
    await expect(page.locator('[data-testid="dashboard"]')).toBeVisible();

    // Create new account
    await page.click('[data-testid="add-account-button"]');
    await page.fill('[data-testid="account-name"]', 'E2E Test Account');
    await page.selectOption('[data-testid="account-type"]', 'checking');
    await page.fill('[data-testid="initial-balance"]', '1000');
    await page.click('[data-testid="create-account-submit"]');

    // Verify account appears in list
    await expect(page.locator('[data-testid="account-list"]')).toContainText('E2E Test Account');
    await expect(page.locator('[data-testid="account-balance"]')).toContainText('$1,000.00');
  });

  test('transaction creation and balance update', async ({ page }) => {
    // Assume user is already logged in and has an account
    await setupTestUserAndAccount(page);

    // Create a transaction
    await page.click('[data-testid="add-transaction-button"]');
    await page.selectOption('[data-testid="transaction-account"]', 'test-account');
    await page.selectOption('[data-testid="transaction-category"]', 'groceries');
    await page.fill('[data-testid="transaction-amount"]', '75.50');
    await page.fill('[data-testid="transaction-description"]', 'Weekly groceries');
    await page.selectOption('[data-testid="transaction-type"]', 'expense');
    await page.click('[data-testid="create-transaction-submit"]');

    // Verify transaction appears in list
    await expect(page.locator('[data-testid="transaction-list"]')).toContainText('Weekly groceries');
    await expect(page.locator('[data-testid="transaction-amount"]')).toContainText('-$75.50');

    // Verify account balance was updated
    await page.click('[data-testid="accounts-tab"]');
    await expect(page.locator('[data-testid="account-balance"]')).toContainText('$924.50'); // 1000 - 75.50
  });
});

async function setupTestUserAndAccount(page) {
  // Helper function to set up test data
  // This could involve API calls or database seeding
}
```

## Performance Testing

### Load Testing API Endpoints

```typescript
// tests/performance/api-load.test.ts
import { describe, it, expect } from 'vitest';
import { apiClient } from '@/api/client';

describe('API Performance Tests', () => {
  it('should handle concurrent account creation', async () => {
    const userId = 'test-user-123';
    const concurrentRequests = 10;

    const startTime = Date.now();

    const promises = Array.from({ length: concurrentRequests }, (_, i) =>
      apiClient.createAccount({
        user_id: userId,
        account_type_id: 'checking-type',
        name: `Test Account ${i}`,
        balance: 1000,
        currency: 'USD'
      })
    );

    const results = await Promise.all(promises);
    const endTime = Date.now();

    expect(results).toHaveLength(concurrentRequests);
    expect(endTime - startTime).toBeLessThan(5000); // Should complete within 5 seconds

    // Verify all accounts were created with unique IDs
    const accountIds = results.map(account => account.id);
    const uniqueIds = new Set(accountIds);
    expect(uniqueIds.size).toBe(concurrentRequests);
  });

  it('should handle large transaction queries efficiently', async () => {
    const userId = 'test-user-123';

    const startTime = Date.now();

    const transactions = await apiClient.getTransactions({
      user_id: userId,
      limit: 1000,
      sort_by: 'transaction_date',
      sort_direction: 'DESC'
    });

    const endTime = Date.now();

    expect(endTime - startTime).toBeLessThan(2000); // Should complete within 2 seconds
    expect(transactions.length).toBeLessThanOrEqual(1000);
  });
});
```

## Test Configuration

### Vitest Configuration

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    coverage: {
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/test/',
        '**/*.d.ts',
        '**/*.config.*',
        '**/routeTree.gen.ts'
      ]
    }
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  }
});
```

### Test Setup

```typescript
// src/test/setup.ts
import { vi } from 'vitest';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Global test utilities
global.testUtils = {
  createMockUser: () => ({
    id: 'test-user-123',
    username: 'testuser',
    email: 'test@example.com',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  }),

  createMockAccount: () => ({
    id: 'test-account-123',
    user_id: 'test-user-123',
    account_type_id: 'checking-type',
    name: 'Test Account',
    balance: 1000.00,
    currency: 'USD',
    is_active: true,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  })
};

```
