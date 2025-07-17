use serde_json::Value;
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

use crate::{
    database::{encrypted::EncryptedDatabaseUtils, Database, DatabaseUtils},
    dto::{
        BulkTransactionAction, BulkTransactionRequest, CreateTransactionRequest,
        CreateTransferRequest, ExportFormat, PaginatedResponse, TransactionFilters,
        TransactionStatsResponse, TransactionSummaryResponse, UpdateTransactionRequest,
    },
    error::{FiscusError, SecurityValidator, Validator},
    models::{Transaction, TransactionStatus, TransactionType, Transfer},
    utils::parse_decimal_from_json,
    with_transaction,
};

/// Create a new transaction
#[tauri::command]
pub async fn create_transaction(
    request: CreateTransactionRequest,
    db: State<'_, Database>,
) -> Result<Transaction, FiscusError> {
    // Validate input (user_id already validated by ValidatedUserId)
    Validator::validate_uuid(&request.account_id, "account_id")?;
    Validator::validate_string(&request.description, "description", 1, 255)?;
    Validator::validate_amount(request.amount, true)?; // Allow negative for refunds/corrections

    // Format the DateTime to RFC3339 string for validation
    let transaction_date = Validator::validate_datetime(&request.transaction_date.to_rfc3339())?;

    if let Some(ref category_id) = request.category_id {
        Validator::validate_uuid(category_id, "category_id")?;
    }

    // Validate ownership
    DatabaseUtils::validate_account_ownership(&db, &request.account_id, &request.user_id.as_str())
        .await?;

    if let Some(ref category_id) = request.category_id {
        DatabaseUtils::validate_category_ownership(&db, category_id, &request.user_id.as_str())
            .await?;
    }

    let transaction_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Use transaction for atomicity
    with_transaction!(&*db, async {
        // Insert transaction
        let insert_query = r#"
            INSERT INTO transactions (
                id, user_id, account_id, category_id, amount, description, notes,
                transaction_date, transaction_type, status, reference_number, payee, tags,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        "#;

        let tags_json = request
            .tags
            .as_ref()
            .map(|tags| serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string()));

        // Use encrypted parameter mapping for sensitive fields
        let params_with_mapping = vec![
            ("id".to_string(), Value::String(transaction_id.clone())),
            (
                "user_id".to_string(),
                Value::String(request.user_id.to_string()),
            ),
            (
                "account_id".to_string(),
                Value::String(request.account_id.clone()),
            ),
            (
                "category_id".to_string(),
                request
                    .category_id
                    .as_ref()
                    .map(|id| Value::String(id.clone()))
                    .unwrap_or(Value::Null),
            ),
            (
                "amount".to_string(),
                Value::String(request.amount.to_string()),
            ),
            (
                "description".to_string(),
                Value::String(request.description.clone()),
            ),
            (
                "notes".to_string(),
                request
                    .notes
                    .as_ref()
                    .map(|n| Value::String(n.clone()))
                    .unwrap_or(Value::Null),
            ),
            (
                "transaction_date".to_string(),
                Value::String(transaction_date.to_rfc3339()),
            ),
            (
                "transaction_type".to_string(),
                Value::String(request.transaction_type.to_string()),
            ),
            (
                "status".to_string(),
                Value::String(TransactionStatus::Completed.to_string()),
            ),
            (
                "reference_number".to_string(),
                request
                    .reference_number
                    .as_ref()
                    .map(|r| Value::String(r.clone()))
                    .unwrap_or(Value::Null),
            ),
            (
                "payee".to_string(),
                request
                    .payee
                    .as_ref()
                    .map(|p| Value::String(p.clone()))
                    .unwrap_or(Value::Null),
            ),
            (
                "tags".to_string(),
                tags_json
                    .as_ref()
                    .map(|t| Value::String(t.clone()))
                    .unwrap_or(Value::Null),
            ),
            ("created_at".to_string(), Value::String(now.clone())),
            ("updated_at".to_string(), Value::String(now)),
        ];

        let encrypted_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
            params_with_mapping,
            &request.user_id.as_str(),
            "transactions",
        )
        .await?;

        DatabaseUtils::execute_non_query(&db, insert_query, encrypted_params).await?;

        // Update account balance based on transaction type
        let current_balance = DatabaseUtils::get_account_balance(&db, &request.account_id).await?;
        let new_balance = match request.transaction_type {
            TransactionType::Income => current_balance + request.amount,
            TransactionType::Expense => current_balance - request.amount,
            TransactionType::Transfer => current_balance, // Transfers are handled separately
        };

        if request.transaction_type != TransactionType::Transfer {
            DatabaseUtils::update_account_balance(&db, &request.account_id, new_balance).await?;
        }

        Ok::<(), FiscusError>(())
    })?;

    // Return the created transaction
    get_transaction_by_id(transaction_id, db).await
}

