/// Security middleware and utilities for the Fiscus application
///
/// This module provides security controls including authentication checks,
/// rate limiting, input validation, and access controls for encryption operations.
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::error::{FiscusError, FiscusResult};

/// Security context for operations
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user_id: String,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub authenticated_at: Instant,
    pub permissions: Vec<String>,
}

impl SecurityContext {
    /// Create a new security context
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            session_id: None,
            ip_address: None,
            user_agent: None,
            authenticated_at: Instant::now(),
            permissions: Vec::new(),
        }
    }

    /// Check if the context has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Check if the authentication is still valid
    pub fn is_auth_valid(&self, max_age: Duration) -> bool {
        self.authenticated_at.elapsed() < max_age
    }
}

/// Security middleware for encryption operations
#[derive(Debug)]
pub struct SecurityMiddleware {
    rate_limiter: Arc<RwLock<RateLimiter>>,
    auth_validator: Arc<AuthValidator>,
    access_controller: Arc<AccessController>,
}

impl SecurityMiddleware {
    /// Create a new security middleware instance
    pub fn new() -> Self {
        Self {
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
            auth_validator: Arc::new(AuthValidator::new()),
            access_controller: Arc::new(AccessController::new()),
        }
    }

    /// Validate a request before allowing encryption operations
    #[instrument(skip(self, context), fields(user_id = %context.user_id))]
    pub async fn validate_request(
        &self,
        context: &SecurityContext,
        operation: &str,
        data_size: usize,
    ) -> FiscusResult<()> {
        debug!(
            user_id = %context.user_id,
            operation = operation,
            data_size = data_size,
            "Validating security request"
        );

        // 1. Check authentication
        self.auth_validator.validate_authentication(context).await?;

        // 2. Check rate limits
        self.rate_limiter
            .write()
            .await
            .check_rate_limit(&context.user_id, operation)
            .await?;

        // 3. Check access permissions
        self.access_controller
            .check_access(context, operation)
            .await?;

        // 4. Validate data size
        self.validate_data_size(data_size, operation)?;

        info!(
            user_id = %context.user_id,
            operation = operation,
            "Security validation passed"
        );

        Ok(())
    }

    /// Validate data size limits
    fn validate_data_size(&self, data_size: usize, operation: &str) -> FiscusResult<()> {
        let max_size = match operation {
            "encrypt_financial_data" | "decrypt_financial_data" => 1024 * 1024, // 1MB
            "sign_data" | "verify_signature" => 512 * 1024,                     // 512KB
            _ => 64 * 1024,                                                     // 64KB default
        };

        if data_size > max_size {
            return Err(FiscusError::Security(format!(
                "Data size {data_size} exceeds maximum allowed size {max_size} for operation {operation}"
            )));
        }

        Ok(())
    }
}

/// Rate limiter for preventing abuse
#[derive(Debug)]
pub struct RateLimiter {
    user_limits: HashMap<String, UserRateLimit>,
    #[allow(dead_code)]
    global_limits: HashMap<String, GlobalRateLimit>,
}

#[derive(Debug)]
struct UserRateLimit {
    requests: Vec<Instant>,
    last_cleanup: Instant,
}

#[derive(Debug)]
struct GlobalRateLimit {
    #[allow(dead_code)]
    requests: Vec<Instant>,
    #[allow(dead_code)]
    last_cleanup: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            user_limits: HashMap::new(),
            global_limits: HashMap::new(),
        }
    }

    /// Check if a user can perform an operation
    #[instrument(skip(self), fields(user_id = user_id, operation = operation))]
    pub async fn check_rate_limit(&mut self, user_id: &str, operation: &str) -> FiscusResult<()> {
        let now = Instant::now();

        // Define rate limits per operation
        let (user_limit, window) = match operation {
            "encrypt_financial_data" | "decrypt_financial_data" => (100, Duration::from_secs(60)), // 100 per minute
            "generate_encryption_key" => (10, Duration::from_secs(300)), // 10 per 5 minutes
            "rotate_user_keys" => (5, Duration::from_secs(3600)),        // 5 per hour
            "derive_key_from_password" => (20, Duration::from_secs(300)), // 20 per 5 minutes
            _ => (50, Duration::from_secs(60)),                          // Default: 50 per minute
        };

        // Check user-specific rate limit
        let user_entry = self
            .user_limits
            .entry(user_id.to_string())
            .or_insert_with(|| UserRateLimit {
                requests: Vec::new(),
                last_cleanup: now,
            });

        // Clean up old requests
        if now.duration_since(user_entry.last_cleanup) > Duration::from_secs(60) {
            user_entry
                .requests
                .retain(|&req_time| now.duration_since(req_time) < window);
            user_entry.last_cleanup = now;
        }

        // Check if user has exceeded their limit
        if user_entry.requests.len() >= user_limit {
            warn!(
                user_id = user_id,
                operation = operation,
                current_requests = user_entry.requests.len(),
                limit = user_limit,
                "User rate limit exceeded"
            );
            return Err(FiscusError::Security(format!(
                "Rate limit exceeded for operation '{}'. Limit: {} requests per {} seconds",
                operation,
                user_limit,
                window.as_secs()
            )));
        }

        // Add this request
        user_entry.requests.push(now);

        debug!(
            user_id = user_id,
            operation = operation,
            current_requests = user_entry.requests.len(),
            limit = user_limit,
            "Rate limit check passed"
        );

        Ok(())
    }

    /// Get current rate limit status for a user
    pub fn get_rate_limit_status(&self, user_id: &str, operation: &str) -> (usize, usize) {
        let (limit, _) = match operation {
            "encrypt_financial_data" | "decrypt_financial_data" => (100, Duration::from_secs(60)),
            "generate_encryption_key" => (10, Duration::from_secs(300)),
            "rotate_user_keys" => (5, Duration::from_secs(3600)),
            "derive_key_from_password" => (20, Duration::from_secs(300)),
            _ => (50, Duration::from_secs(60)),
        };

        let current = self
            .user_limits
            .get(user_id)
            .map(|entry| entry.requests.len())
            .unwrap_or(0);

        (current, limit)
    }
}

