use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::HashMap;

/// Utility functions for common data parsing operations
///
/// Parse a decimal value from a JSON HashMap field
///
/// This function safely extracts and parses a decimal value from a HashMap containing
/// JSON values. It handles the common pattern of:
/// 1. Getting the field from the HashMap
/// 2. Converting the JSON Value to a string
/// 3. Parsing the string as a Decimal
/// 4. Providing a default value if any step fails
///
/// # Arguments
///
/// * `data` - The HashMap containing JSON values
/// * `field_name` - The name of the field to extract
///
/// # Returns
///
/// Returns the parsed Decimal value, or Decimal::ZERO if parsing fails
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use serde_json::Value;
/// use fiscus_lib::utils::parse_decimal_from_json;
///
/// let mut data = HashMap::new();
/// data.insert("amount".to_string(), Value::String("123.45".to_string()));
///
/// let amount = parse_decimal_from_json(&data, "amount");
/// assert_eq!(amount.to_string(), "123.45");
/// ```
pub fn parse_decimal_from_json(data: &HashMap<String, Value>, field_name: &str) -> Decimal {
    parse_decimal_from_json_with_default(data, field_name, Decimal::ZERO)
}

/// Parse a decimal value from a JSON HashMap field with a custom default
///
/// This function safely extracts and parses a decimal value from a HashMap containing
/// JSON values, allowing you to specify a custom default value.
///
/// # Arguments
///
/// * `data` - The HashMap containing JSON values
/// * `field_name` - The name of the field to extract
/// * `default` - The default value to return if parsing fails
///
/// # Returns
///
/// Returns the parsed Decimal value, or the provided default if parsing fails
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use serde_json::Value;
/// use rust_decimal::Decimal;
/// use fiscus_lib::utils::parse_decimal_from_json_with_default;
///
/// let mut data = HashMap::new();
/// data.insert("amount".to_string(), Value::String("invalid".to_string()));
///
/// let amount = parse_decimal_from_json_with_default(&data, "amount", Decimal::new(100, 0));
/// assert_eq!(amount.to_string(), "100");
/// ```
pub fn parse_decimal_from_json_with_default(
    data: &HashMap<String, Value>,
    field_name: &str,
    default: Decimal,
) -> Decimal {
    data.get(field_name)
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_parse_decimal_from_json_success() {
        let mut data = HashMap::new();
        data.insert("amount".to_string(), Value::String("123.45".to_string()));

        let result = parse_decimal_from_json(&data, "amount");
        assert_eq!(result.to_string(), "123.45");
    }

    #[test]
    fn test_parse_decimal_from_json_missing_field() {
        let data = HashMap::new();

        let result = parse_decimal_from_json(&data, "amount");
        assert_eq!(result, Decimal::ZERO);
    }

    #[test]
    fn test_parse_decimal_from_json_invalid_value() {
        let mut data = HashMap::new();
        data.insert("amount".to_string(), Value::String("invalid".to_string()));

        let result = parse_decimal_from_json(&data, "amount");
        assert_eq!(result, Decimal::ZERO);
    }

    #[test]
    fn test_parse_decimal_from_json_non_string_value() {
        let mut data = HashMap::new();
        data.insert(
            "amount".to_string(),
            Value::Number(serde_json::Number::from(123)),
        );

        let result = parse_decimal_from_json(&data, "amount");
        assert_eq!(result, Decimal::ZERO); // Should fail because it's not a string
    }

    #[test]
    fn test_parse_decimal_from_json_with_custom_default() {
        let mut data = HashMap::new();
        data.insert("amount".to_string(), Value::String("invalid".to_string()));

        let custom_default = Decimal::new(999, 0);
        let result = parse_decimal_from_json_with_default(&data, "amount", custom_default);
        assert_eq!(result, custom_default);
    }

    #[test]
    fn test_parse_decimal_from_json_zero_value() {
        let mut data = HashMap::new();
        data.insert("amount".to_string(), Value::String("0".to_string()));

        let result = parse_decimal_from_json(&data, "amount");
        assert_eq!(result, Decimal::ZERO);
    }

    #[test]
    fn test_parse_decimal_from_json_negative_value() {
        let mut data = HashMap::new();
        data.insert("amount".to_string(), Value::String("-123.45".to_string()));

        let result = parse_decimal_from_json(&data, "amount");
        assert_eq!(result.to_string(), "-123.45");
    }
}