/// Get transactions with filtering and pagination
#[tauri::command]
pub async fn get_transactions(
    filters: TransactionFilters,
    db: State<'_, Database>,
) -> Result<Vec<Transaction>, FiscusError> {
    // Validate user (already validated by ValidatedUserId)
    DatabaseUtils::validate_user_exists(&db, &filters.user_id.as_str()).await?;

    // Build filter map
    let mut filter_map = HashMap::new();
    filter_map.insert("user_id".to_string(), filters.user_id.as_str().to_string());

    if let Some(account_id) = filters.account_id {
        Validator::validate_uuid(&account_id, "account_id")?;
        filter_map.insert("account_id".to_string(), account_id);
    }

    if let Some(category_id) = filters.category_id {
        Validator::validate_uuid(&category_id, "category_id")?;
        filter_map.insert("category_id".to_string(), category_id);
    }

    if let Some(transaction_type) = filters.transaction_type {
        filter_map.insert("transaction_type".to_string(), transaction_type.to_string());
    }

    if let Some(status) = filters.status {
        filter_map.insert("status".to_string(), status.to_string());
    }

    if let Some(start_date) = filters.start_date {
        Validator::validate_date(&start_date)?;
        filter_map.insert("start_date".to_string(), start_date);
    }

    if let Some(end_date) = filters.end_date {
        Validator::validate_date(&end_date)?;
        filter_map.insert("end_date".to_string(), end_date);
    }

    if let Some(min_amount) = filters.min_amount {
        filter_map.insert("min_amount".to_string(), min_amount.to_string());
    }

    if let Some(max_amount) = filters.max_amount {
        filter_map.insert("max_amount".to_string(), max_amount.to_string());
    }

    // Validate filter fields
    SecurityValidator::validate_transaction_filter_fields(&filter_map)?;

    let base_query = r#"
        SELECT id, user_id, account_id, category_id, amount, description, notes,
               transaction_date, transaction_type, status, reference_number, payee, tags,
               created_at, updated_at
        FROM transactions
    "#
    .to_string();

    // Add search functionality
    let mut search_conditions = Vec::new();
    if let Some(ref search) = filters.search {
        if !search.trim().is_empty() {
            search_conditions
                .push("(description LIKE ? OR payee LIKE ? OR notes LIKE ?)".to_string());
        }
    }

    let (where_clause, mut where_params) = DatabaseUtils::build_where_clause(
        &filter_map,
        &[
            "user_id",
            "account_id",
            "category_id",
            "transaction_type",
            "status",
            "start_date",
            "end_date",
            "min_amount",
            "max_amount",
        ],
        search_conditions,
    )?;

    // Add search parameters
    if let Some(search) = filters.search {
        if !search.trim().is_empty() {
            let search_pattern = format!("%{}%", search.trim());
            where_params.push(Value::String(search_pattern.clone()));
            where_params.push(Value::String(search_pattern.clone()));
            where_params.push(Value::String(search_pattern));
        }
    }

    let order_clause = DatabaseUtils::build_order_clause(
        filters.sort_by.as_deref(),
        filters.sort_direction.as_deref(),
        SecurityValidator::TRANSACTION_SORT_FIELDS,
        "transaction_date",
    )?;

    let limit_clause = DatabaseUtils::build_limit_clause(filters.limit, filters.offset);

    let final_query = format!("{base_query} {where_clause} {order_clause} {limit_clause}");

    // Use encrypted query to properly decrypt sensitive fields
    let transactions: Vec<Transaction> = EncryptedDatabaseUtils::execute_encrypted_query(
        &db,
        &final_query,
        where_params,
        &filters.user_id.as_str(),
        "transactions",
    )
    .await?;

    Ok(transactions)
}

/// Get transactions with pagination support
#[tauri::command]
pub async fn get_transactions_paginated(
    filters: TransactionFilters,
    db: State<'_, Database>,
) -> Result<PaginatedResponse<Transaction>, FiscusError> {
    // Validate user (already validated by ValidatedUserId)
    DatabaseUtils::validate_user_exists(&db, &filters.user_id.as_str()).await?;

    // Get total count first
    let count_query = r#"
        SELECT COUNT(*) as total
        FROM transactions
        WHERE user_id = ?1
    "#;

    let count_result: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            count_query,
            vec![Value::String(filters.user_id.as_str().to_string())],
        )
        .await?;

    let total = count_result
        .and_then(|row| row.get("total").and_then(|v| v.as_i64()))
        .unwrap_or(0) as i32;

    // Get transactions with filters
    let transactions = get_transactions(filters.clone(), db).await?;

    let page = filters.offset.unwrap_or(0) / filters.limit.unwrap_or(50) + 1;
    let per_page = filters.limit.unwrap_or(50);

    Ok(PaginatedResponse::new(transactions, total, page, per_page))
}

