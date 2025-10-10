// Error handling for Tauri commands
// Provides a unified error type that can be serialized to the frontend

use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Application error type that can be sent to the frontend
#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Agent execution error: {0}")]
    Agent(String),

    #[error("Deployment error: {0}")]
    Deployment(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Disk error: {0}")]
    Disk(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convert anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

/// Convert std::io::Error to AppError
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

/// Convert rusqlite::Error to AppError
impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

/// Result type alias for Tauri commands
pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_serialization() {
        let err = AppError::Validation("Invalid input".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("Validation"));
        assert!(json.contains("Invalid input"));
    }
}
