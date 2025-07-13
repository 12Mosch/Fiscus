use serde_json::Value;
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

use crate::{
    database::{encrypted::EncryptedDatabaseUtils, Database, DatabaseUtils},
    dto::{AccountFilters, AccountSummaryResponse, CreateAccountRequest, UpdateAccountRequest},
    error::{FiscusError, SecurityValidator, Validator},
    models::Account,
};

/// Create a new account
#[tauri::command]
pub async fn create_account(
    request: CreateAccountRequest,
    db: State<'_, Database>,
) -> Result<Account, FiscusError> {
    // Validate input
    Validator::validate_uuid(&request.user_id, "user_id")?;
    Validator::validate_uuid(&request.account_type_id, "account_type_id")?;
    Validator::validate_string(&request.name, "name", 1, 100)?;
    Validator::validate_string(&request.currency, "currency", 3, 3)?; // ISO currency codes are 3 chars

    // Validate user exists
    DatabaseUtils::validate_user_exists(&db, &request.user_id).await?;

    // Validate account type exists
    let account_type_query = "SELECT id FROM account_types WHERE id = ?1";
    let account_type_exists: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            account_type_query,
            vec![Value::String(request.account_type_id.clone())],
        )
        .await?;

    if account_type_exists.is_none() {
        return Err(FiscusError::NotFound("Account type not found".to_string()));
    }

    // Validate initial balance if provided
    let initial_balance = request.balance.unwrap_or(rust_decimal::Decimal::ZERO);
    Validator::validate_amount(initial_balance, true)?; // Allow negative for credit accounts

    let account_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let insert_query = r#"
        INSERT INTO accounts (id, user_id, account_type_id, name, balance, currency, account_number, is_active, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
    "#;

    // Use encrypted parameter mapping for sensitive fields
    let params_with_mapping = vec![
        ("id".to_string(), Value::String(account_id.clone())),
        (
            "user_id".to_string(),
            Value::String(request.user_id.clone()),
        ),
        (
            "account_type_id".to_string(),
            Value::String(request.account_type_id.clone()),
        ),
        ("name".to_string(), Value::String(request.name.clone())),
        (
            "balance".to_string(),
            Value::String(initial_balance.to_string()),
        ),
        (
            "currency".to_string(),
            Value::String(request.currency.clone()),
        ),
        (
            "account_number".to_string(),
            request
                .account_number
                .as_ref()
                .map(|n| Value::String(n.clone()))
                .unwrap_or(Value::Null),
        ),
        ("is_active".to_string(), Value::Bool(true)),
        ("created_at".to_string(), Value::String(now.clone())),
        ("updated_at".to_string(), Value::String(now)),
    ];

    let encrypted_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
        params_with_mapping,
        &request.user_id,
        "accounts",
    )
    .await?;

    DatabaseUtils::execute_non_query(&db, insert_query, encrypted_params).await?;

    // Return the created account
    get_account_by_id(account_id, db).await
}

/// Get all accounts for a user with optional filtering
#[tauri::command]
pub async fn get_accounts(
    filters: AccountFilters,
    db: State<'_, Database>,
) -> Result<Vec<Account>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&filters.user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &filters.user_id).await?;

    // Build query with filters
    let mut filter_map = HashMap::new();
    filter_map.insert("user_id".to_string(), filters.user_id.clone());

    if let Some(account_type_id) = filters.account_type_id {
        Validator::validate_uuid(&account_type_id, "account_type_id")?;
        filter_map.insert("account_type_id".to_string(), account_type_id);
    }

    if let Some(is_active) = filters.is_active {
        filter_map.insert("is_active".to_string(), is_active.to_string());
    }

    // Validate filter fields
    SecurityValidator::validate_account_filter_fields(&filter_map)?;

    let base_query = r#"
        SELECT a.id, a.user_id, a.account_type_id, a.name, a.balance, a.currency, 
               a.account_number, a.is_active, a.created_at, a.updated_at
        FROM accounts a
    "#;

    let (where_clause, where_params) = DatabaseUtils::build_where_clause(
        &filter_map,
        &["user_id", "account_type_id", "is_active"],
        vec![],
    )?;

    let order_clause = DatabaseUtils::build_order_clause(
        filters.sort_by.as_deref(),
        filters.sort_direction.as_deref(),
        SecurityValidator::ACCOUNT_SORT_FIELDS,
        "created_at",
    )?;

    let limit_clause = DatabaseUtils::build_limit_clause(filters.limit, filters.offset);

    let final_query = format!("{base_query} {where_clause} {order_clause} {limit_clause}");

    // Use encrypted query to properly decrypt sensitive fields
    let accounts: Vec<Account> = EncryptedDatabaseUtils::execute_encrypted_query(
        &db,
        &final_query,
        where_params,
        &filters.user_id,
        "accounts",
    )
    .await?;

    Ok(accounts)
}

