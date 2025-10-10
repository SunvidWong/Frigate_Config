// T120-T124: Deployment-related Tauri commands
// Provides frontend interface for deployment execution, health checking, and rollback
// REQUIREMENT: FR-037, FR-038, FR-039, FR-040, FR-041, FR-042

use crate::deployment::executor::{
    execute_deployment, get_deployment_status, stop_deployment, get_container_logs,
    DeploymentRequest,
};
use crate::deployment::health::{
    perform_comprehensive_health_check,
};
use crate::deployment::rollback::{
    execute_rollback, load_deployment_history, get_deployment_by_id,
    save_deployment_snapshot, DeploymentSnapshot, RollbackRequest,
};
use crate::models::deployment_state::{DeploymentMethod, DeploymentStatus};
use crate::error::AppError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{info, warn};

// ========== Deployment Commands (T120) ==========

/// Deploy Frigate with specified configuration
#[tauri::command]
pub async fn deploy_frigate(
    request: DeploymentRequestInput,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<DeploymentResponse, AppError> {
    info!("Deploying Frigate with config: {:?}", request.config_path);

    let deployment_request = DeploymentRequest {
        config_path: PathBuf::from(&request.config_path),
        method: match request.method.as_str() {
            "DockerCompose" => DeploymentMethod::DockerCompose,
            _ => DeploymentMethod::DockerRun,
        },
        devices: request.devices,
        volumes: request.volumes.into_iter().map(|v| (v.host_path, v.container_path)).collect(),
        ports: request.ports.into_iter().map(|p| (p.host_port, p.container_port)).collect(),
        environment: request.environment,
    };

    let result = execute_deployment(&deployment_request)
        .map_err(|e| AppError::Deployment(format!("Deployment failed: {}", e)))?;

    // Save deployment snapshot if successful
    if result.success {
        if let Some(ref container_id) = result.container_id {
            let snapshot = DeploymentSnapshot {
                id: uuid::Uuid::new_v4().to_string(),
                container_id: container_id.clone(),
                config_path: PathBuf::from(&request.config_path),
                deployment_time: SystemTime::now(),
                command: result.command.clone(),
                status: DeploymentStatus::Running,
            };

            if let Err(e) = save_deployment_snapshot(&snapshot) {
                warn!("Failed to save deployment snapshot: {}", e);
            }
        }
    }

    Ok(DeploymentResponse {
        success: result.success,
        container_id: result.container_id,
        command: result.command,
        stdout: result.stdout,
        stderr: result.stderr,
        exit_code: result.exit_code,
        deployment_time: chrono::Utc::now().to_rfc3339(),
    })
}

/// Get deployment status
#[tauri::command]
pub async fn get_deployment_status_cmd(
    container_id: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<DeploymentStatusResponse, AppError> {
    info!("Getting deployment status for container: {}", container_id);

    let status = get_deployment_status(&container_id)
        .map_err(|e| AppError::Deployment(format!("Failed to get status: {}", e)))?;

    Ok(DeploymentStatusResponse {
        container_id,
        status: format!("{:?}", status),
        message: match status {
            DeploymentStatus::Running => "Container is running".to_string(),
            DeploymentStatus::Pending => "Container is starting".to_string(),
            DeploymentStatus::Failed => "Container has failed".to_string(),
            DeploymentStatus::Completed => "Container has completed".to_string(),
            DeploymentStatus::RolledBack => "Container was rolled back".to_string(),
        },
    })
}

/// Stop deployment
#[tauri::command]
pub async fn stop_deployment_cmd(
    container_id: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<StopDeploymentResponse, AppError> {
    info!("Stopping deployment for container: {}", container_id);

    stop_deployment(&container_id)
        .map_err(|e| AppError::Deployment(format!("Failed to stop deployment: {}", e)))?;

    Ok(StopDeploymentResponse {
        success: true,
        container_id,
        message: "Deployment stopped successfully".to_string(),
        stopped_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Get deployment logs
#[tauri::command]
pub async fn get_deployment_logs(
    container_id: String,
    lines: Option<usize>,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<DeploymentLogsResponse, AppError> {
    info!("Getting logs for container: {}", container_id);

    let logs = get_container_logs(&container_id, lines)
        .map_err(|e| AppError::Deployment(format!("Failed to get logs: {}", e)))?;

    let lines_returned = logs.len();

    Ok(DeploymentLogsResponse {
        container_id,
        logs,
        lines_returned,
    })
}

// ========== Health Check Commands (T121) ==========

/// Check deployment health
#[tauri::command]
pub async fn check_deployment_health(
    container_id: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<HealthCheckResponse, AppError> {
    info!("Checking health for container: {}", container_id);

    let health_result = perform_comprehensive_health_check(&container_id)
        .map_err(|e| AppError::Deployment(format!("Health check failed: {}", e)))?;

    Ok(HealthCheckResponse {
        container_id,
        status: format!("{:?}", health_result.status),
        checks: health_result.checks.into_iter().map(|c| HealthCheckDetail {
            name: c.name,
            passed: c.passed,
            message: c.message,
            details: c.details,
        }).collect(),
        response_time_ms: health_result.response_time_ms,
        timestamp: chrono::DateTime::<chrono::Utc>::from(health_result.timestamp).to_rfc3339(),
    })
}

// ========== Rollback Commands (T122) ==========

/// Rollback deployment
#[tauri::command]
pub async fn rollback_deployment(
    request: RollbackRequestInput,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<RollbackResponse, AppError> {
    info!("Rolling back deployment: {}", request.reason);

    let rollback_request = RollbackRequest {
        reason: request.reason,
        target_snapshot_id: request.target_snapshot_id,
        preserve_data: request.preserve_data,
        create_backup: request.create_backup,
    };

    let result = execute_rollback(&rollback_request)
        .map_err(|e| AppError::Deployment(format!("Rollback failed: {}", e)))?;

    Ok(RollbackResponse {
        success: result.success,
        previous_container_id: result.previous_container_id,
        restored_container_id: result.restored_container_id,
        rollback_time: chrono::DateTime::<chrono::Utc>::from(result.rollback_time).to_rfc3339(),
        errors: result.errors,
    })
}

// ========== Deployment History Commands (T123) ==========

/// List deployment history
#[tauri::command]
pub async fn list_deployment_history(
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<DeploymentHistoryResponse, AppError> {
    info!("Listing deployment history");

    let history = load_deployment_history()
        .map_err(|e| AppError::Deployment(format!("Failed to load history: {}", e)))?;

    let deployments: Vec<DeploymentHistoryItem> = history.into_iter().map(|snapshot| DeploymentHistoryItem {
        id: snapshot.id,
        container_id: snapshot.container_id,
        config_path: snapshot.config_path.to_string_lossy().to_string(),
        deployment_time: chrono::DateTime::<chrono::Utc>::from(snapshot.deployment_time).to_rfc3339(),
        command: snapshot.command,
        status: format!("{:?}", snapshot.status),
    }).collect();

    let total_count = deployments.len();

    Ok(DeploymentHistoryResponse {
        deployments,
        total_count,
    })
}

/// Get deployment by ID
#[tauri::command]
pub async fn get_deployment_by_id_cmd(
    deployment_id: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Option<DeploymentHistoryItem>, AppError> {
    info!("Getting deployment: {}", deployment_id);

    let snapshot = get_deployment_by_id(&deployment_id)
        .map_err(|e| AppError::Deployment(format!("Failed to get deployment: {}", e)))?;

    Ok(snapshot.map(|s| DeploymentHistoryItem {
        id: s.id,
        container_id: s.container_id,
        config_path: s.config_path.to_string_lossy().to_string(),
        deployment_time: chrono::DateTime::<chrono::Utc>::from(s.deployment_time).to_rfc3339(),
        command: s.command,
        status: format!("{:?}", s.status),
    }))
}

// ========== Request/Response Types ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentRequestInput {
    pub config_path: String,
    pub method: String,
    pub devices: Vec<String>,
    pub volumes: Vec<VolumeMapping>,
    pub ports: Vec<PortMapping>,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeMapping {
    pub host_path: String,
    pub container_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub success: bool,
    pub container_id: Option<String>,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub deployment_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentStatusResponse {
    pub container_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopDeploymentResponse {
    pub success: bool,
    pub container_id: String,
    pub message: String,
    pub stopped_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentLogsResponse {
    pub container_id: String,
    pub logs: Vec<String>,
    pub lines_returned: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub container_id: String,
    pub status: String,
    pub checks: Vec<HealthCheckDetail>,
    pub response_time_ms: u64,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckDetail {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RollbackRequestInput {
    pub reason: String,
    pub target_snapshot_id: Option<String>,
    pub preserve_data: bool,
    pub create_backup: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RollbackResponse {
    pub success: bool,
    pub previous_container_id: Option<String>,
    pub restored_container_id: Option<String>,
    pub rollback_time: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentHistoryResponse {
    pub deployments: Vec<DeploymentHistoryItem>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentHistoryItem {
    pub id: String,
    pub container_id: String,
    pub config_path: String,
    pub deployment_time: String,
    pub command: String,
    pub status: String,
}
