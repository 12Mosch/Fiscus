/// Tauri commands organized by domain
/// This module provides a clean separation of concerns for different
/// areas of the personal finance application
pub mod accounts;
pub mod auth;
pub mod budgets;
pub mod categories;
pub mod encryption;
pub mod goals;
pub mod reports;
pub mod secure_storage;
pub mod transactions;

// Re-export all command functions for easy registration
pub use accounts::*;
pub use auth::*;
pub use budgets::*;
pub use categories::*;
pub use encryption::*;
pub use goals::*;
pub use reports::*;
pub use secure_storage::*;
pub use transactions::*;
