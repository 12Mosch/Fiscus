use serde_json::Value;
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

use crate::{
    database::{encrypted::EncryptedDatabaseUtils, Database, DatabaseUtils},
    dto::{CreateGoalRequest, GoalFilters, UpdateGoalRequest},
    error::{FiscusError, Validator},
    models::{Goal, GoalStatus},
    utils::parse_decimal_from_json,
};
use rust_decimal::prelude::ToPrimitive;

/// Create a new financial goal
#[tauri::command]
pub async fn create_goal(
    request: CreateGoalRequest,
    db: State<'_, Database>,
) -> Result<Goal, FiscusError> {
    // Validate input (user_id already validated by ValidatedUserId)
    Validator::validate_string(&request.name, "name", 1, 100)?;
    Validator::validate_amount(request.target_amount, false)?; // Goals must have positive targets

    if let Some(ref description) = request.description {
        Validator::validate_string(description, "description", 0, 500)?;
    }

    let target_date = if let Some(ref date_str) = request.target_date {
        Some(Validator::validate_date(date_str)?)
    } else {
        None
    };

    let priority = request.priority.unwrap_or(1).clamp(1, 5);

    // Validate user exists
    DatabaseUtils::validate_user_exists(&db, &request.user_id.as_str()).await?;

    let goal_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let insert_query = r#"
        INSERT INTO goals (
            id, user_id, name, description, target_amount, current_amount,
            target_date, priority, status, category, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
    "#;

    // Use encrypted parameter mapping for sensitive fields
    let params_with_mapping = vec![
        ("id".to_string(), Value::String(goal_id.clone())),
        (
            "user_id".to_string(),
            Value::String(request.user_id.as_str()),
        ),
        ("name".to_string(), Value::String(request.name.clone())),
        (
            "description".to_string(),
            request
                .description
                .as_ref()
                .map(|d| Value::String(d.clone()))
                .unwrap_or(Value::Null),
        ),
        (
            "target_amount".to_string(),
            Value::String(request.target_amount.to_string()),
        ),
        (
            "current_amount".to_string(),
            Value::String(rust_decimal::Decimal::ZERO.to_string()),
        ),
        (
            "target_date".to_string(),
            target_date
                .as_ref()
                .map(|d| Value::String(d.to_string()))
                .unwrap_or(Value::Null),
        ),
        (
            "priority".to_string(),
            Value::Number(serde_json::Number::from(priority as i64)),
        ),
        (
            "status".to_string(),
            Value::String(GoalStatus::Active.to_string()),
        ),
        (
            "category".to_string(),
            request
                .category
                .as_ref()
                .map(|c| Value::String(c.clone()))
                .unwrap_or(Value::Null),
        ),
        ("created_at".to_string(), Value::String(now.clone())),
        ("updated_at".to_string(), Value::String(now)),
    ];

    let encrypted_params = EncryptedDatabaseUtils::encrypt_params_with_mapping(
        params_with_mapping,
        &request.user_id.as_str(),
        "goals",
    )
    .await?;

    DatabaseUtils::execute_non_query(&db, insert_query, encrypted_params).await?;

    // Return the created goal
    get_goal_by_id(goal_id, db).await
}

/// Get goals with filtering
#[tauri::command]
pub async fn get_goals(
    filters: GoalFilters,
    db: State<'_, Database>,
) -> Result<Vec<Goal>, FiscusError> {
    // Validate user (already validated by ValidatedUserId)
    DatabaseUtils::validate_user_exists(&db, &filters.user_id.as_str()).await?;

    // Build query with filters
    let mut filter_map = HashMap::new();
    filter_map.insert("user_id".to_string(), filters.user_id.as_str().to_string());

    if let Some(status) = filters.status {
        filter_map.insert("status".to_string(), status.to_string());
    }

    if let Some(category) = filters.category {
        filter_map.insert("category".to_string(), category);
    }

    let base_query = r#"
        SELECT id, user_id, name, description, target_amount, current_amount,
               target_date, priority, status, category, created_at, updated_at
        FROM goals
    "#;

    let (where_clause, where_params) =
        DatabaseUtils::build_where_clause(&filter_map, &["user_id", "status", "category"], vec![])?;

    let order_clause = DatabaseUtils::build_order_clause(
        filters.sort_by.as_deref(),
        filters.sort_direction.as_deref(),
        &[
            "name",
            "target_amount",
            "current_amount",
            "target_date",
            "priority",
            "created_at",
            "updated_at",
        ],
        "priority",
    )?;

    let final_query = format!("{base_query} {where_clause} {order_clause}");

    // Use encrypted query to properly decrypt sensitive fields
    let goals: Vec<Goal> = EncryptedDatabaseUtils::execute_encrypted_query(
        &db,
        &final_query,
        where_params,
        &filters.user_id.as_str(),
        "goals",
    )
    .await?;

    Ok(goals)
}

/// Get a single goal by ID
#[tauri::command]
pub async fn get_goal_by_id(goal_id: String, db: State<'_, Database>) -> Result<Goal, FiscusError> {
    Validator::validate_uuid(&goal_id, "goal_id")?;

    // First, get the user_id for this goal (this field is not encrypted)
    let user_query = "SELECT user_id FROM goals WHERE id = ?1";
    let user_result: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(&db, user_query, vec![Value::String(goal_id.clone())])
            .await?;

    let user_id = user_result
        .and_then(|row| {
            row.get("user_id")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        })
        .ok_or_else(|| FiscusError::NotFound("Goal not found".to_string()))?;

    let query = r#"
        SELECT id, user_id, name, description, target_amount, current_amount,
               target_date, priority, status, category, created_at, updated_at
        FROM goals
        WHERE id = ?1
    "#;

    // Use encrypted query to properly decrypt sensitive fields
    let goals: Vec<Goal> = EncryptedDatabaseUtils::execute_encrypted_query(
        &db,
        query,
        vec![Value::String(goal_id.clone())],
        &user_id,
        "goals",
    )
    .await?;

    goals
        .into_iter()
        .next()
        .ok_or_else(|| FiscusError::NotFound("Goal not found".to_string()))
}

/// Update a goal
#[tauri::command]
pub async fn update_goal(
    goal_id: String,
    user_id: String,
    request: UpdateGoalRequest,
    db: State<'_, Database>,
) -> Result<Goal, FiscusError> {
    // Validate input
    Validator::validate_uuid(&goal_id, "goal_id")?;
    Validator::validate_uuid(&user_id, "user_id")?;

    // Validate goal exists and belongs to user
    let goal_query = "SELECT id FROM goals WHERE id = ?1 AND user_id = ?2";
    let goal_exists: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            goal_query,
            vec![
                Value::String(goal_id.clone()),
                Value::String(user_id.clone()),
            ],
        )
        .await?;

    if goal_exists.is_none() {
        return Err(FiscusError::Authorization("Goal access denied".to_string()));
    }

    // Build update query dynamically with encrypted parameter mapping
    let mut update_fields = Vec::new();
    let mut params_with_mapping = Vec::new();
    let mut param_index = 1;

    if let Some(name) = &request.name {
        Validator::validate_string(name, "name", 1, 100)?;
        update_fields.push(format!("\"name\" = ?{param_index}"));
        params_with_mapping.push(("name".to_string(), Value::String(name.clone())));
        param_index += 1;
    }

    if let Some(description) = &request.description {
        Validator::validate_string(description, "description", 0, 500)?;
        update_fields.push(format!("\"description\" = ?{param_index}"));
        params_with_mapping.push((
            "description".to_string(),
            Value::String(description.clone()),
        ));
        param_index += 1;
    }

    if let Some(target_amount) = request.target_amount {
        Validator::validate_amount(target_amount, false)?;
        update_fields.push(format!("\"target_amount\" = ?{param_index}"));
        params_with_mapping.push((
            "target_amount".to_string(),
            Value::String(target_amount.to_string()),
        ));
        param_index += 1;
    }

    if let Some(current_amount) = request.current_amount {
        Validator::validate_amount(current_amount, false)?;
        update_fields.push(format!("\"current_amount\" = ?{param_index}"));
        params_with_mapping.push((
            "current_amount".to_string(),
            Value::String(current_amount.to_string()),
        ));
        param_index += 1;
    }

    if let Some(target_date) = &request.target_date {
        let parsed_date = Validator::validate_date(target_date)?;
        update_fields.push(format!("\"target_date\" = ?{param_index}"));
        params_with_mapping.push((
            "target_date".to_string(),
            Value::String(parsed_date.to_string()),
        ));
        param_index += 1;
    }

    if let Some(priority) = request.priority {
        let validated_priority = priority.clamp(1, 5);
        update_fields.push(format!("\"priority\" = ?{param_index}"));
        params_with_mapping.push((
            "priority".to_string(),
            Value::Number(serde_json::Number::from(validated_priority as i64)),
        ));
        param_index += 1;
    }

    if let Some(status) = &request.status {
        update_fields.push(format!("\"status\" = ?{param_index}"));
        params_with_mapping.push(("status".to_string(), Value::String(status.to_string())));
        param_index += 1;
    }

    if let Some(category) = &request.category {
        update_fields.push(format!("\"category\" = ?{param_index}"));
        params_with_mapping.push(("category".to_string(), Value::String(category.clone())));
        param_index += 1;
    }

    if update_fields.is_empty() {
        return Err(FiscusError::InvalidInput("No fields to update".to_string()));
    }

    // Add updated_at timestamp
    update_fields.push(format!("\"updated_at\" = ?{param_index}"));
    params_with_mapping.push((
        "updated_at".to_string(),
        Value::String(chrono::Utc::now().to_rfc3339()),
    ));
    param_index += 1;

    // Add goal_id for WHERE clause
    params_with_mapping.push(("id".to_string(), Value::String(goal_id.clone())));

    let update_query = format!(
        "UPDATE goals SET {} WHERE id = ?{}",
        update_fields.join(", "),
        param_index
    );

    // Encrypt sensitive parameters before update
    let encrypted_params =
        EncryptedDatabaseUtils::encrypt_params_with_mapping(params_with_mapping, &user_id, "goals")
            .await?;

    let affected_rows =
        DatabaseUtils::execute_non_query(&db, &update_query, encrypted_params).await?;

    if affected_rows == 0 {
        return Err(FiscusError::NotFound("Goal not found".to_string()));
    }

    // Return updated goal
    get_goal_by_id(goal_id, db).await
}

