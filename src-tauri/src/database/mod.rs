// Database module for SQLite operations
// Handles backup history, deployment states, and audit logging

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use tracing::info;

pub mod queries;

/// Database connection manager
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Initialize database at the given path
    /// Creates tables if they don't exist
    pub fn new(db_path: PathBuf) -> Result<Self> {
        info!("Initializing database at {:?}", db_path);

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create database directory")?;
        }

        let conn = Connection::open(&db_path).context("Failed to open database connection")?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys")?;

        let db = Database { conn };
        db.init_schema()?;

        info!("Database initialized successfully");
        Ok(db)
    }

    /// Execute schema initialization SQL
    fn init_schema(&self) -> Result<()> {
        let schema_sql = include_str!("schema.sql");

        self.conn
            .execute_batch(schema_sql)
            .context("Failed to execute schema initialization")?;

        info!("Database schema initialized");
        Ok(())
    }

    /// Get database connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Run database migrations if needed
    pub fn migrate(&self) -> Result<()> {
        // Check current schema version
        let version: i32 = self
            .conn
            .query_row(
                "SELECT value FROM app_metadata WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        info!("Current schema version: {}", version);

        // Future: Add migration logic here when schema changes
        // match version {
        //     0 => self.migrate_to_v1()?,
        //     1 => self.migrate_to_v2()?,
        //     _ => {}
        // }

        Ok(())
    }

    /// Get the application data directory
    /// Defaults to ~/.frigate-config-tool/
    pub fn get_app_data_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to get home directory")?;

        let app_dir = home.join(".frigate-config-tool");

        std::fs::create_dir_all(&app_dir).context("Failed to create app data directory")?;

        Ok(app_dir)
    }

    /// Get the default database path
    pub fn get_default_db_path() -> Result<PathBuf> {
        let app_dir = Self::get_app_data_dir()?;
        Ok(app_dir.join("database.sqlite"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_initialization() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let db = Database::new(db_path).unwrap();

        // Verify tables exist
        let tables: Vec<String> = db
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"config_snapshots".to_string()));
        assert!(tables.contains(&"deployment_states".to_string()));
        assert!(tables.contains(&"audit_log".to_string()));
        assert!(tables.contains(&"app_metadata".to_string()));
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let db = Database::new(db_path).unwrap();

        let fk_enabled: i32 = db
            .conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();

        assert_eq!(fk_enabled, 1);
    }
}
