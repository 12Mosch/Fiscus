# Database Integration Guide

## Overview

Fiscus uses the Tauri SQL plugin with SQLite for local data storage. This document covers the database setup, schema, and usage patterns for the personal finance application.

## Architecture

### Technology Stack

- **Database**: SQLite (via Tauri SQL plugin)
- **ORM/Query Builder**: Custom TypeScript repositories
- **Migrations**: SQL-based migrations with version control
- **Connection Management**: Singleton pattern with error handling

### Key Components

- **Secure API Service**: `src/lib/api-service/index.ts`
- **React Hooks**: `src/lib/database/hooks.ts` (migrated to use API service)
- **Tauri Commands**: `src-tauri/src/commands/` (secure backend)
- **Type Definitions**: `src/types/api.ts`

## Database Schema

### Core Tables

#### Users

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

#### Account Types

```sql
CREATE TABLE account_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    is_asset BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

#### Accounts

```sql
CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    account_type_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    initial_balance DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    current_balance DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    currency TEXT NOT NULL DEFAULT 'USD',
    is_active BOOLEAN NOT NULL DEFAULT 1,
    institution_name TEXT,
    account_number TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (account_type_id) REFERENCES account_types(id)
);
```

#### Categories

```sql
CREATE TABLE categories (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,
    icon TEXT,
    parent_category_id TEXT,
    is_income BOOLEAN NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_category_id) REFERENCES categories(id)
);
```

#### Transactions

```sql
CREATE TABLE transactions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    category_id TEXT,
    amount DECIMAL(15,2) NOT NULL,
    description TEXT NOT NULL,
    notes TEXT,
    transaction_date DATE NOT NULL,
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('income', 'expense', 'transfer')),
    status TEXT NOT NULL DEFAULT 'completed' CHECK (status IN ('pending', 'completed', 'cancelled')),
    reference_number TEXT,
    payee TEXT,
    tags TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id)
);
```

### Additional Tables

- **transfers**: For tracking money movement between accounts
- **budget_periods**: For managing budget timeframes
- **budgets**: For category-based budget allocations
- **goals**: For financial goal tracking

## Configuration

### Tauri Configuration

#### tauri.conf.json

```json
{
  "plugins": {
    "sql": {
      "preload": ["sqlite:fiscus.db"]
    }
  }
}
```

#### Capabilities (src-tauri/capabilities/default.json)

```json
{
  "permissions": [
    "core:default",
    "opener:default",
    "sql:default",
    "sql:allow-execute",
    "sql:allow-select",
    "sql:allow-load",
    "sql:allow-close"
  ]
}
```

### Rust Backend Configuration

#### Cargo.toml

```toml
[dependencies]
tauri-plugin-sql = { version = "2.0", features = ["sqlite"] }
```

#### lib.rs

```rust
use tauri_plugin_sql::{Migration, MigrationKind};

let migrations = vec![Migration {
    version: 1,
    description: "create_initial_tables",
    sql: include_str!("../migrations/001_initial_schema.sql"),
    kind: MigrationKind::Up,
}];

tauri::Builder::default()
    .plugin(
        tauri_plugin_sql::Builder::default()
            .add_migrations("sqlite:fiscus.db", migrations)
            .build(),
    )
    // ... other plugins
```

## Usage Patterns

### Basic Database Operations

#### Using Secure API Service

```typescript
import { apiService } from '@/lib/api-service';

// Create an account
const newAccount = await apiService.accounts.create({
  user_id: 'user-123',
  account_type_id: 'checking',
  name: 'My Checking Account',
  initial_balance: 1000.00,
  currency: 'USD'
});

// Find accounts with type information
const accounts = await apiService.accounts.findWithType('user-123');

// Create a transaction with balance update
const transaction = await apiService.transactions.createWithBalanceUpdate({
  user_id: 'user-123',
  account_id: 'account-456',
  category_id: 'groceries',
  amount: -50.00,
  description: 'Grocery shopping',
  transaction_date: '2024-01-15',
  transaction_type: 'expense'
});
```

#### Using React Hooks (Secure)

```typescript
import { useAccounts, useTransactions, useAccountOperations } from '@/lib/database/hooks';

function AccountsPage() {
  const { accounts, loading, error, refetch } = useAccounts('user-123');
  const { createAccount, updateAccount, deleteAccount, loading: operationLoading } = useAccountOperations();

  const handleCreateAccount = async (accountData) => {
    try {
      await createAccount(accountData);
      refetch(); // Refresh the accounts list
    } catch (error) {
      console.error('Failed to create account:', error);
    }
  };

  const handleDeleteAccount = async (accountId: string) => {
    try {
      const result = await deleteAccount(accountId);
      console.log(`Account ${result.id} deleted: ${result.deleted}`);
      refetch(); // Refresh the accounts list
    } catch (error) {
      console.error('Failed to delete account:', error);
    }
  };

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;

  return (
    <div>
      {accounts.map(account => (
        <div key={account.id}>
          {account.name}: {account.current_balance}
          <button onClick={() => handleDeleteAccount(account.id)}>Delete</button>
        </div>
      ))}
    </div>
  );
}
```

### Hook Return Types

All hooks follow consistent patterns for return types and error handling.

#### Delete Operations Return Types

The `deleteAccount` and `deleteTransaction` functions return a promise that resolves to:

```typescript
{ id: string; deleted: boolean }
```

This provides:

- `id`: The ID of the deleted record
- `deleted`: Boolean indicating whether the deletion was successful

**Example Usage**:

```typescript
const { deleteAccount } = useAccountOperations();

