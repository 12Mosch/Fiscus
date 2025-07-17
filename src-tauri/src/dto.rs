use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::encryption::types::{EncryptionAlgorithm, KeyDerivationAlgorithm, KeyType};
use crate::error::{ValidatedCurrency, ValidatedUserId};
use crate::models::{GoalStatus, TransactionStatus, TransactionType};
use crate::security::data_protection::SensitiveData;

/// Request DTOs for creating entities

#[derive(Debug, Deserialize, Clone)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: SensitiveData<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub user_id: ValidatedUserId,
    pub account_type_id: String,
    pub name: String,
    pub balance: Option<Decimal>,
    pub currency: ValidatedCurrency,
    pub account_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub user_id: ValidatedUserId,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub parent_category_id: Option<String>,
    pub is_income: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub user_id: ValidatedUserId,
    pub account_id: String,
    pub category_id: Option<String>,
    pub amount: Decimal,
    pub description: String,
    pub notes: Option<String>,
    pub transaction_date: DateTime<Utc>,
    pub transaction_type: TransactionType,
    pub reference_number: Option<String>,
    pub payee: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBudgetPeriodRequest {
    pub user_id: ValidatedUserId,
    pub name: String,
    pub start_date: String, // YYYY-MM-DD format
    pub end_date: String,   // YYYY-MM-DD format
}

