use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::HashMap;
use tempfile::NamedTempFile;
use uuid::Uuid;

use crate::{
    dto::*,
    error::{FiscusResult, ValidatedCurrency, ValidatedUserId},
    models::*,
    security::data_protection::SensitiveData,
};

/// Test utilities for creating mock data and setting up test environments
pub struct TestUtils;

#[allow(dead_code)]
impl TestUtils {
    /// Create a test user with default values
    pub fn create_test_user() -> User {
        User::new(
            "testuser".to_string(),
            Some("test@example.com".to_string()),
            "$argon2id$v=19$m=65536,t=3,p=4$test_salt$test_hash".to_string(),
        )
    }

    /// Create a test user with custom values
    pub fn create_test_user_with_values(username: &str, email: Option<&str>) -> User {
        User::new(
            username.to_string(),
            email.map(|e| e.to_string()),
            "$argon2id$v=19$m=65536,t=3,p=4$test_salt$test_hash".to_string(),
        )
    }

    /// Create a test account
    pub fn create_test_account(user_id: &str) -> Account {
        Account::new(
            user_id.to_string(),
            "checking".to_string(),
            "Test Checking Account".to_string(),
            "USD".to_string(),
        )
    }

    /// Create a test account with custom values
    pub fn create_test_account_with_values(
        user_id: &str,
        account_type_id: &str,
        name: &str,
        balance: Decimal,
    ) -> Account {
        let mut account = Account::new(
            user_id.to_string(),
            account_type_id.to_string(),
            name.to_string(),
            "USD".to_string(),
        );
        account.balance = balance;
        account
    }

    /// Create a test category
    pub fn create_test_category(user_id: &str, name: &str, is_income: bool) -> Category {
        Category::new(user_id.to_string(), name.to_string(), is_income)
    }

