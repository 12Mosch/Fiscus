# Fiscus API Reference

## Overview

Fiscus provides a comprehensive API for personal finance management through Tauri commands (Rust backend) and a TypeScript client (React frontend). This document serves as the complete API reference with practical examples.

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   React App     │    │  TypeScript API  │    │  Rust Commands  │
│   Components    │◄──►│     Client       │◄──►│   (Tauri)       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │   Type Safety    │    │   SQLite DB     │
                       │   Error Handling │    │   Validation    │
                       └──────────────────┘    └─────────────────┘
```

## Getting Started

### Basic Setup

```typescript
import { apiClient } from '@/api/client';

// All API calls return promises and include error handling
try {
  const user = await apiClient.createUser({
    username: 'john_doe',
    email: 'john@example.com',
    password: 'securePassword123'
  });
  console.log('User created:', user);
} catch (error) {
  console.error('API Error:', error.message);
}
```

### Error Handling

All API methods throw `FiscusApiError` with structured error information:

```typescript
import { FiscusApiError } from '@/api/client';

try {
  await apiClient.loginUser({ username: 'invalid', password: 'wrong' });
} catch (error) {
  if (error instanceof FiscusApiError) {
    console.error('Error Code:', error.code);
    console.error('Message:', error.message);
    console.error('Status:', error.statusCode);
  }
}
```

## Authentication API

### Create User

Creates a new user account with validation.

**Method:** `createUser(request: CreateUserRequest): Promise<User>`

**Parameters:**

- `username`: String (3-50 characters, required)
- `email`: String (optional, must be valid email)
- `password`: String (8-128 characters, required)

**Example:**

```typescript
const newUser = await apiClient.createUser({
  username: 'jane_smith',
  email: 'jane@example.com',
  password: 'mySecurePassword123'
});

console.log('User ID:', newUser.id);
console.log('Created:', newUser.created_at);
```

**Rust Command:** `create_user`

- Validates input parameters
- Checks for existing username
- Hashes password securely
- Returns user data (password excluded)

### Login User

Authenticates user credentials and returns session information.

**Method:** `loginUser(request: LoginRequest): Promise<LoginResponse>`

**Example:**

```typescript
const loginResponse = await apiClient.loginUser({
  username: 'jane_smith',
  password: 'mySecurePassword123'
});

const { user, session_token } = loginResponse;
console.log('Logged in as:', user.username);
```

### Change Password

Updates user password with current password verification.

**Method:** `changePassword(request: ChangePasswordRequest): Promise<boolean>`

**Example:**

```typescript
const success = await apiClient.changePassword({
  user_id: 'user-uuid',
  current_password: 'oldPassword',
  new_password: 'newSecurePassword123'
});

if (success) {
  console.log('Password updated successfully');
}
```

## Account Management API

### Create Account

Creates a new financial account for a user.

**Method:** `createAccount(request: CreateAccountRequest): Promise<Account>`

**Parameters:**

- `user_id`: String (UUID, required)
- `account_type_id`: String (UUID, required)
- `name`: String (1-100 characters, required)
- `balance`: Number (optional, defaults to 0)
- `currency`: String (3 characters, e.g., 'USD', required)
- `account_number`: String (optional)

**Example:**

```typescript
const account = await apiClient.createAccount({
  user_id: 'user-uuid',
  account_type_id: 'checking-type-uuid',
  name: 'My Checking Account',
  balance: 1500.00,
  currency: 'USD',
  account_number: '****1234'
});

console.log('Account created:', account.name);
console.log('Balance:', account.balance);
```

### Get Accounts with Filtering

Retrieves accounts with optional filtering and sorting.

**Method:** `getAccounts(filters: AccountFilters): Promise<Account[]>`

**Example:**

```typescript
const accounts = await apiClient.getAccounts({
  user_id: 'user-uuid',
  is_active: true,
  sort_by: 'name',
  sort_direction: 'ASC',
  limit: 10
});

