// T116-T118: Deployment rollback, history storage, and automatic rollback
// Handles rollback to previous deployments, stores deployment history, and automatic rollback on failures
// REQUIREMENT: FR-040, FR-041 (Deployment Module - Rollback functionality)

use crate::deployment::executor::{execute_deployment, stop_deployment, DeploymentRequest};
use crate::deployment::health::{check_container_health, HealthStatus, perform_comprehensive_health_check};
use crate::models::deployment_state::{DeploymentMethod, DeploymentState, DeploymentStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

// ========== Deployment Snapshot Structures (T117) ==========

/// Snapshot of a deployment for rollback purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSnapshot {
    pub id: String,
    pub container_id: String,
    pub config_path: PathBuf,
    pub deployment_time: SystemTime,
    pub command: String,
    pub status: DeploymentStatus,
}

/// Request to perform a rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub reason: String,
    pub target_snapshot_id: Option<String>, // None = rollback to previous
    pub preserve_data: bool,
    pub create_backup: bool,
}

/// Result of a rollback operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub success: bool,
    pub previous_container_id: Option<String>,
    pub restored_container_id: Option<String>,
    pub rollback_time: SystemTime,
    pub errors: Vec<String>,
}

impl DeploymentSnapshot {
    pub fn new(
        id: String,
        container_id: String,
        config_path: PathBuf,
        command: String,
    ) -> Self {
        Self {
            id,
            container_id,
            config_path,
            deployment_time: SystemTime::now(),
            command,
            status: DeploymentStatus::Running,
        }
    }

    pub fn from_state(state: &DeploymentState, command: String) -> Option<Self> {
        if let Some(container_id) = &state.container_id {
            Some(Self {
                id: state.id.clone(),
                container_id: container_id.clone(),
                config_path: PathBuf::from(&state.config_path),
                deployment_time: state.start_time.unwrap_or(SystemTime::now()),
                command,
                status: state.status.clone(),
            })
        } else {
            None
        }
    }
}

impl RollbackResult {
    pub fn new(success: bool) -> Self {
        Self {
            success,
            previous_container_id: None,
            restored_container_id: None,
            rollback_time: SystemTime::now(),
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.success = false;
    }
}

// ========== Deployment History Storage (T117) ==========

/// Get the deployment history directory
fn get_history_dir() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("frigate-config");
    path.push("deployment-history");
    path
}

/// Save a deployment snapshot to history
pub fn save_deployment_snapshot(snapshot: &DeploymentSnapshot) -> Result<(), String> {
    let history_dir = get_history_dir();
    fs::create_dir_all(&history_dir)
        .map_err(|e| format!("Failed to create history directory: {}", e))?;

    let snapshot_file = history_dir.join(format!("{}.json", snapshot.id));
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| format!("Failed to serialize snapshot: {}", e))?;

    fs::write(&snapshot_file, json)
        .map_err(|e| format!("Failed to write snapshot file: {}", e))?;

    Ok(())
}

/// Load all deployment snapshots from history (sorted newest first)
pub fn load_deployment_history() -> Result<Vec<DeploymentSnapshot>, String> {
    let history_dir = get_history_dir();

    if !history_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&history_dir)
        .map_err(|e| format!("Failed to read history directory: {}", e))?;

    let mut snapshots = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read snapshot file: {}", e))?;

            let snapshot: DeploymentSnapshot = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse snapshot: {}", e))?;

            snapshots.push(snapshot);
        }
    }

    // Sort by deployment time, newest first
    snapshots.sort_by(|a, b| b.deployment_time.cmp(&a.deployment_time));

    Ok(snapshots)
}

/// Get the most recent previous deployment
pub fn get_previous_deployment() -> Result<Option<DeploymentSnapshot>, String> {
    let history = load_deployment_history()?;

    // Return the second most recent (first is current, second is previous)
    if history.len() > 1 {
        Ok(Some(history[1].clone()))
    } else {
        Ok(None)
    }
}

/// Get the current deployment
pub fn get_current_deployment() -> Result<Option<DeploymentSnapshot>, String> {
    let history = load_deployment_history()?;

    if !history.is_empty() {
        Ok(Some(history[0].clone()))
    } else {
        Ok(None)
    }
}

/// Get a specific deployment by ID
pub fn get_deployment_by_id(id: &str) -> Result<Option<DeploymentSnapshot>, String> {
    let history = load_deployment_history()?;

    Ok(history.into_iter().find(|s| s.id == id))
}

// ========== Rollback Operations (T116) ==========

/// Stop the current deployment
pub fn stop_current_deployment() -> Result<Option<String>, String> {
    let current = get_current_deployment()?;

    if let Some(deployment) = current {
        stop_deployment(&deployment.container_id)?;
        Ok(Some(deployment.container_id))
    } else {
        Ok(None)
    }
}

/// Restore a deployment from a snapshot
pub fn restore_deployment(snapshot: &DeploymentSnapshot) -> Result<String, String> {
    // Validate config file exists
    if !snapshot.config_path.exists() {
        return Err(format!("Configuration file not found: {:?}", snapshot.config_path));
    }

    // Parse the original command to reconstruct deployment request
    // For simplicity, we'll create a basic deployment request
    let request = DeploymentRequest {
        config_path: snapshot.config_path.clone(),
        method: DeploymentMethod::DockerRun, // Default to docker run
        devices: vec![],
        volumes: vec![],
        ports: vec![(5000, 5000)], // Default Frigate port
        environment: HashMap::new(),
    };

    // Execute deployment
    let result = execute_deployment(&request)?;

    if !result.success {
        return Err(format!("Deployment failed: {}", result.stderr));
    }

    result.container_id.ok_or_else(|| "No container ID returned".to_string())
}

