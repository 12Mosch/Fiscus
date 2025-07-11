use tauri_plugin_sql::{Migration, MigrationKind};

// Module declarations
mod commands;
mod database;
mod dto;
mod error;
mod models;

// Re-export for easier access
pub use database::*;
pub use dto::*;
pub use error::*;
pub use models::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Define database migrations for the personal finance application
    let migrations = vec![Migration {
        version: 1,
        description: "create_initial_tables",
        sql: include_str!("../migrations/001_initial_schema.sql"),
        kind: MigrationKind::Up,
    }];

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:fiscus.db", migrations)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            // Authentication commands
            commands::create_user,
            commands::login_user,
            commands::change_password,
            commands::get_current_user,
            // Account commands
            commands::create_account,
            commands::get_accounts,
            commands::get_account_by_id,
            commands::update_account,
            commands::delete_account,
            commands::get_account_summary,
            // Transaction commands
            commands::create_transaction,
            commands::get_transactions,
            commands::get_transaction_by_id,
            commands::update_transaction,
            commands::delete_transaction,
            commands::create_transfer,
            commands::get_transfer_by_id,
            commands::get_transaction_summary,
            // Category commands
            commands::create_category,
            commands::get_categories,
            commands::get_category_by_id,
            commands::update_category,
            commands::delete_category,
            commands::get_category_hierarchy,
            // Budget commands
            commands::create_budget_period,
            commands::get_budget_periods,
            commands::get_budget_period_by_id,
            commands::create_budget,
            commands::get_budgets,
            commands::get_budget_by_id,
            commands::update_budget,
            commands::delete_budget,
            commands::get_budget_summary,
            // Goal commands
            commands::create_goal,
            commands::get_goals,
            commands::get_goal_by_id,
            commands::update_goal,
            commands::delete_goal,
            commands::update_goal_progress,
            commands::get_goal_progress_summary,
            // Report commands
            commands::get_financial_overview,
            commands::get_spending_by_category,
            commands::get_monthly_spending_trend,
            commands::get_account_balance_history,
            commands::get_budget_performance,
            commands::get_net_worth_progression,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
