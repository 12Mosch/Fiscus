use serde_json::Value;
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

use crate::{
    database::{encrypted::EncryptedDatabaseUtils, Database, DatabaseUtils},
    dto::{
        BudgetFilters, BudgetSummaryResponse, CreateBudgetPeriodRequest, CreateBudgetRequest,
        UpdateBudgetRequest,
    },
    error::{FiscusError, SecurityValidator, Validator},
    models::{Budget, BudgetPeriod},
    utils::parse_decimal_from_json,
};

/// Create a new budget period
#[tauri::command]
pub async fn create_budget_period(
    request: CreateBudgetPeriodRequest,
    db: State<'_, Database>,
) -> Result<BudgetPeriod, FiscusError> {
    // Validate input
    Validator::validate_uuid(&request.user_id.as_str(), "user_id")?;
    Validator::validate_string(&request.name, "name", 1, 100)?;

    let start_date = Validator::validate_date(&request.start_date)?;
    let end_date = Validator::validate_date(&request.end_date)?;

    if end_date <= start_date {
        return Err(FiscusError::InvalidInput(
            "End date must be after start date".to_string(),
        ));
    }

    // Validate user exists
    DatabaseUtils::validate_user_exists(&db, &request.user_id.as_str()).await?;

    // Check for overlapping budget periods
    let overlap_query = r#"
        SELECT id FROM budget_periods 
        WHERE user_id = ?1 AND is_active = 1 
        AND ((start_date <= ?2 AND end_date >= ?2) OR (start_date <= ?3 AND end_date >= ?3)
             OR (start_date >= ?2 AND end_date <= ?3))
    "#;

    let overlap_result: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            overlap_query,
            vec![
                Value::String(request.user_id.as_str()),
                Value::String(request.start_date.clone()),
                Value::String(request.end_date.clone()),
            ],
        )
        .await?;

    if overlap_result.is_some() {
        return Err(FiscusError::Conflict(
            "Budget period overlaps with existing period".to_string(),
        ));
    }

    let period_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let insert_query = r#"
        INSERT INTO budget_periods (id, user_id, name, start_date, end_date, is_active, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
    "#;

    let params = vec![
        Value::String(period_id.clone()),
        Value::String(request.user_id.as_str()),
        Value::String(request.name.clone()),
        Value::String(request.start_date),
        Value::String(request.end_date),
        Value::Bool(true),
        Value::String(now.clone()),
        Value::String(now),
    ];

    DatabaseUtils::execute_non_query(&db, insert_query, params).await?;

    // Return the created budget period
    get_budget_period_by_id(period_id, db).await
}

/// Get budget periods for a user
#[tauri::command]
pub async fn get_budget_periods(
    user_id: String,
    is_active: Option<bool>,
    db: State<'_, Database>,
) -> Result<Vec<BudgetPeriod>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    let mut query = r#"
        SELECT id, user_id, name, start_date, end_date, is_active, created_at, updated_at
        FROM budget_periods
        WHERE user_id = ?1
    "#
    .to_string();

    let mut params = vec![Value::String(user_id)];

    if let Some(active) = is_active {
        query.push_str(" AND is_active = ?2");
        params.push(Value::Bool(active));
    }

    query.push_str(" ORDER BY start_date DESC");

    let periods: Vec<BudgetPeriod> = DatabaseUtils::execute_query(&db, &query, params).await?;

    Ok(periods)
}

/// Get a budget period by ID
#[tauri::command]
pub async fn get_budget_period_by_id(
    period_id: String,
    db: State<'_, Database>,
) -> Result<BudgetPeriod, FiscusError> {
    Validator::validate_uuid(&period_id, "period_id")?;

    let query = r#"
        SELECT id, user_id, name, start_date, end_date, is_active, created_at, updated_at
        FROM budget_periods 
        WHERE id = ?1
    "#;

    let period: Option<BudgetPeriod> =
        DatabaseUtils::execute_query_single(&db, query, vec![Value::String(period_id.clone())])
            .await?;

    period.ok_or_else(|| FiscusError::NotFound("Budget period not found".to_string()))
}