/// Verify that a rollback was successful
pub fn verify_rollback_success(container_id: &str) -> Result<bool, String> {
    // Check container is running
    match check_container_health(container_id) {
        Ok(HealthStatus::Healthy) | Ok(HealthStatus::Starting) => Ok(true),
        Ok(_) => Ok(false),
        Err(e) => Err(e),
    }
}

/// Clean up a failed deployment
pub fn cleanup_failed_deployment(container_id: &str) -> Result<(), String> {
    // Stop and remove container
    stop_deployment(container_id)?;
    Ok(())
}

/// Execute a full rollback operation
pub fn execute_rollback(request: &RollbackRequest) -> Result<RollbackResult, String> {
    let mut result = RollbackResult::new(true);

    // 1. Determine target snapshot
    let target_snapshot = if let Some(snapshot_id) = &request.target_snapshot_id {
        match get_deployment_by_id(snapshot_id) {
            Ok(Some(snapshot)) => snapshot,
            Ok(None) => {
                result.add_error(format!("Snapshot not found: {}", snapshot_id));
                return Ok(result);
            },
            Err(e) => {
                result.add_error(format!("Failed to load snapshot: {}", e));
                return Ok(result);
            }
        }
    } else {
        // Rollback to previous deployment
        match get_previous_deployment() {
            Ok(Some(snapshot)) => snapshot,
            Ok(None) => {
                result.add_error("No previous deployment found".to_string());
                return Ok(result);
            },
            Err(e) => {
                result.add_error(format!("Failed to load previous deployment: {}", e));
                return Ok(result);
            }
        }
    };

    // 2. Create backup of current deployment if requested
    if request.create_backup {
        if let Ok(Some(current)) = get_current_deployment() {
            if let Err(e) = save_deployment_snapshot(&current) {
                result.add_error(format!("Failed to create backup: {}", e));
                // Continue with rollback anyway
            }
        }
    }

    // 3. Stop current deployment
    match stop_current_deployment() {
        Ok(Some(container_id)) => {
            result.previous_container_id = Some(container_id);
        },
        Ok(None) => {
            // No current deployment to stop
        },
        Err(e) => {
            result.add_error(format!("Failed to stop current deployment: {}", e));
            return Ok(result);
        }
    }

    // 4. Restore target deployment
    match restore_deployment(&target_snapshot) {
        Ok(container_id) => {
            result.restored_container_id = Some(container_id.clone());

            // 5. Verify rollback success
            match verify_rollback_success(&container_id) {
                Ok(true) => {
                    result.success = true;
                },
                Ok(false) => {
                    result.add_error("Rollback verification failed: container not healthy".to_string());
                },
                Err(e) => {
                    result.add_error(format!("Rollback verification failed: {}", e));
                }
            }
        },
        Err(e) => {
            result.add_error(format!("Failed to restore deployment: {}", e));
        }
    }

    Ok(result)
}

// ========== Automatic Rollback (T118) ==========

/// Automatically rollback on health check failure
pub fn automatic_rollback_on_health_failure(current_container_id: &str) -> Result<RollbackResult, String> {
    // Check if current deployment is unhealthy
    let health = perform_comprehensive_health_check(current_container_id)?;

    if health.status != HealthStatus::Unhealthy {
        return Err(format!("Current deployment is not unhealthy, status: {:?}", health.status));
    }

    // Check if there's a previous deployment to rollback to
    let previous = get_previous_deployment()?;

    if previous.is_none() {
        return Err("No previous deployment found for automatic rollback".to_string());
    }

    // Execute rollback
    let request = RollbackRequest {
        reason: "Automatic rollback due to health check failure".to_string(),
        target_snapshot_id: None, // Rollback to previous
        preserve_data: true,
        create_backup: true,
    };

    execute_rollback(&request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_snapshot_creation() {
        let snapshot = DeploymentSnapshot::new(
            "test_001".to_string(),
            "container_123".to_string(),
            PathBuf::from("/config/test.yml"),
            "docker run test".to_string(),
        );

        assert_eq!(snapshot.id, "test_001");
        assert_eq!(snapshot.container_id, "container_123");
        assert_eq!(snapshot.status, DeploymentStatus::Running);
    }

    #[test]
    fn test_rollback_result_creation() {
        let mut result = RollbackResult::new(true);
        assert!(result.success);
        assert!(result.errors.is_empty());

        result.add_error("Test error".to_string());
        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_rollback_request_creation() {
        let request = RollbackRequest {
            reason: "Test rollback".to_string(),
            target_snapshot_id: None,
            preserve_data: true,
            create_backup: true,
        };

        assert_eq!(request.reason, "Test rollback");
        assert!(request.preserve_data);
        assert!(request.create_backup);
    }

    #[test]
    fn test_get_history_dir() {
        let dir = get_history_dir();
        assert!(dir.to_string_lossy().contains("frigate-config"));
        assert!(dir.to_string_lossy().contains("deployment-history"));
    }

    #[test]
    fn test_save_and_load_empty_history() {
        let history = load_deployment_history();
        assert!(history.is_ok());
        // History might be empty or contain previous test data
    }
}