#[derive(Debug, Deserialize)]
pub struct CreateBudgetRequest {
    pub user_id: ValidatedUserId,
    pub budget_period_id: String,
    pub category_id: String,
    pub allocated_amount: Decimal,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGoalRequest {
    pub user_id: ValidatedUserId,
    pub name: String,
    pub description: Option<String>,
    pub target_amount: Decimal,
    pub target_date: Option<String>, // YYYY-MM-DD format
    pub priority: Option<i32>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransferRequest {
    pub user_id: ValidatedUserId,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: Decimal,
    pub description: String,
    pub transfer_date: String, // ISO 8601 format
}

/// Update DTOs for modifying entities

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub balance: Option<Decimal>,
    pub account_number: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub parent_category_id: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTransactionRequest {
    pub category_id: Option<String>,
    pub amount: Option<Decimal>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub transaction_date: Option<String>,
    pub transaction_type: Option<TransactionType>,
    pub status: Option<TransactionStatus>,
    pub reference_number: Option<String>,
    pub payee: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBudgetRequest {
    pub allocated_amount: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGoalRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub target_amount: Option<Decimal>,
    pub current_amount: Option<Decimal>,
    pub target_date: Option<String>,
    pub priority: Option<i32>,
    pub status: Option<GoalStatus>,
    pub category: Option<String>,
}

/// Filter and query DTOs

#[derive(Debug, Deserialize)]
pub struct AccountFilters {
    pub user_id: ValidatedUserId,
    pub account_type_id: Option<String>,
    pub is_active: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransactionFilters {
    pub user_id: ValidatedUserId,
    pub account_id: Option<String>,
    pub category_id: Option<String>,
    pub transaction_type: Option<TransactionType>,
    pub status: Option<TransactionStatus>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CategoryFilters {
    pub user_id: ValidatedUserId,
    pub parent_category_id: Option<String>,
    pub is_income: Option<bool>,
    pub is_active: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BudgetFilters {
    pub user_id: ValidatedUserId,
    pub budget_period_id: Option<String>,
    pub category_id: Option<String>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoalFilters {
    pub user_id: ValidatedUserId,
    pub status: Option<GoalStatus>,
    pub category: Option<String>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

/// Authentication DTOs

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: SensitiveData<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub user_id: ValidatedUserId,
    pub current_password: SensitiveData<String>,
    pub new_password: SensitiveData<String>,
}

/// Response DTOs

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub session_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSummaryResponse {
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    pub net_worth: Decimal,
    pub account_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetSummaryResponse {
    pub total_allocated: Decimal,
    pub total_spent: Decimal,
    pub remaining: Decimal,
    pub categories_over_budget: i32,
    pub categories_under_budget: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSummaryResponse {
    pub total_income: Decimal,
    pub total_expenses: Decimal,
    pub net_income: Decimal,
    pub transaction_count: i32,
    pub average_transaction: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionStatsResponse {
    pub total_transactions: i32,
    pub total_income: Decimal,
    pub total_expenses: Decimal,
    pub net_income: Decimal,
    pub average_transaction_amount: Decimal,
    pub largest_expense: Option<Decimal>,
    pub largest_income: Option<Decimal>,
    pub most_frequent_category: Option<String>,
    pub transactions_by_type: HashMap<String, i32>,
    pub transactions_by_status: HashMap<String, i32>,
}

#[derive(Debug, Deserialize)]
pub struct BulkTransactionRequest {
    pub user_id: ValidatedUserId,
    pub transaction_ids: Vec<String>,
    pub action: BulkTransactionAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkTransactionAction {
    Delete,
    UpdateCategory { category_id: Option<String> },
    UpdateStatus { status: TransactionStatus },
    Export { format: ExportFormat },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Csv,
    Json,
}

/// Utility functions for DTOs
impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i32, page: i32, per_page: i32) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as i32;
        Self {
            data,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}

/// Encryption-related DTOs

#[derive(Debug, Deserialize)]
pub struct EncryptDataRequest {
    pub user_id: ValidatedUserId,
    pub data_type: String,
    pub data: String, // Base64 encoded data
}

#[derive(Debug, Serialize)]
pub struct EncryptDataResponse {
    pub encrypted_data: String, // Base64 encoded
    pub nonce: String,          // Base64 encoded
    pub algorithm: EncryptionAlgorithm,
    pub key_id: String,
    pub encrypted_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct DecryptDataRequest {
    pub user_id: ValidatedUserId,
    pub data_type: String,
    pub encrypted_data: String, // Base64 encoded
    pub nonce: String,          // Base64 encoded
    pub algorithm: EncryptionAlgorithm,
    pub key_id: String,
}

#[derive(Debug, Serialize)]
pub struct DecryptDataResponse {
    pub data: String, // Base64 encoded decrypted data
    pub decrypted_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateKeyRequest {
    pub user_id: ValidatedUserId,
    pub algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Serialize)]
pub struct GenerateKeyResponse {
    pub key_id: String,
    pub algorithm: EncryptionAlgorithm,
    pub key_type: KeyType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RotateKeysRequest {
    pub user_id: ValidatedUserId,
}

#[derive(Debug, Serialize)]
pub struct EncryptionStatsResponse {
    pub total_keys: usize,
    pub active_keys: usize,
    pub rotated_keys: usize,
    pub encryption_operations: u64,
    pub decryption_operations: u64,
    pub key_derivation_operations: u64,
    pub last_key_rotation: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct DeriveKeyRequest {
    pub password: SensitiveData<String>,
    pub algorithm: KeyDerivationAlgorithm,
    pub salt: Option<String>, // Base64 encoded salt
}

#[derive(Debug, Serialize)]
pub struct DeriveKeyResponse {
    pub key_id: String,
    pub algorithm: KeyDerivationAlgorithm,
    pub derived_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SignDataRequest {
    pub user_id: ValidatedUserId,
    pub data: String, // Base64 encoded data to sign
    pub private_key_id: String,
    pub algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Serialize)]
pub struct SignDataResponse {
    pub signature: String, // Base64 encoded signature
    pub algorithm: EncryptionAlgorithm,
    pub signed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct VerifySignatureRequest {
    pub data: String,       // Base64 encoded original data
    pub signature: String,  // Base64 encoded signature
    pub public_key: String, // Base64 encoded public key
    pub algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Serialize)]
pub struct VerifySignatureResponse {
    pub is_valid: bool,
    pub algorithm: EncryptionAlgorithm,
    pub verified_at: DateTime<Utc>,
}

/// Secure storage DTOs
#[derive(Debug, Deserialize)]
pub struct SecureStoreRequest {
    pub user_id: ValidatedUserId,
    pub data_type: String,
    pub encrypted_data: String, // Base64 encoded encrypted data
    pub nonce: String,          // Base64 encoded nonce
    pub algorithm: EncryptionAlgorithm,
    pub key_id: String,
}

#[derive(Debug, Serialize)]
pub struct SecureStoreResponse {
    pub stored: bool,
    pub storage_key: String,
    pub stored_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SecureRetrieveRequest {
    pub user_id: ValidatedUserId,
    pub data_type: String,
}

#[derive(Debug, Serialize)]
pub struct SecureRetrieveResponse {
    pub encrypted_data: String, // Base64 encoded encrypted data
    pub nonce: String,          // Base64 encoded nonce
    pub algorithm: EncryptionAlgorithm,
    pub key_id: String,
    pub stored_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SecureDeleteRequest {
    pub user_id: ValidatedUserId,
    pub data_type: String,
}

#[derive(Debug, Serialize)]
pub struct SecureDeleteResponse {
    pub deleted: bool,
    pub deleted_at: DateTime<Utc>,
}

impl From<crate::models::User> for UserResponse {
    fn from(user: crate::models::User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_create_user_request_deserialization() {
        let json = r#"{
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123"
        }"#;

        let request: CreateUserRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "testuser");
        assert_eq!(request.email, Some("test@example.com".to_string()));
        assert_eq!(request.password.expose(), "password123");
    }

    #[test]
    fn test_create_user_request_without_email() {
        let json = r#"{
            "username": "testuser",
            "password": "password123"
        }"#;

        let request: CreateUserRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "testuser");
        assert_eq!(request.email, None);
        assert_eq!(request.password.expose(), "password123");
    }

    #[test]
    fn test_login_request_deserialization() {
        let json = r#"{
            "username": "testuser",
            "password": "password123"
        }"#;

        let request: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "testuser");
        assert_eq!(request.password.expose(), "password123");
    }

    #[test]
    fn test_create_account_request_deserialization() {
        let json = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "account_type_id": "checking",
            "name": "My Checking Account",
            "balance": 1000.50,
            "currency": "USD",
            "account_number": "123456789"
        }"#;

        let request: CreateAccountRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.user_id.as_str(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(request.account_type_id, "checking");
        assert_eq!(request.name, "My Checking Account");
        assert_eq!(request.balance, Some(Decimal::new(100050, 2)));
        assert_eq!(request.currency.as_str(), "USD");
        assert_eq!(request.account_number, Some("123456789".to_string()));
    }

    #[test]
    fn test_create_transaction_request_deserialization() {
        let json = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "account_id": "account-456",
            "category_id": "category-789",
            "amount": -50.25,
            "description": "Grocery shopping",
            "notes": "Weekly groceries",
            "transaction_date": "2023-12-25T10:30:00Z",
            "transaction_type": "expense",
            "reference_number": "REF123",
            "payee": "Grocery Store",
            "tags": ["food", "weekly"]
        }"#;

        let request: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.user_id.as_str(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(request.account_id, "account-456");
        assert_eq!(request.category_id, Some("category-789".to_string()));
        assert_eq!(request.amount, Decimal::new(-5025, 2));
        assert_eq!(request.description, "Grocery shopping");
        assert_eq!(request.notes, Some("Weekly groceries".to_string()));
        assert_eq!(
            request.transaction_date,
            DateTime::parse_from_rfc3339("2023-12-25T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
        assert_eq!(
            request.transaction_type,
            crate::models::TransactionType::Expense
        );
        assert_eq!(request.reference_number, Some("REF123".to_string()));
        assert_eq!(request.payee, Some("Grocery Store".to_string()));
        assert_eq!(
            request.tags,
            Some(vec!["food".to_string(), "weekly".to_string()])
        );
    }

    #[test]
    fn test_create_category_request_deserialization() {
        let json = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Groceries",
            "description": "Food and household items",
            "icon": "shopping_cart",
            "parent_category_id": "parent-456",
            "is_income": false
        }"#;

        let request: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.user_id.as_str(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(request.name, "Groceries");
        assert_eq!(
            request.description,
            Some("Food and household items".to_string())
        );
        assert_eq!(request.color, None);
        assert_eq!(request.icon, Some("shopping_cart".to_string()));
        assert_eq!(request.parent_category_id, Some("parent-456".to_string()));
        assert!(!request.is_income);
    }

    #[test]
    fn test_create_goal_request_deserialization() {
        let json = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Emergency Fund",
            "description": "Save for emergencies",
            "target_amount": 10000.00,
            "target_date": "2024-12-31",
            "priority": 1,
            "category": "savings"
        }"#;

        let request: CreateGoalRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.user_id.as_str(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(request.name, "Emergency Fund");
        assert_eq!(
            request.description,
            Some("Save for emergencies".to_string())
        );
        assert_eq!(request.target_amount, Decimal::new(1000000, 2));
        assert_eq!(request.target_date, Some("2024-12-31".to_string()));
        assert_eq!(request.priority, Some(1));
        assert_eq!(request.category, Some("savings".to_string()));
    }

    #[test]
    fn test_account_filters_deserialization() {
        let json = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "account_type_id": "checking",
            "is_active": true,
            "sort_by": "name",
            "sort_direction": "asc",
            "limit": 10,
            "offset": 0
        }"#;

        let filters: AccountFilters = serde_json::from_str(json).unwrap();
        assert_eq!(
            filters.user_id.as_str(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(filters.account_type_id, Some("checking".to_string()));
        assert_eq!(filters.is_active, Some(true));
        assert_eq!(filters.sort_by, Some("name".to_string()));
        assert_eq!(filters.sort_direction, Some("asc".to_string()));
        assert_eq!(filters.limit, Some(10));
        assert_eq!(filters.offset, Some(0));
    }

    #[test]
    fn test_transaction_filters_deserialization() {
        let json = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "account_id": "account-456",
            "category_id": "category-789",
            "transaction_type": "expense",
            "status": "completed",
            "start_date": "2023-01-01",
            "end_date": "2023-12-31",
            "min_amount": 0.01,
            "max_amount": 1000.00,
            "search": "grocery",
            "sort_by": "transaction_date",
            "sort_direction": "desc",
            "limit": 20,
            "offset": 0
        }"#;

        let filters: TransactionFilters = serde_json::from_str(json).unwrap();
        assert_eq!(
            filters.user_id.as_str(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(filters.account_id, Some("account-456".to_string()));
        assert_eq!(filters.category_id, Some("category-789".to_string()));
        assert_eq!(
            filters.transaction_type,
            Some(crate::models::TransactionType::Expense)
        );
        assert_eq!(
            filters.status,
            Some(crate::models::TransactionStatus::Completed)
        );
        assert_eq!(filters.start_date, Some("2023-01-01".to_string()));
        assert_eq!(filters.end_date, Some("2023-12-31".to_string()));
        assert_eq!(filters.min_amount, Some(Decimal::new(1, 2)));
        assert_eq!(filters.max_amount, Some(Decimal::new(100000, 2)));
        assert_eq!(filters.search, Some("grocery".to_string()));
        assert_eq!(filters.sort_by, Some("transaction_date".to_string()));
        assert_eq!(filters.sort_direction, Some("desc".to_string()));
        assert_eq!(filters.limit, Some(20));
        assert_eq!(filters.offset, Some(0));
    }

    #[test]
    fn test_login_response_serialization() {
        use chrono::Utc;

        let user_response = UserResponse {
            id: "user-123".to_string(),
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let login_response = LoginResponse {
            user: user_response,
            session_token: Some("token123".to_string()),
        };

        let serialized = serde_json::to_string(&login_response).unwrap();
        let deserialized: LoginResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.user.id, "user-123");
        assert_eq!(deserialized.user.username, "testuser");
        assert_eq!(deserialized.session_token, Some("token123".to_string()));
    }

    #[test]
    fn test_paginated_response_creation() {
        let data = vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
        ];
        let response = PaginatedResponse::new(data.clone(), 100, 1, 10);

        assert_eq!(response.data, data);
        assert_eq!(response.total, 100);
        assert_eq!(response.page, 1);
        assert_eq!(response.per_page, 10);
        assert_eq!(response.total_pages, 10); // 100 / 10 = 10
    }

    #[test]
    fn test_paginated_response_with_remainder() {
        let data = vec!["item1".to_string()];
        let response = PaginatedResponse::new(data, 25, 3, 10);

        assert_eq!(response.total, 25);
        assert_eq!(response.page, 3);
        assert_eq!(response.per_page, 10);
        assert_eq!(response.total_pages, 3); // ceil(25 / 10) = 3
    }

    #[test]
    fn test_account_summary_response_serialization() {
        let summary = AccountSummaryResponse {
            total_assets: Decimal::new(150000, 2),     // $1500.00
            total_liabilities: Decimal::new(50000, 2), // $500.00
            net_worth: Decimal::new(100000, 2),        // $1000.00
            account_count: 5,
        };

        let serialized = serde_json::to_string(&summary).unwrap();
        let deserialized: AccountSummaryResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.total_assets, Decimal::new(150000, 2));
        assert_eq!(deserialized.total_liabilities, Decimal::new(50000, 2));
        assert_eq!(deserialized.net_worth, Decimal::new(100000, 2));
        assert_eq!(deserialized.account_count, 5);
    }

    #[test]
    fn test_budget_summary_response_serialization() {
        let summary = BudgetSummaryResponse {
            total_allocated: Decimal::new(200000, 2), // $2000.00
            total_spent: Decimal::new(150000, 2),     // $1500.00
            remaining: Decimal::new(50000, 2),        // $500.00
            categories_over_budget: 2,
            categories_under_budget: 8,
        };

        let serialized = serde_json::to_string(&summary).unwrap();
        let deserialized: BudgetSummaryResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.total_allocated, Decimal::new(200000, 2));
        assert_eq!(deserialized.total_spent, Decimal::new(150000, 2));
        assert_eq!(deserialized.remaining, Decimal::new(50000, 2));
        assert_eq!(deserialized.categories_over_budget, 2);
        assert_eq!(deserialized.categories_under_budget, 8);
    }

    #[test]
    fn test_transaction_summary_response_serialization() {
        let summary = TransactionSummaryResponse {
            total_income: Decimal::new(500000, 2),   // $5000.00
            total_expenses: Decimal::new(300000, 2), // $3000.00
            net_income: Decimal::new(200000, 2),     // $2000.00
            transaction_count: 25,
            average_transaction: Decimal::new(12000, 2), // $120.00
        };

        let serialized = serde_json::to_string(&summary).unwrap();
        let deserialized: TransactionSummaryResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.total_income, Decimal::new(500000, 2));
        assert_eq!(deserialized.total_expenses, Decimal::new(300000, 2));
        assert_eq!(deserialized.net_income, Decimal::new(200000, 2));
        assert_eq!(deserialized.transaction_count, 25);
        assert_eq!(deserialized.average_transaction, Decimal::new(12000, 2));
    }

    #[test]
    fn test_update_requests_deserialization() {
        // Test UpdateAccountRequest
        let json = r#"{
            "name": "Updated Account Name",
            "account_number": "987654321"
        }"#;

        let request: UpdateAccountRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, Some("Updated Account Name".to_string()));
        assert_eq!(request.account_number, Some("987654321".to_string()));

        // Test UpdateTransactionRequest
        let json = r#"{
            "amount": 75.50,
            "description": "Updated description",
            "category_id": "new-category-id"
        }"#;

        let request: UpdateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.amount, Some(Decimal::new(7550, 2)));
        assert_eq!(request.description, Some("Updated description".to_string()));
        assert_eq!(request.category_id, Some("new-category-id".to_string()));
    }

    #[test]
    fn test_change_password_request_deserialization() {
        let json = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "current_password": "oldpass123",
            "new_password": "newpass456"
        }"#;

        let request: ChangePasswordRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.user_id.as_str(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(request.current_password.expose(), "oldpass123");
        assert_eq!(request.new_password.expose(), "newpass456");
    }

    #[test]
    fn test_create_account_request_validation_errors() {
        // Invalid user_id
        let json_invalid_user_id = r#"{
            "user_id": "invalid-uuid",
            "account_type_id": "checking",
            "name": "My Account",
            "currency": "USD"
        }"#;
        assert!(serde_json::from_str::<CreateAccountRequest>(json_invalid_user_id).is_err());

        // Invalid currency
        let json_invalid_currency = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "account_type_id": "checking",
            "name": "My Account",
            "currency": "INVALID"
        }"#;
        assert!(serde_json::from_str::<CreateAccountRequest>(json_invalid_currency).is_err());

        // Empty currency
        let json_empty_currency = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "account_type_id": "checking",
            "name": "My Account",
            "currency": ""
        }"#;
        assert!(serde_json::from_str::<CreateAccountRequest>(json_empty_currency).is_err());

        // Lowercase currency (should be normalized to uppercase)
        let json_lowercase_currency = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "account_type_id": "checking",
            "name": "My Account",
            "currency": "eur"
        }"#;
        let request: CreateAccountRequest = serde_json::from_str(json_lowercase_currency).unwrap();
        assert_eq!(request.currency.as_str(), "EUR");
    }

    #[test]
    fn test_transaction_filters_validation() {
        // Valid filters
        let json_valid = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "account_id": "account-123",
            "limit": 10
        }"#;
        let filters: TransactionFilters = serde_json::from_str(json_valid).unwrap();
        assert_eq!(
            filters.user_id.as_str(),
            "550e8400-e29b-41d4-a716-446655440000"
        );

        // Invalid user_id
        let json_invalid = r#"{
            "user_id": "not-a-uuid",
            "limit": 10
        }"#;
        assert!(serde_json::from_str::<TransactionFilters>(json_invalid).is_err());
    }

