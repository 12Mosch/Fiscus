# API Security Guide

## Overview

This guide covers security best practices for the Fiscus API, including authentication, authorization, data protection, and secure coding practices for both Rust commands and TypeScript client code.

## Security Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Frontend      │    │   Validation     │    │   Backend       │
│   Input         │◄──►│   & Sanitization │◄──►│   Commands      │
│   Validation    │    │                  │    │   (Rust)        │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Type Safety   │    │   Field          │    │   Database      │
│   Error         │    │   Whitelisting   │    │   Security      │
│   Handling      │    │                  │    │   (SQLite)      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Authentication & Authorization

### User Authentication Flow

```rust
// src-tauri/src/commands/auth.rs
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::error::FiscusError;

/// Secure password hashing
fn hash_password(password: &str) -> Result<String, FiscusError> {
    hash(password, DEFAULT_COST)
        .map_err(|e| FiscusError::Internal(format!("Password hashing failed: {}", e)))
}

/// Secure password verification
fn verify_password(password: &str, hash: &str) -> Result<bool, FiscusError> {
    verify(password, hash)
        .map_err(|e| FiscusError::Authentication(format!("Password verification failed: {}", e)))
}

#[tauri::command]
pub async fn login_user(
    request: LoginRequest,
    db: State<'_, Database>,
) -> Result<LoginResponse, FiscusError> {
    // Input validation
    Validator::validate_string(&request.username, "username", 1, 50)?;
    Validator::validate_string(&request.password, "password", 1, 128)?;

    // Rate limiting (implement based on requirements)
    check_rate_limit(&request.username)?;

    // Fetch user securely
    let user_data = get_user_by_username(&db, &request.username).await?
        .ok_or_else(|| FiscusError::Authentication("Invalid credentials".to_string()))?;

    // Verify password
    let password_hash = user_data.get("password_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| FiscusError::Internal("Invalid user data".to_string()))?;

    if !verify_password(&request.password, password_hash)? {
        return Err(FiscusError::Authentication("Invalid credentials".to_string()));
    }

    // Create session (if using sessions)
    let session_token = create_secure_session(&user_data.id)?;

    Ok(LoginResponse {
        user: User::from_database_row(user_data)?,
        session_token: Some(session_token),
    })
}
```

### Session Management

```rust
// src-tauri/src/auth/session.rs
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

pub struct SessionManager {
    sessions: HashMap<String, SessionData>,
}

#[derive(Debug, Clone)]
pub struct SessionData {
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

impl SessionManager {
    pub fn create_session(&mut self, user_id: String) -> Result<String, FiscusError> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let session = SessionData {
            user_id,
            created_at: now,
            expires_at: now + Duration::hours(24), // 24-hour sessions
            last_activity: now,
        };

        self.sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    pub fn validate_session(&mut self, session_id: &str) -> Result<String, FiscusError> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| FiscusError::Authentication("Invalid session".to_string()))?;

        if Utc::now() > session.expires_at {
            self.sessions.remove(session_id);
            return Err(FiscusError::Authentication("Session expired".to_string()));
        }

        // Update last activity
        session.last_activity = Utc::now();
        Ok(session.user_id.clone())
    }

    pub fn revoke_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }
}
```

### Authorization Patterns

```rust
// src-tauri/src/auth/authorization.rs

/// Verify user owns the resource
pub async fn verify_user_ownership(
    db: &Database,
    user_id: &str,
    resource_type: &str,
    resource_id: &str,
) -> Result<(), FiscusError> {
    let query = match resource_type {
        "account" => "SELECT user_id FROM accounts WHERE id = ?1",
        "transaction" => "SELECT user_id FROM transactions WHERE id = ?1",
        "category" => "SELECT user_id FROM categories WHERE id = ?1",
        "budget" => "SELECT user_id FROM budgets WHERE id = ?1",
        "goal" => "SELECT user_id FROM goals WHERE id = ?1",
        _ => return Err(FiscusError::InvalidInput("Invalid resource type".to_string())),
    };

    let result: Option<HashMap<String, Value>> = DatabaseUtils::execute_query_single(
        db,
        query,
        vec![Value::String(resource_id.to_string())],
    ).await?;

    let resource_user_id = result
        .and_then(|row| row.get("user_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| FiscusError::NotFound("Resource not found".to_string()))?;

    if resource_user_id != user_id {
        return Err(FiscusError::Authorization("Access denied".to_string()));
    }

    Ok(())
}

/// Macro for easy ownership verification
macro_rules! verify_ownership {
    ($db:expr, $user_id:expr, $resource_type:literal, $resource_id:expr) => {
        verify_user_ownership($db, $user_id, $resource_type, $resource_id).await?
    };
}

// Usage in commands
#[tauri::command]
pub async fn update_account(
    account_id: String,
    user_id: String,
    request: UpdateAccountRequest,
    db: State<'_, Database>,
) -> Result<Account, FiscusError> {
    // Verify ownership before proceeding
    verify_ownership!(&db, &user_id, "account", &account_id);
    
    // Proceed with update...
}
```

