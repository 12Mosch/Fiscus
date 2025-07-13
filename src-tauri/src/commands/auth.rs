use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;
use serde_json::Value;
use tauri::State;
use uuid::Uuid;

use crate::{
    database::{encrypted::EncryptedDatabaseUtils, Database, DatabaseUtils},
    dto::{ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, UserResponse},
    error::{FiscusError, FiscusResult, Validator},
};

/// Create a new user account
#[tauri::command]
pub async fn create_user(
    request: CreateUserRequest,
    db: State<'_, Database>,
) -> Result<UserResponse, FiscusError> {
    // Validate input
    Validator::validate_string(&request.username, "username", 3, 50)?;
    Validator::validate_string(&request.password, "password", 8, 128)?;

    if let Some(ref email) = request.email {
        Validator::validate_email(email)?;
    }

    // Check if username already exists
    let existing_user_query = "SELECT id FROM users WHERE username = ?1";
    let existing_user: Option<std::collections::HashMap<String, Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            existing_user_query,
            vec![Value::String(request.username.clone())],
        )
        .await?;

    if existing_user.is_some() {
        return Err(FiscusError::Conflict("Username already exists".to_string()));
    }

    // Check if email already exists (if provided)
    if let Some(ref email) = request.email {
        let existing_email_query = "SELECT id FROM users WHERE email = ?1";
        let existing_email: Option<std::collections::HashMap<String, Value>> =
            DatabaseUtils::execute_query_single(
                &db,
                existing_email_query,
                vec![Value::String(email.clone())],
            )
            .await?;

        if existing_email.is_some() {
            return Err(FiscusError::Conflict("Email already exists".to_string()));
        }
    }

    // Hash password
    let password_hash = hash_password(&request.password)?;

    // Create user
    let user_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let insert_query = r#"
        INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
    "#;

    // Use encrypted parameter mapping for sensitive fields
    let params_with_mapping = vec![
        ("id".to_string(), Value::String(user_id.clone())),
        (
            "username".to_string(),
            Value::String(request.username.clone()),
        ),
        (
            "email".to_string(),
            request
                .email
                .as_ref()
                .map(|e| Value::String(e.clone()))
                .unwrap_or(Value::Null),
        ),
        ("password_hash".to_string(), Value::String(password_hash)),
        ("created_at".to_string(), Value::String(now.clone())),
        ("updated_at".to_string(), Value::String(now)),
    ];

    let encrypted_params =
        EncryptedDatabaseUtils::encrypt_params_with_mapping(params_with_mapping, &user_id, "users")
            .await?;

    DatabaseUtils::execute_non_query(&db, insert_query, encrypted_params).await?;

    // Return user response (without password hash)
    Ok(UserResponse {
        id: user_id,
        username: request.username,
        email: request.email,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

/// Authenticate user login
#[tauri::command]
pub async fn login_user(
    request: LoginRequest,
    db: State<'_, Database>,
) -> Result<LoginResponse, FiscusError> {
    // Validate input
    Validator::validate_string(&request.username, "username", 1, 50)?;
    Validator::validate_string(&request.password, "password", 1, 128)?;

    // Find user by username - first get user_id for encryption context
    let user_id_query = "SELECT id FROM users WHERE username = ?1";
    let user_id_row: Option<std::collections::HashMap<String, Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            user_id_query,
            vec![Value::String(request.username.clone())],
        )
        .await?;

    let user_id = user_id_row
        .and_then(|row| {
            row.get("id")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        })
        .ok_or_else(|| FiscusError::Authentication("Invalid credentials".to_string()))?;

    // Now get full user data with decryption
    let user_query = "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE username = ?1";
    let user_rows: Vec<std::collections::HashMap<String, Value>> =
        EncryptedDatabaseUtils::execute_encrypted_query(
            &db,
            user_query,
            vec![Value::String(request.username)],
            &user_id,
            "users",
        )
        .await?;

    let user_row = user_rows.into_iter().next();

    let user_data =
        user_row.ok_or_else(|| FiscusError::Authentication("Invalid credentials".to_string()))?;

    // Extract password hash
    let stored_hash = user_data
        .get("password_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| FiscusError::Database("Invalid user data".to_string()))?;

    // Verify password
    if !verify_password(&request.password, stored_hash)? {
        return Err(FiscusError::Authentication(
            "Invalid credentials".to_string(),
        ));
    }

    // Create user response
    let user_response = UserResponse {
        id: user_data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        username: user_data
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        email: user_data
            .get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        created_at: user_data
            .get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now),
        updated_at: user_data
            .get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now),
    };

    Ok(LoginResponse {
        user: user_response,
        session_token: None, // TODO: Implement session tokens if needed
    })
}

