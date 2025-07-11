-- Fiscus Personal Finance Application - Initial Database Schema
-- This migration creates the foundational tables for the personal finance application

-- Users table (for future multi-user support)
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Account types (checking, savings, credit card, investment, etc.)
CREATE TABLE account_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    is_asset BOOLEAN NOT NULL DEFAULT 1, -- 1 for assets, 0 for liabilities
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Financial accounts
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

-- Transaction categories
CREATE TABLE categories (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT, -- Hex color code for UI
    icon TEXT, -- Icon identifier for UI
    parent_category_id TEXT, -- For subcategories
    is_income BOOLEAN NOT NULL DEFAULT 0, -- 1 for income categories, 0 for expense
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_category_id) REFERENCES categories(id)
);

-- Financial transactions
CREATE TABLE transactions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    category_id TEXT,
    amount DECIMAL(15,2) NOT NULL,
    description TEXT NOT NULL,
    notes TEXT,
    transaction_date DATETIME NOT NULL,
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('income', 'expense', 'transfer')),
    status TEXT NOT NULL DEFAULT 'completed' CHECK (status IN ('pending', 'completed', 'cancelled')),
    reference_number TEXT, -- Check number, confirmation number, etc.
    payee TEXT, -- Who the transaction was with
    tags TEXT CHECK (tags IS NULL OR json_valid(tags)), -- JSON array of tags for flexible categorization
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

-- Transfer transactions (for tracking money movement between accounts)
CREATE TABLE transfers (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    from_account_id TEXT NOT NULL,
    to_account_id TEXT NOT NULL,
    from_transaction_id TEXT NOT NULL,
    to_transaction_id TEXT NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    description TEXT,
    transfer_date DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (from_account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (to_account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (from_transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
    FOREIGN KEY (to_transaction_id) REFERENCES transactions(id) ON DELETE CASCADE
);

-- Budget periods (monthly, yearly, custom)
CREATE TABLE budget_periods (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Budget categories and limits
CREATE TABLE budgets (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    budget_period_id TEXT NOT NULL,
    category_id TEXT NOT NULL,
    allocated_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    spent_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (budget_period_id) REFERENCES budget_periods(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE,
    UNIQUE(budget_period_id, category_id)
);

-- Financial goals
CREATE TABLE goals (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    target_amount DECIMAL(15,2) NOT NULL,
    current_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    target_date DATE,
    priority INTEGER DEFAULT 1 CHECK (priority BETWEEN 1 AND 5),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'paused', 'cancelled')),
    category TEXT, -- emergency_fund, vacation, house, car, etc.
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Performance indexes for better query performance
CREATE INDEX idx_transactions_account_date ON transactions(account_id, transaction_date);
CREATE INDEX idx_transactions_category ON transactions(category_id);
CREATE INDEX idx_transactions_date ON transactions(transaction_date);
CREATE INDEX idx_transactions_user ON transactions(user_id);
CREATE INDEX idx_accounts_user ON accounts(user_id);
CREATE INDEX idx_budgets_user_period ON budgets(user_id, budget_period_id);
CREATE INDEX idx_categories_user ON categories(user_id);
CREATE INDEX idx_goals_user ON goals(user_id);

-- Insert default account types
INSERT INTO account_types (id, name, description, is_asset) VALUES
('checking', 'Checking Account', 'Standard checking account for daily transactions', 1),
('savings', 'Savings Account', 'Savings account for storing money', 1),
('credit_card', 'Credit Card', 'Credit card account', 0),
('investment', 'Investment Account', 'Investment and brokerage accounts', 1),
('loan', 'Loan Account', 'Loan and debt accounts', 0),
('cash', 'Cash', 'Physical cash', 1),
('other_asset', 'Other Asset', 'Other asset accounts', 1),
('other_liability', 'Other Liability', 'Other liability accounts', 0);

-- Insert default categories for a new user (will be associated with user_id when user is created)
-- These are common categories that most users will need
-- Note: In a real application, these would be inserted when a user is created
-- For now, we'll create them with a placeholder user_id that can be updated later
-- TODO: Inserted when a user is created