## Input Validation & Sanitization

### Comprehensive Validation

```rust
// src-tauri/src/validation.rs
use regex::Regex;
use once_cell::sync::Lazy;
use uuid::Uuid;

// Lazy static regex for email validation - compiled once for better performance
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")
        .expect("Failed to compile email regex - this should never happen")
});

pub struct Validator;

impl Validator {
    /// Validate string length and content
    pub fn validate_string(value: &str, field_name: &str, min_len: usize, max_len: usize) -> Result<(), FiscusError> {
        if value.len() < min_len {
            return Err(FiscusError::Validation(
                format!("{} must be at least {} characters", field_name, min_len)
            ));
        }
        
        if value.len() > max_len {
            return Err(FiscusError::Validation(
                format!("{} must be at most {} characters", field_name, max_len)
            ));
        }

        // Check for null bytes and control characters
        if value.contains('\0') || value.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
            return Err(FiscusError::Validation(
                format!("{} contains invalid characters", field_name)
            ));
        }

        Ok(())
    }

    /// Validate UUID format
    pub fn validate_uuid(value: &str, field_name: &str) -> Result<(), FiscusError> {
        Uuid::parse_str(value)
            .map_err(|_| FiscusError::Validation(format!("{} must be a valid UUID", field_name)))?;
        Ok(())
    }

    /// Validate email format
    /// Uses a lazy static regex for optimal performance - compiled once on first use
    pub fn validate_email(email: &str) -> Result<(), FiscusError> {
        // EMAIL_REGEX is a lazy static compiled once for better performance
        if !EMAIL_REGEX.is_match(email) {
            return Err(FiscusError::Validation("Invalid email format".to_string()));
        }

        Ok(())
    }

    /// Validate currency code (ISO 4217)
    pub fn validate_currency(currency: &str) -> Result<(), FiscusError> {
        const VALID_CURRENCIES: &[&str] = &[
            "USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "CNY", "SEK", "NZD"
        ];

        if !VALID_CURRENCIES.contains(&currency) {
            return Err(FiscusError::Validation(
                format!("Unsupported currency: {}", currency)
            ));
        }

        Ok(())
    }

    /// Validate monetary amount
    pub fn validate_amount(amount: f64, field_name: &str) -> Result<(), FiscusError> {
        if amount.is_nan() || amount.is_infinite() {
            return Err(FiscusError::Validation(
                format!("{} must be a valid number", field_name)
            ));
        }

        // Check for reasonable bounds (adjust as needed)
        if amount.abs() > 999_999_999.99 {
            return Err(FiscusError::Validation(
                format!("{} exceeds maximum allowed value", field_name)
            ));
        }

        // Check decimal precision (2 decimal places max)
        let rounded = (amount * 100.0).round() / 100.0;
        if (amount - rounded).abs() > f64::EPSILON {
            return Err(FiscusError::Validation(
                format!("{} must have at most 2 decimal places", field_name)
            ));
        }

        Ok(())
    }

    /// Validate date format (YYYY-MM-DD)
    pub fn validate_date(date_str: &str, field_name: &str) -> Result<(), FiscusError> {
        let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$")
            .map_err(|e| FiscusError::Internal(format!("Regex compilation failed: {}", e)))?;

        if !date_regex.is_match(date_str) {
            return Err(FiscusError::Validation(
                format!("{} must be in YYYY-MM-DD format", field_name)
            ));
        }

        // Additional validation with chrono
        chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| FiscusError::Validation(
                format!("{} is not a valid date", field_name)
            ))?;

        Ok(())
    }
}
```