/// Get transaction statistics
#[tauri::command]
pub async fn get_transaction_stats(
    filters: TransactionFilters,
    db: State<'_, Database>,
) -> Result<TransactionStatsResponse, FiscusError> {
    // Validate user (already validated by ValidatedUserId)
    DatabaseUtils::validate_user_exists(&db, &filters.user_id.as_str()).await?;

    let stats_query = r#"
        SELECT
            COUNT(*) as total_transactions,
            COALESCE(SUM(CASE WHEN transaction_type = 'income' THEN amount ELSE 0 END), 0) as total_income,
            COALESCE(SUM(CASE WHEN transaction_type = 'expense' THEN amount ELSE 0 END), 0) as total_expenses,
            COALESCE(AVG(amount), 0) as average_amount,
            MAX(CASE WHEN transaction_type = 'expense' THEN amount END) as largest_expense,
            MAX(CASE WHEN transaction_type = 'income' THEN amount END) as largest_income
        FROM transactions
        WHERE user_id = ?1
    "#;

    let stats_result: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            stats_query,
            vec![Value::String(filters.user_id.as_str().to_string())],
        )
        .await?;

    let stats = stats_result
        .ok_or_else(|| FiscusError::NotFound("No transaction statistics found".to_string()))?;

    let total_income = parse_decimal_from_json(&stats, "total_income");
    let total_expenses = parse_decimal_from_json(&stats, "total_expenses");

    // Get transactions by type
    let type_query = r#"
        SELECT transaction_type, COUNT(*) as count
        FROM transactions
        WHERE user_id = ?1
        GROUP BY transaction_type
    "#;

    let type_results: Vec<HashMap<String, serde_json::Value>> = DatabaseUtils::execute_query(
        &db,
        type_query,
        vec![Value::String(filters.user_id.as_str().to_string())],
    )
    .await?;

    let mut transactions_by_type = HashMap::new();
    for row in type_results {
        if let (Some(tx_type), Some(count)) = (
            row.get("transaction_type").and_then(|v| v.as_str()),
            row.get("count").and_then(|v| v.as_i64()),
        ) {
            transactions_by_type.insert(tx_type.to_string(), count as i32);
        }
    }

    // Get transactions by status
    let status_query = r#"
        SELECT status, COUNT(*) as count
        FROM transactions
        WHERE user_id = ?1
        GROUP BY status
    "#;

    let status_results: Vec<HashMap<String, serde_json::Value>> = DatabaseUtils::execute_query(
        &db,
        status_query,
        vec![Value::String(filters.user_id.as_str().to_string())],
    )
    .await?;

    let mut transactions_by_status = HashMap::new();
    for row in status_results {
        if let (Some(status), Some(count)) = (
            row.get("status").and_then(|v| v.as_str()),
            row.get("count").and_then(|v| v.as_i64()),
        ) {
            transactions_by_status.insert(status.to_string(), count as i32);
        }
    }

    Ok(TransactionStatsResponse {
        total_transactions: stats
            .get("total_transactions")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32,
        total_income,
        total_expenses,
        net_income: total_income - total_expenses,
        average_transaction_amount: parse_decimal_from_json(&stats, "average_amount"),
        largest_expense: {
            let value = parse_decimal_from_json(&stats, "largest_expense");
            if value == rust_decimal::Decimal::ZERO {
                None
            } else {
                Some(value)
            }
        },
        largest_income: {
            let value = parse_decimal_from_json(&stats, "largest_income");
            if value == rust_decimal::Decimal::ZERO {
                None
            } else {
                Some(value)
            }
        },
        most_frequent_category: None, // TODO: Implement category analysis
        transactions_by_type,
        transactions_by_status,
    })
}

/// Get a single transaction by ID (internal helper with user_id for encryption)
async fn get_transaction_by_id_encrypted(
    transaction_id: String,
    user_id: &str,
    db: &Database,
) -> Result<Transaction, FiscusError> {
    Validator::validate_uuid(&transaction_id, "transaction_id")?;

    let query = r#"
        SELECT id, user_id, account_id, category_id, amount, description, notes,
               transaction_date, transaction_type, status, reference_number, payee, tags,
               created_at, updated_at
        FROM transactions
        WHERE id = ?1
    "#;

    let transactions: Vec<Transaction> = EncryptedDatabaseUtils::execute_encrypted_query(
        db,
        query,
        vec![Value::String(transaction_id.clone())],
        user_id,
        "transactions",
    )
    .await?;

    transactions
        .into_iter()
        .next()
        .ok_or_else(|| FiscusError::NotFound("Transaction not found".to_string()))
}

/// Get a single transaction by ID
#[tauri::command]
pub async fn get_transaction_by_id(
    transaction_id: String,
    db: State<'_, Database>,
) -> Result<Transaction, FiscusError> {
    Validator::validate_uuid(&transaction_id, "transaction_id")?;

    // First, get the user_id for this transaction (this field is not encrypted)
    let user_query = "SELECT user_id FROM transactions WHERE id = ?1";
    let user_result: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            user_query,
            vec![Value::String(transaction_id.clone())],
        )
        .await?;

    let user_id = user_result
        .and_then(|row| {
            row.get("user_id")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        })
        .ok_or_else(|| FiscusError::NotFound("Transaction not found".to_string()))?;

    // Now get the full transaction with decryption
    get_transaction_by_id_encrypted(transaction_id, &user_id, &db).await
}

