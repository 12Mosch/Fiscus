use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::{GoalStatus, TransactionStatus, TransactionType};

/// Request DTOs for creating entities

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub user_id: String,
    pub account_type_id: String,
    pub name: String,
    pub balance: Option<Decimal>,
    pub currency: String,
    pub account_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub parent_category_id: Option<String>,
    pub is_income: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub user_id: String,
    pub account_id: String,
    pub category_id: Option<String>,
    pub amount: Decimal,
    pub description: String,
    pub notes: Option<String>,
    pub transaction_date: String, // ISO 8601 format
    pub transaction_type: TransactionType,
    pub reference_number: Option<String>,
    pub payee: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBudgetPeriodRequest {
    pub user_id: String,
    pub name: String,
    pub start_date: String, // YYYY-MM-DD format
    pub end_date: String,   // YYYY-MM-DD format
}

#[derive(Debug, Deserialize)]
pub struct CreateBudgetRequest {
    pub user_id: String,
    pub budget_period_id: String,
    pub category_id: String,
    pub allocated_amount: Decimal,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGoalRequest {
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub target_amount: Decimal,
    pub target_date: Option<String>, // YYYY-MM-DD format
    pub priority: Option<i32>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransferRequest {
    pub user_id: String,
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
    pub user_id: String,
    pub account_type_id: Option<String>,
    pub is_active: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionFilters {
    pub user_id: String,
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
    pub user_id: String,
    pub parent_category_id: Option<String>,
    pub is_income: Option<bool>,
    pub is_active: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BudgetFilters {
    pub user_id: String,
    pub budget_period_id: Option<String>,
    pub category_id: Option<String>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoalFilters {
    pub user_id: String,
    pub status: Option<GoalStatus>,
    pub category: Option<String>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

/// Authentication DTOs

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub user_id: String,
    pub current_password: String,
    pub new_password: String,
}

/// Response DTOs

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub session_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

#[derive(Debug, Serialize)]
pub struct AccountSummaryResponse {
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    pub net_worth: Decimal,
    pub account_count: i32,
}

#[derive(Debug, Serialize)]
pub struct BudgetSummaryResponse {
    pub total_allocated: Decimal,
    pub total_spent: Decimal,
    pub remaining: Decimal,
    pub categories_over_budget: i32,
    pub categories_under_budget: i32,
}

#[derive(Debug, Serialize)]
pub struct TransactionSummaryResponse {
    pub total_income: Decimal,
    pub total_expenses: Decimal,
    pub net_income: Decimal,
    pub transaction_count: i32,
    pub average_transaction: Decimal,
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
