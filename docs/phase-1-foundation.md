# Phase 1: Foundation - Implementation Guide

## Fiscus Personal Finance Management Application

### Overview

Phase 1 establishes the foundational architecture for Fiscus, focusing on data persistence, security, API design, and development infrastructure. This phase is critical as all subsequent features depend on these core systems.

**Timeline**: Months 1-2
**Priority**: High
**Dependencies**: None (Foundation phase)

---

## 1. Database Integration ✅ COMPLETED

### 1.1 Technology Selection

**Implemented**: SQLite with Tauri SQL Plugin

- **Rationale**: Lightweight, serverless, ACID compliant, perfect for desktop applications
- **Status**: ✅ Fully implemented with comprehensive schema and TypeScript integration

### 1.2 Database Schema Design

#### Core Tables Structure

```sql
-- Users table (for future multi-user support)
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Accounts table
CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('checking', 'savings', 'credit', 'investment')),
    balance DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    currency TEXT NOT NULL DEFAULT 'USD',
    account_number TEXT,
    credit_limit DECIMAL(15,2),
    is_active BOOLEAN DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Categories table
CREATE TABLE categories (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT,
    color TEXT,
    icon TEXT,
    is_system BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (parent_id) REFERENCES categories(id)
);

-- Transactions table
CREATE TABLE transactions (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    description TEXT NOT NULL,
    category_id TEXT,
    transaction_date DATE NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('income', 'expense', 'transfer')),
    status TEXT NOT NULL DEFAULT 'completed' CHECK (status IN ('pending', 'completed', 'failed')),
    merchant TEXT,
    notes TEXT,
    tags TEXT, -- JSON array of tags
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

-- Budgets table
CREATE TABLE budgets (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    category_id TEXT NOT NULL,
    name TEXT NOT NULL,
    allocated_amount DECIMAL(15,2) NOT NULL,
    period TEXT NOT NULL CHECK (period IN ('weekly', 'monthly', 'yearly')),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

-- Financial Goals table
CREATE TABLE financial_goals (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    target_amount DECIMAL(15,2) NOT NULL,
    current_amount DECIMAL(15,2) DEFAULT 0.00,
    deadline DATE,
    category TEXT,
    priority TEXT CHECK (priority IN ('low', 'medium', 'high')),
    is_active BOOLEAN DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

### 1.3 Database Indexes

```sql
-- Performance indexes
CREATE INDEX idx_transactions_account_date ON transactions(account_id, transaction_date);
CREATE INDEX idx_transactions_category ON transactions(category_id);
CREATE INDEX idx_transactions_date ON transactions(transaction_date);
CREATE INDEX idx_accounts_user ON accounts(user_id);
CREATE INDEX idx_budgets_user_period ON budgets(user_id, start_date, end_date);
CREATE INDEX idx_categories_user ON categories(user_id);
```

### 1.4 Migration System

```typescript
// src/database/migrations/001_initial_schema.ts
export const migration001 = {
  version: 1,
  name: 'initial_schema',
  up: async (db: Database) => {
    // Execute schema creation SQL
  },
  down: async (db: Database) => {
    // Rollback SQL
  }
};

// src/database/migrator.ts
export class DatabaseMigrator {
  async migrate(targetVersion?: number): Promise<void> {
    // Migration logic
  }

  async rollback(targetVersion: number): Promise<void> {
    // Rollback logic
  }
}
```

### 1.5 Data Access Layer (DAL)

```typescript
// src/database/repositories/BaseRepository.ts
export abstract class BaseRepository<T> {
  constructor(protected db: Database) {}

  abstract create(entity: Omit<T, 'id' | 'created_at' | 'updated_at'>): Promise<T>;
  abstract findById(id: string): Promise<T | null>;
  abstract update(id: string, updates: Partial<T>): Promise<T>;
  abstract delete(id: string): Promise<boolean>;
  abstract findAll(filters?: Record<string, any>): Promise<T[]>;
}

// src/database/repositories/AccountRepository.ts
export class AccountRepository extends BaseRepository<Account> {
  async findByUserId(userId: string): Promise<Account[]> {
    // Implementation
  }

  async updateBalance(accountId: string, newBalance: number): Promise<void> {
    // Implementation with transaction
  }
}
```

### 1.6 Implementation Tasks

- [x] Install and configure Tauri SQL plugin
- [x] Create database schema and migration files
- [x] Implement migration system
- [x] Create base repository pattern
- [x] Implement specific repositories for each entity
- [x] Add database connection management
- [x] Create database testing utilities
- [x] Create database seeding for development
- [ ] Add database backup and restore functionality
- [ ] Implement database performance monitoring

**Estimated Time**: 2-3 weeks
**Complexity**: High
**Dependencies**: Tauri SQL plugin setup

---

## 2. API Architecture

### 2.1 Tauri Command Structure

```typescript
// src-tauri/src/commands/mod.rs
pub mod accounts;
pub mod transactions;
pub mod budgets;
pub mod categories;
pub mod goals;
pub mod auth;

