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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "testuser".to_string(),
            Some("test@example.com".to_string()),
            "password_hash".to_string(),
        );

        assert!(!user.id.is_empty());
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert_eq!(user.password_hash, "password_hash");
        assert!(user.created_at <= Utc::now());
        assert!(user.updated_at <= Utc::now());
        assert_eq!(user.created_at, user.updated_at);
    }

    #[test]
    fn test_user_entity_trait() {
        let user = User::new("testuser".to_string(), None, "password_hash".to_string());

        assert_eq!(user.id(), &user.id);
        assert_eq!(user.created_at(), &user.created_at);
        assert_eq!(user.updated_at(), &user.updated_at);
    }

    #[test]
    fn test_account_creation() {
        let account = Account::new(
            "user-id".to_string(),
            "checking".to_string(),
            "Test Account".to_string(),
            "USD".to_string(),
        );

        assert!(!account.id.is_empty());
        assert_eq!(account.user_id, "user-id");
        assert_eq!(account.account_type_id, "checking");
        assert_eq!(account.name, "Test Account");
        assert_eq!(account.balance, Decimal::ZERO);
        assert_eq!(account.currency, "USD");
        assert_eq!(account.account_number, None);
        assert!(account.is_active);
        assert!(account.created_at <= Utc::now());
        assert!(account.updated_at <= Utc::now());
    }

    #[test]
    fn test_account_entity_trait() {
        let account = Account::new(
            "user-id".to_string(),
            "checking".to_string(),
            "Test Account".to_string(),
            "USD".to_string(),
        );

        assert_eq!(account.id(), &account.id);
        assert_eq!(account.created_at(), &account.created_at);
        assert_eq!(account.updated_at(), &account.updated_at);
    }

    #[test]
    fn test_category_creation() {
        let category = Category::new("user-id".to_string(), "Groceries".to_string(), false);

        assert!(!category.id.is_empty());
        assert_eq!(category.user_id, "user-id");
        assert_eq!(category.name, "Groceries");
        assert_eq!(category.description, None);
        assert_eq!(category.color, None);
        assert_eq!(category.icon, None);
        assert_eq!(category.parent_category_id, None);
        assert!(!category.is_income);
        assert!(category.is_active);
        assert!(category.created_at <= Utc::now());
        assert!(category.updated_at <= Utc::now());
    }

    #[test]
    fn test_category_entity_trait() {
        let category = Category::new("user-id".to_string(), "Salary".to_string(), true);

        assert_eq!(category.id(), &category.id);
        assert_eq!(category.created_at(), &category.created_at);
        assert_eq!(category.updated_at(), &category.updated_at);
    }

    #[test]
    fn test_transaction_entity_trait() {
        let now = Utc::now();
        let transaction = Transaction {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            account_id: "account-id".to_string(),
            category_id: None,
            amount: Decimal::new(10000, 2), // $100.00
            description: "Test transaction".to_string(),
            notes: None,
            transaction_date: now,
            transaction_type: TransactionType::Expense,
            status: TransactionStatus::Completed,
            reference_number: None,
            payee: None,
            tags: None,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(transaction.id(), "test-id");
        assert_eq!(transaction.created_at(), &now);
        assert_eq!(transaction.updated_at(), &now);
    }

    #[test]
    fn test_goal_entity_trait() {
        let now = Utc::now();
        let goal = Goal {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            name: "Emergency Fund".to_string(),
            description: Some("Save for emergencies".to_string()),
            target_amount: Decimal::new(100000, 2), // $1000.00
            current_amount: Decimal::ZERO,
            target_date: None,
            priority: 1,
            status: GoalStatus::Active,
            category: Some("savings".to_string()),
            created_at: now,
            updated_at: now,
        };

        assert_eq!(goal.id(), "test-id");
        assert_eq!(goal.created_at(), &now);
        assert_eq!(goal.updated_at(), &now);
    }

    #[test]
    fn test_budget_entity_trait() {
        let now = Utc::now();
        let budget = Budget {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            budget_period_id: "period-id".to_string(),
            category_id: "category-id".to_string(),
            allocated_amount: Decimal::new(50000, 2), // $500.00
            spent_amount: Decimal::ZERO,
            notes: None,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(budget.id(), "test-id");
        assert_eq!(budget.created_at(), &now);
        assert_eq!(budget.updated_at(), &now);
    }

    #[test]
    fn test_budget_period_entity_trait() {
        let now = Utc::now();
        let start_date = now.date_naive();
        let end_date = start_date + chrono::Duration::days(30);

        let budget_period = BudgetPeriod {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            name: "January 2024".to_string(),
            start_date,
            end_date,
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(budget_period.id(), "test-id");
        assert_eq!(budget_period.created_at(), &now);
        assert_eq!(budget_period.updated_at(), &now);
    }

    #[test]
    fn test_transaction_type_serialization() {
        // Test serialization
        let income = TransactionType::Income;
        let expense = TransactionType::Expense;
        let transfer = TransactionType::Transfer;

        let income_json = serde_json::to_string(&income).unwrap();
        let expense_json = serde_json::to_string(&expense).unwrap();
        let transfer_json = serde_json::to_string(&transfer).unwrap();

        assert_eq!(income_json, "\"income\"");
        assert_eq!(expense_json, "\"expense\"");
        assert_eq!(transfer_json, "\"transfer\"");

        // Test deserialization
        let income_deserialized: TransactionType = serde_json::from_str(&income_json).unwrap();
        let expense_deserialized: TransactionType = serde_json::from_str(&expense_json).unwrap();
        let transfer_deserialized: TransactionType = serde_json::from_str(&transfer_json).unwrap();

        assert_eq!(income_deserialized, TransactionType::Income);
        assert_eq!(expense_deserialized, TransactionType::Expense);
        assert_eq!(transfer_deserialized, TransactionType::Transfer);
    }

    #[test]
    fn test_transaction_status_serialization() {
        let pending = TransactionStatus::Pending;
        let completed = TransactionStatus::Completed;
        let cancelled = TransactionStatus::Cancelled;

        let pending_json = serde_json::to_string(&pending).unwrap();
        let completed_json = serde_json::to_string(&completed).unwrap();
        let cancelled_json = serde_json::to_string(&cancelled).unwrap();

        assert_eq!(pending_json, "\"pending\"");
        assert_eq!(completed_json, "\"completed\"");
        assert_eq!(cancelled_json, "\"cancelled\"");

        // Test deserialization
        let pending_deserialized: TransactionStatus = serde_json::from_str(&pending_json).unwrap();
        let completed_deserialized: TransactionStatus =
            serde_json::from_str(&completed_json).unwrap();
        let cancelled_deserialized: TransactionStatus =
            serde_json::from_str(&cancelled_json).unwrap();

        assert_eq!(pending_deserialized, TransactionStatus::Pending);
        assert_eq!(completed_deserialized, TransactionStatus::Completed);
        assert_eq!(cancelled_deserialized, TransactionStatus::Cancelled);
    }

    #[test]
    fn test_goal_status_serialization() {
        let active = GoalStatus::Active;
        let completed = GoalStatus::Completed;
        let paused = GoalStatus::Paused;
        let cancelled = GoalStatus::Cancelled;

        let active_json = serde_json::to_string(&active).unwrap();
        let completed_json = serde_json::to_string(&completed).unwrap();
        let paused_json = serde_json::to_string(&paused).unwrap();
        let cancelled_json = serde_json::to_string(&cancelled).unwrap();

        assert_eq!(active_json, "\"active\"");
        assert_eq!(completed_json, "\"completed\"");
        assert_eq!(paused_json, "\"paused\"");
        assert_eq!(cancelled_json, "\"cancelled\"");

        // Test deserialization
        let active_deserialized: GoalStatus = serde_json::from_str(&active_json).unwrap();
        let completed_deserialized: GoalStatus = serde_json::from_str(&completed_json).unwrap();
        let paused_deserialized: GoalStatus = serde_json::from_str(&paused_json).unwrap();
        let cancelled_deserialized: GoalStatus = serde_json::from_str(&cancelled_json).unwrap();

        assert_eq!(active_deserialized, GoalStatus::Active);
        assert_eq!(completed_deserialized, GoalStatus::Completed);
        assert_eq!(paused_deserialized, GoalStatus::Paused);
        assert_eq!(cancelled_deserialized, GoalStatus::Cancelled);
    }

    #[test]
    fn test_model_serialization() {
        let user = User::new(
            "testuser".to_string(),
            Some("test@example.com".to_string()),
            "password_hash".to_string(),
        );

        // Test that the model can be serialized and deserialized
        let serialized = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&serialized).unwrap();

        assert_eq!(user.id, deserialized.id);
        assert_eq!(user.username, deserialized.username);
        assert_eq!(user.email, deserialized.email);
        assert_eq!(user.password_hash, deserialized.password_hash);
        assert_eq!(user.created_at, deserialized.created_at);
        assert_eq!(user.updated_at, deserialized.updated_at);
    }

    #[test]
    fn test_account_serialization() {
        let account = Account::new(
            "user-id".to_string(),
            "checking".to_string(),
            "Test Account".to_string(),
            "USD".to_string(),
        );

        let serialized = serde_json::to_string(&account).unwrap();
        let deserialized: Account = serde_json::from_str(&serialized).unwrap();

        assert_eq!(account.id, deserialized.id);
        assert_eq!(account.user_id, deserialized.user_id);
        assert_eq!(account.account_type_id, deserialized.account_type_id);
        assert_eq!(account.name, deserialized.name);
        assert_eq!(account.balance, deserialized.balance);
        assert_eq!(account.currency, deserialized.currency);
        assert_eq!(account.is_active, deserialized.is_active);
    }

    #[test]
    fn test_category_serialization() {
        let category = Category::new("user-id".to_string(), "Groceries".to_string(), false);

        let serialized = serde_json::to_string(&category).unwrap();
        let deserialized: Category = serde_json::from_str(&serialized).unwrap();

        assert_eq!(category.id, deserialized.id);
        assert_eq!(category.user_id, deserialized.user_id);
        assert_eq!(category.name, deserialized.name);
        assert_eq!(category.is_income, deserialized.is_income);
        assert_eq!(category.is_active, deserialized.is_active);
    }
}