/// Create a new budget
#[tauri::command]
pub async fn create_budget(
    request: CreateBudgetRequest,
    db: State<'_, Database>,
) -> Result<Budget, FiscusError> {
    // Validate input
    Validator::validate_uuid(&request.user_id.as_str(), "user_id")?;
    Validator::validate_uuid(&request.budget_period_id, "budget_period_id")?;
    Validator::validate_uuid(&request.category_id, "category_id")?;
    Validator::validate_amount(request.allocated_amount, false)?; // Budget amounts must be positive

    // Validate user exists
    DatabaseUtils::validate_user_exists(&db, &request.user_id.as_str()).await?;

    // Validate budget period exists and belongs to user
    let period_query = "SELECT id FROM budget_periods WHERE id = ?1 AND user_id = ?2";
    let period_exists: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            period_query,
            vec![
                Value::String(request.budget_period_id.clone()),
                Value::String(request.user_id.as_str()),
            ],
        )
        .await?;

    if period_exists.is_none() {
        return Err(FiscusError::NotFound("Budget period not found".to_string()));
    }

    // Validate category exists and belongs to user
    DatabaseUtils::validate_category_ownership(
        &db,
        &request.category_id,
        &request.user_id.as_str(),
    )
    .await?;

    // Check if budget already exists for this period and category
    let existing_query = "SELECT id FROM budgets WHERE budget_period_id = ?1 AND category_id = ?2";
    let existing: Option<HashMap<String, serde_json::Value>> = DatabaseUtils::execute_query_single(
        &db,
        existing_query,
        vec![
            Value::String(request.budget_period_id.clone()),
            Value::String(request.category_id.clone()),
        ],
    )
    .await?;

    if existing.is_some() {
        return Err(FiscusError::Conflict(
            "Budget already exists for this category and period".to_string(),
        ));
    }

    let budget_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let insert_query = r#"
        INSERT INTO budgets (
            id, user_id, budget_period_id, category_id, allocated_amount, 
            spent_amount, notes, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
    "#;

    // Use encrypted parameter mapping for sensitive fields
    let params_with_mapping = vec![
        ("id".to_string(), Value::String(budget_id.clone())),
        (
            "user_id".to_string(),
            Value::String(request.user_id.as_str()),
        ),
        (
            "budget_period_id".to_string(),
            Value::String(request.budget_period_id.clone()),
        ),
        (
            "category_id".to_string(),
            Value::String(request.category_id.clone()),
        ),
        (
            "allocated_amount".to_string(),
            Value::String(request.allocated_amount.to_string()),
        ),
        (
            "spent_amount".to_string(),
            Value::String(rust_decimal::Decimal::ZERO.to_string()),
        ),
        (
            "notes".to_string(),
            request
                .notes
                .as_ref()
                .map(|n| Value::String(n.clone()))
                .unwrap_or(Value::Null),
        ),
        ("created_at".to_string(), Value::String(now.clone())),
        ("updated_at".to_string(), Value::String(now)),
    ];

    let encrypted_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
        params_with_mapping,
        &request.user_id.as_str(),
        "budgets",
    )
    .await?;

    DatabaseUtils::execute_non_query(&db, insert_query, encrypted_params).await?;

    // Return the created budget
    get_budget_by_id(budget_id, db).await
}

