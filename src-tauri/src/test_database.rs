use mockall::mock;
use serde_json::Value;
use std::collections::HashMap;
use tempfile::NamedTempFile;

use crate::error::{FiscusError, FiscusResult};

mock! {
    /// Mock database for testing
    pub Database {}

    impl Clone for Database {
        fn clone(&self) -> Self;
    }
}

/// Test database setup utilities
#[allow(dead_code)]
pub struct TestDatabase {
    pub temp_file: Option<NamedTempFile>,
    pub db_url: String,
}

#[allow(dead_code)]
impl TestDatabase {
    /// Create a new in-memory SQLite database for testing
    pub async fn new_in_memory() -> FiscusResult<Self> {
        Ok(Self {
            temp_file: None,
            db_url: "sqlite::memory:".to_string(),
        })
    }

    /// Create a new temporary file-based SQLite database for testing
    pub async fn new_temp_file() -> FiscusResult<Self> {
        let temp_file = NamedTempFile::new()
            .map_err(|e| FiscusError::Internal(format!("Failed to create temp file: {e}")))?;

        let db_url = format!("sqlite:{}", temp_file.path().display());

        Ok(Self {
            temp_file: Some(temp_file),
            db_url,
        })
    }

    /// Initialize the test database with schema
    pub async fn init_schema(&self) -> FiscusResult<()> {
        // This would normally use the actual database connection
        // For now, we'll simulate schema initialization
        Ok(())
    }

    /// Seed the database with test data
    pub async fn seed_test_data(&self) -> FiscusResult<TestDataSet> {
        let test_data = TestDataSet::new();
        // In a real implementation, this would insert the test data into the database
        Ok(test_data)
    }

    /// Clean up the database
    pub async fn cleanup(&self) -> FiscusResult<()> {
        // Cleanup is automatic for temp files and in-memory databases
        Ok(())
    }
}