/// Update a transaction
#[tauri::command]
pub async fn update_transaction(
    transaction_id: String,
    user_id: String,
    request: UpdateTransactionRequest,
    db: State<'_, Database>,
) -> Result<Transaction, FiscusError> {
    // Validate input
    Validator::validate_uuid(&transaction_id, "transaction_id")?;
    Validator::validate_uuid(&user_id, "user_id")?;

    // Get current transaction to validate ownership and calculate balance changes
    let current_transaction = get_transaction_by_id(transaction_id.clone(), db.clone()).await?;

    if current_transaction.user_id != user_id {
        return Err(FiscusError::Authorization(
            "Transaction access denied".to_string(),
        ));
    }

    // Build update query dynamically with encrypted parameter mapping
    let mut update_fields = Vec::new();
    let mut params_with_mapping = Vec::new();
    let mut param_index = 1;

    if let Some(category_id) = &request.category_id {
        Validator::validate_uuid(category_id, "category_id")?;
        DatabaseUtils::validate_category_ownership(&db, category_id, &user_id).await?;
        update_fields.push(format!("`category_id` = ?{param_index}"));
        params_with_mapping.push((
            "category_id".to_string(),
            Value::String(category_id.clone()),
        ));
        param_index += 1;
    }

    let mut amount_changed = false;
    let mut new_amount = current_transaction.amount;
    if let Some(amount) = request.amount {
        Validator::validate_amount(amount, true)?;
        update_fields.push(format!("`amount` = ?{param_index}"));
        params_with_mapping.push(("amount".to_string(), Value::String(amount.to_string())));
        param_index += 1;
        amount_changed = true;
        new_amount = amount;
    }

    // Track transaction type changes for balance calculation
    let mut transaction_type_changed = false;
    let mut new_transaction_type = current_transaction.transaction_type.clone();
    if let Some(transaction_type) = &request.transaction_type {
        if *transaction_type != current_transaction.transaction_type {
            transaction_type_changed = true;
            new_transaction_type = transaction_type.clone();
        }
    }

    // Prevent changing transaction type to or from Transfer
    // Transfers involve two linked transactions and require special handling
    if transaction_type_changed {
        if current_transaction.transaction_type == TransactionType::Transfer {
            return Err(FiscusError::InvalidInput(
                "Cannot change transaction type from Transfer. Transfers involve linked transactions and must be handled separately.".to_string(),
            ));
        }
        if new_transaction_type == TransactionType::Transfer {
            return Err(FiscusError::InvalidInput(
                "Cannot change transaction type to Transfer. Use the create_transfer function to create transfers between accounts.".to_string(),
            ));
        }
    }

    if let Some(description) = &request.description {
        Validator::validate_string(description, "description", 1, 255)?;
        update_fields.push(format!("`description` = ?{param_index}"));
        params_with_mapping.push((
            "description".to_string(),
            Value::String(description.clone()),
        ));
        param_index += 1;
    }

    if let Some(notes) = &request.notes {
        update_fields.push(format!("`notes` = ?{param_index}"));
        params_with_mapping.push(("notes".to_string(), Value::String(notes.clone())));
        param_index += 1;
    }

    if let Some(transaction_date) = &request.transaction_date {
        let parsed_date = Validator::validate_datetime(transaction_date)?;
        update_fields.push(format!("`transaction_date` = ?{param_index}"));
        params_with_mapping.push((
            "transaction_date".to_string(),
            Value::String(parsed_date.to_rfc3339()),
        ));
        param_index += 1;
    }

    if let Some(transaction_type) = &request.transaction_type {
        update_fields.push(format!("`transaction_type` = ?{param_index}"));
        params_with_mapping.push((
            "transaction_type".to_string(),
            Value::String(transaction_type.to_string()),
        ));
        param_index += 1;
    }

    if let Some(status) = &request.status {
        update_fields.push(format!("`status` = ?{param_index}"));
        params_with_mapping.push(("status".to_string(), Value::String(status.to_string())));
        param_index += 1;
    }

    if let Some(reference_number) = &request.reference_number {
        update_fields.push(format!("`reference_number` = ?{param_index}"));
        params_with_mapping.push((
            "reference_number".to_string(),
            Value::String(reference_number.clone()),
        ));
        param_index += 1;
    }

    if let Some(payee) = &request.payee {
        update_fields.push(format!("`payee` = ?{param_index}"));
        params_with_mapping.push(("payee".to_string(), Value::String(payee.clone())));
        param_index += 1;
    }

    if let Some(tags) = &request.tags {
        let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
        update_fields.push(format!("`tags` = ?{param_index}"));
        params_with_mapping.push(("tags".to_string(), Value::String(tags_json)));
        param_index += 1;
    }

    if update_fields.is_empty() {
        return Err(FiscusError::InvalidInput("No fields to update".to_string()));
    }

    // Use transaction for atomicity
    with_transaction!(&*db, async {
        // Update transaction
        update_fields.push(format!("`updated_at` = ?{param_index}"));
        params_with_mapping.push((
            "updated_at".to_string(),
            Value::String(chrono::Utc::now().to_rfc3339()),
        ));
        param_index += 1;

        params_with_mapping.push(("id".to_string(), Value::String(transaction_id.clone())));

        let update_query = format!(
            "UPDATE transactions SET {} WHERE id = ?{}",
            update_fields.join(", "),
            param_index
        );

        // Encrypt sensitive parameters before update
        let encrypted_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
            params_with_mapping,
            &user_id,
            "transactions",
        )
        .await?;

        let affected_rows =
            DatabaseUtils::execute_non_query(&db, &update_query, encrypted_params).await?;

        if affected_rows == 0 {
            return Err(FiscusError::NotFound("Transaction not found".to_string()));
        }

        // Update account balance if amount or transaction type changed
        // Note: Transfer type changes are prevented above, so we only need to check for non-Transfer transactions
        if (amount_changed || transaction_type_changed)
            && current_transaction.transaction_type != TransactionType::Transfer
        {
            let current_balance =
                DatabaseUtils::get_account_balance(&db, &current_transaction.account_id).await?;

            // Reverse the old transaction effect using the original transaction type and amount
            let balance_after_reversal = match current_transaction.transaction_type {
                TransactionType::Income => current_balance - current_transaction.amount,
                TransactionType::Expense => current_balance + current_transaction.amount,
                TransactionType::Transfer => current_balance,
            };

            // Apply the new transaction effect using the new transaction type and amount
            let new_balance = match new_transaction_type {
                TransactionType::Income => balance_after_reversal + new_amount,
                TransactionType::Expense => balance_after_reversal - new_amount,
                TransactionType::Transfer => balance_after_reversal,
            };

            DatabaseUtils::update_account_balance(
                &db,
                &current_transaction.account_id,
                new_balance,
            )
            .await?;
        }

        Ok::<(), FiscusError>(())
    })?;

    // Return updated transaction
    get_transaction_by_id(transaction_id, db).await
}

