use tauri_plugin_sql::{Migration, MigrationKind};

// Module declarations
mod commands;
mod database;
mod dto;
pub mod encryption;
pub mod error;
mod logging;
mod models;
pub mod security;
mod services;
pub mod utils;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod test_database;

// Re-export for easier access
pub use database::{
    config as database_config, connection, encrypted, secure_storage_repository, sqlite, Database,
    DatabaseConfig, DatabaseConnection, DatabaseType, DatabaseUtils, PoolStats, SQLiteManager,
    SQLiteStats,
};
pub use dto::*;
pub use error::*;
pub use logging::{
    config as logging_config, create_db_logger, create_middleware, create_sanitizer, init,
    middleware, performance, sanitizer, DataSanitizer, DatabaseLogger, Environment, ExtractUserId,
    LogFormat, LoggingConfig, LoggingMiddleware, PerformanceMonitor, PerformanceSummary,
    RequestContext, Sanitizable,
};
pub use models::*;
pub use utils::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging system first
    if let Err(e) = logging::init() {
        eprintln!("Failed to initialize logging: {e}");
        // Continue without logging rather than crash
    }

    tracing::info!("Starting Fiscus application");

    // Initialize encryption service
    if let Err(e) = commands::encryption::initialize_encryption_service() {
        tracing::error!("Failed to initialize encryption service: {e}");
        // Encryption is critical for security - fail fast
        panic!("Failed to initialize encryption service: {e}");
    } else {
        tracing::info!("Encryption service initialized successfully");
    }

    // Define database migrations for the personal finance application
    let migrations = vec![
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("../migrations/001_initial_schema.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_secure_storage",
            sql: include_str!("../migrations/002_secure_storage.sql"),
            kind: MigrationKind::Up,
        },
    ];

    tracing::info!(
        "Configuring Tauri application with {} migrations",
        migrations.len()
    );

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
            // Encryption commands
            commands::encrypt_financial_data,
            commands::decrypt_financial_data,
            commands::generate_encryption_key,
            commands::rotate_user_keys,
            commands::get_encryption_stats,
            commands::derive_key_from_password,
            // Secure storage commands
            commands::secure_store,
            commands::secure_retrieve,
            commands::secure_delete,
            commands::secure_cleanup_expired,
            commands::secure_get_statistics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
