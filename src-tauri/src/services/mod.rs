/// Services module for Fiscus application
///
/// This module contains long-running services and background tasks
/// that provide additional functionality beyond the basic Tauri commands.
pub mod secure_storage_service;

pub use secure_storage_service::get_secure_storage_service;
