use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde_json::Value;
use tauri::State;
use uuid::Uuid;

use crate::{
    database::{Database, DatabaseUtils},
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

    let params = vec![
        Value::String(user_id.clone()),
        Value::String(request.username.clone()),
        request
            .email
            .as_ref()
            .map(|e| Value::String(e.clone()))
            .unwrap_or(Value::Null),
        Value::String(password_hash),
        Value::String(now.clone()),
        Value::String(now),
    ];

    DatabaseUtils::execute_non_query(&db, insert_query, params).await?;

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

    // Find user by username
    let user_query = "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE username = ?1";
    let user_row: Option<std::collections::HashMap<String, Value>> =
        DatabaseUtils::execute_query_single(&db, user_query, vec![Value::String(request.username)])
            .await?;

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
    let user_row: Option<std::collections::HashMap<String, Value>> =
        DatabaseUtils::execute_query_single(&db, user_query, vec![Value::String(user_id)]).await?;

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
