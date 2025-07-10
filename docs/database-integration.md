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
- **Connection Layer**: `src/lib/database/connection.ts`
- **Type Definitions**: `src/lib/database/types.ts`
- **Repository Pattern**: `src/lib/database/repositories/`
- **React Hooks**: `src/lib/database/hooks.ts`
- **Service Layer**: `src/lib/database/index.ts`

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

#### Using Repository Pattern
```typescript
import { databaseService } from '@/lib/database';

// Create an account
const newAccount = await databaseService.accounts.create({
  user_id: 'user-123',
  account_type_id: 'checking',
  name: 'My Checking Account',
  initial_balance: 1000.00,
  current_balance: 1000.00,
  currency: 'USD'
});

// Find accounts with type information
const accounts = await databaseService.accounts.findWithType('user-123');

// Create a transaction with balance update
const transaction = await databaseService.transactions.createWithBalanceUpdate({
  user_id: 'user-123',
  account_id: 'account-456',
  category_id: 'groceries',
  amount: -50.00,
  description: 'Grocery shopping',
  transaction_date: '2024-01-15',
  transaction_type: 'expense'
});
```

#### Using React Hooks
```typescript
import { useAccounts, useTransactions, useAccountOperations } from '@/lib/database/hooks';

function AccountsPage() {
  const { accounts, loading, error, refetch } = useAccounts('user-123');
  const { createAccount, updateAccount, loading: operationLoading } = useAccountOperations();

  const handleCreateAccount = async (accountData) => {
    try {
      await createAccount(accountData);
      refetch(); // Refresh the accounts list
    } catch (error) {
      console.error('Failed to create account:', error);
    }
  };

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;

  return (
    <div>
      {accounts.map(account => (
        <div key={account.id}>{account.name}: {account.current_balance}</div>
      ))}
    </div>
  );
}
```

### Advanced Queries

#### Financial Summaries
```typescript
// Get net worth
const netWorth = await databaseService.accounts.getNetWorth('user-123');

// Get category spending
const categorySpending = await databaseService.transactions.getCategorySpending(
  'user-123',
  '2024-01-01',
  '2024-01-31'
);

// Get monthly spending trends
const monthlySpending = await databaseService.transactions.getMonthlySpending('user-123', 2024);
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

### Database Errors
```typescript
import { DatabaseError } from '@/lib/database';

try {
  await databaseService.accounts.create(accountData);
} catch (error) {
  if (error instanceof DatabaseError) {
    console.error('Database error:', error.message);
    console.error('Error code:', error.code);
    console.error('Details:', error.details);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

### Connection Management
```typescript
import { isDatabaseConnected, getDatabaseVersion } from '@/lib/database';

// Check connection status
const isConnected = await isDatabaseConnected();

// Get database version for debugging
const version = await getDatabaseVersion();

// Initialize database service
await databaseService.initialize();
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