/// Get budgets with filtering
#[tauri::command]
pub async fn get_budgets(
    filters: BudgetFilters,
    db: State<'_, Database>,
) -> Result<Vec<Budget>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&filters.user_id.as_str(), "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &filters.user_id.as_str()).await?;

    // Build query with filters
    let mut filter_map = HashMap::new();
    filter_map.insert("user_id".to_string(), filters.user_id.as_str());

    if let Some(period_id) = filters.budget_period_id {
        Validator::validate_uuid(&period_id, "budget_period_id")?;
        filter_map.insert("budget_period_id".to_string(), period_id);
    }

    if let Some(category_id) = filters.category_id {
        Validator::validate_uuid(&category_id, "category_id")?;
        filter_map.insert("category_id".to_string(), category_id);
    }

    let base_query = r#"
        SELECT id, user_id, budget_period_id, category_id, allocated_amount,
               spent_amount, notes, created_at, updated_at
        FROM budgets
    "#;

    let (where_clause, where_params) = DatabaseUtils::build_where_clause(
        &filter_map,
        &["user_id", "budget_period_id", "category_id"],
        vec![],
    )?;

    let order_clause = DatabaseUtils::build_order_clause(
        filters.sort_by.as_deref(),
        filters.sort_direction.as_deref(),
        SecurityValidator::BUDGET_SORT_FIELDS,
        "created_at",
    )?;

    let final_query = format!("{base_query} {where_clause} {order_clause}");

    // Use encrypted query to properly decrypt sensitive fields
    let budgets: Vec<Budget> = EncryptedDatabaseUtils::execute_encrypted_query(
        &db,
        &final_query,
        where_params,
        &filters.user_id.as_str(),
        "budgets",
    )
    .await?;

    Ok(budgets)
}

/// Get a single budget by ID
#[tauri::command]
pub async fn get_budget_by_id(
    budget_id: String,
    db: State<'_, Database>,
) -> Result<Budget, FiscusError> {
    Validator::validate_uuid(&budget_id, "budget_id")?;

    // First, get the user_id for this budget (this field is not encrypted)
    let user_query = "SELECT user_id FROM budgets WHERE id = ?1";
    let user_result: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            user_query,
            vec![Value::String(budget_id.clone())],
        )
        .await?;

    let user_id = user_result
        .and_then(|row| {
            row.get("user_id")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        })
        .ok_or_else(|| FiscusError::NotFound("Budget not found".to_string()))?;

    let query = r#"
        SELECT id, user_id, budget_period_id, category_id, allocated_amount,
               spent_amount, notes, created_at, updated_at
        FROM budgets
        WHERE id = ?1
    "#;

    // Use encrypted query to properly decrypt sensitive fields
    let budgets: Vec<Budget> = EncryptedDatabaseUtils::execute_encrypted_query(
        &db,
        query,
        vec![Value::String(budget_id.clone())],
        &user_id,
        "budgets",
    )
    .await?;

    budgets
        .into_iter()
        .next()
        .ok_or_else(|| FiscusError::NotFound("Budget not found".to_string()))
}

/// Update a budget
#[tauri::command]
pub async fn update_budget(
    budget_id: String,
    user_id: String,
    request: UpdateBudgetRequest,
    db: State<'_, Database>,
) -> Result<Budget, FiscusError> {
    // Validate input
    Validator::validate_uuid(&budget_id, "budget_id")?;
    Validator::validate_uuid(&user_id, "user_id")?;

    // Validate budget exists and belongs to user
    let budget_query = "SELECT id FROM budgets WHERE id = ?1 AND user_id = ?2";
    let budget_exists: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            budget_query,
            vec![
                Value::String(budget_id.clone()),
                Value::String(user_id.clone()),
            ],
        )
        .await?;

    if budget_exists.is_none() {
        return Err(FiscusError::Authorization(
            "Budget access denied".to_string(),
        ));
    }

    // Build update query dynamically with encrypted parameter mapping
    let mut update_fields = Vec::new();
    let mut params_with_mapping = Vec::new();
    let mut param_index = 1;

    if let Some(allocated_amount) = request.allocated_amount {
        Validator::validate_amount(allocated_amount, false)?;
        update_fields.push(format!("`allocated_amount` = ?{param_index}"));
        params_with_mapping.push((
            "allocated_amount".to_string(),
            Value::String(allocated_amount.to_string()),
        ));
        param_index += 1;
    }

    if let Some(notes) = &request.notes {
        update_fields.push(format!("`notes` = ?{param_index}"));
        params_with_mapping.push(("notes".to_string(), Value::String(notes.clone())));
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

    // Add budget_id for WHERE clause
    params_with_mapping.push(("id".to_string(), Value::String(budget_id.clone())));

    let update_query = format!(
        "UPDATE budgets SET {} WHERE id = ?{}",
        update_fields.join(", "),
        param_index
    );

    // Encrypt sensitive parameters before update
    let encrypted_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
        params_with_mapping,
        &user_id,
        "budgets",
    )
    .await?;

    let affected_rows =
        DatabaseUtils::execute_non_query(&db, &update_query, encrypted_params).await?;

    if affected_rows == 0 {
        return Err(FiscusError::NotFound("Budget not found".to_string()));
    }

    // Return updated budget
    get_budget_by_id(budget_id, db).await
}