// src-tauri/src/commands/accounts.rs
use tauri::State;
use crate::database::Database;
use crate::models::Account;

#[tauri::command]
pub async fn create_account(
    account_data: CreateAccountRequest,
    db: State<'_, Database>
) -> Result<Account, String> {
    // Implementation
}

#[tauri::command]
pub async fn get_accounts(
    user_id: String,
    db: State<'_, Database>
) -> Result<Vec<Account>, String> {
    // Implementation
}

#[tauri::command]
pub async fn update_account(
    account_id: String,
    updates: UpdateAccountRequest,
    db: State<'_, Database>
) -> Result<Account, String> {
    // Implementation
}

#[tauri::command]
pub async fn delete_account(
    account_id: String,
    db: State<'_, Database>
) -> Result<bool, String> {
    // Implementation
}
```

### 2.2 Frontend API Client

```typescript
// src/api/client.ts
import { invoke } from '@tauri-apps/api/core';

export class ApiClient {
  // Account operations
  async createAccount(accountData: CreateAccountRequest): Promise<Account> {
    return invoke('create_account', { accountData });
  }

  async getAccounts(userId: string): Promise<Account[]> {
    return invoke('get_accounts', { userId });
  }

  async updateAccount(accountId: string, updates: UpdateAccountRequest): Promise<Account> {
    return invoke('update_account', { accountId, updates });
  }

  async deleteAccount(accountId: string): Promise<boolean> {
    return invoke('delete_account', { accountId });
  }

  // Transaction operations
  async createTransaction(transactionData: CreateTransactionRequest): Promise<Transaction> {
    return invoke('create_transaction', { transactionData });
  }

  async getTransactions(filters: TransactionFilters): Promise<Transaction[]> {
    return invoke('get_transactions', { filters });
  }

  // Budget operations
  async createBudget(budgetData: CreateBudgetRequest): Promise<Budget> {
    return invoke('create_budget', { budgetData });
  }

  async getBudgets(userId: string, period?: string): Promise<Budget[]> {
    return invoke('get_budgets', { userId, period });
  }
}

// Singleton instance
export const apiClient = new ApiClient();
```

### 2.3 Type-Safe API Interfaces

```typescript
// src/types/api.ts
export interface CreateAccountRequest {
  name: string;
  type: AccountType;
  balance: number;
  currency: string;
  accountNumber?: string;
  creditLimit?: number;
}

export interface UpdateAccountRequest {
  name?: string;
  balance?: number;
  creditLimit?: number;
  isActive?: boolean;
}

export interface CreateTransactionRequest {
  accountId: string;
  amount: number;
  description: string;
  categoryId?: string;
  transactionDate: string;
  type: TransactionType;
  merchant?: string;
  notes?: string;
  tags?: string[];
}

export interface TransactionFilters {
  accountId?: string;
  categoryId?: string;
  startDate?: string;
  endDate?: string;
  type?: TransactionType;
  minAmount?: number;
  maxAmount?: number;
  search?: string;
  limit?: number;
  offset?: number;
}
```

### 2.4 Error Handling

```typescript
// src/api/errors.ts
export class ApiError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode?: number
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export const handleApiError = (error: unknown): ApiError => {
  if (error instanceof ApiError) {
    return error;
  }

  if (typeof error === 'string') {
    return new ApiError(error, 'UNKNOWN_ERROR');
  }

  return new ApiError('An unexpected error occurred', 'UNKNOWN_ERROR');
};

// src/hooks/useApiCall.ts
export const useApiCall = <T>(
  apiCall: () => Promise<T>
) => {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ApiError | null>(null);

  const execute = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const result = await apiCall();
      setData(result);
      return result;
    } catch (err) {
      const apiError = handleApiError(err);
      setError(apiError);
      throw apiError;
    } finally {
      setLoading(false);
    }
  }, [apiCall]);

  return { data, loading, error, execute };
};
```

### 2.5 Implementation Tasks

- [x] Design and implement Tauri command structure
- [x] Create type-safe API interfaces
- [x] Implement frontend API client
- [x] Add comprehensive error handling
- [x] Create API testing utilities
- [x] Implement request/response logging
- [x] Add API performance monitoring
- [x] Create API documentation
- [ ] Implement API versioning strategy
- [ ] Add API rate limiting and throttling

**Estimated Time**: 2-3 weeks
**Complexity**: High
**Dependencies**: Database integration

---

## 3. Data Security Implementation

### 3.1 Encryption Strategy

```rust
// src-tauri/src/security/encryption.rs
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{rand_core::OsRng, SaltString}};

pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // Implementation
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, String> {
        // Implementation
    }
}

pub struct PasswordService;

impl PasswordService {
    pub fn hash_password(password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| e.to_string())?
            .to_string();

        Ok(password_hash)
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| e.to_string())?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
```

### 3.2 Secure Storage

```typescript
// src/security/secureStorage.ts
import { Store } from '@tauri-apps/plugin-store';

export class SecureStorage {
  private store: Store;

  constructor() {
    this.store = new Store('.secure.dat');
  }

  async setSecure(key: string, value: any): Promise<void> {
    // Encrypt value before storing
    const encrypted = await this.encrypt(JSON.stringify(value));
    await this.store.set(key, encrypted);
    await this.store.save();
  }

  async getSecure<T>(key: string): Promise<T | null> {
    const encrypted = await this.store.get<string>(key);
    if (!encrypted) return null;

    try {
      const decrypted = await this.decrypt(encrypted);
      return JSON.parse(decrypted);
    } catch {
      return null;
    }
  }

  private async encrypt(data: string): Promise<string> {
    // Use Tauri's secure encryption
    return invoke('encrypt_data', { data });
  }

  private async decrypt(encryptedData: string): Promise<string> {
    return invoke('decrypt_data', { encryptedData });
  }
}

export const secureStorage = new SecureStorage();
```

### 3.3 Authentication System

```typescript
// src/auth/authService.ts
export interface User {
  id: string;
  username: string;
  email?: string;
  createdAt: Date;
}

export interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

export class AuthService {
  async login(username: string, password: string): Promise<User> {
    const user = await invoke<User>('authenticate_user', { username, password });
    await secureStorage.setSecure('currentUser', user);
    return user;
  }

  async logout(): Promise<void> {
    await invoke('logout_user');
    await secureStorage.setSecure('currentUser', null);
  }

  async getCurrentUser(): Promise<User | null> {
    return secureStorage.getSecure<User>('currentUser');
  }

  async createUser(username: string, password: string, email?: string): Promise<User> {
    return invoke<User>('create_user', { username, password, email });
  }

  async changePassword(currentPassword: string, newPassword: string): Promise<void> {
    return invoke('change_password', { currentPassword, newPassword });
  }
}

export const authService = new AuthService();
```

### 3.4 Implementation Tasks

- [x] Implement encryption service in Rust
- [ ] Create secure storage wrapper
- [x] Implement password hashing and verification
- [x] Create authentication service
- [ ] Add session management
- [ ] Implement secure key management
- [ ] Add data field encryption for sensitive information
- [ ] Create security audit logging
- [ ] Implement secure backup and restore
- [ ] Add security testing and validation

**Estimated Time**: 2-3 weeks
**Complexity**: High
**Dependencies**: Database integration

---

## 4. Enhanced State Management

### 4.1 Zustand Store Architecture

```typescript
// src/stores/authStore.ts
interface AuthStore {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  initialize: () => Promise<void>;
}

export const useAuthStore = create<AuthStore>()(
  devtools(
    persist(
      (set, get) => ({
        user: null,
        isAuthenticated: false,
        isLoading: false,

        login: async (username: string, password: string) => {
          set({ isLoading: true });
          try {
            const user = await authService.login(username, password);
            set({ user, isAuthenticated: true, isLoading: false });
          } catch (error) {
            set({ isLoading: false });
            throw error;
          }
        },

        logout: async () => {
          await authService.logout();
          set({ user: null, isAuthenticated: false });
        },

        initialize: async () => {
          set({ isLoading: true });
          try {
            const user = await authService.getCurrentUser();
            set({ user, isAuthenticated: !!user, isLoading: false });
          } catch {
            set({ user: null, isAuthenticated: false, isLoading: false });
          }
        }
      }),
      {
        name: 'auth-storage',
        storage: createJSONStorage(() => secureStorage),
      }
    ),
    { name: 'auth-store' }
  )
);
```

### 4.2 Data Stores with Optimistic Updates

```typescript
// src/stores/accountStore.ts
interface AccountStore {
  accounts: Account[];
  loading: boolean;
  error: string | null;
  fetchAccounts: () => Promise<void>;
  createAccount: (accountData: CreateAccountRequest) => Promise<void>;
  updateAccount: (id: string, updates: UpdateAccountRequest) => Promise<void>;
  deleteAccount: (id: string) => Promise<void>;
}