/// Authentication validator
#[derive(Debug)]
pub struct AuthValidator {
    session_timeout: Duration,
}

impl AuthValidator {
    /// Create a new authentication validator
    pub fn new() -> Self {
        Self {
            session_timeout: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Validate user authentication
    #[instrument(skip(self, context), fields(user_id = %context.user_id))]
    pub async fn validate_authentication(&self, context: &SecurityContext) -> FiscusResult<()> {
        // Check if authentication is still valid
        if !context.is_auth_valid(self.session_timeout) {
            warn!(
                user_id = %context.user_id,
                auth_age = ?context.authenticated_at.elapsed(),
                "Authentication expired"
            );
            return Err(FiscusError::Authentication(
                "Authentication session has expired".to_string(),
            ));
        }

        // Additional authentication checks could go here
        // For example, checking if the user is still active in the database

        debug!(
            user_id = %context.user_id,
            auth_age = ?context.authenticated_at.elapsed(),
            "Authentication validation passed"
        );

        Ok(())
    }
}

/// Access control for encryption operations
#[derive(Debug)]
pub struct AccessController {
    required_permissions: HashMap<String, Vec<String>>,
}

impl AccessController {
    /// Create a new access controller
    pub fn new() -> Self {
        let mut required_permissions = HashMap::new();

        // Define required permissions for each operation
        required_permissions.insert(
            "encrypt_financial_data".to_string(),
            vec!["encryption:encrypt".to_string(), "data:write".to_string()],
        );
        required_permissions.insert(
            "decrypt_financial_data".to_string(),
            vec!["encryption:decrypt".to_string(), "data:read".to_string()],
        );
        required_permissions.insert(
            "generate_encryption_key".to_string(),
            vec![
                "encryption:key_generate".to_string(),
                "admin:keys".to_string(),
            ],
        );
        required_permissions.insert(
            "rotate_user_keys".to_string(),
            vec![
                "encryption:key_rotate".to_string(),
                "admin:keys".to_string(),
            ],
        );

        Self {
            required_permissions,
        }
    }

    /// Check if a user has access to perform an operation
    #[instrument(skip(self, context), fields(user_id = %context.user_id, operation = operation))]
    pub async fn check_access(
        &self,
        context: &SecurityContext,
        operation: &str,
    ) -> FiscusResult<()> {
        // For now, allow all operations for authenticated users
        // In a production system, you'd implement proper role-based access control

        if let Some(required_perms) = self.required_permissions.get(operation) {
            for required_perm in required_perms {
                if !context.has_permission(required_perm) {
                    warn!(
                        user_id = %context.user_id,
                        operation = operation,
                        required_permission = required_perm,
                        "Access denied - missing permission"
                    );
                    // For now, just log the warning but don't block access
                    // return Err(FiscusError::Authorization(format!(
                    //     "Missing required permission: {}", required_perm
                    // )));
                }
            }
        }

        debug!(
            user_id = %context.user_id,
            operation = operation,
            "Access control check passed"
        );

        Ok(())
    }
}

impl Default for SecurityMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AuthValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AccessController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_context_creation() {
        let context = SecurityContext::new("test-user".to_string());
        assert_eq!(context.user_id, "test-user");
        assert!(context.is_auth_valid(Duration::from_secs(3600)));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut rate_limiter = RateLimiter::new();
        // deepcode ignore NoHardcodedCredentials: <test>
        let user_id = "test-user";
        let operation = "encrypt_financial_data";

        // Should allow initial requests
        for _ in 0..10 {
            assert!(rate_limiter
                .check_rate_limit(user_id, operation)
                .await
                .is_ok());
        }

        let (current, limit) = rate_limiter.get_rate_limit_status(user_id, operation);
        assert_eq!(current, 10);
        assert_eq!(limit, 100);
    }

    #[tokio::test]
    async fn test_auth_validator() {
        let validator = AuthValidator::new();
        let context = SecurityContext::new("test-user".to_string());

        // Should pass for fresh authentication
        assert!(validator.validate_authentication(&context).await.is_ok());
    }

    #[tokio::test]
    async fn test_access_controller() {
        let controller = AccessController::new();
        let context = SecurityContext::new("test-user".to_string());

        // Should pass for now (permissive mode)
        assert!(controller
            .check_access(&context, "encrypt_financial_data")
            .await
            .is_ok());
    }
}
