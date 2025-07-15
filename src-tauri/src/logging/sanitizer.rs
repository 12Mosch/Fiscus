use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashSet;

/// Data sanitizer for removing sensitive information from logs
#[derive(Debug, Clone)]
pub struct DataSanitizer {
    /// Field names that should be sanitized
    sensitive_fields: HashSet<String>,
    /// Regex patterns for detecting sensitive data
    patterns: Vec<SensitivePattern>,
    /// Replacement text for sanitized data
    replacement: String,
}

/// Pattern for detecting sensitive data
#[derive(Debug, Clone)]
struct SensitivePattern {
    #[allow(dead_code)]
    name: String,
    regex: Regex,
    replacement: String,
}

impl Default for DataSanitizer {
    fn default() -> Self {
        let mut sanitizer = Self {
            sensitive_fields: HashSet::new(),
            patterns: Vec::new(),
            replacement: "[REDACTED]".to_string(),
        };

        // Add default sensitive field names
        sanitizer.add_sensitive_fields(&[
            "password",
            "password_hash",
            "token",
            "secret",
            "key",
            "api_key",
            "auth_token",
            "session_token",
            "access_token",
            "refresh_token",
            "account_number",
            "account_num",
            "routing_number",
            "ssn",
            "social_security_number",
            "credit_card",
            "card_number",
            "card_num",
            "cvv",
            "cvc",
            "pin",
            "email",
            "phone",
            "phone_number",
            "address",
            "street_address",
            "postal_code",
            "zip_code",
        ]);

        // Add regex patterns for common sensitive data
        sanitizer.add_patterns();

        sanitizer
    }
}

impl DataSanitizer {
    /// Create a new data sanitizer
    pub fn new() -> Self {
        Self::default()
    }

    /// Add sensitive field names
    pub fn add_sensitive_fields(&mut self, fields: &[&str]) {
        for field in fields {
            self.sensitive_fields.insert(field.to_lowercase());
        }
    }

    /// Add regex patterns for detecting sensitive data
    fn add_patterns(&mut self) {
        let patterns = vec![
            // Credit card numbers (basic pattern)
            ("credit_card", r"\b(?:\d{4}[-\s]?){3}\d{4}\b", "[CARD-****]"),
            // SSN pattern
            ("ssn", r"\b\d{3}-?\d{2}-?\d{4}\b", "[SSN-***]"),
            // Email addresses
            (
                "email",
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",
                "[EMAIL-***]",
            ),
            // Phone numbers (US format)
            (
                "phone",
                r"\b(?:\+?1[-.\s]?)?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})\b",
                "[PHONE-***]",
            ),
            // Account numbers (8-20 digits)
            ("account_number", r"\b\d{8,20}\b", "[ACCOUNT-****]"),
            // API keys (common patterns)
            ("api_key", r"\b[A-Za-z0-9]{32,}\b", "[API-KEY-***]"),
            // JWT tokens
            (
                "jwt_token",
                r"\beyJ[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*\b",
                "[JWT-***]",
            ),
        ];

        for (name, pattern, replacement) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.patterns.push(SensitivePattern {
                    name: name.to_string(),
                    regex,
                    replacement: replacement.to_string(),
                });
            }
        }
    }

    /// Sanitize a JSON value
    pub fn sanitize_json(&self, value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut sanitized_map = Map::new();
                for (key, val) in map {
                    let sanitized_key = key.to_lowercase();
                    if self.sensitive_fields.contains(&sanitized_key) {
                        sanitized_map.insert(key.clone(), Value::String(self.replacement.clone()));
                    } else {
                        sanitized_map.insert(key.clone(), self.sanitize_json(val));
                    }
                }
                Value::Object(sanitized_map)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(|v| self.sanitize_json(v)).collect()),
            Value::String(s) => Value::String(self.sanitize_string(s)),
            _ => value.clone(),
        }
    }

    /// Sanitize a string value
    pub fn sanitize_string(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Apply regex patterns
        for pattern in &self.patterns {
            result = pattern
                .regex
                .replace_all(&result, &pattern.replacement)
                .to_string();
        }

        result
    }

    /// Sanitize a serde_json::Value that might contain sensitive data
    pub fn sanitize_value(&self, value: &Value) -> Value {
        self.sanitize_json(value)
    }

    /// Sanitize SQL query parameters
    pub fn sanitize_sql_params(&self, params: &[Value]) -> Vec<Value> {
        params
            .iter()
            .map(|param| self.sanitize_json(param))
            .collect()
    }

    /// Check if a field name is considered sensitive
    pub fn is_sensitive_field(&self, field_name: &str) -> bool {
        self.sensitive_fields.contains(&field_name.to_lowercase())
    }

    /// Sanitize a struct that implements serde::Serialize
    pub fn sanitize_serializable<T>(&self, data: &T) -> Value
    where
        T: serde::Serialize,
    {
        match serde_json::to_value(data) {
            Ok(value) => self.sanitize_json(&value),
            Err(_) => Value::String("[SERIALIZATION_ERROR]".to_string()),
        }
    }

    /// Create a sanitized version of a HashMap-like structure
    pub fn sanitize_map<K, V>(&self, map: &std::collections::HashMap<K, V>) -> Value
    where
        K: AsRef<str>,
        V: serde::Serialize,
    {
        let mut sanitized = Map::new();
        for (key, value) in map {
            let key_str = key.as_ref();
            if self.is_sensitive_field(key_str) {
                sanitized.insert(key_str.to_string(), Value::String(self.replacement.clone()));
            } else {
                match serde_json::to_value(value) {
                    Ok(val) => {
                        sanitized.insert(key_str.to_string(), self.sanitize_json(&val));
                    }
                    Err(_) => {
                        sanitized.insert(key_str.to_string(), Value::String("[ERROR]".to_string()));
                    }
                }
            }
        }
        Value::Object(sanitized)
    }

    /// Sanitize error messages that might contain sensitive data
    pub fn sanitize_error_message(&self, error_msg: &str) -> String {
        self.sanitize_string(error_msg)
    }

    /// Create a partial sanitizer that only redacts specific fields
    pub fn partial_sanitizer(fields: &[&str]) -> Self {
        let mut sanitizer = Self {
            sensitive_fields: HashSet::new(),
            patterns: Vec::new(),
            replacement: "[REDACTED]".to_string(),
        };
        sanitizer.add_sensitive_fields(fields);
        sanitizer
    }
}