export const useAccountStore = create<AccountStore>()(
  devtools(
    (set, get) => ({
      accounts: [],
      loading: false,
      error: null,

      fetchAccounts: async () => {
        set({ loading: true, error: null });
        try {
          const { user } = useAuthStore.getState();
          if (!user) throw new Error('User not authenticated');

          const accounts = await apiClient.getAccounts(user.id);
          set({ accounts, loading: false });
        } catch (error) {
          set({ error: error.message, loading: false });
        }
      },

      createAccount: async (accountData) => {
        const tempId = `temp-${Date.now()}`;
        const tempAccount: Account = {
          id: tempId,
          ...accountData,
          createdAt: new Date(),
          updatedAt: new Date(),
        };

        // Optimistic update
        set(state => ({ accounts: [...state.accounts, tempAccount] }));

        try {
          const newAccount = await apiClient.createAccount(accountData);
          set(state => ({
            accounts: state.accounts.map(acc =>
              acc.id === tempId ? newAccount : acc
            )
          }));
        } catch (error) {
          // Rollback optimistic update
          set(state => ({
            accounts: state.accounts.filter(acc => acc.id !== tempId),
            error: error.message
          }));
          throw error;
        }
      },

      updateAccount: async (id, updates) => {
        const originalAccount = get().accounts.find(acc => acc.id === id);
        if (!originalAccount) return;

        // Optimistic update
        set(state => ({
          accounts: state.accounts.map(acc =>
            acc.id === id ? { ...acc, ...updates } : acc
          )
        }));

        try {
          const updatedAccount = await apiClient.updateAccount(id, updates);
          set(state => ({
            accounts: state.accounts.map(acc =>
              acc.id === id ? updatedAccount : acc
            )
          }));
        } catch (error) {
          // Rollback optimistic update
          set(state => ({
            accounts: state.accounts.map(acc =>
              acc.id === id ? originalAccount : acc
            ),
            error: error.message
          }));
          throw error;
        }
      },

      deleteAccount: async (id) => {
        const originalAccounts = get().accounts;

        // Optimistic update
        set(state => ({
          accounts: state.accounts.filter(acc => acc.id !== id)
        }));

        try {
          await apiClient.deleteAccount(id);
        } catch (error) {
          // Rollback optimistic update
          set({ accounts: originalAccounts, error: error.message });
          throw error;
        }
      }
    }),
    { name: 'account-store' }
  )
);
```

### 4.3 Implementation Tasks

- [x] Create authentication store with secure persistence
- [x] Implement data stores for all entities (accounts, transactions, budgets, goals)
- [x] Add optimistic updates with rollback capability
- [x] Implement store synchronization and conflict resolution
- [ ] Create store debugging and devtools integration
- [ ] Add store performance monitoring
- [x] Implement store testing utilities
- [ ] Create store documentation and usage guides
- [ ] Add store migration system for schema changes
- [ ] Implement store cleanup and memory management

**Estimated Time**: 1-2 weeks
**Complexity**: Medium
**Dependencies**: API architecture

---

## 5. Testing Framework

### 5.1 Unit Testing Setup

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/test/',
        '**/*.d.ts',
        '**/*.config.*',
      ],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});

// src/test/setup.ts
import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-store', () => ({
  Store: vi.fn().mockImplementation(() => ({
    get: vi.fn(),
    set: vi.fn(),
    save: vi.fn(),
  })),
}));
```

### 5.2 Component Testing

```typescript
// src/components/__tests__/FinancialCard.test.tsx
import { render, screen } from '@testing-library/react';
import { FinancialCard } from '../dashboard/FinancialCard';

describe('FinancialCard', () => {
  it('renders financial card with basic props', () => {
    render(
      <FinancialCard
        title="Total Balance"
        value={1234.56}
      />
    );

    expect(screen.getByText('Total Balance')).toBeInTheDocument();
    expect(screen.getByText('$1,234.56')).toBeInTheDocument();
  });

  it('displays change indicator when provided', () => {
    render(
      <FinancialCard
        title="Monthly Income"
        value={5000}
        change={{
          value: 2.5,
          type: 'increase',
          period: 'last month'
        }}
      />
    );

    expect(screen.getByText('+2.5%')).toBeInTheDocument();
    expect(screen.getByText('vs last month')).toBeInTheDocument();
  });
});
```

### 5.3 Store Testing