/// Change user password
#[tauri::command]
pub async fn change_password(
    request: ChangePasswordRequest,
    db: State<'_, Database>,
) -> Result<bool, FiscusError> {
    // Validate input
    Validator::validate_uuid(&request.user_id, "user_id")?;
    Validator::validate_string(&request.current_password, "current_password", 1, 128)?;
    Validator::validate_string(&request.new_password, "new_password", 8, 128)?;

    // Get current user data
    let user_query = "SELECT password_hash FROM users WHERE id = ?1";
    let user_row: Option<std::collections::HashMap<String, Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            user_query,
            vec![Value::String(request.user_id.clone())],
        )
        .await?;

    let user_data = user_row.ok_or_else(|| FiscusError::NotFound("User not found".to_string()))?;

    // Verify current password
    let stored_hash = user_data
        .get("password_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| FiscusError::Database("Invalid user data".to_string()))?;

    if !verify_password(&request.current_password, stored_hash)? {
        return Err(FiscusError::Authentication(
            "Current password is incorrect".to_string(),
        ));
    }

    // Hash new password
    let new_password_hash = hash_password(&request.new_password)?;

    // Update password
    let update_query = "UPDATE users SET password_hash = ?1, updated_at = ?2 WHERE id = ?3";
    let params = vec![
        Value::String(new_password_hash),
        Value::String(chrono::Utc::now().to_rfc3339()),
        Value::String(request.user_id),
    ];

    let affected_rows = DatabaseUtils::execute_non_query(&db, update_query, params).await?;

    Ok(affected_rows > 0)
}

/// Get current user information
#[tauri::command]
pub async fn get_current_user(
    user_id: String,
    db: State<'_, Database>,
) -> Result<UserResponse, FiscusError> {
    // Validate input
    Validator::validate_uuid(&user_id, "user_id")?;

    let user_query = "SELECT id, username, email, created_at, updated_at FROM users WHERE id = ?1";

    // Use encrypted query to properly decrypt email field
    let user_rows: Vec<std::collections::HashMap<String, Value>> =
        EncryptedDatabaseUtils::execute_encrypted_query(
            &db,
            user_query,
            vec![Value::String(user_id.clone())],
            &user_id,
            "users",
        )
        .await?;

    let user_row = user_rows.into_iter().next();

    let user_data = user_row.ok_or_else(|| FiscusError::NotFound("User not found".to_string()))?;

    Ok(UserResponse {
        id: user_data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        username: user_data
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        email: user_data
            .get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        created_at: user_data
            .get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now),
        updated_at: user_data
            .get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now),
    })
}

/// Hash a password using Argon2
fn hash_password(password: &str) -> FiscusResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(FiscusError::from)?
        .to_string();

    Ok(password_hash)
}