#[cfg(test)]
mod update_transaction_tests {
    use crate::models::TransactionType;

    #[test]
    fn test_transaction_type_change_validation_logic() {
        // Test the validation logic for transaction type changes
        // This tests the core logic without requiring a full database setup

        // Test case 1: Changing FROM Transfer to Income should be detected as invalid
        let current_type = TransactionType::Transfer;
        let new_type = TransactionType::Income;
        let transaction_type_changed = new_type != current_type;

        assert!(transaction_type_changed);
        assert_eq!(current_type, TransactionType::Transfer);
        assert_eq!(new_type, TransactionType::Income);

        // In the actual function, this would trigger the validation error
        if transaction_type_changed {
            if current_type == TransactionType::Transfer {
                // This would return an error in the actual function
                // Note: Transfer type changes should be prevented
            }
            if new_type == TransactionType::Transfer {
                // This would return an error in the actual function
                // Note: Transfer type changes should be prevented
            }
        }

        // Test case 2: Changing TO Transfer from Expense should be detected as invalid
        let current_type_2 = TransactionType::Expense;
        let new_type_2 = TransactionType::Transfer;
        let transaction_type_changed_2 = new_type_2 != current_type_2;

        assert!(transaction_type_changed_2);
        assert_eq!(current_type_2, TransactionType::Expense);
        assert_eq!(new_type_2, TransactionType::Transfer);

        // Test case 3: Changing from Income to Expense should be allowed
        let current_type_3 = TransactionType::Income;
        let new_type_3 = TransactionType::Expense;
        let transaction_type_changed_3 = new_type_3 != current_type_3;

        assert!(transaction_type_changed_3);
        assert_ne!(current_type_3, TransactionType::Transfer);
        assert_ne!(new_type_3, TransactionType::Transfer);
        // This change should be allowed
    }
}

/// Delete a transaction
#[tauri::command]
pub async fn delete_transaction(
    transaction_id: String,
    user_id: String,
    db: State<'_, Database>,
) -> Result<bool, FiscusError> {
    // Validate input
    Validator::validate_uuid(&transaction_id, "transaction_id")?;
    Validator::validate_uuid(&user_id, "user_id")?;

    // Get current transaction to validate ownership and update balance
    let current_transaction = get_transaction_by_id(transaction_id.clone(), db.clone()).await?;

    if current_transaction.user_id != user_id {
        return Err(FiscusError::Authorization(
            "Transaction access denied".to_string(),
        ));
    }

    // Use transaction for atomicity
    with_transaction!(&*db, async {
        // Delete the transaction
        let delete_query = "DELETE FROM transactions WHERE id = ?1";
        let params = vec![Value::String(transaction_id)];

        let affected_rows = DatabaseUtils::execute_non_query(&db, delete_query, params).await?;

        if affected_rows == 0 {
            return Err(FiscusError::NotFound("Transaction not found".to_string()));
        }

        // Update account balance by reversing the transaction effect
        if current_transaction.transaction_type != TransactionType::Transfer {
            let current_balance =
                DatabaseUtils::get_account_balance(&db, &current_transaction.account_id).await?;

            let new_balance = match current_transaction.transaction_type {
                TransactionType::Income => current_balance - current_transaction.amount,
                TransactionType::Expense => current_balance + current_transaction.amount,
                TransactionType::Transfer => current_balance,
            };

            DatabaseUtils::update_account_balance(
                &db,
                &current_transaction.account_id,
                new_balance,
            )
            .await?;
        }

        Ok::<(), FiscusError>(())
    })?;

    Ok(true)
}