/// Delete a goal
#[tauri::command]
pub async fn delete_goal(
    goal_id: String,
    user_id: String,
    db: State<'_, Database>,
) -> Result<bool, FiscusError> {
    // Validate input
    Validator::validate_uuid(&goal_id, "goal_id")?;
    Validator::validate_uuid(&user_id, "user_id")?;

    // Validate goal exists and belongs to user
    let goal_query = "SELECT id FROM goals WHERE id = ?1 AND user_id = ?2";
    let goal_exists: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            goal_query,
            vec![Value::String(goal_id.clone()), Value::String(user_id)],
        )
        .await?;

    if goal_exists.is_none() {
        return Err(FiscusError::Authorization("Goal access denied".to_string()));
    }

    let delete_query = "DELETE FROM goals WHERE id = ?1";
    let params = vec![Value::String(goal_id)];

    let affected_rows = DatabaseUtils::execute_non_query(&db, delete_query, params).await?;
    Ok(affected_rows > 0)
}

/// Update goal progress (add to current amount)
#[tauri::command]
pub async fn update_goal_progress(
    goal_id: String,
    user_id: String,
    amount: rust_decimal::Decimal,
    db: State<'_, Database>,
) -> Result<Goal, FiscusError> {
    // Validate input
    Validator::validate_uuid(&goal_id, "goal_id")?;
    Validator::validate_uuid(&user_id, "user_id")?;
    Validator::validate_amount(amount, false)?; // Progress additions must be positive

    // Get current goal to validate ownership and calculate new amount
    let current_goal = get_goal_by_id(goal_id.clone(), db.clone()).await?;

    if current_goal.user_id != user_id {
        return Err(FiscusError::Authorization("Goal access denied".to_string()));
    }

    let new_current_amount = current_goal.current_amount + amount;
    let mut new_status = current_goal.status;

    // Auto-complete goal if target is reached
    if new_current_amount >= current_goal.target_amount && new_status == GoalStatus::Active {
        new_status = GoalStatus::Completed;
    }

    let update_query =
        "UPDATE goals SET current_amount = ?1, status = ?2, updated_at = ?3 WHERE id = ?4";

    // Use encrypted parameter mapping for sensitive fields
    let params_with_mapping = vec![
        (
            "current_amount".to_string(),
            Value::String(new_current_amount.to_string()),
        ),
        ("status".to_string(), Value::String(new_status.to_string())),
        (
            "updated_at".to_string(),
            Value::String(chrono::Utc::now().to_rfc3339()),
        ),
        ("id".to_string(), Value::String(goal_id.clone())),
    ];

    let encrypted_params =
        EncryptedDatabaseUtils::encrypt_params_with_mapping(params_with_mapping, &user_id, "goals")
            .await?;

    let affected_rows =
        DatabaseUtils::execute_non_query(&db, update_query, encrypted_params).await?;

    if affected_rows == 0 {
        return Err(FiscusError::NotFound("Goal not found".to_string()));
    }

    // Return updated goal
    get_goal_by_id(goal_id, db).await
}

