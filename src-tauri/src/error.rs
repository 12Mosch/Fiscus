use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Custom error types for the Fiscus application
/// These errors can be serialized across the Tauri bridge
#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum FiscusError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Security violation: {0}")]
    Security(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("External service error: {0}")]
    External(String),
}

impl From<tauri_plugin_sql::Error> for FiscusError {
    fn from(err: tauri_plugin_sql::Error) -> Self {
        FiscusError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for FiscusError {
    fn from(err: serde_json::Error) -> Self {
        FiscusError::InvalidInput(format!("JSON parsing error: {err}"))
    }
}

impl From<uuid::Error> for FiscusError {
    fn from(err: uuid::Error) -> Self {
        FiscusError::InvalidInput(format!("UUID error: {err}"))
    }
}

impl From<chrono::ParseError> for FiscusError {
    fn from(err: chrono::ParseError) -> Self {
        FiscusError::InvalidInput(format!("Date parsing error: {err}"))
    }
}

impl From<rust_decimal::Error> for FiscusError {
    fn from(err: rust_decimal::Error) -> Self {
        FiscusError::InvalidInput(format!("Decimal parsing error: {err}"))
    }
}

impl From<argon2::password_hash::Error> for FiscusError {
    fn from(err: argon2::password_hash::Error) -> Self {
        FiscusError::Authentication(format!("Password hashing error: {err}"))
    }
}

impl From<anyhow::Error> for FiscusError {
    fn from(err: anyhow::Error) -> Self {
        FiscusError::Internal(err.to_string())
    }
}

/// Result type alias for Fiscus operations
pub type FiscusResult<T> = Result<T, FiscusError>;

/// Validation utilities
pub struct Validator;

impl Validator {
    /// Validate that a string is not empty and within length limits
    pub fn validate_string(
        value: &str,
        field_name: &str,
        min_len: usize,
        max_len: usize,
    ) -> FiscusResult<()> {
        if value.trim().is_empty() {
            return Err(FiscusError::Validation(format!(
                "{field_name} cannot be empty"
            )));
        }

        if value.len() < min_len {
            return Err(FiscusError::Validation(format!(
                "{field_name} must be at least {min_len} characters"
            )));
        }

        if value.len() > max_len {
            return Err(FiscusError::Validation(format!(
                "{field_name} cannot exceed {max_len} characters"
            )));
        }

        Ok(())
    }

    /// Validate email format
    pub fn validate_email(email: &str) -> FiscusResult<()> {
        let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")
            .map_err(|e| FiscusError::Internal(format!("Regex compilation error: {e}")))?;

        if !email_regex.is_match(email) {
            return Err(FiscusError::Validation("Invalid email format".to_string()));
        }

        Ok(())
    }

    /// Validate UUID format
    pub fn validate_uuid(id: &str, field_name: &str) -> FiscusResult<uuid::Uuid> {
        uuid::Uuid::parse_str(id)
            .map_err(|_| FiscusError::Validation(format!("Invalid {field_name} format")))
    }

    /// Validate amount (must be positive for most operations)
    pub fn validate_amount(
        amount: rust_decimal::Decimal,
        allow_negative: bool,
    ) -> FiscusResult<()> {
        if !allow_negative && amount < rust_decimal::Decimal::ZERO {
            return Err(FiscusError::Validation(
                "Amount cannot be negative".to_string(),
            ));
        }

        // Check for reasonable limits (prevent overflow)
        let max_amount = rust_decimal::Decimal::from(999_999_999_999i64);
        if amount.abs() > max_amount {
            return Err(FiscusError::Validation(
                "Amount exceeds maximum allowed value".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate date string
    pub fn validate_date(date_str: &str) -> FiscusResult<chrono::NaiveDate> {
        chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            FiscusError::Validation("Invalid date format. Expected YYYY-MM-DD".to_string())
        })
    }

    /// Validate datetime string
    pub fn validate_datetime(datetime_str: &str) -> FiscusResult<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::parse_from_rfc3339(datetime_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|_| {
                FiscusError::Validation("Invalid datetime format. Expected RFC3339".to_string())
            })
    }
}

/// Security utilities for field whitelisting and SQL injection prevention
pub struct SecurityValidator;

impl SecurityValidator {
    /// Allowed fields for account sorting
    pub const ACCOUNT_SORT_FIELDS: &'static [&'static str] =
        &["name", "type", "balance", "created_at", "updated_at"];

    /// Allowed fields for transaction sorting
    pub const TRANSACTION_SORT_FIELDS: &'static [&'static str] = &[
        "amount",
        "description",
        "transaction_date",
        "created_at",
        "updated_at",
    ];

    /// Allowed fields for category sorting
    pub const CATEGORY_SORT_FIELDS: &'static [&'static str] = &["name", "created_at", "updated_at"];

    /// Allowed fields for budget sorting
    pub const BUDGET_SORT_FIELDS: &'static [&'static str] = &[
        "allocated_amount",
        "spent_amount",
        "created_at",
        "updated_at",
    ];

    /// Validate sort field against whitelist
    pub fn validate_sort_field(field: &str, allowed_fields: &[&str]) -> FiscusResult<String> {
        if allowed_fields.contains(&field) {
            Ok(format!("`{field}`")) // Quote the field name for SQL safety
        } else {
            Err(FiscusError::Security(format!(
                "Invalid sort field: {field}"
            )))
        }
    }

    /// Validate sort direction
    pub fn validate_sort_direction(direction: &str) -> FiscusResult<String> {
        match direction.to_uppercase().as_str() {
            "ASC" | "DESC" => Ok(direction.to_uppercase()),
            _ => Err(FiscusError::Security(
                "Invalid sort direction. Must be ASC or DESC".to_string(),
            )),
        }
    }

    /// Validate filter fields for accounts
    pub fn validate_account_filter_fields(
        filters: &std::collections::HashMap<String, String>,
    ) -> FiscusResult<()> {
        const ALLOWED_FILTERS: &[&str] = &["user_id", "type", "is_active"];

        for key in filters.keys() {
            if !ALLOWED_FILTERS.contains(&key.as_str()) {
                return Err(FiscusError::Security(format!(
                    "Invalid filter field: {key}"
                )));
            }
        }

        Ok(())
    }

    /// Validate filter fields for transactions
    pub fn validate_transaction_filter_fields(
        filters: &std::collections::HashMap<String, String>,
    ) -> FiscusResult<()> {
        const ALLOWED_FILTERS: &[&str] = &[
            "user_id",
            "account_id",
            "category_id",
            "transaction_type",
            "status",
            "start_date",
            "end_date",
            "min_amount",
            "max_amount",
        ];

        for key in filters.keys() {
            if !ALLOWED_FILTERS.contains(&key.as_str()) {
                return Err(FiscusError::Security(format!(
                    "Invalid filter field: {key}"
                )));
            }
        }

        Ok(())
    }
}