```typescript
// src/stores/__tests__/accountStore.test.ts
import { renderHook, act } from '@testing-library/react';
import { vi } from 'vitest';
import { useAccountStore } from '../accountStore';
import { apiClient } from '../../api/client';

vi.mock('../../api/client');

describe('AccountStore', () => {
  beforeEach(() => {
    useAccountStore.getState().accounts = [];
    vi.clearAllMocks();
  });

  it('fetches accounts successfully', async () => {
    const mockAccounts = [
      { id: '1', name: 'Checking', type: 'checking', balance: 1000 }
    ];

    vi.mocked(apiClient.getAccounts).mockResolvedValue(mockAccounts);

    const { result } = renderHook(() => useAccountStore());

    await act(async () => {
      await result.current.fetchAccounts();
    });

    expect(result.current.accounts).toEqual(mockAccounts);
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBe(null);
  });

  it('handles optimistic updates correctly', async () => {
    const newAccountData = {
      name: 'New Account',
      type: 'savings' as const,
      balance: 500,
      currency: 'USD'
    };

    const createdAccount = { id: 'real-id', ...newAccountData };
    vi.mocked(apiClient.createAccount).mockResolvedValue(createdAccount);

    const { result } = renderHook(() => useAccountStore());

    await act(async () => {
      await result.current.createAccount(newAccountData);
    });

    expect(result.current.accounts).toHaveLength(1);
    expect(result.current.accounts[0]).toEqual(createdAccount);
  });
});
```

### 5.4 Integration Testing

```typescript
// src/test/integration/accountFlow.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import { App } from '../../App';
import { apiClient } from '../../api/client';

vi.mock('../../api/client');

describe('Account Management Flow', () => {
  it('creates and displays new account', async () => {
    const user = userEvent.setup();

    vi.mocked(apiClient.getAccounts).mockResolvedValue([]);
    vi.mocked(apiClient.createAccount).mockResolvedValue({
      id: '1',
      name: 'Test Account',
      type: 'checking',
      balance: 1000,
      currency: 'USD'
    });

    render(<App />);

    // Navigate to accounts page
    await user.click(screen.getByText('Accounts'));

    // Click add account button
    await user.click(screen.getByText('Add Account'));

    // Fill form
    await user.type(screen.getByLabelText('Account Name'), 'Test Account');
    await user.selectOptions(screen.getByLabelText('Account Type'), 'checking');
    await user.type(screen.getByLabelText('Initial Balance'), '1000');

    // Submit form
    await user.click(screen.getByText('Create Account'));

    // Verify account appears in list
    await waitFor(() => {
      expect(screen.getByText('Test Account')).toBeInTheDocument();
      expect(screen.getByText('$1,000.00')).toBeInTheDocument();
    });
  });
});
```

### 5.5 Implementation Tasks

- [x] Configure Vitest with React Testing Library
- [x] Create comprehensive test utilities and helpers
- [x] Implement component testing suite
- [x] Create store testing framework
- [x] Add integration testing setup
- [x] Implement API mocking and fixtures
- [x] Create test data factories
- [ ] Add visual regression testing
- [ ] Implement performance testing
- [ ] Create testing documentation and guidelines

**Estimated Time**: 1-2 weeks
**Complexity**: Medium
**Dependencies**: Component and store implementation

---

## Phase 1 Success Criteria

### Technical Deliverables

- [x] **Database**: Fully functional SQLite database with schema, migrations, and repositories
- [x] **API**: Complete Tauri command structure with type-safe frontend client
- [/] **Security**: Encryption, authentication, and secure storage implementation (Authentication ✅, Encryption ❌)
- [x] **State Management**: Enhanced Zustand stores with optimistic updates and persistence
- [/] **Testing**: Comprehensive testing framework with >80% code coverage (Framework ✅, Coverage reporting ❌)

### Quality Gates

- [x] All unit tests passing with >80% coverage
- [x] Integration tests covering critical user flows
- [ ] Security audit and penetration testing completed
- [/] Performance benchmarks established and met (Basic implementation ✅, Monitoring ❌)
- [x] Code review and documentation completed

### Documentation

- [ ] API documentation with examples
- [x] Database schema documentation
- [ ] Security implementation guide
- [/] Testing guidelines and best practices (Basic setup ✅, Guidelines ❌)
- [x] Development setup and contribution guide

**Total Estimated Time**: 6-8 weeks
**Team Size**: 2-3 developers
**Risk Level**: Medium (foundational work with clear requirements)

This foundation phase is critical for the success of all subsequent phases. The robust architecture established here will enable rapid development of user-facing features while maintaining security, performance, and maintainability standards.