const handleDelete = async (accountId: string) => {
  try {
    const result = await deleteAccount(accountId);
    if (result.deleted) {
      console.log(`Successfully deleted account ${result.id}`);
    } else {
      console.log(`Failed to delete account ${result.id}`);
    }
  } catch (error) {
    console.error('Delete operation failed:', error);
  }
};
```

This approach ensures type safety and prevents runtime errors that could occur with incomplete object returns.

### Advanced Queries

#### Financial Summaries

```typescript
// Get net worth
const netWorth = await apiService.accounts.getNetWorth('user-123');

// Get category spending
const categorySpending = await apiService.transactions.getCategorySpending(
  'user-123',
  '2024-01-01',
  '2024-01-31'
);

// Get monthly spending trends
const monthlySpending = await apiService.transactions.getMonthlySpending('user-123', 2024);
```

#### Dashboard Data

```typescript
import { useDashboard } from '@/lib/database/hooks';

function Dashboard() {
  const { dashboard, loading, error } = useDashboard('user-123');

  if (loading) return <div>Loading dashboard...</div>;
  if (error) return <div>Error: {error}</div>;

  return (
    <div>
      <h2>Net Worth: ${dashboard.net_worth}</h2>
      <h3>Monthly Income: ${dashboard.monthly_income}</h3>
      <h3>Monthly Expenses: ${dashboard.monthly_expenses}</h3>
      
      <h4>Recent Transactions:</h4>
      {dashboard.recent_transactions.map(tx => (
        <div key={tx.id}>{tx.description}: ${tx.amount}</div>
      ))}
    </div>
  );
}
```

## Security Considerations

### Data Protection

- All financial data is stored locally in SQLite
- Database file is protected by OS-level permissions
- No sensitive data is transmitted over network
- UUIDs used for all record identifiers

### Access Control

- API service enforces authentication and authorization
- All data access goes through validated Tauri commands
- Encryption/decryption handled transparently by the API layer
- No SQL queries exposed to frontend code
- Row Level Security through user_id filtering
- All queries scoped to authenticated user
- Prepared statements prevent SQL injection
- Input validation at TypeScript level

### Best Practices

- Always use parameterized queries
- Validate input data before database operations
- Handle errors gracefully with user-friendly messages
- Use transactions for multi-step operations
- Regular database backups (future enhancement)

## Error Handling

### API Errors

```typescript
import { FiscusApiError } from '@/api/client';

try {
  await apiService.accounts.create(accountData);
} catch (error) {
  if (error instanceof FiscusApiError) {
    console.error('API error:', error.message);
    console.error('Error code:', error.code);
    console.error('Details:', error.details);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

### API Service Management

```typescript
import { apiService, apiUtils } from '@/lib/api-service';

// Initialize API service
await apiService.initialize();

// Date formatting utilities with validation
const dateString = apiUtils.formatDate(new Date()); // "2024-01-15"
const dateTimeString = apiUtils.formatDateTime("2024-01-15T10:30:00Z"); // "2024-01-15T10:30:00.000Z"

// Generate secure IDs
const newId = apiUtils.generateId();

// Validate IDs
const isValid = apiUtils.isValidId(newId);

// Error handling for invalid dates
try {
  apiUtils.formatDate("invalid-date");
} catch (error) {
  console.error("Invalid date provided:", error.message);
}
```

## Testing

### Unit Tests

- Repository methods tested with mock data
- Hook behavior tested with React Testing Library
- Error scenarios covered with appropriate assertions

### Integration Tests

- Database operations tested end-to-end
- Migration scripts validated
- Performance benchmarks for common queries

## Migration Management

### Adding New Migrations

1. Create new SQL file in `src-tauri/migrations/`
2. Update migration array in `lib.rs`
3. Increment version number
4. Test migration on development database

### Migration Best Practices

- Always backup before running migrations
- Test migrations on copy of production data
- Include rollback procedures for complex changes
- Document schema changes in this file

## Performance Optimization

### Indexing Strategy

- Primary keys on all tables
- Foreign key indexes for joins
- Composite indexes for common query patterns
- Date-based indexes for time-series queries

### Query Optimization

- Use LIMIT for paginated results
- Avoid N+1 queries with proper joins
- Cache frequently accessed data
- Use prepared statements for repeated queries

## Future Enhancements

### Planned Features

- Database encryption at rest
- Automated backup and restore
- Data export/import functionality
- Multi-user support with proper isolation
- Real-time sync capabilities (optional cloud sync)

### Performance Improvements

- Connection pooling
- Query result caching
- Background data processing
- Optimized indexes based on usage patterns
