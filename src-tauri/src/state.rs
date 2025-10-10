// Application state management for Tauri
// Provides shared state across all command handlers

use crate::database::Database;
use anyhow::Result;
use std::sync::{Arc, Mutex};

/// Application state shared across Tauri commands
pub struct AppState {
    /// Database connection
    pub db: Arc<Mutex<Database>>,
}

impl AppState {
    /// Initialize application state
    pub fn new() -> Result<Self> {
        let db_path = Database::get_default_db_path()?;
        let db = Database::new(db_path)?;

        Ok(AppState {
            db: Arc::new(Mutex::new(db)),
        })
    }
}