accounts.forEach(account => {
  console.log(`${account.name}: ${account.balance} ${account.currency}`);
});
```

### Update Account

Updates account information with ownership validation.

**Method:** `updateAccount(accountId: string, userId: string, request: UpdateAccountRequest): Promise<Account>`

**Example:**

```typescript
const updatedAccount = await apiClient.updateAccount(
  'account-uuid',
  'user-uuid',
  {
    name: 'Updated Account Name',
    balance: 2000.00
  }
);
```

### Delete Account (Soft Delete)

Deactivates an account (sets is_active to false).

**Method:** `deleteAccount(accountId: string, userId: string): Promise<boolean>`

**Example:**

```typescript
const deleted = await apiClient.deleteAccount('account-uuid', 'user-uuid');
if (deleted) {
  console.log('Account deactivated successfully');
}
```

## Transaction Management API

### Create Transaction

Creates a new financial transaction with automatic balance updates.

**Method:** `createTransaction(request: CreateTransactionRequest): Promise<Transaction>`

**Example:**

```typescript
const transaction = await apiClient.createTransaction({
  user_id: 'user-uuid',
  account_id: 'account-uuid',
  category_id: 'groceries-uuid',
  amount: -75.50,
  description: 'Weekly grocery shopping',
  transaction_date: '2024-01-15',
  transaction_type: 'expense',
  payee: 'SuperMarket Inc',
  tags: ['groceries', 'weekly']
});
```

### Get Transactions with Advanced Filtering

Retrieves transactions with comprehensive filtering options.

**Method:** `getTransactions(filters: TransactionFilters): Promise<Transaction[]>`

**Example:**

```typescript
const transactions = await apiClient.getTransactions({
  user_id: 'user-uuid',
  account_id: 'account-uuid',
  transaction_type: 'expense',
  start_date: '2024-01-01',
  end_date: '2024-01-31',
  min_amount: 10.00,
  search: 'grocery',
  sort_by: 'transaction_date',
  sort_direction: 'DESC',
  limit: 50
});
```

### Create Transfer

Creates a transfer between two accounts (creates two linked transactions).

**Method:** `createTransfer(request: CreateTransferRequest): Promise<Transfer>`

**Example:**

```typescript
const transfer = await apiClient.createTransfer({
  user_id: 'user-uuid',
  from_account_id: 'checking-uuid',
  to_account_id: 'savings-uuid',
  amount: 500.00,
  description: 'Monthly savings transfer',
  transfer_date: '2024-01-15'
});

console.log('Transfer ID:', transfer.id);
console.log('From Transaction:', transfer.from_transaction_id);
console.log('To Transaction:', transfer.to_transaction_id);
```

## Category Management API

### Create Category

Creates a new transaction category with optional parent-child relationships.

**Method:** `createCategory(request: CreateCategoryRequest): Promise<Category>`

**Example:**

```typescript
// Create parent category
const parentCategory = await apiClient.createCategory({
  user_id: 'user-uuid',
  name: 'Food & Dining',
  description: 'All food-related expenses',
  color: '#FF6B6B',
  icon: 'utensils',
  is_income: false
});

// Create subcategory
const subCategory = await apiClient.createCategory({
  user_id: 'user-uuid',
  name: 'Groceries',
  parent_category_id: parentCategory.id,
  color: '#4ECDC4',
  icon: 'shopping-cart',
  is_income: false
});
```

### Get Category Hierarchy

Retrieves categories in hierarchical tree structure.

**Method:** `getCategoryHierarchy(userId: string, isIncome?: boolean): Promise<Category[]>`

**Example:**

```typescript
const expenseCategories = await apiClient.getCategoryHierarchy('user-uuid', false);
const incomeCategories = await apiClient.getCategoryHierarchy('user-uuid', true);

// Categories are returned with parent-child relationships preserved
expenseCategories.forEach(category => {
  console.log(`${category.name} (${category.parent_category_id ? 'Sub' : 'Main'})`);
});
```

## Budget Management API

### Create Budget Period

Creates a time period for budget tracking.

**Method:** `createBudgetPeriod(request: CreateBudgetPeriodRequest): Promise<BudgetPeriod>`

**Example:**

```typescript
const budgetPeriod = await apiClient.createBudgetPeriod({
  user_id: 'user-uuid',
  name: 'January 2024 Budget',
  start_date: '2024-01-01',
  end_date: '2024-01-31'
});
```

### Create Budget

Creates a budget allocation for a category within a period.

**Method:** `createBudget(request: CreateBudgetRequest): Promise<Budget>`

**Example:**

```typescript
const budget = await apiClient.createBudget({
  user_id: 'user-uuid',
  budget_period_id: 'period-uuid',
  category_id: 'groceries-uuid',
  allocated_amount: 400.00,
  notes: 'Monthly grocery budget'
});
```

### Get Budget Summary

Retrieves budget performance summary with spending analysis.

**Method:** `getBudgetSummary(userId: string, budgetPeriodId?: string): Promise<BudgetSummaryResponse>`

**Example:**

```typescript
const summary = await apiClient.getBudgetSummary('user-uuid', 'period-uuid');

console.log('Total Allocated:', summary.total_allocated);
console.log('Total Spent:', summary.total_spent);
console.log('Remaining:', summary.remaining);
console.log('Over Budget:', summary.over_budget);
```

## Goal Management API

### Create Goal

Creates a financial goal with target amount and optional deadline.

**Method:** `createGoal(request: CreateGoalRequest): Promise<Goal>`

**Example:**
```typescript
const goal = await apiClient.createGoal({
  user_id: 'user-uuid',
  name: 'Emergency Fund',
  description: 'Build 6-month emergency fund',
  target_amount: 10000.00,
  current_amount: 0.00,
  target_date: '2024-12-31',
  priority: 1,
  category: 'savings'
});
```

### Update Goal Progress

Adds to the current progress of a goal.

**Method:** `updateGoalProgress(goalId: string, userId: string, amount: number): Promise<Goal>`

**Example:**

```typescript
const updatedGoal = await apiClient.updateGoalProgress(
  'goal-uuid',
  'user-uuid',
  250.00 // Add $250 to current progress
);