/// Verify a password against its hash
fn verify_password(password: &str, hash: &str) -> FiscusResult<bool> {
    let parsed_hash = PasswordHash::new(hash).map_err(FiscusError::from)?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestUtils;

    #[test]
    fn test_hash_password() {
        // deepcode ignore HardcodedPassword: <test>
        let password = "test_password_123";
        let result = hash_password(password);

        assert!(result.is_ok());
        let hash = result.unwrap();

        // Hash should not be empty
        assert!(!hash.is_empty());

        // Hash should be different from the original password
        assert_ne!(hash, password);

        // Hash should start with $argon2id$ (Argon2 format)
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_hash_password_different_salts() {
        // deepcode ignore HardcodedPassword: <test>
        let password = "test_password_123";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Same password should produce different hashes due to different salts
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_password_correct() {
        // deepcode ignore HardcodedPassword: <test>
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        // deepcode ignore HardcodedPassword: <test>
        let password = "test_password_123";
        // deepcode ignore HardcodedPassword: <test>
        let wrong_password = "wrong_password";
        let hash = hash_password(password).unwrap();

        let result = verify_password(wrong_password, &hash);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        // deepcode ignore HardcodedPassword: <test>
        let password = "test_password_123";
        let invalid_hash = "invalid_hash_format";

        let result = verify_password(password, invalid_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_user_validation_logic() {
        // Test username validation
        let result = Validator::validate_string("ab", "username", 3, 50);
        assert!(result.is_err());

        let result = Validator::validate_string("validuser", "username", 3, 50);
        assert!(result.is_ok());

        // Test password validation
        let result = Validator::validate_string("short", "password", 8, 128);
        assert!(result.is_err());

        let result = Validator::validate_string("validpassword123", "password", 8, 128);
        assert!(result.is_ok());

        // Test email validation
        let result = Validator::validate_email("invalid-email");
        assert!(result.is_err());

        let result = Validator::validate_email("valid@example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_user_request_structure() {
        let request = CreateUserRequest {
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            password: "password123".to_string(),
        };

        // Test that the request structure is correct
        assert_eq!(request.username, "testuser");
        assert_eq!(request.email, Some("test@example.com".to_string()));
        assert_eq!(request.password, "password123");

        // Test cloning works
        let cloned = request.clone();
        assert_eq!(cloned.username, request.username);
        assert_eq!(cloned.email, request.email);
        assert_eq!(cloned.password, request.password);
    }

    #[test]
    fn test_login_validation_logic() {
        // Test username validation for login
        let result = Validator::validate_string("", "username", 1, 50);
        assert!(result.is_err());

        let result = Validator::validate_string("validuser", "username", 1, 50);
        assert!(result.is_ok());

        // Test password validation for login (different from create - min 1 instead of 8)
        let result = Validator::validate_string("", "password", 1, 128);
        assert!(result.is_err());

        let result = Validator::validate_string("anypassword", "password", 1, 128);
        assert!(result.is_ok());
    }

    #[test]
    fn test_change_password_validation_logic() {
        // Test UUID validation
        let result = Validator::validate_uuid("invalid-uuid", "user_id");
        assert!(result.is_err());

        let valid_uuid = uuid::Uuid::new_v4().to_string();
        let result = Validator::validate_uuid(&valid_uuid, "user_id");
        assert!(result.is_ok());

        // Test current password validation
        let result = Validator::validate_string("", "current_password", 1, 128);
        assert!(result.is_err());

        let result = Validator::validate_string("current123", "current_password", 1, 128);
        assert!(result.is_ok());

        // Test new password validation
        let result = Validator::validate_string("short", "new_password", 8, 128);
        assert!(result.is_err());

        let result = Validator::validate_string("newpassword123", "new_password", 8, 128);
        assert!(result.is_ok());
    }

    #[test]
    fn test_change_password_request_structure() {
        let request = ChangePasswordRequest {
            user_id: uuid::Uuid::new_v4().to_string(),
            current_password: "current123".to_string(),
            new_password: "newpassword123".to_string(),
        };

        // Test that the request structure is correct
        assert!(!request.user_id.is_empty());
        assert_eq!(request.current_password, "current123");
        assert_eq!(request.new_password, "newpassword123");
    }

    #[test]
    fn test_get_current_user_validation_logic() {
        // Test UUID validation
        let result = Validator::validate_uuid("invalid-uuid", "user_id");
        assert!(result.is_err());

        let valid_uuid = uuid::Uuid::new_v4().to_string();
        let result = Validator::validate_uuid(&valid_uuid, "user_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_hashing_security() {
        let passwords = vec![
            "simple123",
            "Complex!Password@2023",
            "very_long_password_with_many_characters_123456789",
            "短密码123", // Unicode characters
            "password with spaces",
        ];

        for password in passwords {
            let hash_result = hash_password(password);
            assert!(hash_result.is_ok(), "Failed to hash password: {password}");

            let hash = hash_result.unwrap();

            // Verify the password can be verified
            let verify_result = verify_password(password, &hash);
            assert!(
                verify_result.is_ok(),
                "Failed to verify password: {password}"
            );
            assert!(
                verify_result.unwrap(),
                "Password verification failed for: {password}"
            );

            // Verify wrong password fails
            let wrong_verify = verify_password("wrong_password", &hash);
            assert!(wrong_verify.is_ok());
            assert!(
                !wrong_verify.unwrap(),
                "Wrong password should not verify for: {password}"
            );
        }
    }

    #[test]
    fn test_password_hash_format() {
        // deepcode ignore HardcodedPassword: <test>
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        // Argon2 hash should have specific format
        let parts: Vec<&str> = hash.split('$').collect();
        assert!(
            parts.len() >= 6,
            "Hash should have at least 6 parts separated by $"
        );
        assert_eq!(parts[1], "argon2id", "Should use argon2id variant");
        assert_eq!(parts[2], "v=19", "Should use version 19");

        // Parameters should be present
        assert!(parts[3].contains("m="), "Should contain memory parameter");
        assert!(parts[3].contains("t="), "Should contain time parameter");
        assert!(
            parts[3].contains("p="),
            "Should contain parallelism parameter"
        );
    }

    #[test]
    fn test_create_user_request_helper() {
        let request =
            TestUtils::create_user_request("testuser", Some("test@example.com"), "password123");

        assert_eq!(request.username, "testuser");
        assert_eq!(request.email, Some("test@example.com".to_string()));
        assert_eq!(request.password, "password123");
    }

    #[test]
    fn test_login_request_helper() {
        let request = TestUtils::login_request("testuser", "password123");

        assert_eq!(request.username, "testuser");
        assert_eq!(request.password, "password123");
    }

    #[test]
    fn test_edge_cases() {
        // Test maximum length username
        let long_username = "a".repeat(50);
        let request = CreateUserRequest {
            username: long_username.clone(),
            email: None,
            password: "password123".to_string(),
        };

        // Should not panic on validation
        let validation_result = Validator::validate_string(&request.username, "username", 3, 50);
        assert!(validation_result.is_ok());

        // Test maximum length password
        let long_password = "a".repeat(128);
        let validation_result = Validator::validate_string(&long_password, "password", 8, 128);
        assert!(validation_result.is_ok());

        // Test password that's too long
        let too_long_password = "a".repeat(129);
        let validation_result = Validator::validate_string(&too_long_password, "password", 8, 128);
        assert!(validation_result.is_err());
    }
}
