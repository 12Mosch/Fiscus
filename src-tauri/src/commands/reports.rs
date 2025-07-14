use serde_json::Value;
use std::collections::HashMap;
use tauri::State;

use crate::{
    database::{Database, DatabaseUtils},
    error::{FiscusError, Validator},
    utils::parse_decimal_from_json,
};

/// Get financial overview report for a user
#[tauri::command]
pub async fn get_financial_overview(
    user_id: String,
    start_date: Option<String>,
    end_date: Option<String>,
    db: State<'_, Database>,
) -> Result<HashMap<String, serde_json::Value>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    let mut date_conditions = Vec::new();
    let mut params = vec![Value::String(user_id.clone())];
    let mut param_index = 2;

    if let Some(start) = &start_date {
        Validator::validate_date(start)?;
        date_conditions.push(format!("DATE(t.transaction_date) >= ?{param_index}"));
        params.push(Value::String(start.clone()));
        param_index += 1;
    }

    if let Some(end) = &end_date {
        Validator::validate_date(end)?;
        date_conditions.push(format!("DATE(t.transaction_date) <= ?{param_index}"));
        params.push(Value::String(end.clone()));
    }

    let date_filter = if date_conditions.is_empty() {
        String::new()
    } else {
        format!("AND {}", date_conditions.join(" AND "))
    };

    let overview_query = format!(
        r#"
        SELECT 
            -- Account Summary
            (SELECT COALESCE(SUM(CASE WHEN at.is_asset = 1 THEN a.balance ELSE 0 END), 0) 
             FROM accounts a 
             JOIN account_types at ON a.account_type_id = at.id 
             WHERE a.user_id = ?1 AND a.is_active = 1) as total_assets,
            
            (SELECT COALESCE(SUM(CASE WHEN at.is_asset = 0 THEN ABS(a.balance) ELSE 0 END), 0) 
             FROM accounts a 
             JOIN account_types at ON a.account_type_id = at.id 
             WHERE a.user_id = ?1 AND a.is_active = 1) as total_liabilities,
            
            -- Transaction Summary
            COALESCE(SUM(CASE WHEN t.transaction_type = 'income' THEN t.amount ELSE 0 END), 0) as total_income,
            COALESCE(SUM(CASE WHEN t.transaction_type = 'expense' THEN t.amount ELSE 0 END), 0) as total_expenses,
            COUNT(CASE WHEN t.transaction_type != 'transfer' THEN 1 END) as transaction_count,
            
            -- Budget Summary
            (SELECT COALESCE(SUM(allocated_amount), 0) FROM budgets WHERE user_id = ?1) as total_budgeted,
            (SELECT COALESCE(SUM(spent_amount), 0) FROM budgets WHERE user_id = ?1) as total_spent_budget,
            
            -- Goal Summary
            (SELECT COUNT(*) FROM goals WHERE user_id = ?1 AND status = 'active') as active_goals,
            (SELECT COALESCE(SUM(target_amount), 0) FROM goals WHERE user_id = ?1 AND status = 'active') as total_goal_targets,
            (SELECT COALESCE(SUM(current_amount), 0) FROM goals WHERE user_id = ?1 AND status = 'active') as total_goal_progress
            
        FROM transactions t
        WHERE t.user_id = ?1 AND t.transaction_type != 'transfer' {date_filter}
    "#
    );

    let overview: Option<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query_single(&db, &overview_query, params).await?;

    let mut result = overview.unwrap_or_default();

    // Calculate derived values
    let total_assets = parse_decimal_from_json(&result, "total_assets");
    let total_liabilities = parse_decimal_from_json(&result, "total_liabilities");

    // Only calculate net worth if we have valid asset or liability data
    if result.contains_key("total_assets") || result.contains_key("total_liabilities") {
        let net_worth = total_assets - total_liabilities;
        result.insert(
            "net_worth".to_string(),
            serde_json::Value::String(net_worth.to_string()),
        );
    }

    let total_income = parse_decimal_from_json(&result, "total_income");
    let total_expenses = parse_decimal_from_json(&result, "total_expenses");

    // Only calculate net income if we have valid income or expense data
    if result.contains_key("total_income") || result.contains_key("total_expenses") {
        let net_income = total_income - total_expenses;
        result.insert(
            "net_income".to_string(),
            serde_json::Value::String(net_income.to_string()),
        );
    }

    Ok(result)
}

