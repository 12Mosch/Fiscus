# Database Seeding Guide

## Overview

The Fiscus application includes a comprehensive database seeding system for development and testing purposes. This system allows you to populate the database with realistic sample data to facilitate development, testing, and demonstration of the application's features.

## Features

- **Configurable Seeding**: Choose what data to seed (users, accounts, categories, transactions, budgets, goals)
- **Realistic Data**: Generated data follows realistic patterns and relationships
- **Multiple Presets**: Pre-configured seeding options for different use cases
- **CLI Interface**: Command-line scripts for automated seeding
- **React Integration**: React hooks and components for in-app seeding
- **Safe Operations**: Clear warnings and confirmations for destructive operations

## Quick Start

### Command Line Seeding

```bash
# Seed with default configuration
npm run seed

# Clear existing data and seed fresh
npm run seed:clear

# Seed only basic data (users, accounts, categories)
npm run seed:minimal

# Seed with demo configuration
npm run seed:demo
```

### Programmatic Seeding

```typescript
import { seedDatabase, clearDatabase } from '@/lib/database/seeding';

// Seed with default options
await seedDatabase();

// Seed with custom options
await seedDatabase({
  clearExisting: true,
  includeTransactions: true,
  transactionsPerAccount: 30
});

// Clear all data
await clearDatabase();
```

## Seeding Options

### SeedOptions Interface

```typescript
interface SeedOptions {
  clearExisting?: boolean;        // Clear existing data before seeding
  includeUsers?: boolean;         // Seed users table
  includeAccounts?: boolean;      // Seed accounts table
  includeCategories?: boolean;    // Seed categories table
  includeTransactions?: boolean;  // Seed transactions table
  includeBudgets?: boolean;       // Seed budgets and budget periods
  includeGoals?: boolean;         // Seed financial goals
  transactionsPerAccount?: number; // Number of transactions per account
}
```

### Default Configuration

```typescript
const DEFAULT_SEED_OPTIONS = {
  clearExisting: false,
  includeUsers: true,
  includeAccounts: true,
  includeCategories: true,
  includeTransactions: true,
  includeBudgets: true,
  includeGoals: true,
  transactionsPerAccount: 20,
};
```

## Seeding Presets

### Available Presets

1. **Full** (`full`)
   - Clears existing data
   - Seeds all data types
   - 25 transactions per account
   - Best for: Complete fresh start

2. **Demo** (`demo`)
   - Clears existing data
   - Seeds all data types
   - 15 transactions per account
   - Best for: Demonstrations and presentations

3. **Minimal** (`minimal`)
   - Preserves existing data
   - Seeds only users, accounts, and categories
   - No transactions, budgets, or goals
   - Best for: Basic setup without overwhelming data

4. **Testing** (`testing`)
   - Clears existing data
   - Seeds users, accounts, categories, and transactions
   - 5 transactions per account
   - No budgets or goals
   - Best for: Unit and integration testing

## Generated Data

### Users
- Creates a demo user with username "demo_user"
- Email: demo@fiscus.app
- Password hash placeholder (for development only)

### Account Types
- Pre-populated in migration (checking, savings, credit_card, investment, etc.)

### Accounts
- **Main Checking**: $5,420.50 balance
- **High Yield Savings**: $15,750.00 balance
- **Rewards Credit Card**: -$1,250.75 balance (debt)
- **Investment Portfolio**: $42,350.25 balance

### Categories
**Expense Categories:**
- Food & Dining
- Transportation
- Entertainment
- Shopping
- Bills & Utilities
- Healthcare
- Education

**Income Categories:**
- Salary
- Freelance
- Investments
- Other Income

### Transactions
- Generated for the past 30 days
- 80% expenses, 20% income
- Realistic amounts and descriptions
- Properly categorized
- Associated with appropriate merchants

### Budgets
- Creates current month budget period
- Allocates budgets for major expense categories:
  - Food & Dining: $600
  - Transportation: $300
  - Entertainment: $200
  - Shopping: $400
  - Bills & Utilities: $800
  - Healthcare: $150