/// Get a single account by ID
#[tauri::command]
pub async fn get_account_by_id(
    account_id: String,
    db: State<'_, Database>,
) -> Result<Account, FiscusError> {
    Validator::validate_uuid(&account_id, "account_id")?;

    // First, get the user_id for this account (this field is not encrypted)
    let user_query = "SELECT user_id FROM accounts WHERE id = ?1";
    let user_result: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            user_query,
            vec![Value::String(account_id.clone())],
        )
        .await?;

    let user_id = user_result
        .and_then(|row| {
            row.get("user_id")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        })
        .ok_or_else(|| FiscusError::NotFound("Account not found".to_string()))?;

    let query = r#"
        SELECT id, user_id, account_type_id, name, balance, currency,
               account_number, is_active, created_at, updated_at
        FROM accounts
        WHERE id = ?1
    "#;

    // Use encrypted query to properly decrypt sensitive fields
    let accounts: Vec<Account> = EncryptedDatabaseUtils::execute_encrypted_query(
        &db,
        query,
        vec![Value::String(account_id.clone())],
        &user_id,
        "accounts",
    )
    .await?;

    accounts
        .into_iter()
        .next()
        .ok_or_else(|| FiscusError::NotFound("Account not found".to_string()))
}

/// Update an account
#[tauri::command]
pub async fn update_account(
    account_id: String,
    user_id: String,
    request: UpdateAccountRequest,
    db: State<'_, Database>,
) -> Result<Account, FiscusError> {
    // Validate input
    Validator::validate_uuid(&account_id, "account_id")?;
    Validator::validate_uuid(&user_id, "user_id")?;

    // Validate ownership
    DatabaseUtils::validate_account_ownership(&db, &account_id, &user_id).await?;

    // Build update query dynamically with encrypted parameter mapping
    let mut update_fields = Vec::new();
    let mut params_with_mapping = Vec::new();
    let mut param_index = 1;

    if let Some(name) = &request.name {
        Validator::validate_string(name, "name", 1, 100)?;
        update_fields.push(format!("`name` = ?{param_index}"));
        params_with_mapping.push(("name".to_string(), Value::String(name.clone())));
        param_index += 1;
    }

    if let Some(balance) = request.balance {
        Validator::validate_amount(balance, true)?;
        update_fields.push(format!("`balance` = ?{param_index}"));
        params_with_mapping.push(("balance".to_string(), Value::String(balance.to_string())));
        param_index += 1;
    }

    if let Some(account_number) = &request.account_number {
        update_fields.push(format!("`account_number` = ?{param_index}"));
        params_with_mapping.push((
            "account_number".to_string(),
            Value::String(account_number.clone()),
        ));
        param_index += 1;
    }

    if let Some(is_active) = request.is_active {
        update_fields.push(format!("`is_active` = ?{param_index}"));
        params_with_mapping.push(("is_active".to_string(), Value::Bool(is_active)));
        param_index += 1;
    }

    if update_fields.is_empty() {
        return Err(FiscusError::InvalidInput("No fields to update".to_string()));
    }

    // Add updated_at timestamp
    update_fields.push(format!("`updated_at` = ?{param_index}"));
    params_with_mapping.push((
        "updated_at".to_string(),
        Value::String(chrono::Utc::now().to_rfc3339()),
    ));
    param_index += 1;

    // Add account_id for WHERE clause
    params_with_mapping.push(("id".to_string(), Value::String(account_id.clone())));

    let update_query = format!(
        "UPDATE accounts SET {} WHERE id = ?{}",
        update_fields.join(", "),
        param_index
    );

    // Encrypt sensitive parameters before update
    let encrypted_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
        params_with_mapping,
        &user_id,
        "accounts",
    )
    .await?;

    let affected_rows =
        DatabaseUtils::execute_non_query(&db, &update_query, encrypted_params).await?;

    if affected_rows == 0 {
        return Err(FiscusError::NotFound("Account not found".to_string()));
    }

    // Return updated account
    get_account_by_id(account_id, db).await
}