### TypeScript Input Validation

```typescript
// src/utils/validation.ts
export class ValidationError extends Error {
  constructor(message: string, public field?: string) {
    super(message);
    this.name = 'ValidationError';
  }
}

export class ClientValidator {
  static validateString(value: string, fieldName: string, minLen: number, maxLen: number): void {
    if (typeof value !== 'string') {
      throw new ValidationError(`${fieldName} must be a string`, fieldName);
    }

    if (value.length < minLen) {
      throw new ValidationError(`${fieldName} must be at least ${minLen} characters`, fieldName);
    }

    if (value.length > maxLen) {
      throw new ValidationError(`${fieldName} must be at most ${maxLen} characters`, fieldName);
    }

    // Check for potentially dangerous characters
    if (value.includes('\0') || /[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/.test(value)) {
      throw new ValidationError(`${fieldName} contains invalid characters`, fieldName);
    }
  }

  static validateEmail(email: string): void {
    const emailRegex = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;
    if (!emailRegex.test(email)) {
      throw new ValidationError('Invalid email format', 'email');
    }
  }

  static validateUUID(value: string, fieldName: string): void {
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
    if (!uuidRegex.test(value)) {
      throw new ValidationError(`${fieldName} must be a valid UUID`, fieldName);
    }
  }

  static validateAmount(amount: number, fieldName: string): void {
    if (typeof amount !== 'number' || isNaN(amount) || !isFinite(amount)) {
      throw new ValidationError(`${fieldName} must be a valid number`, fieldName);
    }

    if (Math.abs(amount) > 999999999.99) {
      throw new ValidationError(`${fieldName} exceeds maximum allowed value`, fieldName);
    }

    // Check decimal precision
    const rounded = Math.round(amount * 100) / 100;
    if (Math.abs(amount - rounded) > Number.EPSILON) {
      throw new ValidationError(`${fieldName} must have at most 2 decimal places`, fieldName);
    }
  }

  static validateDate(dateStr: string, fieldName: string): void {
    const dateRegex = /^\d{4}-\d{2}-\d{2}$/;
    if (!dateRegex.test(dateStr)) {
      throw new ValidationError(`${fieldName} must be in YYYY-MM-DD format`, fieldName);
    }

    const date = new Date(dateStr);
    if (isNaN(date.getTime())) {
      throw new ValidationError(`${fieldName} is not a valid date`, fieldName);
    }
  }

  // Validate API request before sending
  static validateCreateUserRequest(request: CreateUserRequest): void {
    this.validateString(request.username, 'username', 3, 50);
    this.validateString(request.password, 'password', 8, 128);
    
    if (request.email) {
      this.validateEmail(request.email);
    }
  }

  static validateCreateAccountRequest(request: CreateAccountRequest): void {
    this.validateUUID(request.user_id, 'user_id');
    this.validateUUID(request.account_type_id, 'account_type_id');
    this.validateString(request.name, 'name', 1, 100);
    this.validateString(request.currency, 'currency', 3, 3);
    
    if (request.balance !== undefined) {
      this.validateAmount(request.balance, 'balance');
    }
  }
}
```

## SQL Injection Prevention

### Field Whitelisting