console.log(`Progress: ${updatedGoal.current_amount}/${updatedGoal.target_amount}`);
```

## Reporting API

### Financial Overview

Gets comprehensive financial overview for a user.

**Method:** `getFinancialOverview(userId: string, startDate?: string, endDate?: string): Promise<ReportData>`

**Example:**

```typescript
const overview = await apiClient.getFinancialOverview(
  'user-uuid',
  '2024-01-01',
  '2024-01-31'
);

console.log('Net Worth:', overview.net_worth);
console.log('Monthly Income:', overview.total_income);
console.log('Monthly Expenses:', overview.total_expenses);
```

### Spending by Category

Analyzes spending patterns by category.

**Method:** `getSpendingByCategory(userId: string, startDate?: string, endDate?: string, limit?: number): Promise<ReportData[]>`

**Example:**

```typescript
const categorySpending = await apiClient.getSpendingByCategory(
  'user-uuid',
  '2024-01-01',
  '2024-01-31',
  10 // Top 10 categories
);

categorySpending.forEach(category => {
  console.log(`${category.name}: $${category.amount}`);
});
```

### Monthly Spending Trend

Gets spending trends over multiple months.

**Method:** `getMonthlySpendingTrend(userId: string, months?: number): Promise<ReportData[]>`

**Example:**

```typescript
const spendingTrend = await apiClient.getMonthlySpendingTrend('user-uuid', 12);

spendingTrend.forEach(month => {
  console.log(`${month.period}: $${month.amount}`);
});
```

## Security Considerations

### Authentication Flow

1. **User Registration**: Passwords are hashed using secure algorithms
2. **Login**: Credentials validated against hashed passwords
3. **Session Management**: Optional session tokens for extended sessions
4. **User Isolation**: All data operations scoped to authenticated user

### Data Protection

- **Local Storage**: All data stored locally in SQLite database
- **No Network Transmission**: Sensitive financial data never leaves device
- **Input Validation**: All inputs validated at both TypeScript and Rust levels
- **SQL Injection Protection**: Parameterized queries and field whitelisting

### Best Practices

```typescript
// ✅ Good: Always validate user ownership
const account = await apiClient.getAccountById('account-uuid');
if (account.user_id !== currentUserId) {
  throw new Error('Unauthorized access');
}

// ✅ Good: Handle errors gracefully
try {
  await apiClient.createTransaction(transactionData);
} catch (error) {
  if (error instanceof FiscusApiError) {
    // Handle specific API errors
    showUserFriendlyError(error.message);
  }
}

// ❌ Bad: Don't expose sensitive data in logs
console.log('User data:', user); // May contain sensitive info
```

## Error Handling Reference

### Error Types

| Error Type | Description | Common Causes |
|------------|-------------|---------------|
| `Database` | Database operation failed | Connection issues, constraint violations |
| `Validation` | Input validation failed | Invalid data format, missing required fields |
| `Authentication` | Authentication failed | Invalid credentials, expired session |
| `Authorization` | Access denied | User doesn't own resource |
| `NotFound` | Resource not found | Invalid ID, deleted resource |
| `Conflict` | Resource conflict | Duplicate username, constraint violation |
| `InvalidInput` | Invalid input data | Malformed data, type mismatch |
| `Security` | Security violation | Suspicious activity, rate limiting |
| `Internal` | Internal server error | Unexpected system error |

### Error Handling Patterns

```typescript
import { FiscusApiError } from '@/api/client';

// Pattern 1: Specific error handling
try {
  await apiClient.createUser(userData);
} catch (error) {
  if (error instanceof FiscusApiError) {
    switch (error.code) {
      case 'Validation':
        showValidationErrors(error.message);
        break;
      case 'Conflict':
        showMessage('Username already exists');
        break;
      default:
        showGenericError();
    }
  }
}

// Pattern 2: Retry logic for transient errors
async function withRetry<T>(operation: () => Promise<T>, maxRetries = 3): Promise<T> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await operation();
    } catch (error) {
      if (error instanceof FiscusApiError && error.code === 'Database' && i < maxRetries - 1) {
        await new Promise(resolve => setTimeout(resolve, 1000 * (i + 1)));
        continue;
      }
      throw error;
    }
  }
  throw new Error('Max retries exceeded');
}
```

## Performance Considerations

### Pagination

Use pagination for large datasets:

```typescript
// Get transactions with pagination
const transactions = await apiClient.getTransactions({
  user_id: 'user-uuid',
  limit: 50,
  offset: 0,
  sort_by: 'transaction_date',
  sort_direction: 'DESC'
});
```

### Filtering

Apply filters to reduce data transfer:

```typescript
// Filter at API level, not in UI
const recentExpenses = await apiClient.getTransactions({
  user_id: 'user-uuid',
  transaction_type: 'expense',
  start_date: '2024-01-01',
  limit: 20
});
```

### Caching

Consider caching frequently accessed data:

```typescript
// Example with React Query or SWR
const { data: accounts, error } = useSWR(
  ['accounts', userId],
  () => apiClient.getAccounts({ user_id: userId }),
  { revalidateOnFocus: false, staleTime: 5 * 60 * 1000 } // 5 minutes
);
```
