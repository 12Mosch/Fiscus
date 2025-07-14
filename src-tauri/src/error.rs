use aes_gcm::aead;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

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

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Key derivation error: {0}")]
    KeyDerivation(String),

    #[error("Key management error: {0}")]
    KeyManagement(String),

    #[error("Cryptographic operation failed: {0}")]
    Cryptographic(String),
}

impl FiscusError {
    /// Log the error with appropriate level and context
    pub fn log_error(&self, context: Option<&str>) {
        let error_msg = self.to_string();
        let error_type = self.error_type();

        match self {
            FiscusError::Database(_) => {
                error!(
                    error_type = error_type,
                    error = %error_msg,
                    context = context,
                    "Database error occurred"
                );
            }
            FiscusError::Security(_) => {
                error!(
                    error_type = error_type,
                    error = %error_msg,
                    context = context,
                    "Security violation detected"
                );
            }
            FiscusError::Encryption(_)
            | FiscusError::KeyDerivation(_)
            | FiscusError::KeyManagement(_)
            | FiscusError::Cryptographic(_) => {
                error!(
                    error_type = error_type,
                    error = %error_msg,
                    context = context,
                    "Cryptographic operation error"
                );
            }
            FiscusError::Authentication(_) | FiscusError::Authorization(_) => {
                error!(
                    error_type = error_type,
                    error = %error_msg,
                    context = context,
                    "Authentication/Authorization error"
                );
            }
            FiscusError::Internal(_) => {
                error!(
                    error_type = error_type,
                    error = %error_msg,
                    context = context,
                    "Internal server error"
                );
            }
            _ => {
                error!(
                    error_type = error_type,
                    error = %error_msg,
                    context = context,
                    "Application error occurred"
                );
            }
        }
    }

    /// Get the error type as a string for logging
    pub fn error_type(&self) -> &'static str {
        match self {
            FiscusError::Database(_) => "database",
            FiscusError::Validation(_) => "validation",
            FiscusError::Authentication(_) => "authentication",
            FiscusError::Authorization(_) => "authorization",
            FiscusError::NotFound(_) => "not_found",
            FiscusError::Conflict(_) => "conflict",
            FiscusError::InvalidInput(_) => "invalid_input",
            FiscusError::Security(_) => "security",
            FiscusError::Internal(_) => "internal",
            FiscusError::External(_) => "external",
            FiscusError::Encryption(_) => "encryption",
            FiscusError::KeyDerivation(_) => "key_derivation",
            FiscusError::KeyManagement(_) => "key_management",
            FiscusError::Cryptographic(_) => "cryptographic",
        }
    }

    /// Check if the error is critical (requires immediate attention)
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            FiscusError::Database(_)
                | FiscusError::Security(_)
                | FiscusError::Internal(_)
                | FiscusError::Encryption(_)
                | FiscusError::KeyManagement(_)
                | FiscusError::Cryptographic(_)
        )
    }

    /// Create a new error with logging
    pub fn new_with_log(error: FiscusError, context: Option<&str>) -> Self {
        error.log_error(context);
        error
    }
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

// Note: aes_gcm::Error and chacha20poly1305::Error are the same type (aead::Error)
// so we can only implement From for one of them. We'll use a generic approach.
impl From<aead::Error> for FiscusError {
    fn from(err: aead::Error) -> Self {
        FiscusError::Encryption(format!("AEAD encryption error: {err}"))
    }
}

impl From<rsa::Error> for FiscusError {
    fn from(err: rsa::Error) -> Self {
        FiscusError::Encryption(format!("RSA error: {err}"))
    }
}

impl From<ed25519_dalek::SignatureError> for FiscusError {
    fn from(err: ed25519_dalek::SignatureError) -> Self {
        FiscusError::Encryption(format!("Ed25519 error: {err}"))
    }
}

impl From<scrypt::errors::InvalidParams> for FiscusError {
    fn from(err: scrypt::errors::InvalidParams) -> Self {
        FiscusError::KeyDerivation(format!("Scrypt parameter error: {err}"))
    }
}

impl From<scrypt::errors::InvalidOutputLen> for FiscusError {
    fn from(err: scrypt::errors::InvalidOutputLen) -> Self {
        FiscusError::KeyDerivation(format!("Scrypt output length error: {err}"))
    }
}