/// Get spending by category report
#[tauri::command]
pub async fn get_spending_by_category(
    user_id: String,
    start_date: Option<String>,
    end_date: Option<String>,
    limit: Option<i32>,
    db: State<'_, Database>,
) -> Result<Vec<HashMap<String, serde_json::Value>>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    let mut date_conditions = vec![
        "t.user_id = ?1".to_string(),
        "t.transaction_type = 'expense'".to_string(),
    ];
    let mut params = vec![Value::String(user_id)];
    let mut param_index = 2;

    if let Some(start) = &start_date {
        Validator::validate_date(start)?;
        date_conditions.push(format!("DATE(t.transaction_date) >= ?{param_index}"));
        params.push(Value::String(start.clone()));
        param_index += 1;
    }

    if let Some(end) = &end_date {
        Validator::validate_date(end)?;
        date_conditions.push(format!("DATE(t.transaction_date) <= ?{param_index}"));
        params.push(Value::String(end.clone()));
    }

    let limit_clause = if let Some(l) = limit {
        format!("LIMIT {}", l.clamp(1, 100))
    } else {
        "LIMIT 20".to_string()
    };

    let category_query = format!(
        r#"
        SELECT 
            COALESCE(c.name, 'Uncategorized') as category_name,
            COALESCE(c.color, '#808080') as category_color,
            COALESCE(SUM(t.amount), 0) as total_amount,
            COUNT(t.id) as transaction_count,
            COALESCE(AVG(t.amount), 0) as average_amount
        FROM transactions t
        LEFT JOIN categories c ON t.category_id = c.id
        WHERE {}
        GROUP BY c.id, c.name, c.color
        ORDER BY total_amount DESC
        {}
    "#,
        date_conditions.join(" AND "),
        limit_clause
    );

    let categories: Vec<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query(&db, &category_query, params).await?;

    Ok(categories)
}

/// Get monthly spending trend
#[tauri::command]
pub async fn get_monthly_spending_trend(
    user_id: String,
    months: Option<i32>,
    db: State<'_, Database>,
) -> Result<Vec<HashMap<String, serde_json::Value>>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    let months_back = months.unwrap_or(12).clamp(1, 24);

    let trend_query = r#"
        SELECT 
            strftime('%Y-%m', transaction_date) as month,
            COALESCE(SUM(CASE WHEN transaction_type = 'income' THEN amount ELSE 0 END), 0) as income,
            COALESCE(SUM(CASE WHEN transaction_type = 'expense' THEN amount ELSE 0 END), 0) as expenses,
            COUNT(CASE WHEN transaction_type != 'transfer' THEN 1 END) as transaction_count
        FROM transactions
        WHERE user_id = ?1 
        AND transaction_type != 'transfer'
        AND transaction_date >= date('now', '-' || ?2 || ' months')
        GROUP BY strftime('%Y-%m', transaction_date)
        ORDER BY month DESC
    "#;

    let params = vec![
        Value::String(user_id),
        Value::Number(serde_json::Number::from(months_back as i64)),
    ];

    let trend: Vec<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query(&db, trend_query, params).await?;

    Ok(trend)
}