/// Delete a budget
#[tauri::command]
pub async fn delete_budget(
    budget_id: String,
    user_id: String,
    db: State<'_, Database>,
) -> Result<bool, FiscusError> {
    // Validate input
    Validator::validate_uuid(&budget_id, "budget_id")?;
    Validator::validate_uuid(&user_id, "user_id")?;

    // Validate budget exists and belongs to user
    let budget_query = "SELECT id FROM budgets WHERE id = ?1 AND user_id = ?2";
    let budget_exists: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            budget_query,
            vec![Value::String(budget_id.clone()), Value::String(user_id)],
        )
        .await?;

    if budget_exists.is_none() {
        return Err(FiscusError::Authorization(
            "Budget access denied".to_string(),
        ));
    }

    let delete_query = "DELETE FROM budgets WHERE id = ?1";
    let params = vec![Value::String(budget_id)];

    let affected_rows = DatabaseUtils::execute_non_query(&db, delete_query, params).await?;
    Ok(affected_rows > 0)
}

/// Get budget summary for a user and period
#[tauri::command]
pub async fn get_budget_summary(
    user_id: String,
    budget_period_id: Option<String>,
    db: State<'_, Database>,
) -> Result<BudgetSummaryResponse, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    // For aggregation on encrypted fields, we need to fetch all budgets first and decrypt them
    let mut where_conditions = vec!["user_id = ?1".to_string()];
    let mut params = vec![Value::String(user_id.clone())];

    if let Some(period_id) = budget_period_id {
        Validator::validate_uuid(&period_id, "budget_period_id")?;
        where_conditions.push("budget_period_id = ?2".to_string());
        params.push(Value::String(period_id));
    }

    let budgets_query = format!(
        r#"
        SELECT id, user_id, budget_period_id, category_id, allocated_amount,
               spent_amount, notes, created_at, updated_at
        FROM budgets
        WHERE {}
    "#,
        where_conditions.join(" AND ")
    );

    // Use encrypted query to properly decrypt amount fields
    let budgets: Vec<HashMap<String, serde_json::Value>> =
        EncryptedDatabaseUtils::execute_encrypted_query(
            &db,
            &budgets_query,
            params,
            &user_id,
            "budgets",
        )
        .await?;

    // Calculate summary from decrypted budget data
    let mut total_allocated = rust_decimal::Decimal::ZERO;
    let mut total_spent = rust_decimal::Decimal::ZERO;
    let mut categories_over_budget = 0i32;
    let mut categories_under_budget = 0i32;

    for budget in budgets {
        let allocated_amount = parse_decimal_from_json(&budget, "allocated_amount");
        let spent_amount = parse_decimal_from_json(&budget, "spent_amount");

        total_allocated += allocated_amount;
        total_spent += spent_amount;

        if spent_amount > allocated_amount {
            categories_over_budget += 1;
        } else {
            categories_under_budget += 1;
        }
    }

    let remaining = total_allocated - total_spent;

    Ok(BudgetSummaryResponse {
        total_allocated,
        total_spent,
        remaining,
        categories_over_budget,
        categories_under_budget,
    })
}