    /// Create a test transaction
    pub fn create_test_transaction(
        user_id: &str,
        account_id: &str,
        amount: Decimal,
        transaction_type: TransactionType,
    ) -> Transaction {
        let now = Utc::now();
        Transaction {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            account_id: account_id.to_string(),
            category_id: None,
            amount,
            description: "Test transaction".to_string(),
            notes: None,
            transaction_date: now,
            transaction_type,
            status: TransactionStatus::Completed,
            reference_number: None,
            payee: None,
            tags: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a test goal
    pub fn create_test_goal(user_id: &str, name: &str, target_amount: Decimal) -> Goal {
        let now = Utc::now();
        Goal {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            name: name.to_string(),
            description: Some("Test goal".to_string()),
            target_amount,
            current_amount: Decimal::ZERO,
            target_date: None,
            priority: 1,
            status: GoalStatus::Active,
            category: Some("savings".to_string()),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a test budget
    pub fn create_test_budget(
        user_id: &str,
        budget_period_id: &str,
        category_id: &str,
        allocated_amount: Decimal,
    ) -> Budget {
        let now = Utc::now();
        Budget {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            budget_period_id: budget_period_id.to_string(),
            category_id: category_id.to_string(),
            allocated_amount,
            spent_amount: Decimal::ZERO,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a test budget period
    pub fn create_test_budget_period(user_id: &str, name: &str) -> BudgetPeriod {
        let now = Utc::now();
        let start_date = now.date_naive();
        let end_date = start_date + chrono::Duration::days(30);

        BudgetPeriod {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            name: name.to_string(),
            start_date,
            end_date,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Generate a random UUID string
    pub fn random_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    /// Create test CreateUserRequest
    pub fn create_user_request(
        username: &str,
        email: Option<&str>,
        password: &str,
    ) -> CreateUserRequest {
        CreateUserRequest {
            username: username.to_string(),
            email: email.map(|e| e.to_string()),
            password: SensitiveData::new(password.to_string()),
        }
    }

    /// Create test LoginRequest
    pub fn login_request(username: &str, password: &str) -> LoginRequest {
        LoginRequest {
            username: username.to_string(),
            password: SensitiveData::new(password.to_string()),
        }
    }

    /// Create test CreateAccountRequest
    pub fn create_account_request(user_id: &str, name: &str) -> CreateAccountRequest {
        CreateAccountRequest {
            user_id: ValidatedUserId::new(user_id).unwrap(),
            account_type_id: "checking".to_string(),
            name: name.to_string(),
            balance: Some(Decimal::ZERO),
            currency: ValidatedCurrency::new("USD").unwrap(),
            account_number: None,
        }
    }

    /// Create test CreateTransactionRequest
    pub fn create_transaction_request(
        user_id: &str,
        account_id: &str,
        amount: Decimal,
        description: &str,
    ) -> CreateTransactionRequest {
        CreateTransactionRequest {
            user_id: ValidatedUserId::new(user_id).unwrap(),
            account_id: account_id.to_string(),
            category_id: None,
            amount,
            description: description.to_string(),
            notes: None,
            transaction_date: Utc::now(),
            transaction_type: TransactionType::Expense,
            reference_number: None,
            payee: None,
            tags: None,
        }
    }

    /// Create test CreateCategoryRequest
    pub fn create_category_request(
        user_id: &str,
        name: &str,
        is_income: bool,
    ) -> CreateCategoryRequest {
        CreateCategoryRequest {
            user_id: ValidatedUserId::new(user_id).unwrap(),
            name: name.to_string(),
            description: None,
            color: None,
            icon: None,
            parent_category_id: None,
            is_income,
        }
    }

    /// Create test filters with default values
    pub fn default_account_filters(user_id: &str) -> AccountFilters {
        AccountFilters {
            user_id: ValidatedUserId::new(user_id).unwrap(),
            account_type_id: None,
            is_active: None,
            sort_by: None,
            sort_direction: None,
            limit: None,
            offset: None,
        }
    }

    /// Create test filters with default values
    pub fn default_transaction_filters(user_id: &str) -> TransactionFilters {
        TransactionFilters {
            user_id: ValidatedUserId::new(user_id).unwrap(),
            account_id: None,
            category_id: None,
            transaction_type: None,
            status: None,
            start_date: None,
            end_date: None,
            min_amount: None,
            max_amount: None,
            search: None,
            sort_by: None,
            sort_direction: None,
            limit: None,
            offset: None,
        }
    }

    /// Create a mock database row as HashMap
    pub fn mock_db_row(fields: Vec<(&str, Value)>) -> HashMap<String, Value> {
        fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect()
    }

    /// Convert a User model to a database row representation
    pub fn user_to_db_row(user: &User) -> HashMap<String, Value> {
        Self::mock_db_row(vec![
            ("id", Value::String(user.id.clone())),
            ("username", Value::String(user.username.clone())),
            (
                "email",
                user.email
                    .as_ref()
                    .map(|e| Value::String(e.clone()))
                    .unwrap_or(Value::Null),
            ),
            ("password_hash", Value::String(user.password_hash.clone())),
            ("created_at", Value::String(user.created_at.to_rfc3339())),
            ("updated_at", Value::String(user.updated_at.to_rfc3339())),
        ])
    }

    /// Convert an Account model to a database row representation
    pub fn account_to_db_row(account: &Account) -> HashMap<String, Value> {
        Self::mock_db_row(vec![
            ("id", Value::String(account.id.clone())),
            ("user_id", Value::String(account.user_id.clone())),
            (
                "account_type_id",
                Value::String(account.account_type_id.clone()),
            ),
            ("name", Value::String(account.name.clone())),
            ("balance", Value::String(account.balance.to_string())),
            ("currency", Value::String(account.currency.clone())),
            (
                "account_number",
                account
                    .account_number
                    .as_ref()
                    .map(|n| Value::String(n.clone()))
                    .unwrap_or(Value::Null),
            ),
            ("is_active", Value::Bool(account.is_active)),
            ("created_at", Value::String(account.created_at.to_rfc3339())),
            ("updated_at", Value::String(account.updated_at.to_rfc3339())),
        ])
    }
}

/// Database test utilities for setting up test databases
pub struct DatabaseTestUtils;

#[allow(dead_code)]
impl DatabaseTestUtils {
    /// Create a temporary SQLite database file for testing
    pub fn create_temp_db() -> FiscusResult<NamedTempFile> {
        let temp_file = NamedTempFile::new().map_err(|e| {
            crate::error::FiscusError::Internal(format!("Failed to create temp file: {e}"))
        })?;
        Ok(temp_file)
    }

    /// Get the database URL for a temporary file
    pub fn get_temp_db_url(temp_file: &NamedTempFile) -> String {
        format!("sqlite:{}", temp_file.path().display())
    }
}

/// Assertion helpers for tests
pub struct TestAssertions;

#[allow(dead_code)]
impl TestAssertions {
    /// Assert that two decimals are equal within a small tolerance
    pub fn assert_decimal_eq(actual: Decimal, expected: Decimal, message: &str) {
        let diff = (actual - expected).abs();
        let tolerance = Decimal::new(1, 6); // 0.000001
        assert!(
            diff < tolerance,
            "{message}: expected {expected}, got {actual} (diff: {diff})"
        );
    }

    /// Assert that a datetime is within a reasonable range of now
    pub fn assert_datetime_recent(datetime: &DateTime<Utc>, message: &str) {
        let now = Utc::now();
        let diff = (now - *datetime).num_seconds().abs();
        assert!(
            diff < 60, // Within 1 minute
            "{message}: datetime {datetime} is not recent (diff: {diff} seconds)"
        );
    }

    /// Assert that a UUID string is valid
    pub fn assert_valid_uuid(uuid_str: &str, message: &str) {
        assert!(
            Uuid::parse_str(uuid_str).is_ok(),
            "{message}: '{uuid_str}' is not a valid UUID"
        );
    }
}