/// Trait for types that can be sanitized for logging
pub trait Sanitizable {
    fn sanitize(&self, sanitizer: &DataSanitizer) -> Value;
}

impl Sanitizable for Value {
    fn sanitize(&self, sanitizer: &DataSanitizer) -> Value {
        sanitizer.sanitize_json(self)
    }
}

/// Helper function to sanitize any serializable type
pub fn sanitize_serializable<T>(data: &T, sanitizer: &DataSanitizer) -> Value
where
    T: serde::Serialize,
{
    sanitizer.sanitize_serializable(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sanitize_sensitive_fields() {
        let sanitizer = DataSanitizer::new();
        let data = json!({
            "username": "john_doe",
            "password": "secret123",
            "email": "john@example.com",
            "balance": 1000.50
        });

        let sanitized = sanitizer.sanitize_json(&data);

        assert_eq!(sanitized["username"], "john_doe");
        assert_eq!(sanitized["password"], "[REDACTED]");
        assert_eq!(sanitized["email"], "[REDACTED]");
        assert_eq!(sanitized["balance"], 1000.50);
    }

    #[test]
    fn test_sanitize_nested_objects() {
        let sanitizer = DataSanitizer::new();
        let data = json!({
            "user": {
                "id": "123",
                "password": "secret",
                "profile": {
                    "email": "test@example.com"
                }
            }
        });

        let sanitized = sanitizer.sanitize_json(&data);

        assert_eq!(sanitized["user"]["id"], "123");
        assert_eq!(sanitized["user"]["password"], "[REDACTED]");
        assert_eq!(sanitized["user"]["profile"]["email"], "[REDACTED]");
    }

    #[test]
    fn test_sanitize_arrays() {
        let sanitizer = DataSanitizer::new();
        let data = json!([
            {"username": "user1", "password": "pass1"},
            {"username": "user2", "password": "pass2"}
        ]);

        let sanitized = sanitizer.sanitize_json(&data);

        if let Value::Array(arr) = sanitized {
            assert_eq!(arr[0]["username"], "user1");
            assert_eq!(arr[0]["password"], "[REDACTED]");
            assert_eq!(arr[1]["username"], "user2");
            assert_eq!(arr[1]["password"], "[REDACTED]");
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_sanitize_string_patterns() {
        let sanitizer = DataSanitizer::new();

        let text = "My email is john@example.com and my phone is 555-123-4567";
        let sanitized = sanitizer.sanitize_string(text);

        assert!(sanitized.contains("[EMAIL-***]"));
        assert!(sanitized.contains("[PHONE-***]"));
        assert!(!sanitized.contains("john@example.com"));
        assert!(!sanitized.contains("555-123-4567"));
    }

    #[test]
    fn test_partial_sanitizer() {
        let sanitizer = DataSanitizer::partial_sanitizer(&["password"]);
        let data = json!({
            "username": "john",
            "password": "secret",
            "email": "john@example.com"
        });

        let sanitized = sanitizer.sanitize_json(&data);

        assert_eq!(sanitized["username"], "john");
        assert_eq!(sanitized["password"], "[REDACTED]");
        assert_eq!(sanitized["email"], "john@example.com"); // Not sanitized
    }
}