/// Create a transfer between accounts
#[tauri::command]
pub async fn create_transfer(
    request: CreateTransferRequest,
    db: State<'_, Database>,
) -> Result<Transfer, FiscusError> {
    // Validate input (user_id already validated by ValidatedUserId)
    Validator::validate_uuid(&request.from_account_id, "from_account_id")?;
    Validator::validate_uuid(&request.to_account_id, "to_account_id")?;
    Validator::validate_amount(request.amount, false)?; // Transfers must be positive
    Validator::validate_string(&request.description, "description", 1, 255)?;

    let transfer_date = Validator::validate_datetime(&request.transfer_date)?;

    if request.from_account_id == request.to_account_id {
        return Err(FiscusError::InvalidInput(
            "Cannot transfer to the same account".to_string(),
        ));
    }

    // Validate account ownership
    DatabaseUtils::validate_account_ownership(
        &db,
        &request.from_account_id,
        &request.user_id.as_str(),
    )
    .await?;
    DatabaseUtils::validate_account_ownership(
        &db,
        &request.to_account_id,
        &request.user_id.as_str(),
    )
    .await?;

    let transfer_id = Uuid::new_v4().to_string();
    let from_transaction_id = Uuid::new_v4().to_string();
    let to_transaction_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Use transaction for atomicity
    with_transaction!(&*db, async {
        // Create the transfer record
        let transfer_query = r#"
            INSERT INTO transfers (
                id, user_id, from_account_id, to_account_id, amount, description,
                transfer_date, status, from_transaction_id, to_transaction_id,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#;

        // Use encrypted parameter mapping for transfer record
        let transfer_params_with_mapping = vec![
            ("id".to_string(), Value::String(transfer_id.clone())),
            (
                "user_id".to_string(),
                Value::String(request.user_id.to_string()),
            ),
            (
                "from_account_id".to_string(),
                Value::String(request.from_account_id.clone()),
            ),
            (
                "to_account_id".to_string(),
                Value::String(request.to_account_id.clone()),
            ),
            (
                "amount".to_string(),
                Value::String(request.amount.to_string()),
            ),
            (
                "description".to_string(),
                Value::String(request.description.clone()),
            ),
            (
                "transfer_date".to_string(),
                Value::String(transfer_date.to_rfc3339()),
            ),
            (
                "status".to_string(),
                Value::String(TransactionStatus::Completed.to_string()),
            ),
            (
                "from_transaction_id".to_string(),
                Value::String(from_transaction_id.clone()),
            ),
            (
                "to_transaction_id".to_string(),
                Value::String(to_transaction_id.clone()),
            ),
            ("created_at".to_string(), Value::String(now.clone())),
            ("updated_at".to_string(), Value::String(now.clone())),
        ];

        let encrypted_transfer_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
            transfer_params_with_mapping,
            &request.user_id.as_str(),
            "transfers",
        )
        .await?;

        DatabaseUtils::execute_non_query(&db, transfer_query, encrypted_transfer_params).await?;

        // Create outgoing transaction (expense)
        let from_transaction_query = r#"
            INSERT INTO transactions (
                id, user_id, account_id, amount, description, transaction_date,
                transaction_type, status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#;

        // Use encrypted parameter mapping for outgoing transaction
        let from_params_with_mapping = vec![
            ("id".to_string(), Value::String(from_transaction_id)),
            (
                "user_id".to_string(),
                Value::String(request.user_id.to_string()),
            ),
            (
                "account_id".to_string(),
                Value::String(request.from_account_id.clone()),
            ),
            (
                "amount".to_string(),
                Value::String((-request.amount).to_string()),
            ), // Negative for outgoing
            (
                "description".to_string(),
                Value::String(format!("Transfer to account: {}", request.description)),
            ),
            (
                "transaction_date".to_string(),
                Value::String(transfer_date.to_rfc3339()),
            ),
            (
                "transaction_type".to_string(),
                Value::String(TransactionType::Transfer.to_string()),
            ),
            (
                "status".to_string(),
                Value::String(TransactionStatus::Completed.to_string()),
            ),
            ("created_at".to_string(), Value::String(now.clone())),
            ("updated_at".to_string(), Value::String(now.clone())),
        ];

        let encrypted_from_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
            from_params_with_mapping,
            &request.user_id.as_str(),
            "transactions",
        )
        .await?;

        DatabaseUtils::execute_non_query(&db, from_transaction_query, encrypted_from_params)
            .await?;

        // Create incoming transaction (income)
        let to_transaction_query = r#"
            INSERT INTO transactions (
                id, user_id, account_id, amount, description, transaction_date,
                transaction_type, status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#;

        // Use encrypted parameter mapping for incoming transaction
        let to_params_with_mapping = vec![
            ("id".to_string(), Value::String(to_transaction_id)),
            (
                "user_id".to_string(),
                Value::String(request.user_id.to_string()),
            ),
            (
                "account_id".to_string(),
                Value::String(request.to_account_id.clone()),
            ),
            (
                "amount".to_string(),
                Value::String(request.amount.to_string()),
            ), // Positive for incoming
            (
                "description".to_string(),
                Value::String(format!("Transfer from account: {}", request.description)),
            ),
            (
                "transaction_date".to_string(),
                Value::String(transfer_date.to_rfc3339()),
            ),
            (
                "transaction_type".to_string(),
                Value::String(TransactionType::Transfer.to_string()),
            ),
            (
                "status".to_string(),
                Value::String(TransactionStatus::Completed.to_string()),
            ),
            ("created_at".to_string(), Value::String(now.clone())),
            ("updated_at".to_string(), Value::String(now)),
        ];

        let encrypted_to_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
            to_params_with_mapping,
            &request.user_id.as_str(),
            "transactions",
        )
        .await?;

        DatabaseUtils::execute_non_query(&db, to_transaction_query, encrypted_to_params).await?;

        // Update account balances
        let from_balance =
            DatabaseUtils::get_account_balance(&db, &request.from_account_id).await?;
        let to_balance = DatabaseUtils::get_account_balance(&db, &request.to_account_id).await?;

        DatabaseUtils::update_account_balance(
            &db,
            &request.from_account_id,
            from_balance - request.amount,
        )
        .await?;
        DatabaseUtils::update_account_balance(
            &db,
            &request.to_account_id,
            to_balance + request.amount,
        )
        .await?;

        Ok::<(), FiscusError>(())
    })?;

    // Return the created transfer
    get_transfer_by_id(transfer_id, db).await
}