/// Test data set for consistent test data across tests
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestDataSet {
    pub users: Vec<TestUser>,
    pub accounts: Vec<TestAccount>,
    pub categories: Vec<TestCategory>,
    pub transactions: Vec<TestTransaction>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestAccount {
    pub id: String,
    pub user_id: String,
    pub account_type_id: String,
    pub name: String,
    pub balance: rust_decimal::Decimal,
    pub currency: String,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestCategory {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub is_income: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestTransaction {
    pub id: String,
    pub user_id: String,
    pub account_id: String,
    pub category_id: Option<String>,
    pub amount: rust_decimal::Decimal,
    pub description: String,
    pub transaction_type: String,
    pub status: String,
}

#[allow(dead_code)]
impl TestDataSet {
    pub fn new() -> Self {
        use rust_decimal::Decimal;
        use uuid::Uuid;

        let user1_id = Uuid::new_v4().to_string();
        let user2_id = Uuid::new_v4().to_string();

        let account1_id = Uuid::new_v4().to_string();
        let account2_id = Uuid::new_v4().to_string();
        let account3_id = Uuid::new_v4().to_string();

        let category1_id = Uuid::new_v4().to_string();
        let category2_id = Uuid::new_v4().to_string();
        let category3_id = Uuid::new_v4().to_string();

        Self {
            users: vec![
                TestUser {
                    id: user1_id.clone(),
                    username: "testuser1".to_string(),
                    email: Some("test1@example.com".to_string()),
                    password_hash: "$argon2id$v=19$m=65536,t=3,p=4$test_salt$test_hash".to_string(),
                },
                TestUser {
                    id: user2_id.clone(),
                    username: "testuser2".to_string(),
                    email: Some("test2@example.com".to_string()),
                    password_hash: "$argon2id$v=19$m=65536,t=3,p=4$test_salt$test_hash".to_string(),
                },
            ],
            accounts: vec![
                TestAccount {
                    id: account1_id.clone(),
                    user_id: user1_id.clone(),
                    account_type_id: "checking".to_string(),
                    name: "Test Checking".to_string(),
                    balance: Decimal::new(100000, 2), // $1000.00
                    currency: "USD".to_string(),
                    is_active: true,
                },
                TestAccount {
                    id: account2_id.clone(),
                    user_id: user1_id.clone(),
                    account_type_id: "savings".to_string(),
                    name: "Test Savings".to_string(),
                    balance: Decimal::new(500000, 2), // $5000.00
                    currency: "USD".to_string(),
                    is_active: true,
                },
                TestAccount {
                    id: account3_id.clone(),
                    user_id: user2_id.clone(),
                    account_type_id: "checking".to_string(),
                    name: "User2 Checking".to_string(),
                    balance: Decimal::new(75000, 2), // $750.00
                    currency: "USD".to_string(),
                    is_active: true,
                },
            ],
            categories: vec![
                TestCategory {
                    id: category1_id.clone(),
                    user_id: user1_id.clone(),
                    name: "Groceries".to_string(),
                    is_income: false,
                    is_active: true,
                },
                TestCategory {
                    id: category2_id.clone(),
                    user_id: user1_id.clone(),
                    name: "Salary".to_string(),
                    is_income: true,
                    is_active: true,
                },
                TestCategory {
                    id: category3_id.clone(),
                    user_id: user2_id.clone(),
                    name: "Entertainment".to_string(),
                    is_income: false,
                    is_active: true,
                },
            ],
            transactions: vec![
                TestTransaction {
                    id: Uuid::new_v4().to_string(),
                    user_id: user1_id.clone(),
                    account_id: account1_id.clone(),
                    category_id: Some(category1_id.clone()),
                    amount: Decimal::new(-5000, 2), // -$50.00
                    description: "Grocery shopping".to_string(),
                    transaction_type: "expense".to_string(),
                    status: "completed".to_string(),
                },
                TestTransaction {
                    id: Uuid::new_v4().to_string(),
                    user_id: user1_id.clone(),
                    account_id: account1_id.clone(),
                    category_id: Some(category2_id.clone()),
                    amount: Decimal::new(300000, 2), // $3000.00
                    description: "Monthly salary".to_string(),
                    transaction_type: "income".to_string(),
                    status: "completed".to_string(),
                },
                TestTransaction {
                    id: Uuid::new_v4().to_string(),
                    user_id: user2_id.clone(),
                    account_id: account3_id.clone(),
                    category_id: Some(category3_id.clone()),
                    amount: Decimal::new(-2500, 2), // -$25.00
                    description: "Movie tickets".to_string(),
                    transaction_type: "expense".to_string(),
                    status: "completed".to_string(),
                },
            ],
        }
    }

    /// Get a user by username
    pub fn get_user_by_username(&self, username: &str) -> Option<&TestUser> {
        self.users.iter().find(|u| u.username == username)
    }

    /// Get accounts for a user
    pub fn get_accounts_for_user(&self, user_id: &str) -> Vec<&TestAccount> {
        self.accounts
            .iter()
            .filter(|a| a.user_id == user_id)
            .collect()
    }

    /// Get categories for a user
    pub fn get_categories_for_user(&self, user_id: &str) -> Vec<&TestCategory> {
        self.categories
            .iter()
            .filter(|c| c.user_id == user_id)
            .collect()
    }

    /// Get transactions for a user
    pub fn get_transactions_for_user(&self, user_id: &str) -> Vec<&TestTransaction> {
        self.transactions
            .iter()
            .filter(|t| t.user_id == user_id)
            .collect()
    }

    /// Get transactions for an account
    pub fn get_transactions_for_account(&self, account_id: &str) -> Vec<&TestTransaction> {
        self.transactions
            .iter()
            .filter(|t| t.account_id == account_id)
            .collect()
    }
}

/// Mock database utilities for testing
pub struct MockDatabaseUtils;

#[allow(dead_code)]
impl MockDatabaseUtils {
    /// Create a mock database response for a single row query
    pub fn mock_single_row_response(
        data: Option<HashMap<String, Value>>,
    ) -> FiscusResult<Option<HashMap<String, Value>>> {
        Ok(data)
    }

    /// Create a mock database response for a multi-row query
    pub fn mock_multi_row_response(
        data: Vec<HashMap<String, Value>>,
    ) -> FiscusResult<Vec<HashMap<String, Value>>> {
        Ok(data)
    }

    /// Create a mock database response for a non-query operation
    pub fn mock_non_query_response(affected_rows: u64) -> FiscusResult<u64> {
        Ok(affected_rows)
    }

    /// Convert test user to database row format
    pub fn test_user_to_row(user: &TestUser) -> HashMap<String, Value> {
        let mut row = HashMap::new();
        row.insert("id".to_string(), Value::String(user.id.clone()));
        row.insert("username".to_string(), Value::String(user.username.clone()));
        row.insert(
            "email".to_string(),
            user.email
                .as_ref()
                .map(|e| Value::String(e.clone()))
                .unwrap_or(Value::Null),
        );
        row.insert(
            "password_hash".to_string(),
            Value::String(user.password_hash.clone()),
        );
        row
    }

    /// Convert test account to database row format
    pub fn test_account_to_row(account: &TestAccount) -> HashMap<String, Value> {
        let mut row = HashMap::new();
        row.insert("id".to_string(), Value::String(account.id.clone()));
        row.insert(
            "user_id".to_string(),
            Value::String(account.user_id.clone()),
        );
        row.insert(
            "account_type_id".to_string(),
            Value::String(account.account_type_id.clone()),
        );
        row.insert("name".to_string(), Value::String(account.name.clone()));
        row.insert(
            "balance".to_string(),
            Value::String(account.balance.to_string()),
        );
        row.insert(
            "currency".to_string(),
            Value::String(account.currency.clone()),
        );
        row.insert("is_active".to_string(), Value::Bool(account.is_active));
        row
    }

    /// Convert test category to database row format
    pub fn test_category_to_row(category: &TestCategory) -> HashMap<String, Value> {
        let mut row = HashMap::new();
        row.insert("id".to_string(), Value::String(category.id.clone()));
        row.insert(
            "user_id".to_string(),
            Value::String(category.user_id.clone()),
        );
        row.insert("name".to_string(), Value::String(category.name.clone()));
        row.insert("is_income".to_string(), Value::Bool(category.is_income));
        row.insert("is_active".to_string(), Value::Bool(category.is_active));
        row
    }

    /// Convert test transaction to database row format
    pub fn test_transaction_to_row(transaction: &TestTransaction) -> HashMap<String, Value> {
        let mut row = HashMap::new();
        row.insert("id".to_string(), Value::String(transaction.id.clone()));
        row.insert(
            "user_id".to_string(),
            Value::String(transaction.user_id.clone()),
        );
        row.insert(
            "account_id".to_string(),
            Value::String(transaction.account_id.clone()),
        );
        row.insert(
            "category_id".to_string(),
            transaction
                .category_id
                .as_ref()
                .map(|c| Value::String(c.clone()))
                .unwrap_or(Value::Null),
        );
        row.insert(
            "amount".to_string(),
            Value::String(transaction.amount.to_string()),
        );
        row.insert(
            "description".to_string(),
            Value::String(transaction.description.clone()),
        );
        row.insert(
            "transaction_type".to_string(),
            Value::String(transaction.transaction_type.clone()),
        );
        row.insert(
            "status".to_string(),
            Value::String(transaction.status.clone()),
        );
        row
    }
}