/// Delete an account (soft delete by setting is_active to false)
#[tauri::command]
pub async fn delete_account(
    account_id: String,
    user_id: String,
    db: State<'_, Database>,
) -> Result<bool, FiscusError> {
    // Validate input
    Validator::validate_uuid(&account_id, "account_id")?;
    Validator::validate_uuid(&user_id, "user_id")?;

    // Validate ownership
    DatabaseUtils::validate_account_ownership(&db, &account_id, &user_id).await?;

    // Check if account has transactions
    let transaction_count_query =
        "SELECT COUNT(*) as count FROM transactions WHERE account_id = ?1";
    let count_result: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            transaction_count_query,
            vec![Value::String(account_id.clone())],
        )
        .await?;

    let transaction_count = count_result
        .and_then(|row| row.get("count").cloned())
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    if transaction_count > 0 {
        // Soft delete - set is_active to false
        let update_query = "UPDATE accounts SET is_active = ?1, updated_at = ?2 WHERE id = ?3";
        let params_with_mapping = vec![
            ("is_active".to_string(), Value::Bool(false)),
            (
                "updated_at".to_string(),
                Value::String(chrono::Utc::now().to_rfc3339()),
            ),
            ("id".to_string(), Value::String(account_id)),
        ];

        let encrypted_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
            params_with_mapping,
            &user_id,
            "accounts",
        )
        .await?;

        let affected_rows =
            DatabaseUtils::execute_non_query(&db, update_query, encrypted_params).await?;
        Ok(affected_rows > 0)
    } else {
        // Hard delete if no transactions - no encryption needed for DELETE
        let delete_query = "DELETE FROM accounts WHERE id = ?1";
        let params = vec![Value::String(account_id)];

        let affected_rows = DatabaseUtils::execute_non_query(&db, delete_query, params).await?;
        Ok(affected_rows > 0)
    }
}

/// Get account summary for a user
#[tauri::command]
pub async fn get_account_summary(
    user_id: String,
    db: State<'_, Database>,
) -> Result<AccountSummaryResponse, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    // For aggregation on encrypted fields, we need to fetch all accounts first and decrypt them
    let accounts_query = r#"
        SELECT a.id, a.user_id, a.account_type_id, a.name, a.balance, a.currency,
               a.account_number, a.is_active, a.created_at, a.updated_at, at.is_asset
        FROM accounts a
        JOIN account_types at ON a.account_type_id = at.id
        WHERE a.user_id = ?1 AND a.is_active = 1
    "#;

    // Use encrypted query to properly decrypt balance fields
    let accounts_with_types: Vec<HashMap<String, serde_json::Value>> =
        EncryptedDatabaseUtils::execute_encrypted_query(
            &db,
            accounts_query,
            vec![Value::String(user_id.clone())],
            &user_id,
            "accounts",
        )
        .await?;

    // Calculate summary from decrypted account data
    let mut total_assets = rust_decimal::Decimal::ZERO;
    let mut total_liabilities = rust_decimal::Decimal::ZERO;
    let account_count = accounts_with_types.len() as i32;

    for account in accounts_with_types {
        let balance = account
            .get("balance")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<rust_decimal::Decimal>().ok())
            .unwrap_or(rust_decimal::Decimal::ZERO);

        let is_asset = account
            .get("is_asset")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if is_asset {
            total_assets += balance;
        } else {
            total_liabilities += balance.abs();
        }
    }

    let net_worth = total_assets - total_liabilities;

    Ok(AccountSummaryResponse {
        total_assets,
        total_liabilities,
        net_worth,
        account_count,
    })
}