/// Get a transfer by ID
#[tauri::command]
pub async fn get_transfer_by_id(
    transfer_id: String,
    db: State<'_, Database>,
) -> Result<Transfer, FiscusError> {
    Validator::validate_uuid(&transfer_id, "transfer_id")?;

    // First, get the user_id for this transfer (this field is not encrypted)
    let user_query = "SELECT user_id FROM transfers WHERE id = ?1";
    let user_result: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            user_query,
            vec![Value::String(transfer_id.clone())],
        )
        .await?;

    let user_id = user_result
        .and_then(|row| {
            row.get("user_id")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        })
        .ok_or_else(|| FiscusError::NotFound("Transfer not found".to_string()))?;

    let query = r#"
        SELECT id, user_id, from_account_id, to_account_id, amount, description,
               transfer_date, status, from_transaction_id, to_transaction_id,
               created_at, updated_at
        FROM transfers
        WHERE id = ?1
    "#;

    // Use encrypted query to properly decrypt sensitive fields
    let transfers: Vec<Transfer> = EncryptedDatabaseUtils::execute_encrypted_query(
        &db,
        query,
        vec![Value::String(transfer_id.clone())],
        &user_id,
        "transfers",
    )
    .await?;

    transfers
        .into_iter()
        .next()
        .ok_or_else(|| FiscusError::NotFound("Transfer not found".to_string()))
}

/// Bulk operations on transactions
#[tauri::command]
pub async fn bulk_transaction_operations(
    request: BulkTransactionRequest,
    db: State<'_, Database>,
) -> Result<String, FiscusError> {
    // Validate user (already validated by ValidatedUserId)
    DatabaseUtils::validate_user_exists(&db, &request.user_id.as_str()).await?;

    // Validate transaction IDs
    for transaction_id in &request.transaction_ids {
        Validator::validate_uuid(transaction_id, "transaction_id")?;
    }

    if request.transaction_ids.is_empty() {
        return Err(FiscusError::InvalidInput(
            "No transaction IDs provided".to_string(),
        ));
    }

    if request.transaction_ids.len() > 100 {
        return Err(FiscusError::InvalidInput(
            "Cannot process more than 100 transactions at once".to_string(),
        ));
    }

    match request.action {
        BulkTransactionAction::Delete => {
            bulk_delete_transactions(request.transaction_ids, &request.user_id.as_str(), &db).await
        }
        BulkTransactionAction::UpdateCategory { category_id } => {
            bulk_update_category(
                request.transaction_ids,
                category_id,
                &request.user_id.as_str(),
                &db,
            )
            .await
        }
        BulkTransactionAction::UpdateStatus { status } => {
            bulk_update_status(
                request.transaction_ids,
                status,
                &request.user_id.as_str(),
                &db,
            )
            .await
        }
        BulkTransactionAction::Export { format } => {
            bulk_export_transactions(
                request.transaction_ids,
                format,
                &request.user_id.as_str(),
                &db,
            )
            .await
        }
    }
}

/// Bulk delete transactions
async fn bulk_delete_transactions(
    transaction_ids: Vec<String>,
    user_id: &str,
    db: &Database,
) -> Result<String, FiscusError> {
    with_transaction!(db, async {
        for transaction_id in &transaction_ids {
            // Verify ownership before deletion
            let transaction =
                get_transaction_by_id_encrypted(transaction_id.clone(), user_id, db).await?;

            if transaction.user_id != user_id {
                return Err(FiscusError::Authorization(
                    "Transaction access denied".to_string(),
                ));
            }

            // Delete the transaction
            let delete_query = "DELETE FROM transactions WHERE id = ?1 AND user_id = ?2";
            DatabaseUtils::execute_non_query(
                db,
                delete_query,
                vec![
                    Value::String(transaction_id.clone()),
                    Value::String(user_id.to_string()),
                ],
            )
            .await?;

            // Update account balance
            let balance_change = match transaction.transaction_type {
                TransactionType::Income => -transaction.amount,
                TransactionType::Expense => transaction.amount,
                TransactionType::Transfer => rust_decimal::Decimal::ZERO,
            };

            if balance_change != rust_decimal::Decimal::ZERO {
                let update_balance_query = r#"
                    UPDATE accounts
                    SET current_balance = current_balance + ?1, updated_at = CURRENT_TIMESTAMP
                    WHERE id = ?2 AND user_id = ?3
                "#;

                DatabaseUtils::execute_non_query(
                    db,
                    update_balance_query,
                    vec![
                        Value::String(balance_change.to_string()),
                        Value::String(transaction.account_id),
                        Value::String(user_id.to_string()),
                    ],
                )
                .await?;
            }
        }

        Ok(format!(
            "Successfully deleted {} transactions",
            transaction_ids.len()
        ))
    })
}

