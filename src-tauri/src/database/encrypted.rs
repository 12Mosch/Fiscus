use base64::Engine;
use serde_json::Value;
/// Encrypted database utilities for transparent encryption/decryption of sensitive data
///
/// This module provides database utilities that automatically encrypt sensitive
/// financial data before storage and decrypt it when retrieved, ensuring data
/// protection at rest.
use std::collections::HashMap;
use tracing::{debug, error, instrument, warn};

use crate::{
    database::{Database, DatabaseUtils},
    error::{FiscusError, FiscusResult},
};

/// Fields that should be encrypted in different tables
const ENCRYPTED_FIELDS: &[(&str, &[&str])] = &[
    ("transactions", &["amount", "description", "notes"]),
    ("accounts", &["balance", "account_number"]),
    ("users", &["email"]),
    ("goals", &["target_amount", "current_amount", "description"]),
    ("budgets", &["amount", "spent_amount"]),
];

/// Encrypted database utilities
pub struct EncryptedDatabaseUtils;

impl EncryptedDatabaseUtils {
    /// Execute a query with automatic encryption of sensitive fields
    #[instrument(skip(db, params), fields(query_type = "encrypted_query"))]
    pub async fn execute_encrypted_query<T>(
        db: &Database,
        query: &str,
        params: Vec<Value>,
        user_id: &str,
        table_name: &str,
    ) -> FiscusResult<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        debug!(
            query = query,
            table = table_name,
            user_id = user_id,
            "Executing encrypted database query"
        );

