use serde_json::Value;
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

use crate::{
    database::{Database, DatabaseUtils},
    dto::{CategoryFilters, CreateCategoryRequest, UpdateCategoryRequest},
    error::{FiscusError, FiscusResult, SecurityValidator, Validator},
    models::Category,
};

/// Create a new category
#[tauri::command]
pub async fn create_category(
    request: CreateCategoryRequest,
    db: State<'_, Database>,
) -> Result<Category, FiscusError> {
    // Validate input
    Validator::validate_uuid(&request.user_id, "user_id")?;
    Validator::validate_string(&request.name, "name", 1, 100)?;

    if let Some(ref description) = request.description {
        Validator::validate_string(description, "description", 0, 500)?;
    }

    if let Some(ref parent_id) = request.parent_category_id {
        Validator::validate_uuid(parent_id, "parent_category_id")?;
    }

    // Validate user exists
    DatabaseUtils::validate_user_exists(&db, &request.user_id).await?;

    // Validate parent category exists and belongs to user (if provided)
    if let Some(ref parent_id) = request.parent_category_id {
        DatabaseUtils::validate_category_ownership(&db, parent_id, &request.user_id).await?;
    }

    // Check if category name already exists for this user
    let existing_query =
        "SELECT id FROM categories WHERE user_id = ?1 AND name = ?2 AND is_active = 1";
    let existing: Option<HashMap<String, serde_json::Value>> = DatabaseUtils::execute_query_single(
        &db,
        existing_query,
        vec![
            Value::String(request.user_id.clone()),
            Value::String(request.name.clone()),
        ],
    )
    .await?;

    if existing.is_some() {
        return Err(FiscusError::Conflict(
            "Category name already exists".to_string(),
        ));
    }

    let category_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let insert_query = r#"
        INSERT INTO categories (
            id, user_id, name, description, color, icon, parent_category_id, 
            is_income, is_active, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
    "#;

    let params = vec![
        Value::String(category_id.clone()),
        Value::String(request.user_id.clone()),
        Value::String(request.name.clone()),
        request
            .description
            .as_ref()
            .map(|d| Value::String(d.clone()))
            .unwrap_or(Value::Null),
        request
            .color
            .as_ref()
            .map(|c| Value::String(c.clone()))
            .unwrap_or(Value::Null),
        request
            .icon
            .as_ref()
            .map(|i| Value::String(i.clone()))
            .unwrap_or(Value::Null),
        request
            .parent_category_id
            .as_ref()
            .map(|p| Value::String(p.clone()))
            .unwrap_or(Value::Null),
        Value::Bool(request.is_income),
        Value::Bool(true),
        Value::String(now.clone()),
        Value::String(now),
    ];

    DatabaseUtils::execute_non_query(&db, insert_query, params).await?;

    // Return the created category
    get_category_by_id(category_id, db).await
}

/// Get all categories for a user with optional filtering
#[tauri::command]
pub async fn get_categories(
    filters: CategoryFilters,
    db: State<'_, Database>,
) -> Result<Vec<Category>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&filters.user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &filters.user_id).await?;

    // Build query with filters
    let mut filter_map = HashMap::new();
    filter_map.insert("user_id".to_string(), filters.user_id);

    if let Some(parent_id) = filters.parent_category_id {
        if parent_id.is_empty() {
            // Filter for root categories (no parent)
            filter_map.insert("parent_category_id".to_string(), "NULL".to_string());
        } else {
            Validator::validate_uuid(&parent_id, "parent_category_id")?;
            filter_map.insert("parent_category_id".to_string(), parent_id);
        }
    }

    if let Some(is_income) = filters.is_income {
        filter_map.insert("is_income".to_string(), is_income.to_string());
    }

    if let Some(is_active) = filters.is_active {
        filter_map.insert("is_active".to_string(), is_active.to_string());
    }

    let base_query = r#"
        SELECT id, user_id, name, description, color, icon, parent_category_id,
               is_income, is_active, created_at, updated_at
        FROM categories
    "#;

    let (where_clause, where_params) = DatabaseUtils::build_where_clause(
        &filter_map,
        &["user_id", "parent_category_id", "is_income", "is_active"],
        vec![],
    )?;

    let order_clause = DatabaseUtils::build_order_clause(
        filters.sort_by.as_deref(),
        filters.sort_direction.as_deref(),
        SecurityValidator::CATEGORY_SORT_FIELDS,
        "name",
    )?;

    let final_query = format!("{base_query} {where_clause} {order_clause}");

    let categories: Vec<Category> =
        DatabaseUtils::execute_query(&db, &final_query, where_params).await?;

    Ok(categories)
}