    #[test]
    fn test_encryption_request_validation() {
        // Valid request
        let json_valid = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "data_type": "sensitive_data",
            "data": "SGVsbG8gV29ybGQ="
        }"#;
        let request: EncryptDataRequest = serde_json::from_str(json_valid).unwrap();
        assert_eq!(
            request.user_id.as_str(),
            "550e8400-e29b-41d4-a716-446655440000"
        );

        // Invalid user_id
        let json_invalid = r#"{
            "user_id": "invalid",
            "data_type": "sensitive_data",
            "data": "SGVsbG8gV29ybGQ="
        }"#;
        assert!(serde_json::from_str::<EncryptDataRequest>(json_invalid).is_err());
    }

    #[test]
    fn test_comprehensive_currency_validation() {
        // Test all major currencies are supported
        let major_currencies = ["USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "CNY"];

        for currency in major_currencies {
            let json = format!(
                r#"{{
                "user_id": "550e8400-e29b-41d4-a716-446655440000",
                "account_type_id": "checking",
                "name": "Test Account",
                "currency": "{currency}"
            }}"#
            );

            let request: CreateAccountRequest = serde_json::from_str(&json).unwrap();
            assert_eq!(request.currency.as_str(), currency);
        }

        // Test unsupported currencies are rejected
        let invalid_currencies = ["XXX", "ABC", "123", "US", "USDD"];

        for currency in invalid_currencies {
            let json = format!(
                r#"{{
                "user_id": "550e8400-e29b-41d4-a716-446655440000",
                "account_type_id": "checking",
                "name": "Test Account",
                "currency": "{currency}"
            }}"#
            );

            assert!(
                serde_json::from_str::<CreateAccountRequest>(&json).is_err(),
                "Currency {currency} should be rejected"
            );
        }
    }
}
