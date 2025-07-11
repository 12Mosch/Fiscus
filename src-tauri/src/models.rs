use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Base trait for all database entities
pub trait Entity {
    fn id(&self) -> &str;
    fn created_at(&self) -> &DateTime<Utc>;
    fn updated_at(&self) -> &DateTime<Utc>;
}

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for User {
    fn id(&self) -> &str {
        &self.id
    }
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

/// Account Type entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountType {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_asset: bool,
    pub created_at: DateTime<Utc>,
}

/// Account entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub user_id: String,
    pub account_type_id: String,
    pub name: String,
    pub balance: Decimal,
    pub currency: String,
    pub account_number: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Account {
    fn id(&self) -> &str {
        &self.id
    }
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

/// Category entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub parent_category_id: Option<String>,
    pub is_income: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Category {
    fn id(&self) -> &str {
        &self.id
    }
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

/// Transaction entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub user_id: String,
    pub account_id: String,
    pub category_id: Option<String>,
    pub amount: Decimal,
    pub description: String,
    pub notes: Option<String>,
    pub transaction_date: DateTime<Utc>,
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub reference_number: Option<String>,
    pub payee: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Transaction {
    fn id(&self) -> &str {
        &self.id
    }
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

/// Transaction type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Income,
    Expense,
    Transfer,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Income => write!(f, "income"),
            TransactionType::Expense => write!(f, "expense"),
            TransactionType::Transfer => write!(f, "transfer"),
        }
    }
}

/// Transaction status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Completed,
    Cancelled,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Completed => write!(f, "completed"),
            TransactionStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Budget Period entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetPeriod {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for BudgetPeriod {
    fn id(&self) -> &str {
        &self.id
    }
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

/// Budget entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: String,
    pub user_id: String,
    pub budget_period_id: String,
    pub category_id: String,
    pub allocated_amount: Decimal,
    pub spent_amount: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Budget {
    fn id(&self) -> &str {
        &self.id
    }
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

/// Goal entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub target_amount: Decimal,
    pub current_amount: Decimal,
    pub target_date: Option<NaiveDate>,
    pub priority: i32,
    pub status: GoalStatus,
    pub category: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Goal {
    fn id(&self) -> &str {
        &self.id
    }
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

/// Goal status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GoalStatus {
    Active,
    Completed,
    Paused,
    Cancelled,
}

impl std::fmt::Display for GoalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoalStatus::Active => write!(f, "active"),
            GoalStatus::Completed => write!(f, "completed"),
            GoalStatus::Paused => write!(f, "paused"),
            GoalStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Transfer entity (for tracking money movement between accounts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub id: String,
    pub user_id: String,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: Decimal,
    pub description: String,
    pub transfer_date: DateTime<Utc>,
    pub status: TransactionStatus,
    pub from_transaction_id: String,
    pub to_transaction_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for Transfer {
    fn id(&self) -> &str {
        &self.id
    }
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

/// Utility functions for model operations
impl User {
    pub fn new(username: String, email: Option<String>, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            email,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Account {
    pub fn new(user_id: String, account_type_id: String, name: String, currency: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            account_type_id,
            name,
            balance: Decimal::ZERO,
            currency,
            account_number: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Category {
    pub fn new(user_id: String, name: String, is_income: bool) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            name,
            description: None,
            color: None,
            icon: None,
            parent_category_id: None,
            is_income,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}