### Goals
- **Emergency Fund**: $15,000 target, $8,500 current
- **Vacation to Europe**: $5,000 target, $1,200 current
- **New Car Down Payment**: $8,000 target, $3,500 current
- **Home Improvement**: $12,000 target, $2,800 current

## React Integration

### useSeeding Hook

```typescript
import { useSeeding, SEEDING_PRESETS } from '@/lib/database/seeding/use-seeding';

function MyComponent() {
  const { 
    isSeeding, 
    isClearing, 
    error, 
    lastSeeded, 
    seed, 
    clear, 
    seedWithClear 
  } = useSeeding();

  const handleSeed = async () => {
    await seed(SEEDING_PRESETS.demo);
  };

  const handleClear = async () => {
    await clear();
  };

  return (
    <div>
      <button onClick={handleSeed} disabled={isSeeding}>
        {isSeeding ? 'Seeding...' : 'Seed Database'}
      </button>
      <button onClick={handleClear} disabled={isClearing}>
        {isClearing ? 'Clearing...' : 'Clear Database'}
      </button>
      {error && <div>Error: {error}</div>}
      {lastSeeded && <div>Last seeded: {lastSeeded.toLocaleString()}</div>}
    </div>
  );
}
```

### DatabaseSeeder Component

A complete UI component for database seeding operations:

```typescript
import { DatabaseSeeder } from '@/components/debug/DatabaseSeeder';

function DevelopmentPage() {
  return (
    <div>
      <h1>Development Tools</h1>
      <DatabaseSeeder />
    </div>
  );
}
```

## CLI Commands

### Available Commands

```bash
# Default seeding (preserves existing data)
npm run seed

# Clear and seed with fresh data
npm run seed:clear

# Seed only basic data
npm run seed:minimal

# Seed with demo configuration
npm run seed:demo
```

### Custom CLI Usage

You can also run the seeding script directly with custom arguments:

```bash
# Available commands: default, clear, minimal, demo, transactions-only, clear-only
npx tsx src/lib/database/seeding/seed-script.ts [command]
```

## Safety Considerations

### Development Only
- Seeding utilities are designed for development and testing
- The DatabaseSeeder component only renders in development mode
- Production builds should exclude seeding functionality

### Data Safety
- **Clear Operations**: Permanently delete all user data
- **Seed Operations**: May create duplicate data if run multiple times
- **Backup**: Always backup important data before seeding operations

### Warnings
- Clear operations cannot be undone
- Seeding may take several seconds for large datasets
- Database connections should be properly closed after operations

## Troubleshooting

### Common Issues

1. **Database Connection Errors**
   - Ensure the Tauri application is running
   - Check database permissions in capabilities
   - Verify migration has been applied

2. **Seeding Failures**
   - Check console logs for specific error messages
   - Ensure all required tables exist
   - Verify foreign key constraints

3. **Performance Issues**
   - Reduce `transactionsPerAccount` for faster seeding
   - Use minimal preset for basic testing
   - Consider seeding in smaller batches

### Debug Mode

Enable detailed logging by setting the environment variable:

```bash
DEBUG=database:seeding npm run seed
```

## Best Practices

1. **Use Appropriate Presets**: Choose the right preset for your use case
2. **Regular Clearing**: Clear data regularly during development to avoid conflicts
3. **Backup Important Data**: Always backup before destructive operations
4. **Test Seeding**: Verify seeded data meets your testing requirements
5. **Monitor Performance**: Watch for slow seeding operations and optimize as needed

## Integration with Testing

### Unit Tests

```typescript
import { seedDatabase, clearDatabase } from '@/lib/database/seeding';

describe('My Feature', () => {
  beforeEach(async () => {
    await clearDatabase();
    await seedDatabase({
      includeUsers: true,
      includeAccounts: true,
      includeTransactions: false
    });
  });

  // Your tests here
});
```

### E2E Tests

```typescript
// In your test setup
await seedDatabase(SEEDING_PRESETS.testing);
```

This seeding system provides a robust foundation for development and testing workflows, ensuring consistent and realistic data across different environments.