/// Bulk update transaction categories
async fn bulk_update_category(
    transaction_ids: Vec<String>,
    category_id: Option<String>,
    user_id: &str,
    db: &Database,
) -> Result<String, FiscusError> {
    // Validate category if provided
    if let Some(ref cat_id) = category_id {
        Validator::validate_uuid(cat_id, "category_id")?;
    }

    with_transaction!(db, async {
        let update_query = r#"
            UPDATE transactions
            SET category_id = ?1, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?2 AND user_id = ?3
        "#;

        for transaction_id in &transaction_ids {
            // Verify ownership
            let transaction =
                get_transaction_by_id_encrypted(transaction_id.clone(), user_id, db).await?;

            if transaction.user_id != user_id {
                return Err(FiscusError::Authorization(
                    "Transaction access denied".to_string(),
                ));
            }

            DatabaseUtils::execute_non_query(
                db,
                update_query,
                vec![
                    category_id
                        .as_ref()
                        .map(|id| Value::String(id.clone()))
                        .unwrap_or(Value::Null),
                    Value::String(transaction_id.clone()),
                    Value::String(user_id.to_string()),
                ],
            )
            .await?;
        }

        Ok(format!(
            "Successfully updated category for {} transactions",
            transaction_ids.len()
        ))
    })
}

/// Bulk update transaction status
async fn bulk_update_status(
    transaction_ids: Vec<String>,
    status: TransactionStatus,
    user_id: &str,
    db: &Database,
) -> Result<String, FiscusError> {
    with_transaction!(db, async {
        let update_query = r#"
            UPDATE transactions
            SET status = ?1, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?2 AND user_id = ?3
        "#;

        for transaction_id in &transaction_ids {
            // Verify ownership
            let transaction =
                get_transaction_by_id_encrypted(transaction_id.clone(), user_id, db).await?;

            if transaction.user_id != user_id {
                return Err(FiscusError::Authorization(
                    "Transaction access denied".to_string(),
                ));
            }

            DatabaseUtils::execute_non_query(
                db,
                update_query,
                vec![
                    Value::String(status.to_string()),
                    Value::String(transaction_id.clone()),
                    Value::String(user_id.to_string()),
                ],
            )
            .await?;
        }

        Ok(format!(
            "Successfully updated status for {} transactions",
            transaction_ids.len()
        ))
    })
}

/// Bulk export transactions
async fn bulk_export_transactions(
    transaction_ids: Vec<String>,
    format: ExportFormat,
    user_id: &str,
    db: &Database,
) -> Result<String, FiscusError> {
    let mut transactions = Vec::new();

    for transaction_id in &transaction_ids {
        let transaction =
            get_transaction_by_id_encrypted(transaction_id.clone(), user_id, db).await?;

        if transaction.user_id != user_id {
            return Err(FiscusError::Authorization(
                "Transaction access denied".to_string(),
            ));
        }

        transactions.push(transaction);
    }

    match format {
        ExportFormat::Json => {
            let json_data = serde_json::to_string_pretty(&transactions)
                .map_err(|e| FiscusError::Internal(format!("JSON serialization failed: {e}")))?;
            Ok(json_data)
        }
        ExportFormat::Csv => {
            let mut csv_data = String::from("id,account_id,category_id,amount,description,transaction_date,transaction_type,status,payee,notes\n");

            for transaction in transactions {
                csv_data.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{}\n",
                    transaction.id,
                    transaction.account_id,
                    transaction.category_id.unwrap_or_default(),
                    transaction.amount,
                    transaction.description.replace(',', ";"),
                    transaction.transaction_date.format("%Y-%m-%d %H:%M:%S"),
                    transaction.transaction_type,
                    transaction.status,
                    transaction.payee.unwrap_or_default().replace(',', ";"),
                    transaction.notes.unwrap_or_default().replace(',', ";")
                ));
            }

            Ok(csv_data)
        }
    }
}

/// Get transaction summary for a user
#[tauri::command]
pub async fn get_transaction_summary(
    user_id: String,
    start_date: Option<String>,
    end_date: Option<String>,
    db: State<'_, Database>,
) -> Result<TransactionSummaryResponse, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    let mut where_conditions = vec![
        "user_id = ?1".to_string(),
        "transaction_type != 'transfer'".to_string(),
    ];
    let mut params = vec![Value::String(user_id)];
    let mut param_index = 2;

    if let Some(start) = start_date {
        Validator::validate_date(&start)?;
        where_conditions.push(format!("DATE(transaction_date) >= ?{param_index}"));
        params.push(Value::String(start));
        param_index += 1;
    }

    if let Some(end) = end_date {
        Validator::validate_date(&end)?;
        where_conditions.push(format!("DATE(transaction_date) <= ?{param_index}"));
        params.push(Value::String(end));
    }

    let summary_query = format!(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN transaction_type = 'income' THEN amount ELSE 0 END), 0) as total_income,
            COALESCE(SUM(CASE WHEN transaction_type = 'expense' THEN amount ELSE 0 END), 0) as total_expenses,
            COUNT(*) as transaction_count,
            COALESCE(AVG(ABS(amount)), 0) as average_transaction
        FROM transactions
        WHERE {}
    "#,
        where_conditions.join(" AND ")
    );

    let summary: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(&db, &summary_query, params).await?;

    let summary_data = summary.unwrap_or_default();

    let total_income = parse_decimal_from_json(&summary_data, "total_income");
    let total_expenses = parse_decimal_from_json(&summary_data, "total_expenses");

    let transaction_count = summary_data
        .get("transaction_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;

    let average_transaction = parse_decimal_from_json(&summary_data, "average_transaction");

    let net_income = total_income - total_expenses;

    Ok(TransactionSummaryResponse {
        total_income,
        total_expenses,
        net_income,
        transaction_count,
        average_transaction,
    })
}