```rust
// src-tauri/src/database/security.rs
use std::collections::HashSet;

pub struct DatabaseSecurity;

impl DatabaseSecurity {
    /// Get allowed fields for sorting/filtering
    pub fn get_allowed_account_fields() -> HashSet<&'static str> {
        [
            "id", "name", "balance", "currency", "created_at", "updated_at", "is_active"
        ].iter().cloned().collect()
    }

    pub fn get_allowed_transaction_fields() -> HashSet<&'static str> {
        [
            "id", "amount", "description", "transaction_date", "transaction_type", 
            "status", "payee", "created_at", "updated_at"
        ].iter().cloned().collect()
    }

    /// Validate and sanitize sort field
    pub fn validate_sort_field(field: &str, allowed_fields: &HashSet<&str>) -> Result<String, FiscusError> {
        if !allowed_fields.contains(field) {
            return Err(FiscusError::Security(
                format!("Invalid sort field: {}", field)
            ));
        }

        // Additional sanitization - ensure field name is safe
        if !field.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(FiscusError::Security(
                format!("Invalid characters in field name: {}", field)
            ));
        }

        Ok(format!("`{}`", field)) // Quote field name
    }

    /// Validate sort direction
    pub fn validate_sort_direction(direction: &str) -> Result<&'static str, FiscusError> {
        match direction.to_uppercase().as_str() {
            "ASC" => Ok("ASC"),
            "DESC" => Ok("DESC"),
            _ => Err(FiscusError::Security(
                format!("Invalid sort direction: {}", direction)
            )),
        }
    }

    /// Build safe WHERE clause with parameter binding
    pub fn build_where_clause(
        filters: &HashMap<String, String>,
        allowed_fields: &HashSet<&str>,
    ) -> Result<(String, Vec<Value>), FiscusError> {
        let mut conditions = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 1;

        for (field, value) in filters {
            if !allowed_fields.contains(field.as_str()) {
                return Err(FiscusError::Security(
                    format!("Invalid filter field: {}", field)
                ));
            }

            conditions.push(format!("`{}` = ?{}", field, param_index));
            params.push(Value::String(value.clone()));
            param_index += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        Ok((where_clause, params))
    }
}
```

### Parameterized Query Examples

```rust
// src-tauri/src/database/queries.rs
use crate::database::security::DatabaseSecurity;

impl DatabaseUtils {
    /// Safe account query with filtering
    pub async fn get_accounts_filtered(
        db: &Database,
        user_id: &str,
        filters: HashMap<String, String>,
        sort_by: Option<String>,
        sort_direction: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Account>, FiscusError> {
        let allowed_fields = DatabaseSecurity::get_allowed_account_fields();

        // Build WHERE clause safely
        let mut base_filters = HashMap::new();
        base_filters.insert("user_id".to_string(), user_id.to_string());
        base_filters.extend(filters);

        let (where_clause, mut params) = DatabaseSecurity::build_where_clause(
            &base_filters,
            &allowed_fields
        )?;

        // Build ORDER BY clause safely
        let order_clause = if let Some(sort_field) = sort_by {
            let safe_field = DatabaseSecurity::validate_sort_field(&sort_field, &allowed_fields)?;
            let safe_direction = sort_direction
                .as_ref()
                .map(|d| DatabaseSecurity::validate_sort_direction(d))
                .transpose()?
                .unwrap_or("ASC");

            format!("ORDER BY {} {}", safe_field, safe_direction)
        } else {
            "ORDER BY created_at DESC".to_string()
        };

        // Build LIMIT clause safely
        let limit_clause = if let Some(limit_val) = limit {
            if limit_val > 1000 {
                return Err(FiscusError::Validation("Limit cannot exceed 1000".to_string()));
            }
            format!("LIMIT {}", limit_val)
        } else {
            "LIMIT 100".to_string() // Default limit
        };

        let offset_clause = if let Some(offset_val) = offset {
            format!("OFFSET {}", offset_val)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT * FROM accounts {} {} {} {}",
            where_clause, order_clause, limit_clause, offset_clause
        );

        DatabaseUtils::execute_query_multiple(db, &query, params).await
    }
}
```

## Data Protection

### Sensitive Data Handling

```rust
// src-tauri/src/security/data_protection.rs
use serde::{Serialize, Deserialize};

/// Wrapper for sensitive data that prevents accidental logging
#[derive(Debug, Clone)]
pub struct SensitiveData<T> {
    inner: T,
}

impl<T> SensitiveData<T> {
    pub fn new(data: T) -> Self {
        Self { inner: data }
    }

    pub fn expose(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

// Prevent accidental serialization of sensitive data
impl<T> Serialize for SensitiveData<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("[REDACTED]")
    }
}

// Custom Debug implementation to prevent logging
impl<T> std::fmt::Display for SensitiveData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// Secure password handling
pub type SecurePassword = SensitiveData<String>;

/// Usage in commands
#[tauri::command]
pub async fn create_user_secure(
    username: String,
    email: Option<String>,
    password: SecurePassword, // Prevents accidental logging
    db: State<'_, Database>,
) -> Result<User, FiscusError> {
    // Validate inputs
    Validator::validate_string(&username, "username", 3, 50)?;
    if let Some(ref email_val) = email {
        Validator::validate_email(email_val)?;
    }
    Validator::validate_string(password.expose(), "password", 8, 128)?;

    // Hash password securely
    let password_hash = hash_password(password.expose())?;

    // Continue with user creation...
}
```