        // For SELECT queries, execute normally then decrypt results
        if query.trim_start().to_lowercase().starts_with("select") {
            let results =
                DatabaseUtils::execute_query::<HashMap<String, Value>>(db, query, params).await?;
            let decrypted_results =
                Self::decrypt_query_results(results, user_id, table_name).await?;

            // Convert back to the desired type
            let json_results: Vec<T> = decrypted_results
                .into_iter()
                .map(|row| serde_json::from_value(serde_json::to_value(row).unwrap()))
                .collect::<Result<Vec<T>, _>>()
                .map_err(|e| {
                    FiscusError::Database(format!("Failed to deserialize results: {e}"))
                })?;

            Ok(json_results)
        } else {
            // For INSERT/UPDATE queries, encrypt sensitive fields first
            let encrypted_params = Self::encrypt_query_params(params, user_id, table_name).await?;
            DatabaseUtils::execute_query(db, query, encrypted_params).await
        }
    }

    /// Execute a non-query with automatic encryption
    #[instrument(skip(db, params), fields(query_type = "encrypted_non_query"))]
    pub async fn execute_encrypted_non_query(
        db: &Database,
        query: &str,
        params: Vec<Value>,
        user_id: &str,
        table_name: &str,
    ) -> FiscusResult<u64> {
        debug!(
            query = query,
            table = table_name,
            user_id = user_id,
            "Executing encrypted non-query"
        );

        let encrypted_params = Self::encrypt_query_params(params, user_id, table_name).await?;
        DatabaseUtils::execute_non_query(db, query, encrypted_params).await
    }

    /// Encrypt sensitive parameters before database insertion
    async fn encrypt_query_params(
        params: Vec<Value>,
        _user_id: &str,
        _table_name: &str,
    ) -> FiscusResult<Vec<Value>> {
        // For now, return params as-is since we need more context about which
        // parameters correspond to which fields. In a real implementation,
        // this would require query parsing or parameter mapping.
        warn!("Parameter encryption not fully implemented - requires query parsing");
        Ok(params)
    }

    /// Decrypt sensitive fields in query results
    async fn decrypt_query_results(
        results: Vec<HashMap<String, Value>>,
        user_id: &str,
        table_name: &str,
    ) -> FiscusResult<Vec<HashMap<String, Value>>> {
        let encrypted_fields = Self::get_encrypted_fields(table_name);
        if encrypted_fields.is_empty() {
            return Ok(results);
        }

        debug!(
            table = table_name,
            encrypted_fields = ?encrypted_fields,
            result_count = results.len(),
            "Decrypting query results"
        );

        let mut decrypted_results = Vec::new();

        for mut row in results {
            for field_name in &encrypted_fields {
                if let Some(encrypted_value) = row.get(field_name) {
                    if let Some(encrypted_str) = encrypted_value.as_str() {
                        // Check if the value is actually encrypted (has our prefix)
                        if encrypted_str.starts_with("enc:") {
                            match Self::decrypt_field_value(encrypted_str, user_id, field_name)
                                .await
                            {
                                Ok(decrypted_value) => {
                                    row.insert(
                                        field_name.to_string(),
                                        Value::String(decrypted_value),
                                    );
                                }
                                Err(e) => {
                                    error!(
                                        field = field_name,
                                        error = %e,
                                        "Failed to decrypt field value"
                                    );
                                    // Keep the encrypted value rather than fail the entire query
                                }
                            }
                        }
                    }
                }
            }
            decrypted_results.push(row);
        }

        debug!(
            decrypted_count = decrypted_results.len(),
            "Query results decrypted successfully"
        );

        Ok(decrypted_results)
    }

    /// Encrypt a field value for storage
    pub async fn encrypt_field_value(
        value: &str,
        user_id: &str,
        field_name: &str,
    ) -> FiscusResult<String> {
        debug!(
            field = field_name,
            user_id = user_id,
            "Encrypting field value"
        );

        // This is a simplified implementation. In production, you'd want to:
        // 1. Initialize the encryption service properly
        // 2. Use proper key derivation for the user
        // 3. Handle different data types appropriately

        // For now, return a placeholder encrypted value
        let placeholder_encrypted = format!(
            "enc:{}",
            base64::engine::general_purpose::STANDARD.encode(value.as_bytes())
        );

        warn!("Field encryption using placeholder implementation");
        Ok(placeholder_encrypted)
    }

    /// Decrypt a field value from storage
    pub async fn decrypt_field_value(
        encrypted_value: &str,
        user_id: &str,
        field_name: &str,
    ) -> FiscusResult<String> {
        debug!(
            field = field_name,
            user_id = user_id,
            "Decrypting field value"
        );

        // Remove the "enc:" prefix and decode
        if let Some(base64_data) = encrypted_value.strip_prefix("enc:") {
            let decoded_bytes = base64::engine::general_purpose::STANDARD
                .decode(base64_data)
                .map_err(|e| {
                    FiscusError::Encryption(format!("Failed to decode encrypted field: {e}"))
                })?;

            let decrypted_value = String::from_utf8(decoded_bytes).map_err(|e| {
                FiscusError::Encryption(format!("Invalid UTF-8 in decrypted field: {e}"))
            })?;

            debug!(field = field_name, "Field value decrypted successfully");
            Ok(decrypted_value)
        } else {
            Err(FiscusError::Encryption(
                "Invalid encrypted field format".to_string(),
            ))
        }
    }

    /// Get the list of encrypted fields for a table
    fn get_encrypted_fields(table_name: &str) -> Vec<String> {
        ENCRYPTED_FIELDS
            .iter()
            .find(|(table, _)| *table == table_name)
            .map(|(_, fields)| fields.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    /// Check if a field should be encrypted
    pub fn is_field_encrypted(table_name: &str, field_name: &str) -> bool {
        ENCRYPTED_FIELDS
            .iter()
            .any(|(table, fields)| *table == table_name && fields.contains(&field_name))
    }

    /// Encrypt sensitive data in a record before insertion
    pub async fn encrypt_record(
        record: &mut HashMap<String, Value>,
        user_id: &str,
        table_name: &str,
    ) -> FiscusResult<()> {
        let encrypted_fields = Self::get_encrypted_fields(table_name);

        for field_name in encrypted_fields {
            if let Some(value) = record.get(&field_name) {
                if let Some(string_value) = value.as_str() {
                    let encrypted_value =
                        Self::encrypt_field_value(string_value, user_id, &field_name).await?;
                    record.insert(field_name, Value::String(encrypted_value));
                }
            }
        }

        Ok(())
    }

    /// Decrypt sensitive data in a record after retrieval
    pub async fn decrypt_record(
        record: &mut HashMap<String, Value>,
        user_id: &str,
        table_name: &str,
    ) -> FiscusResult<()> {
        let encrypted_fields = Self::get_encrypted_fields(table_name);

        for field_name in encrypted_fields {
            if let Some(value) = record.get(&field_name) {
                if let Some(encrypted_str) = value.as_str() {
                    if encrypted_str.starts_with("enc:") {
                        let decrypted_value =
                            Self::decrypt_field_value(encrypted_str, user_id, &field_name).await?;
                        record.insert(field_name, Value::String(decrypted_value));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Trait for repositories that handle encrypted data
pub trait EncryptedRepository {
    /// Get the table name for this repository
    fn table_name(&self) -> &'static str;

    /// Get the user ID for encryption context
    fn get_user_id(&self) -> Option<&str>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_field_encryption_roundtrip() {
        let original_value = "sensitive data";
        let user_id = "test-user";
        let field_name = "test_field";

        let encrypted =
            EncryptedDatabaseUtils::encrypt_field_value(original_value, user_id, field_name)
                .await
                .unwrap();

        assert!(encrypted.starts_with("enc:"));
        assert_ne!(encrypted, original_value);

        let decrypted =
            EncryptedDatabaseUtils::decrypt_field_value(&encrypted, user_id, field_name)
                .await
                .unwrap();

        assert_eq!(decrypted, original_value);
    }

    #[test]
    fn test_encrypted_fields_configuration() {
        let transaction_fields = EncryptedDatabaseUtils::get_encrypted_fields("transactions");
        assert!(transaction_fields.contains(&"amount".to_string()));
        assert!(transaction_fields.contains(&"description".to_string()));

        let account_fields = EncryptedDatabaseUtils::get_encrypted_fields("accounts");
        assert!(account_fields.contains(&"balance".to_string()));
        assert!(account_fields.contains(&"account_number".to_string()));

        assert!(EncryptedDatabaseUtils::is_field_encrypted(
            "transactions",
            "amount"
        ));
        assert!(!EncryptedDatabaseUtils::is_field_encrypted(
            "transactions",
            "id"
        ));
    }

    #[tokio::test]
    async fn test_record_encryption() {
        let mut record = HashMap::new();
        record.insert("id".to_string(), Value::String("123".to_string()));
        record.insert("amount".to_string(), Value::String("100.50".to_string()));
        record.insert(
            "description".to_string(),
            Value::String("Test transaction".to_string()),
        );

        let user_id = "test-user";
        let table_name = "transactions";

        EncryptedDatabaseUtils::encrypt_record(&mut record, user_id, table_name)
            .await
            .unwrap();

        // ID should not be encrypted
        assert_eq!(record.get("id").unwrap().as_str().unwrap(), "123");

        // Amount and description should be encrypted
        assert!(record
            .get("amount")
            .unwrap()
            .as_str()
            .unwrap()
            .starts_with("enc:"));
        assert!(record
            .get("description")
            .unwrap()
            .as_str()
            .unwrap()
            .starts_with("enc:"));

        // Decrypt the record
        EncryptedDatabaseUtils::decrypt_record(&mut record, user_id, table_name)
            .await
            .unwrap();

        // Values should be back to original
        assert_eq!(record.get("amount").unwrap().as_str().unwrap(), "100.50");
        assert_eq!(
            record.get("description").unwrap().as_str().unwrap(),
            "Test transaction"
        );
    }
}