impl From<base64::DecodeError> for FiscusError {
    fn from(err: base64::DecodeError) -> Self {
        FiscusError::InvalidInput(format!("Base64 decode error: {err}"))
    }
}

/// Result type alias for Fiscus operations
pub type FiscusResult<T> = Result<T, FiscusError>;

/// Lazy static regex for email validation - compiled once for better performance
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")
        .expect("Failed to compile email regex - this should never happen")
});

/// Lazy static regex for currency code validation - compiled once for better performance
static CURRENCY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Z]{3}$").expect("Failed to compile currency regex - this should never happen")
});

/// ISO 4217 currency codes - comprehensive list of commonly supported currencies
static VALID_CURRENCY_CODES: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    [
        // Major currencies
        "USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD", "NZD", // Asian currencies
        "CNY", "HKD", "SGD", "KRW", "INR", "THB", "MYR", "IDR", "PHP", "VND",
        // European currencies
        "SEK", "NOK", "DKK", "PLN", "CZK", "HUF", "RON", "BGN", "HRK",
        // Middle Eastern currencies
        "AED", "SAR", "QAR", "KWD", "BHD", "OMR", "JOD", "ILS", // African currencies
        "ZAR", "EGP", "NGN", "KES", "GHS", "MAD", "TND", // Latin American currencies
        "BRL", "MXN", "ARS", "CLP", "COP", "PEN", "UYU", // Other important currencies
        "RUB", "TRY", "PKR", "BDT", "LKR", "NPR", "MMK",
    ]
    .iter()
    .copied()
    .collect()
});

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
        if !EMAIL_REGEX.is_match(email) {
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

    /// Validate currency code according to ISO 4217 standard
    /// Ensures the currency code is a 3-letter uppercase code and is in the supported list
    pub fn validate_currency_code(currency: &str) -> FiscusResult<()> {
        // Check if empty or whitespace
        if currency.trim().is_empty() {
            return Err(FiscusError::Validation(
                "Currency code cannot be empty".to_string(),
            ));
        }

        // Normalize to uppercase for validation
        let currency_upper = currency.trim().to_uppercase();

        // Check format (3 uppercase letters)
        if !CURRENCY_REGEX.is_match(&currency_upper) {
            return Err(FiscusError::Validation(
                "Currency code must be exactly 3 uppercase letters (e.g., USD, EUR, GBP)"
                    .to_string(),
            ));
        }

        // Check if currency is supported
        if !VALID_CURRENCY_CODES.contains(currency_upper.as_str()) {
            return Err(FiscusError::Validation(format!(
                "Unsupported currency code: {currency_upper}. Please use a valid ISO 4217 currency code"
            )));
        }

        Ok(())
    }

    /// Validate user ID format and content
    /// Ensures the user ID is a valid UUID format and not empty
    pub fn validate_user_id(user_id: &str) -> FiscusResult<uuid::Uuid> {
        // Check if empty or whitespace
        if user_id.trim().is_empty() {
            return Err(FiscusError::Validation(
                "User ID cannot be empty".to_string(),
            ));
        }

        // Validate UUID format and return the parsed UUID
        Self::validate_uuid(user_id, "user ID")
    }
}

/// Validated wrapper type for user IDs
/// Ensures user IDs are valid UUIDs and not empty
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatedUserId(uuid::Uuid);

impl ValidatedUserId {
    /// Create a new ValidatedUserId from a string
    pub fn new(user_id: &str) -> FiscusResult<Self> {
        let uuid = Validator::validate_user_id(user_id)?;
        Ok(Self(uuid))
    }

    /// Get the inner UUID value
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }

    /// Get the string representation
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }

    /// Check if the user ID is empty (always false for valid UUIDs)
    pub fn is_empty(&self) -> bool {
        false
    }
}

impl std::fmt::Display for ValidatedUserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ValidatedUserId {
    type Err = FiscusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl serde::Serialize for ValidatedUserId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for ValidatedUserId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
    }
}

/// Validated wrapper type for currency codes
/// Ensures currency codes follow ISO 4217 standard
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatedCurrency(String);

impl ValidatedCurrency {
    /// Create a new ValidatedCurrency from a string
    pub fn new(currency: &str) -> FiscusResult<Self> {
        let currency_upper = currency.trim().to_uppercase();
        Validator::validate_currency_code(&currency_upper)?;
        Ok(Self(currency_upper))
    }