### Data Sanitization for Logging

```rust
// src-tauri/src/logging/sanitizer.rs
use serde_json::{Value, Map};
use std::collections::HashSet;

pub struct DataSanitizer {
    sensitive_fields: HashSet<String>,
}

impl DataSanitizer {
    pub fn new() -> Self {
        let mut sensitive_fields = HashSet::new();
        sensitive_fields.insert("password".to_string());
        sensitive_fields.insert("password_hash".to_string());
        sensitive_fields.insert("session_token".to_string());
        sensitive_fields.insert("api_key".to_string());
        sensitive_fields.insert("secret".to_string());
        sensitive_fields.insert("token".to_string());

        Self { sensitive_fields }
    }

    /// Sanitize JSON value for logging
    pub fn sanitize_json(&self, value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut sanitized_map = Map::new();
                for (key, val) in map {
                    if self.is_sensitive_field(key) {
                        sanitized_map.insert(key.clone(), Value::String("[REDACTED]".to_string()));
                    } else {
                        sanitized_map.insert(key.clone(), self.sanitize_json(val));
                    }
                }
                Value::Object(sanitized_map)
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| self.sanitize_json(v)).collect())
            }
            _ => value.clone(),
        }
    }

    fn is_sensitive_field(&self, field_name: &str) -> bool {
        let lower_field = field_name.to_lowercase();
        self.sensitive_fields.iter().any(|sensitive| lower_field.contains(sensitive))
    }

    /// Sanitize string for logging (remove potential PII)
    pub fn sanitize_string(&self, input: &str) -> String {
        // Remove potential email addresses
        let email_regex = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        let sanitized = email_regex.replace_all(input, "[EMAIL_REDACTED]");

        // Remove potential phone numbers
        let phone_regex = regex::Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap();
        let sanitized = phone_regex.replace_all(&sanitized, "[PHONE_REDACTED]");

        sanitized.to_string()
    }
}

// Usage in logging middleware
pub fn log_api_call(command_name: &str, params: &Value, result: &Result<Value, FiscusError>) {
    let sanitizer = DataSanitizer::new();
    let sanitized_params = sanitizer.sanitize_json(params);

    match result {
        Ok(response) => {
            let sanitized_response = sanitizer.sanitize_json(response);
            info!("API call successful: {} with params: {} -> response: {}",
                  command_name, sanitized_params, sanitized_response);
        }
        Err(error) => {
            warn!("API call failed: {} with params: {} -> error: {}",
                  command_name, sanitized_params, error);
        }
    }
}
```

## Frontend Security

### Secure API Client Usage