/// Get a single category by ID
#[tauri::command]
pub async fn get_category_by_id(
    category_id: String,
    db: State<'_, Database>,
) -> Result<Category, FiscusError> {
    Validator::validate_uuid(&category_id, "category_id")?;

    let query = r#"
        SELECT id, user_id, name, description, color, icon, parent_category_id,
               is_income, is_active, created_at, updated_at
        FROM categories 
        WHERE id = ?1
    "#;

    let category: Option<Category> =
        DatabaseUtils::execute_query_single(&db, query, vec![Value::String(category_id.clone())])
            .await?;

    category.ok_or_else(|| FiscusError::NotFound("Category not found".to_string()))
}

/// Update a category
#[tauri::command]
pub async fn update_category(
    category_id: String,
    user_id: String,
    request: UpdateCategoryRequest,
    db: State<'_, Database>,
) -> Result<Category, FiscusError> {
    // Validate input
    Validator::validate_uuid(&category_id, "category_id")?;
    Validator::validate_uuid(&user_id, "user_id")?;

    // Validate ownership
    DatabaseUtils::validate_category_ownership(&db, &category_id, &user_id).await?;

    // Build update query dynamically
    let mut update_fields = Vec::new();
    let mut params = Vec::new();
    let mut param_index = 1;

    if let Some(name) = &request.name {
        Validator::validate_string(name, "name", 1, 100)?;

        // Check if new name conflicts with existing categories
        let existing_query = "SELECT id FROM categories WHERE user_id = ?1 AND name = ?2 AND id != ?3 AND is_active = 1";
        let existing: Option<HashMap<String, serde_json::Value>> =
            DatabaseUtils::execute_query_single(
                &db,
                existing_query,
                vec![
                    Value::String(user_id.clone()),
                    Value::String(name.clone()),
                    Value::String(category_id.clone()),
                ],
            )
            .await?;

        if existing.is_some() {
            return Err(FiscusError::Conflict(
                "Category name already exists".to_string(),
            ));
        }

        update_fields.push(format!("`name` = ?{param_index}"));
        params.push(Value::String(name.clone()));
        param_index += 1;
    }

    if let Some(description) = &request.description {
        Validator::validate_string(description, "description", 0, 500)?;
        update_fields.push(format!("`description` = ?{param_index}"));
        params.push(Value::String(description.clone()));
        param_index += 1;
    }

    if let Some(color) = &request.color {
        update_fields.push(format!("`color` = ?{param_index}"));
        params.push(Value::String(color.clone()));
        param_index += 1;
    }

    if let Some(icon) = &request.icon {
        update_fields.push(format!("`icon` = ?{param_index}"));
        params.push(Value::String(icon.clone()));
        param_index += 1;
    }

    if let Some(parent_id) = &request.parent_category_id {
        if !parent_id.is_empty() {
            Validator::validate_uuid(parent_id, "parent_category_id")?;
            DatabaseUtils::validate_category_ownership(&db, parent_id, &user_id).await?;

            // Prevent circular reference
            if parent_id == &category_id {
                return Err(FiscusError::InvalidInput(
                    "Category cannot be its own parent".to_string(),
                ));
            }

            // Check if this would create a circular reference through the hierarchy
            if is_circular_reference(&db, &category_id, parent_id).await? {
                return Err(FiscusError::InvalidInput(
                    "This would create a circular reference".to_string(),
                ));
            }
        }

        update_fields.push(format!("`parent_category_id` = ?{param_index}"));
        if parent_id.is_empty() {
            params.push(Value::Null);
        } else {
            params.push(Value::String(parent_id.clone()));
        }
        param_index += 1;
    }

    if let Some(is_active) = request.is_active {
        update_fields.push(format!("`is_active` = ?{param_index}"));
        params.push(Value::Bool(is_active));
        param_index += 1;
    }

    if update_fields.is_empty() {
        return Err(FiscusError::InvalidInput("No fields to update".to_string()));
    }

    // Add updated_at timestamp
    update_fields.push(format!("`updated_at` = ?{param_index}"));
    params.push(Value::String(chrono::Utc::now().to_rfc3339()));
    param_index += 1;

    // Add category_id for WHERE clause
    params.push(Value::String(category_id.clone()));

    let update_query = format!(
        "UPDATE categories SET {} WHERE id = ?{}",
        update_fields.join(", "),
        param_index
    );

    let affected_rows = DatabaseUtils::execute_non_query(&db, &update_query, params).await?;

    if affected_rows == 0 {
        return Err(FiscusError::NotFound("Category not found".to_string()));
    }

    // Return updated category
    get_category_by_id(category_id, db).await
}

