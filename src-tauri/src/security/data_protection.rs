use serde::{Deserialize, Deserializer};
use std::fmt;

/// Wrapper for sensitive data that prevents accidental logging or serialization
///
/// This type wraps sensitive data like passwords to prevent them from being
/// accidentally logged, printed, or serialized. It provides controlled access
/// to the inner value through explicit methods.
#[derive(Clone, PartialEq)]
pub struct SensitiveData<T> {
    inner: T,
}

impl<T> SensitiveData<T> {
    /// Create a new SensitiveData wrapper
    pub fn new(data: T) -> Self {
        Self { inner: data }
    }

    /// Expose the inner value (use with caution)
    ///
    /// This method provides access to the wrapped sensitive data.
    /// Use this method carefully and ensure the exposed data is handled securely.
    pub fn expose(&self) -> &T {
        &self.inner
    }

    /// Consume the wrapper and return the inner value
    ///
    /// This method consumes the SensitiveData and returns the wrapped value.
    /// Use this method carefully and ensure the returned data is handled securely.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

/// Custom Debug implementation to prevent logging sensitive data
impl<T> fmt::Debug for SensitiveData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SensitiveData")
            .field("inner", &"[REDACTED]")
            .finish()
    }
}

/// Custom Display implementation to prevent printing sensitive data
impl<T> fmt::Display for SensitiveData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// Custom Deserialize implementation for SensitiveData<String>
///
/// This allows JSON strings to be deserialized directly into SensitiveData<String>
impl<'de> Deserialize<'de> for SensitiveData<String> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SensitiveData::new(s))
    }
}

/// Type alias for sensitive password data
pub type SensitivePassword = SensitiveData<String>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_sensitive_data_creation() {
        let sensitive = SensitiveData::new("secret".to_string());
        assert_eq!(sensitive.expose(), "secret");
    }

    #[test]
    fn test_sensitive_data_into_inner() {
        let sensitive = SensitiveData::new("secret".to_string());
        let inner = sensitive.into_inner();
        assert_eq!(inner, "secret");
    }

    #[test]
    fn test_sensitive_data_debug() {
        let sensitive = SensitiveData::new("secret".to_string());
        let debug_str = format!("{sensitive:?}");
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("secret"));
    }

    #[test]
    fn test_sensitive_data_display() {
        let sensitive = SensitiveData::new("secret".to_string());
        let display_str = format!("{sensitive}");
        assert_eq!(display_str, "[REDACTED]");
    }

    #[test]
    fn test_sensitive_data_deserialization() {
        let json = r#""my_password""#;
        let sensitive: SensitiveData<String> = serde_json::from_str(json).unwrap();
        assert_eq!(sensitive.expose(), "my_password");
    }

    #[test]
    fn test_sensitive_password_type_alias() {
        let password: SensitivePassword = SensitiveData::new("password123".to_string());
        assert_eq!(password.expose(), "password123");
    }

    #[test]
    fn test_sensitive_data_clone() {
        let original = SensitiveData::new("secret".to_string());
        let cloned = original.clone();
        assert_eq!(original.expose(), cloned.expose());
    }

    #[test]
    fn test_json_deserialization_in_struct() {
        #[derive(Deserialize)]
        struct TestStruct {
            password: SensitiveData<String>,
        }

        let json = r#"{"password": "test123"}"#;
        let test_struct: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(test_struct.password.expose(), "test123");
    }
}