```typescript
// src/api/secure-client.ts
import { apiClient, FiscusApiError } from './client';
import { ClientValidator } from '@/utils/validation';

export class SecureApiClient {
  private static instance: SecureApiClient;
  private rateLimitMap = new Map<string, number[]>();

  static getInstance(): SecureApiClient {
    if (!SecureApiClient.instance) {
      SecureApiClient.instance = new SecureApiClient();
    }
    return SecureApiClient.instance;
  }

  private checkRateLimit(operation: string, maxRequests = 10, windowMs = 60000): void {
    const now = Date.now();
    const requests = this.rateLimitMap.get(operation) || [];

    // Remove old requests outside the window
    const validRequests = requests.filter(time => now - time < windowMs);

    if (validRequests.length >= maxRequests) {
      throw new Error(`Rate limit exceeded for ${operation}. Try again later.`);
    }

    validRequests.push(now);
    this.rateLimitMap.set(operation, validRequests);
  }

  async createUserSecure(request: CreateUserRequest): Promise<User> {
    // Client-side validation
    ClientValidator.validateCreateUserRequest(request);

    // Rate limiting
    this.checkRateLimit('create_user', 5, 300000); // 5 requests per 5 minutes

    // Sanitize input
    const sanitizedRequest = {
      username: this.sanitizeInput(request.username),
      email: request.email ? this.sanitizeInput(request.email) : undefined,
      password: request.password // Don't log or sanitize passwords
    };

    try {
      return await apiClient.createUser(sanitizedRequest);
    } catch (error) {
      // Log error without sensitive data
      console.error('User creation failed:', {
        username: sanitizedRequest.username,
        hasEmail: !!sanitizedRequest.email,
        error: error instanceof FiscusApiError ? error.code : 'Unknown'
      });
      throw error;
    }
  }

  async loginUserSecure(request: LoginRequest): Promise<LoginResponse> {
    // Rate limiting for login attempts
    this.checkRateLimit(`login_${request.username}`, 5, 900000); // 5 attempts per 15 minutes

    const sanitizedRequest = {
      username: this.sanitizeInput(request.username),
      password: request.password
    };

    try {
      const response = await apiClient.loginUser(sanitizedRequest);

      // Store session securely (if using sessions)
      if (response.session_token) {
        this.storeSessionSecurely(response.session_token);
      }

      return response;
    } catch (error) {
      // Log failed login attempt (without password)
      console.warn('Login attempt failed:', {
        username: sanitizedRequest.username,
        timestamp: new Date().toISOString(),
        error: error instanceof FiscusApiError ? error.code : 'Unknown'
      });
      throw error;
    }
  }

  private sanitizeInput(input: string): string {
    // Remove potentially dangerous characters
    return input
      .replace(/[<>'"&]/g, '') // Remove HTML/XML characters
      .replace(/[^\w\s@.-]/g, '') // Keep only alphanumeric, whitespace, @, ., -
      .trim()
      .substring(0, 1000); // Limit length
  }

  private storeSessionSecurely(token: string): void {
    // Store in secure storage (not localStorage for sensitive data)
    // Consider using encrypted storage or secure cookies
    sessionStorage.setItem('fiscus_session', token);
  }

  getStoredSession(): string | null {
    return sessionStorage.getItem('fiscus_session');
  }

  clearSession(): void {
    sessionStorage.removeItem('fiscus_session');
    // Clear any other session-related data
  }
}

export const secureApiClient = SecureApiClient.getInstance();
```

## Performance Optimizations

### Regex Compilation Optimization

For frequently used validation patterns like email validation, compile regex patterns once using lazy static initialization instead of compiling on every validation call:

```rust
use once_cell::sync::Lazy;
use regex::Regex;

// ❌ Bad: Compiles regex on every call
pub fn validate_email_slow(email: &str) -> Result<(), FiscusError> {
    let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")?;
    if !email_regex.is_match(email) {
        return Err(FiscusError::Validation("Invalid email format".to_string()));
    }
    Ok(())
}

// ✅ Good: Compiles regex once, reuses for all calls
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")
        .expect("Failed to compile email regex")
});

pub fn validate_email(email: &str) -> Result<(), FiscusError> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(FiscusError::Validation("Invalid email format".to_string()));
    }
    Ok(())
}
```

### Benefits

- **Performance**: Regex compilation is expensive; doing it once improves performance significantly
- **Memory**: Reduces memory allocations for repeated validations
- **Reliability**: Compilation errors are caught at startup rather than during validation
- **Thread Safety**: `Lazy<Regex>` is thread-safe and can be shared across threads

### Implementation Guidelines

1. Use `once_cell::sync::Lazy` for static regex patterns
2. Handle compilation errors with `expect()` for patterns that should never fail
3. Group related patterns together for better organization
4. Document the performance benefit in code comments

## Security Best Practices

### Secure Coding Checklist

#### Rust Backend
- ✅ **Input Validation**: All inputs validated at command entry points
- ✅ **SQL Injection Prevention**: Parameterized queries and field whitelisting
- ✅ **Password Security**: Bcrypt hashing with appropriate cost factor
- ✅ **Error Handling**: No sensitive data in error messages
- ✅ **Authorization**: User ownership verification for all resources
- ✅ **Rate Limiting**: Implement rate limiting for sensitive operations
- ✅ **Logging Security**: Sanitize logs to prevent data leakage