/// Delete a category (soft delete by setting is_active to false)
#[tauri::command]
pub async fn delete_category(
    category_id: String,
    user_id: String,
    db: State<'_, Database>,
) -> Result<bool, FiscusError> {
    // Validate input
    Validator::validate_uuid(&category_id, "category_id")?;
    Validator::validate_uuid(&user_id, "user_id")?;

    // Validate ownership
    DatabaseUtils::validate_category_ownership(&db, &category_id, &user_id).await?;

    // Check if category has transactions
    let transaction_count_query =
        "SELECT COUNT(*) as count FROM transactions WHERE category_id = ?1";
    let count_result: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            transaction_count_query,
            vec![Value::String(category_id.clone())],
        )
        .await?;

    let transaction_count = count_result
        .and_then(|row| row.get("count").cloned())
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // Check if category has subcategories
    let subcategory_count_query =
        "SELECT COUNT(*) as count FROM categories WHERE parent_category_id = ?1 AND is_active = 1";
    let subcat_result: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(
            &db,
            subcategory_count_query,
            vec![Value::String(category_id.clone())],
        )
        .await?;

    let subcategory_count = subcat_result
        .and_then(|row| row.get("count").cloned())
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    if transaction_count > 0 || subcategory_count > 0 {
        // Soft delete - set is_active to false
        let update_query = "UPDATE categories SET is_active = ?1, updated_at = ?2 WHERE id = ?3";
        let params = vec![
            Value::Bool(false),
            Value::String(chrono::Utc::now().to_rfc3339()),
            Value::String(category_id),
        ];

        let affected_rows = DatabaseUtils::execute_non_query(&db, update_query, params).await?;
        Ok(affected_rows > 0)
    } else {
        // Hard delete if no transactions or subcategories
        let delete_query = "DELETE FROM categories WHERE id = ?1";
        let params = vec![Value::String(category_id)];

        let affected_rows = DatabaseUtils::execute_non_query(&db, delete_query, params).await?;
        Ok(affected_rows > 0)
    }
}

/// Get category hierarchy (tree structure)
#[tauri::command]
pub async fn get_category_hierarchy(
    user_id: String,
    is_income: Option<bool>,
    db: State<'_, Database>,
) -> Result<Vec<Category>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    let mut base_query = r#"
        SELECT id, user_id, name, description, color, icon, parent_category_id,
               is_income, is_active, created_at, updated_at
        FROM categories
        WHERE user_id = ?1 AND is_active = 1
    "#
    .to_string();

    let mut params = vec![Value::String(user_id)];

    if let Some(income_filter) = is_income {
        base_query.push_str(" AND is_income = ?2");
        params.push(Value::Bool(income_filter));
    }

    base_query.push_str(" ORDER BY parent_category_id NULLS FIRST, name");

    let categories: Vec<Category> = DatabaseUtils::execute_query(&db, &base_query, params).await?;

    Ok(categories)
}

/// Helper function to check for circular references in category hierarchy
async fn is_circular_reference(
    db: &Database,
    category_id: &str,
    proposed_parent_id: &str,
) -> FiscusResult<bool> {
    let mut current_parent = Some(proposed_parent_id.to_string());
    let mut visited = std::collections::HashSet::new();

    while let Some(parent_id) = current_parent {
        if parent_id == category_id {
            return Ok(true); // Circular reference detected
        }

        if visited.contains(&parent_id) {
            return Ok(true); // Circular reference detected
        }

        visited.insert(parent_id.clone());

        // Get the parent of the current parent
        let query = "SELECT parent_category_id FROM categories WHERE id = ?1";
        let result: Option<HashMap<String, serde_json::Value>> =
            DatabaseUtils::execute_query_single(db, query, vec![Value::String(parent_id)]).await?;

        current_parent = result
            .and_then(|row| row.get("parent_category_id").cloned())
            .and_then(|v| v.as_str().map(|s| s.to_string()));
    }

    Ok(false)
}