/// Get goal progress summary for a user
#[tauri::command]
pub async fn get_goal_progress_summary(
    user_id: String,
    db: State<'_, Database>,
) -> Result<HashMap<String, serde_json::Value>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    // For aggregation on encrypted fields, we need to fetch all goals first and decrypt them
    let goals_query = r#"
        SELECT id, user_id, name, description, target_amount, current_amount,
               target_date, priority, status, category, created_at, updated_at
        FROM goals
        WHERE user_id = ?1
    "#;

    // Use encrypted query to properly decrypt amount fields
    let goals: Vec<HashMap<String, serde_json::Value>> =
        EncryptedDatabaseUtils::execute_encrypted_query(
            &db,
            goals_query,
            vec![Value::String(user_id.clone())],
            &user_id,
            "goals",
        )
        .await?;

    // Calculate summary from decrypted goal data
    let total_goals = goals.len() as i64;
    let mut active_goals = 0i64;
    let mut completed_goals = 0i64;
    let mut paused_goals = 0i64;
    let mut total_target_amount = rust_decimal::Decimal::ZERO;
    let mut total_current_amount = rust_decimal::Decimal::ZERO;
    let mut progress_sum = 0.0;
    let mut goals_with_target = 0;

    for goal in goals {
        let status = goal.get("status").and_then(|v| v.as_str()).unwrap_or("");
        match status {
            "active" => active_goals += 1,
            "completed" => completed_goals += 1,
            "paused" => paused_goals += 1,
            _ => {}
        }

        let target_amount = parse_decimal_from_json(&goal, "target_amount");
        let current_amount = parse_decimal_from_json(&goal, "current_amount");

        total_target_amount += target_amount;
        total_current_amount += current_amount;

        if target_amount > rust_decimal::Decimal::ZERO {
            let progress_percentage = (current_amount / target_amount
                * rust_decimal::Decimal::from(100))
            .to_f64()
            .unwrap_or(0.0);
            progress_sum += progress_percentage;
            goals_with_target += 1;
        }
    }

    let average_progress_percentage = if goals_with_target > 0 {
        progress_sum / goals_with_target as f64
    } else {
        0.0
    };

    let mut summary = HashMap::new();
    summary.insert(
        "total_goals".to_string(),
        serde_json::Value::Number(serde_json::Number::from(total_goals)),
    );
    summary.insert(
        "active_goals".to_string(),
        serde_json::Value::Number(serde_json::Number::from(active_goals)),
    );
    summary.insert(
        "completed_goals".to_string(),
        serde_json::Value::Number(serde_json::Number::from(completed_goals)),
    );
    summary.insert(
        "paused_goals".to_string(),
        serde_json::Value::Number(serde_json::Number::from(paused_goals)),
    );
    summary.insert(
        "total_target_amount".to_string(),
        serde_json::Value::String(total_target_amount.to_string()),
    );
    summary.insert(
        "total_current_amount".to_string(),
        serde_json::Value::String(total_current_amount.to_string()),
    );
    summary.insert(
        "average_progress_percentage".to_string(),
        serde_json::Value::Number(
            serde_json::Number::from_f64(average_progress_percentage)
                .unwrap_or(serde_json::Number::from(0)),
        ),
    );

    Ok(summary)
}