#### TypeScript Frontend
- ✅ **Client Validation**: Validate inputs before sending to backend
- ✅ **Error Handling**: Handle API errors gracefully without exposing internals
- ✅ **Session Management**: Secure session storage and cleanup
- ✅ **Rate Limiting**: Client-side rate limiting for user experience
- ✅ **Input Sanitization**: Sanitize user inputs to prevent XSS
- ✅ **Secure Storage**: Use appropriate storage mechanisms for sensitive data

### Security Testing

```typescript
// src/security/__tests__/security.test.ts
import { describe, it, expect } from 'vitest';
import { ClientValidator } from '@/utils/validation';
import { secureApiClient } from '@/api/secure-client';

describe('Security Tests', () => {
  describe('Input Validation', () => {
    it('should reject malicious input', () => {
      expect(() => {
        ClientValidator.validateString('<script>alert("xss")</script>', 'username', 3, 50);
      }).toThrow();

      expect(() => {
        ClientValidator.validateString('user\0name', 'username', 3, 50);
      }).toThrow();
    });

    it('should validate email format strictly', () => {
      expect(() => {
        ClientValidator.validateEmail('invalid-email');
      }).toThrow();

      expect(() => {
        ClientValidator.validateEmail('user@domain');
      }).toThrow();

      // Valid email should not throw
      expect(() => {
        ClientValidator.validateEmail('user@domain.com');
      }).not.toThrow();
    });
  });

  describe('Rate Limiting', () => {
    it('should enforce rate limits', async () => {
      const client = secureApiClient;

      // This would require mocking the rate limit functionality
      // to test without actually hitting rate limits
    });
  });

  describe('Data Sanitization', () => {
    it('should sanitize potentially dangerous input', () => {
      const client = secureApiClient as any; // Access private method for testing

      const maliciousInput = '<script>alert("xss")</script>';
      const sanitized = client.sanitizeInput(maliciousInput);

      expect(sanitized).not.toContain('<script>');
      expect(sanitized).not.toContain('alert');
    });
  });
});
```

### Deployment Security

#### Environment Configuration
```bash
# .env.production
NODE_ENV=production
TAURI_PRIVATE_KEY=path/to/private.key
TAURI_KEY_PASSWORD=secure_password

# Security headers
CSP_POLICY="default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';"
```

#### Tauri Security Configuration
```json
// src-tauri/tauri.conf.json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline';",
      "capabilities": ["default"],
      "dangerousDisableAssetCspModification": false
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "createUpdaterArtifacts": true,
    "publisher": "Your Organization",
    "copyright": "Copyright © 2024",
    "category": "Finance",
    "shortDescription": "Personal Finance Manager",
    "longDescription": "Secure personal finance management application"
  }
}
```

## Incident Response

### Security Monitoring

```rust
// src-tauri/src/security/monitoring.rs
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct SecurityMonitor {
    failed_login_attempts: AtomicU64,
    suspicious_activities: Mutex<HashMap<String, u64>>,
}

impl SecurityMonitor {
    pub fn new() -> Self {
        Self {
            failed_login_attempts: AtomicU64::new(0),
            suspicious_activities: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_failed_login(&self, username: &str) {
        self.failed_login_attempts.fetch_add(1, Ordering::Relaxed);

        let mut activities = self.suspicious_activities.lock().unwrap();
        let count = activities.entry(format!("failed_login_{}", username)).or_insert(0);
        *count += 1;

        if *count > 5 {
            warn!("Multiple failed login attempts for user: {}", username);
            // Consider implementing account lockout or alerting
        }
    }

    pub fn record_suspicious_activity(&self, activity_type: &str, details: &str) {
        warn!("Suspicious activity detected: {} - {}", activity_type, details);

        let mut activities = self.suspicious_activities.lock().unwrap();
        let count = activities.entry(activity_type.to_string()).or_insert(0);
        *count += 1;
    }

    pub fn get_security_metrics(&self) -> HashMap<String, u64> {
        let mut metrics = HashMap::new();
        metrics.insert("failed_logins".to_string(), self.failed_login_attempts.load(Ordering::Relaxed));

        let activities = self.suspicious_activities.lock().unwrap();
        for (key, value) in activities.iter() {
            metrics.insert(key.clone(), *value);
        }

        metrics
    }
}
```

This comprehensive API security guide covers all major security aspects of the Fiscus application, from authentication and authorization to data protection and incident response. The implementation provides multiple layers of security to protect sensitive financial data.
```