/// Get account balance history
#[tauri::command]
pub async fn get_account_balance_history(
    user_id: String,
    account_id: Option<String>,
    days: Option<i32>,
    db: State<'_, Database>,
) -> Result<Vec<HashMap<String, serde_json::Value>>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    let mut conditions = vec!["t.user_id = ?1".to_string()];
    let mut params = vec![Value::String(user_id.clone())];
    let param_index = 2;

    if let Some(acc_id) = &account_id {
        Validator::validate_uuid(acc_id, "account_id")?;
        DatabaseUtils::validate_account_ownership(&db, acc_id, &user_id).await?;
        conditions.push(format!("t.account_id = ?{param_index}"));
        params.push(Value::String(acc_id.clone()));
    }

    let days_back = days.unwrap_or(30).clamp(1, 365);
    conditions.push(format!(
        "DATE(t.transaction_date) >= date('now', '-{days_back} days')"
    ));

    let history_query = format!(
        r#"
        SELECT 
            DATE(t.transaction_date) as date,
            t.account_id,
            a.name as account_name,
            SUM(CASE WHEN t.transaction_type = 'income' OR (t.transaction_type = 'transfer' AND t.amount > 0) 
                     THEN t.amount ELSE 0 END) as daily_inflow,
            SUM(CASE WHEN t.transaction_type = 'expense' OR (t.transaction_type = 'transfer' AND t.amount < 0) 
                     THEN ABS(t.amount) ELSE 0 END) as daily_outflow,
            COUNT(t.id) as transaction_count
        FROM transactions t
        JOIN accounts a ON t.account_id = a.id
        WHERE {}
        GROUP BY DATE(t.transaction_date), t.account_id, a.name
        ORDER BY date DESC, a.name
    "#,
        conditions.join(" AND ")
    );

    let history: Vec<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query(&db, &history_query, params).await?;

    Ok(history)
}

/// Get budget performance report
#[tauri::command]
pub async fn get_budget_performance(
    user_id: String,
    budget_period_id: Option<String>,
    db: State<'_, Database>,
) -> Result<Vec<HashMap<String, serde_json::Value>>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    let mut conditions = vec!["b.user_id = ?1".to_string()];
    let mut params = vec![Value::String(user_id)];

    if let Some(period_id) = &budget_period_id {
        Validator::validate_uuid(period_id, "budget_period_id")?;
        conditions.push("b.budget_period_id = ?2".to_string());
        params.push(Value::String(period_id.clone()));
    }

    let performance_query = format!(
        r#"
        SELECT 
            b.id as budget_id,
            c.name as category_name,
            c.color as category_color,
            bp.name as period_name,
            bp.start_date,
            bp.end_date,
            b.allocated_amount,
            b.spent_amount,
            (b.allocated_amount - b.spent_amount) as remaining_amount,
            CASE 
                WHEN b.allocated_amount > 0 THEN (b.spent_amount * 100.0 / b.allocated_amount)
                ELSE 0 
            END as percentage_used,
            CASE 
                WHEN b.spent_amount > b.allocated_amount THEN 1
                ELSE 0 
            END as is_over_budget
        FROM budgets b
        JOIN categories c ON b.category_id = c.id
        JOIN budget_periods bp ON b.budget_period_id = bp.id
        WHERE {}
        ORDER BY is_over_budget DESC, percentage_used DESC
    "#,
        conditions.join(" AND ")
    );

    let performance: Vec<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query(&db, &performance_query, params).await?;

    Ok(performance)
}

/// Get net worth progression over time
#[tauri::command]
pub async fn get_net_worth_progression(
    user_id: String,
    months: Option<i32>,
    db: State<'_, Database>,
) -> Result<Vec<HashMap<String, serde_json::Value>>, FiscusError> {
    // Validate user
    Validator::validate_uuid(&user_id, "user_id")?;
    DatabaseUtils::validate_user_exists(&db, &user_id).await?;

    let months_back = months.unwrap_or(12).clamp(1, 24);

    // This is a simplified version - in a real application, you'd want to track
    // historical balance snapshots for more accurate net worth progression
    let progression_query = r#"
        SELECT 
            strftime('%Y-%m', transaction_date) as month,
            SUM(CASE WHEN transaction_type = 'income' THEN amount 
                     WHEN transaction_type = 'expense' THEN -amount 
                     ELSE 0 END) as net_change,
            COUNT(CASE WHEN transaction_type != 'transfer' THEN 1 END) as transaction_count
        FROM transactions
        WHERE user_id = ?1 
        AND transaction_type != 'transfer'
        AND transaction_date >= date('now', '-' || ?2 || ' months')
        GROUP BY strftime('%Y-%m', transaction_date)
        ORDER BY month ASC
    "#;

    let params = vec![
        Value::String(user_id),
        Value::Number(serde_json::Number::from(months_back as i64)),
    ];

    let progression: Vec<HashMap<String, serde_json::Value>> =
        DatabaseUtils::execute_query(&db, progression_query, params).await?;

    Ok(progression)
}