    /// Get the currency code as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ValidatedCurrency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ValidatedCurrency {
    type Err = FiscusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl serde::Serialize for ValidatedCurrency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for ValidatedCurrency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).map_err(serde::de::Error::custom)
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

    /// Validate data size limits
    pub fn validate_data_size(data: &[u8], max_size: usize, field_name: &str) -> FiscusResult<()> {
        if data.len() > max_size {
            return Err(FiscusError::Security(format!(
                "{} size exceeds maximum limit: {} > {}",
                field_name,
                data.len(),
                max_size
            )));
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_fiscus_error_display() {
        let error = FiscusError::Database("Connection failed".to_string());
        assert_eq!(error.to_string(), "Database error: Connection failed");

        let error = FiscusError::Validation("Invalid input".to_string());
        assert_eq!(error.to_string(), "Validation error: Invalid input");

        let error = FiscusError::Authentication("Invalid credentials".to_string());
        assert_eq!(
            error.to_string(),
            "Authentication error: Invalid credentials"
        );

        let error = FiscusError::Authorization("Access denied".to_string());
        assert_eq!(error.to_string(), "Authorization error: Access denied");

        let error = FiscusError::NotFound("User not found".to_string());
        assert_eq!(error.to_string(), "Not found: User not found");

        let error = FiscusError::Conflict("Username already exists".to_string());
        assert_eq!(error.to_string(), "Conflict: Username already exists");

        let error = FiscusError::InvalidInput("Invalid JSON".to_string());
        assert_eq!(error.to_string(), "Invalid input: Invalid JSON");

        let error = FiscusError::Security("SQL injection attempt".to_string());
        assert_eq!(
            error.to_string(),
            "Security violation: SQL injection attempt"
        );

        let error = FiscusError::Internal("Server error".to_string());
        assert_eq!(error.to_string(), "Internal server error: Server error");

        let error = FiscusError::External("API error".to_string());
        assert_eq!(error.to_string(), "External service error: API error");
    }

    #[test]
    fn test_fiscus_error_serialization() {
        let error = FiscusError::Database("Connection failed".to_string());
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: FiscusError = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            FiscusError::Database(msg) => assert_eq!(msg, "Connection failed"),
            _ => panic!("Expected Database error"),
        }
    }

    #[test]
    fn test_error_conversions() {
        // Test serde_json::Error conversion
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_error.is_err());
        let fiscus_error: FiscusError = json_error.unwrap_err().into();
        match fiscus_error {
            FiscusError::InvalidInput(msg) => assert!(msg.contains("JSON parsing error")),
            _ => panic!("Expected InvalidInput error"),
        }

        // Test uuid::Error conversion
        let uuid_error = uuid::Uuid::parse_str("invalid-uuid");
        assert!(uuid_error.is_err());
        let fiscus_error: FiscusError = uuid_error.unwrap_err().into();
        match fiscus_error {
            FiscusError::InvalidInput(msg) => assert!(msg.contains("UUID error")),
            _ => panic!("Expected InvalidInput error"),
        }

        // Test chrono::ParseError conversion
        let date_error = chrono::DateTime::parse_from_rfc3339("invalid-date");
        assert!(date_error.is_err());
        let fiscus_error: FiscusError = date_error.unwrap_err().into();
        match fiscus_error {
            FiscusError::InvalidInput(msg) => assert!(msg.contains("Date parsing error")),
            _ => panic!("Expected InvalidInput error"),
        }

        // Test rust_decimal::Error conversion
        let decimal_error = Decimal::from_str_exact("invalid-decimal");
        assert!(decimal_error.is_err());
        let fiscus_error: FiscusError = decimal_error.unwrap_err().into();
        match fiscus_error {
            FiscusError::InvalidInput(msg) => assert!(msg.contains("Decimal parsing error")),
            _ => panic!("Expected InvalidInput error"),
        }
    }

    mod validator_tests {
        use super::*;

        #[test]
        fn test_validate_string() {
            // Valid strings
            assert!(Validator::validate_string("hello", "field", 1, 10).is_ok());
            assert!(Validator::validate_string("a", "field", 1, 10).is_ok());
            assert!(Validator::validate_string("1234567890", "field", 1, 10).is_ok());

            // Empty string when min_length > 0
            assert!(Validator::validate_string("", "field", 1, 10).is_err());

            // String too short
            assert!(Validator::validate_string("ab", "field", 3, 10).is_err());

            // String too long
            assert!(Validator::validate_string("12345678901", "field", 1, 10).is_err());

            // Empty string is never allowed due to trim() check
            assert!(Validator::validate_string("", "field", 0, 10).is_err());
        }

        #[test]
        fn test_validate_email() {
            // Valid emails
            assert!(Validator::validate_email("test@example.com").is_ok());
            assert!(Validator::validate_email("user.name@domain.co.uk").is_ok());
            assert!(Validator::validate_email("user+tag@example.org").is_ok());

            // Invalid emails
            assert!(Validator::validate_email("invalid-email").is_err());
            assert!(Validator::validate_email("@example.com").is_err());
            assert!(Validator::validate_email("test@").is_err());
            assert!(Validator::validate_email("test@.com").is_err());
            assert!(Validator::validate_email("").is_err());
        }

        #[test]
        fn test_email_validation_performance() {
            // Test that multiple calls to validate_email work efficiently
            // This test ensures the lazy static regex is working correctly
            let test_emails = vec![
                "test1@example.com",
                "test2@example.com",
                "test3@example.com",
                "invalid-email",
                "test4@example.com",
            ];

            // Multiple iterations to ensure regex is reused
            for _ in 0..100 {
                for email in &test_emails {
                    let _ = Validator::validate_email(email);
                }
            }

            // Verify the regex still works correctly after many calls
            assert!(Validator::validate_email("final@test.com").is_ok());
            assert!(Validator::validate_email("invalid").is_err());
        }

        #[test]
        fn test_validate_uuid() {
            // Valid UUIDs
            let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
            assert!(Validator::validate_uuid(valid_uuid, "id").is_ok());

            let uuid_v4 = uuid::Uuid::new_v4().to_string();
            assert!(Validator::validate_uuid(&uuid_v4, "id").is_ok());

            // Invalid UUIDs
            assert!(Validator::validate_uuid("invalid-uuid", "id").is_err());
            assert!(Validator::validate_uuid("", "id").is_err());
            assert!(Validator::validate_uuid("550e8400-e29b-41d4-a716", "id").is_err());
        }

        #[test]
        fn test_validate_amount() {
            // Valid amounts
            assert!(Validator::validate_amount(Decimal::new(100, 2), false).is_ok()); // $1.00
            assert!(Validator::validate_amount(Decimal::new(0, 0), false).is_ok()); // $0.00
            assert!(Validator::validate_amount(Decimal::new(999999999, 2), false).is_ok()); // Large amount

            // Negative amounts
            assert!(Validator::validate_amount(Decimal::new(-100, 2), true).is_ok()); // Allowed
            assert!(Validator::validate_amount(Decimal::new(-100, 2), false).is_err()); // Not allowed

            // Test maximum amount validation
            let max_amount = Decimal::new(999999999999i64, 2); // Very large amount
            assert!(Validator::validate_amount(max_amount, false).is_ok());

            let too_large = Decimal::new(i64::MAX, 0);
            // This should be invalid as it exceeds our max limit
            assert!(Validator::validate_amount(too_large, false).is_err());
        }

        #[test]
        fn test_validate_date() {
            // Valid dates
            assert!(Validator::validate_date("2023-12-25").is_ok());
            assert!(Validator::validate_date("2000-01-01").is_ok());
            assert!(Validator::validate_date("2024-02-29").is_ok()); // Leap year

            // Invalid dates
            assert!(Validator::validate_date("2023-13-01").is_err()); // Invalid month
            assert!(Validator::validate_date("2023-12-32").is_err()); // Invalid day
            assert!(Validator::validate_date("2023/12/25").is_err()); // Wrong format
            assert!(Validator::validate_date("invalid-date").is_err());
            assert!(Validator::validate_date("").is_err());
        }

        #[test]
        fn test_validate_datetime() {
            // Valid datetimes
            assert!(Validator::validate_datetime("2023-12-25T10:30:00Z").is_ok());
            assert!(Validator::validate_datetime("2023-12-25T10:30:00+00:00").is_ok());
            assert!(Validator::validate_datetime("2023-12-25T10:30:00-05:00").is_ok());

            // Invalid datetimes
            assert!(Validator::validate_datetime("2023-12-25 10:30:00").is_err()); // Wrong format
            assert!(Validator::validate_datetime("2023-12-25T25:30:00Z").is_err()); // Invalid hour
            assert!(Validator::validate_datetime("invalid-datetime").is_err());
            assert!(Validator::validate_datetime("").is_err());
        }
    }

    mod security_validator_tests {
        use super::*;
        use std::collections::HashMap;

        #[test]
        fn test_validate_sort_field() {
            // Valid account sort fields
            assert!(SecurityValidator::validate_sort_field(
                "name",
                SecurityValidator::ACCOUNT_SORT_FIELDS
            )
            .is_ok());
            assert!(SecurityValidator::validate_sort_field(
                "balance",
                SecurityValidator::ACCOUNT_SORT_FIELDS
            )
            .is_ok());
            assert!(SecurityValidator::validate_sort_field(
                "created_at",
                SecurityValidator::ACCOUNT_SORT_FIELDS
            )
            .is_ok());

            // Invalid account sort fields
            assert!(SecurityValidator::validate_sort_field(
                "password",
                SecurityValidator::ACCOUNT_SORT_FIELDS
            )
            .is_err());
            assert!(SecurityValidator::validate_sort_field(
                "DROP TABLE",
                SecurityValidator::ACCOUNT_SORT_FIELDS
            )
            .is_err());
            assert!(SecurityValidator::validate_sort_field(
                "",
                SecurityValidator::ACCOUNT_SORT_FIELDS
            )
            .is_err());

            // Valid transaction sort fields
            assert!(SecurityValidator::validate_sort_field(
                "amount",
                SecurityValidator::TRANSACTION_SORT_FIELDS
            )
            .is_ok());
            assert!(SecurityValidator::validate_sort_field(
                "description",
                SecurityValidator::TRANSACTION_SORT_FIELDS
            )
            .is_ok());

            // Test that field names are quoted in response
            let result = SecurityValidator::validate_sort_field(
                "name",
                SecurityValidator::ACCOUNT_SORT_FIELDS,
            )
            .unwrap();
            assert_eq!(result, "`name`");
        }

        #[test]
        fn test_validate_sort_direction() {
            // Valid directions
            assert!(SecurityValidator::validate_sort_direction("ASC").is_ok());
            assert!(SecurityValidator::validate_sort_direction("DESC").is_ok());
            assert!(SecurityValidator::validate_sort_direction("asc").is_ok());
            assert!(SecurityValidator::validate_sort_direction("desc").is_ok());

            // Test case normalization
            assert_eq!(
                SecurityValidator::validate_sort_direction("asc").unwrap(),
                "ASC"
            );
            assert_eq!(
                SecurityValidator::validate_sort_direction("desc").unwrap(),
                "DESC"
            );

            // Invalid directions
            assert!(SecurityValidator::validate_sort_direction("INVALID").is_err());
            assert!(SecurityValidator::validate_sort_direction("ORDER BY").is_err());
            assert!(SecurityValidator::validate_sort_direction("").is_err());
            assert!(SecurityValidator::validate_sort_direction("ASC; DROP TABLE").is_err());
        }

        #[test]
        fn test_validate_account_filter_fields() {
            // Valid filters
            let mut valid_filters = HashMap::new();
            valid_filters.insert("user_id".to_string(), "test-user-id".to_string());
            valid_filters.insert("type".to_string(), "checking".to_string());
            valid_filters.insert("is_active".to_string(), "true".to_string());
            assert!(SecurityValidator::validate_account_filter_fields(&valid_filters).is_ok());

            // Invalid filters
            let mut invalid_filters = HashMap::new();
            invalid_filters.insert("password".to_string(), "secret".to_string());
            assert!(SecurityValidator::validate_account_filter_fields(&invalid_filters).is_err());

            let mut sql_injection_filters = HashMap::new();
            sql_injection_filters
                .insert("user_id; DROP TABLE users".to_string(), "value".to_string());
            assert!(
                SecurityValidator::validate_account_filter_fields(&sql_injection_filters).is_err()
            );
        }

        #[test]
        fn test_validate_transaction_filter_fields() {
            // Valid filters
            let mut valid_filters = HashMap::new();
            valid_filters.insert("user_id".to_string(), "test-user-id".to_string());
            valid_filters.insert("account_id".to_string(), "test-account-id".to_string());
            valid_filters.insert("category_id".to_string(), "test-category-id".to_string());
            valid_filters.insert("transaction_type".to_string(), "expense".to_string());
            valid_filters.insert("status".to_string(), "completed".to_string());
            valid_filters.insert("start_date".to_string(), "2023-01-01".to_string());
            valid_filters.insert("end_date".to_string(), "2023-12-31".to_string());
            valid_filters.insert("min_amount".to_string(), "0".to_string());
            valid_filters.insert("max_amount".to_string(), "1000".to_string());
            assert!(SecurityValidator::validate_transaction_filter_fields(&valid_filters).is_ok());

            // Invalid filters
            let mut invalid_filters = HashMap::new();
            invalid_filters.insert("password".to_string(), "secret".to_string());
            assert!(
                SecurityValidator::validate_transaction_filter_fields(&invalid_filters).is_err()
            );

            let mut sql_injection_filters = HashMap::new();
            sql_injection_filters.insert(
                "user_id; DROP TABLE transactions".to_string(),
                "value".to_string(),
            );
            assert!(
                SecurityValidator::validate_transaction_filter_fields(&sql_injection_filters)
                    .is_err()
            );
        }

        #[test]
        fn test_sort_field_constants() {
            // Test that all expected fields are present
            assert!(SecurityValidator::ACCOUNT_SORT_FIELDS.contains(&"name"));
            assert!(SecurityValidator::ACCOUNT_SORT_FIELDS.contains(&"type"));
            assert!(SecurityValidator::ACCOUNT_SORT_FIELDS.contains(&"balance"));
            assert!(SecurityValidator::ACCOUNT_SORT_FIELDS.contains(&"created_at"));
            assert!(SecurityValidator::ACCOUNT_SORT_FIELDS.contains(&"updated_at"));

            assert!(SecurityValidator::TRANSACTION_SORT_FIELDS.contains(&"amount"));
            assert!(SecurityValidator::TRANSACTION_SORT_FIELDS.contains(&"description"));
            assert!(SecurityValidator::TRANSACTION_SORT_FIELDS.contains(&"transaction_date"));
            assert!(SecurityValidator::TRANSACTION_SORT_FIELDS.contains(&"created_at"));
            assert!(SecurityValidator::TRANSACTION_SORT_FIELDS.contains(&"updated_at"));

            assert!(SecurityValidator::CATEGORY_SORT_FIELDS.contains(&"name"));
            assert!(SecurityValidator::CATEGORY_SORT_FIELDS.contains(&"created_at"));
            assert!(SecurityValidator::CATEGORY_SORT_FIELDS.contains(&"updated_at"));

            assert!(SecurityValidator::BUDGET_SORT_FIELDS.contains(&"allocated_amount"));
            assert!(SecurityValidator::BUDGET_SORT_FIELDS.contains(&"spent_amount"));
            assert!(SecurityValidator::BUDGET_SORT_FIELDS.contains(&"created_at"));
            assert!(SecurityValidator::BUDGET_SORT_FIELDS.contains(&"updated_at"));
        }

        #[test]
        fn test_sql_injection_prevention() {
            // Test that malicious field names are rejected
            let malicious_fields = vec![
                "name; DROP TABLE users",
                "name' OR '1'='1",
                "name UNION SELECT password FROM users",
                "name/*comment*/",
                "name--comment",
                "name\x00",
            ];

            for field in malicious_fields {
                assert!(
                    SecurityValidator::validate_sort_field(
                        field,
                        SecurityValidator::ACCOUNT_SORT_FIELDS
                    )
                    .is_err(),
                    "Should reject malicious field: {field}"
                );
            }

            // Test that malicious sort directions are rejected
            let malicious_directions = vec![
                "ASC; DROP TABLE users",
                "ASC' OR '1'='1",
                "ASC UNION SELECT password FROM users",
                "ASC/*comment*/",
                "ASC--comment",
            ];

            for direction in malicious_directions {
                assert!(
                    SecurityValidator::validate_sort_direction(direction).is_err(),
                    "Should reject malicious direction: {direction}"
                );
            }
        }

        #[test]
        fn test_validate_currency_code() {
            // Valid currency codes
            assert!(Validator::validate_currency_code("USD").is_ok());
            assert!(Validator::validate_currency_code("EUR").is_ok());
            assert!(Validator::validate_currency_code("GBP").is_ok());
            assert!(Validator::validate_currency_code("JPY").is_ok());
            assert!(Validator::validate_currency_code("CAD").is_ok());

            // Invalid currency codes - empty/whitespace
            assert!(Validator::validate_currency_code("").is_err());
            assert!(Validator::validate_currency_code("   ").is_err());

            // Invalid currency codes - wrong format
            assert!(Validator::validate_currency_code("us").is_err()); // too short
            assert!(Validator::validate_currency_code("USDD").is_err()); // too long
            assert!(Validator::validate_currency_code("US1").is_err()); // contains number
            assert!(Validator::validate_currency_code("U$D").is_err()); // contains symbol

            // Valid currency codes - case normalization should work
            assert!(Validator::validate_currency_code("usd").is_ok()); // lowercase normalized to uppercase
            assert!(Validator::validate_currency_code("eur").is_ok()); // lowercase normalized to uppercase

            // Invalid currency codes - unsupported
            assert!(Validator::validate_currency_code("XXX").is_err());
            assert!(Validator::validate_currency_code("ABC").is_err());
            assert!(Validator::validate_currency_code("ZZZ").is_err());
        }

        #[test]
        fn test_validate_user_id() {
            // Valid UUIDs
            assert!(Validator::validate_user_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
            assert!(Validator::validate_user_id("6ba7b810-9dad-11d1-80b4-00c04fd430c8").is_ok());
            assert!(Validator::validate_user_id("6ba7b811-9dad-11d1-80b4-00c04fd430c8").is_ok());

            // Invalid user IDs - empty/whitespace
            assert!(Validator::validate_user_id("").is_err());
            assert!(Validator::validate_user_id("   ").is_err());

            // Invalid user IDs - wrong format
            assert!(Validator::validate_user_id("not-a-uuid").is_err());
            assert!(Validator::validate_user_id("550e8400-e29b-41d4-a716").is_err()); // too short
            assert!(
                Validator::validate_user_id("550e8400-e29b-41d4-a716-446655440000-extra").is_err()
            ); // too long
            assert!(Validator::validate_user_id("550e8400-e29b-41d4-a716-44665544000g").is_err());
            // invalid character
            assert!(Validator::validate_user_id("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx").is_err());
            // invalid hex characters
        }

        #[test]
        fn test_validated_user_id() {
            // Valid creation
            let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
            let user_id = ValidatedUserId::new(valid_uuid).unwrap();
            assert_eq!(user_id.as_str(), valid_uuid);
            assert_eq!(user_id.to_string(), valid_uuid);

            // Invalid creation
            assert!(ValidatedUserId::new("").is_err());
            assert!(ValidatedUserId::new("not-a-uuid").is_err());
            assert!(ValidatedUserId::new("550e8400-e29b-41d4-a716").is_err());

            // Serialization/Deserialization
            let user_id = ValidatedUserId::new(valid_uuid).unwrap();
            let serialized = serde_json::to_string(&user_id).unwrap();
            let deserialized: ValidatedUserId = serde_json::from_str(&serialized).unwrap();
            assert_eq!(user_id, deserialized);

            // FromStr trait
            let user_id_from_str: ValidatedUserId = valid_uuid.parse().unwrap();
            assert_eq!(user_id_from_str.as_str(), valid_uuid);
        }

        #[test]
        fn test_validated_currency() {
            // Valid creation
            let currency = ValidatedCurrency::new("USD").unwrap();
            assert_eq!(currency.as_str(), "USD");
            assert_eq!(currency.to_string(), "USD");

            // Case normalization
            let currency_lower = ValidatedCurrency::new("usd").unwrap();
            assert_eq!(currency_lower.as_str(), "USD");

            // Invalid creation
            assert!(ValidatedCurrency::new("").is_err());
            assert!(ValidatedCurrency::new("us").is_err());
            assert!(ValidatedCurrency::new("USDD").is_err());
            assert!(ValidatedCurrency::new("XXX").is_err());

            // Serialization/Deserialization
            let currency = ValidatedCurrency::new("EUR").unwrap();
            let serialized = serde_json::to_string(&currency).unwrap();
            let deserialized: ValidatedCurrency = serde_json::from_str(&serialized).unwrap();
            assert_eq!(currency, deserialized);

            // FromStr trait
            let currency_from_str: ValidatedCurrency = "GBP".parse().unwrap();
            assert_eq!(currency_from_str.as_str(), "GBP");
        }
    }
}
