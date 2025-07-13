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
    commands::encryption::get_encryption_service,
    database::{Database, DatabaseUtils},
    encryption::types::EncryptedData,
    error::{FiscusError, FiscusResult},
};

/// Fields that should be encrypted in different tables
const ENCRYPTED_FIELDS: &[(&str, &[&str])] = &[
    ("transactions", &["amount", "description", "notes"]),
    ("accounts", &["balance", "account_number"]),
    ("users", &["email"]),
    ("goals", &["target_amount", "current_amount", "description"]),
    ("budgets", &["allocated_amount", "spent_amount"]),
    ("transfers", &["amount", "description"]),
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
    ///
    /// SECURITY CRITICAL: This function encrypts parameters that will be inserted into
    /// sensitive database fields. It must not be bypassed in production.
    async fn encrypt_query_params(
        params: Vec<Value>,
        user_id: &str,
        table_name: &str,
    ) -> FiscusResult<Vec<Value>> {
        // PRODUCTION GUARD: Fail fast if this is called in production without proper implementation
        #[cfg(not(debug_assertions))]
        {
            error!(
                table = table_name,
                user_id = user_id,
                param_count = params.len(),
                "CRITICAL SECURITY ERROR: encrypt_query_params called in production without proper parameter mapping"
            );
            return Err(FiscusError::Security(
                "Parameter encryption not implemented for production use. This would expose sensitive data.".to_string()
            ));
        }

        let encrypted_fields = Self::get_encrypted_fields(table_name);

        // If no fields need encryption for this table, return params as-is
        if encrypted_fields.is_empty() {
            debug!(
                table = table_name,
                "No encrypted fields configured for table, returning parameters unchanged"
            );
            return Ok(params);
        }

        // FAIL-FAST: If we have encrypted fields but no proper mapping strategy,
        // we must not allow potentially sensitive data to pass through unencrypted
        error!(
            table = table_name,
            user_id = user_id,
            encrypted_fields = ?encrypted_fields,
            param_count = params.len(),
            "SECURITY VIOLATION: Cannot safely encrypt parameters without field mapping. Sensitive data may be exposed."
        );

        // In development, log the issue but continue with a warning
        #[cfg(debug_assertions)]
        {
            warn!(
                "DEVELOPMENT MODE: Parameter encryption not fully implemented. Use encrypt_record() for structured data operations."
            );
            Ok(params)
        }
    }

    /// Encrypt parameters with explicit field mapping (RECOMMENDED APPROACH)
    ///
    /// This function provides a safer alternative to encrypt_query_params by requiring
    /// explicit mapping of parameters to field names, preventing accidental exposure
    /// of sensitive data.
    ///
    /// # Arguments
    /// * `params` - Parameters with their corresponding field names
    /// * `user_id` - User ID for key derivation
    /// * `table_name` - Table name for encryption field lookup
    ///
    /// # Returns
    /// * `FiscusResult<Vec<Value>>` - Encrypted parameters in the same order
    pub async fn encrypt_params_with_mapping(
        params: Vec<(String, Value)>, // (field_name, value) pairs
        user_id: &str,
        table_name: &str,
    ) -> FiscusResult<Vec<Value>> {
        debug!(
            table = table_name,
            user_id = user_id,
            param_count = params.len(),
            "Encrypting parameters with explicit field mapping"
        );

        let mut encrypted_params = Vec::with_capacity(params.len());

        for (field_name, value) in params {
            let encrypted_value = if Self::is_field_encrypted(table_name, &field_name) {
                // Encrypt sensitive field
                if let Some(string_value) = value.as_str() {
                    let encrypted =
                        Self::encrypt_field_value(string_value, user_id, &field_name).await?;
                    Value::String(encrypted)
                } else {
                    warn!(
                        field = field_name,
                        table = table_name,
                        "Non-string value in encrypted field, passing through unchanged"
                    );
                    value
                }
            } else {
                // Non-sensitive field, pass through unchanged
                value
            };

            encrypted_params.push(encrypted_value);
        }

        debug!(
            table = table_name,
            encrypted_count = encrypted_params.len(),
            "Parameters encrypted successfully with field mapping"
        );

        Ok(encrypted_params)
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

    /// Encrypt a field value for storage using AES-256-GCM
    pub async fn encrypt_field_value(
        value: &str,
        user_id: &str,
        field_name: &str,
    ) -> FiscusResult<String> {
        debug!(
            field = field_name,
            user_id = user_id,
            "Encrypting field value with AES-256-GCM"
        );

        // Get the global encryption service
        let encryption_service = get_encryption_service().map_err(|e| {
            error!("Failed to get encryption service: {}", e);
            FiscusError::Encryption("Encryption service not available".to_string())
        })?;

        // Encrypt the field value using AES-256-GCM with user-specific key derivation
        let encrypted_data = encryption_service
            .encrypt_financial_data(value.as_bytes(), user_id, field_name)
            .await
            .map_err(|e| {
                error!("Failed to encrypt field value: {}", e);
                FiscusError::Encryption(format!("Field encryption failed: {e}"))
            })?;

        // Serialize the encrypted data to JSON and base64 encode for storage
        let serialized = serde_json::to_string(&encrypted_data).map_err(|e| {
            error!("Failed to serialize encrypted data: {}", e);
            FiscusError::Encryption(format!("Failed to serialize encrypted data: {e}"))
        })?;

        let encoded = base64::engine::general_purpose::STANDARD.encode(serialized.as_bytes());
        let result = format!("enc:{encoded}");

        debug!(
            field = field_name,
            user_id = user_id,
            "Field value encrypted successfully with AES-256-GCM"
        );
        Ok(result)
    }

    /// Decrypt a field value from storage using AES-256-GCM
    pub async fn decrypt_field_value(
        encrypted_value: &str,
        user_id: &str,
        field_name: &str,
    ) -> FiscusResult<String> {
        debug!(
            field = field_name,
            user_id = user_id,
            "Decrypting field value with AES-256-GCM"
        );

        // Remove the "enc:" prefix and decode
        if let Some(base64_data) = encrypted_value.strip_prefix("enc:") {
            // Decode the base64 data
            let decoded_bytes = base64::engine::general_purpose::STANDARD
                .decode(base64_data)
                .map_err(|e| {
                    error!("Failed to decode base64 encrypted field: {}", e);
                    FiscusError::Encryption(format!("Failed to decode encrypted field: {e}"))
                })?;

            // Deserialize the JSON to EncryptedData
            let serialized_data = String::from_utf8(decoded_bytes).map_err(|e| {
                error!("Invalid UTF-8 in serialized encrypted data: {}", e);
                FiscusError::Encryption(format!("Invalid UTF-8 in encrypted field: {e}"))
            })?;

            let encrypted_data: EncryptedData =
                serde_json::from_str(&serialized_data).map_err(|e| {
                    error!("Failed to deserialize encrypted data: {}", e);
                    FiscusError::Encryption(format!("Failed to deserialize encrypted data: {e}"))
                })?;

            // Get the global encryption service
            let encryption_service = get_encryption_service().map_err(|e| {
                error!("Failed to get encryption service: {}", e);
                FiscusError::Encryption("Encryption service not available".to_string())
            })?;

            // Decrypt the data using AES-256-GCM
            let decrypted_bytes = encryption_service
                .decrypt_financial_data(&encrypted_data, user_id, field_name)
                .await
                .map_err(|e| {
                    error!("Failed to decrypt field value: {}", e);
                    FiscusError::Encryption(format!("Field decryption failed: {e}"))
                })?;

            let decrypted_value = String::from_utf8(decrypted_bytes).map_err(|e| {
                error!("Invalid UTF-8 in decrypted field value: {}", e);
                FiscusError::Encryption(format!("Invalid UTF-8 in decrypted field: {e}"))
            })?;

            debug!(
                field = field_name,
                user_id = user_id,
                "Field value decrypted successfully with AES-256-GCM"
            );
            Ok(decrypted_value)
        } else {
            Err(FiscusError::Encryption(
                "Invalid encrypted field format - missing 'enc:' prefix".to_string(),
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
        // Initialize the encryption service for testing
        crate::commands::encryption::initialize_encryption_service()
            .expect("Failed to initialize encryption service for test");

        let original_value = "sensitive data";
        // deepcode ignore NoHardcodedCredentials: <test>
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

    #[tokio::test]
    async fn test_encrypt_params_with_mapping() {
        // Initialize the encryption service for testing
        crate::commands::encryption::initialize_encryption_service()
            .expect("Failed to initialize encryption service for test");

        let params = vec![
            ("id".to_string(), Value::String("123".to_string())),
            ("amount".to_string(), Value::String("100.50".to_string())),
            (
                "description".to_string(),
                Value::String("Test transaction".to_string()),
            ),
            ("category".to_string(), Value::String("food".to_string())),
        ];

        // deepcode ignore NoHardcodedCredentials: <test>
        let user_id = "test-user";
        let table_name = "transactions";

        let encrypted_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
            params.clone(),
            user_id,
            table_name,
        )
        .await
        .unwrap();

        // Should have same number of parameters
        assert_eq!(encrypted_params.len(), 4);

        // Non-encrypted fields should be unchanged
        assert_eq!(encrypted_params[0], Value::String("123".to_string())); // id
        assert_eq!(encrypted_params[3], Value::String("food".to_string())); // category

        // Encrypted fields should be different and start with "enc:"
        if let Value::String(encrypted_amount) = &encrypted_params[1] {
            assert!(encrypted_amount.starts_with("enc:"));
            assert_ne!(encrypted_amount, "100.50");
        } else {
            panic!("Expected encrypted amount to be a string");
        }

        if let Value::String(encrypted_desc) = &encrypted_params[2] {
            assert!(encrypted_desc.starts_with("enc:"));
            assert_ne!(encrypted_desc, "Test transaction");
        } else {
            panic!("Expected encrypted description to be a string");
        }
    }

    #[tokio::test]
    async fn test_encrypt_query_params_security_guard() {
        // deepcode ignore NoHardcodedCredentials: <test>
        let params = vec![Value::String("sensitive_data".to_string())];
        // deepcode ignore NoHardcodedCredentials: <test>
        let user_id = "test-user";
        let table_name = "transactions"; // Has encrypted fields

        // In debug mode, this should return params with a warning
        #[cfg(debug_assertions)]
        {
            let result =
                EncryptedDatabaseUtils::encrypt_query_params(params, user_id, table_name).await;
            assert!(result.is_ok());
        }

        // Test with table that has no encrypted fields
        let result = EncryptedDatabaseUtils::encrypt_query_params(
            vec![Value::String("data".to_string())],
            user_id,
            "non_encrypted_table",
        )
        .await;
        assert!(result.is_ok());
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
        
        // deepcode ignore NoHardcodedCredentials: <test>
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
