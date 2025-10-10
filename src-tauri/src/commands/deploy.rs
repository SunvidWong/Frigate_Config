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

    // Load saved hardware devices and merge with request devices
    let saved_devices = load_hardware_devices().await.unwrap_or_else(|e| {
        warn!("Failed to load hardware devices: {}", e);
        Vec::new()
    });

    let mut all_devices = request.devices.clone();
    let mut device_warnings = Vec::new();

    for hw_device in saved_devices {
        if !all_devices.contains(&hw_device.device_path) {
            info!("Adding hardware device from config: {} ({})", hw_device.device_name, hw_device.device_path);

            // Validate device existence
            if !validate_device_path(&hw_device.device_path) {
                let warning = format!(
                    "⚠️ 设备 {} ({}) 不存在,但仍会添加到配置中。请确保设备在部署时可用。",
                    hw_device.device_name,
                    hw_device.device_path
                );
                warn!("{}", warning);
                device_warnings.push(warning);
            }

            all_devices.push(hw_device.device_path);
        }
    }

    let deployment_request = DeploymentRequest {
        config_path: PathBuf::from(&request.config_path),
        method: match request.method.as_str() {
            "DockerCompose" => DeploymentMethod::DockerCompose,
            _ => DeploymentMethod::DockerRun,
        },
        devices: all_devices,
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
        warnings: device_warnings,
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
    pub warnings: Vec<String>,
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

// ========== Hardware Device Commands ==========

/// Add hardware device to deployment configuration
#[tauri::command]
pub async fn add_hardware_device_to_config(
    device_path: String,
    device_type: String,
    device_name: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<AddHardwareDeviceResponse, AppError> {
    info!("Adding hardware device to config: {} ({})", device_name, device_path);

    // Validate device path
    if device_path.is_empty() {
        return Err(AppError::Deployment("Device path cannot be empty".to_string()));
    }

    // Get or create hardware config file path
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Deployment("Failed to get config directory".to_string()))?
        .join("frigate-config-tool");

    tokio::fs::create_dir_all(&config_dir).await
        .map_err(|e| AppError::Deployment(format!("Failed to create config directory: {}", e)))?;

    let config_path = config_dir.join("hardware_devices.json");

    // Load existing devices or create new list
    let mut devices: Vec<HardwareDeviceConfig> = if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path).await
            .map_err(|e| AppError::Deployment(format!("Failed to read config: {}", e)))?;
        serde_json::from_str(&content)
            .unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };

    // Check if device already exists
    if !devices.iter().any(|d| d.device_path == device_path) {
        devices.push(HardwareDeviceConfig {
            device_path: device_path.clone(),
            device_type: device_type.clone(),
            device_name: device_name.clone(),
            enabled: true,
        });

        // Save updated config
        let json = serde_json::to_string_pretty(&devices)
            .map_err(|e| AppError::Deployment(format!("Failed to serialize config: {}", e)))?;

        tokio::fs::write(&config_path, json).await
            .map_err(|e| AppError::Deployment(format!("Failed to write config: {}", e)))?;

        info!("Saved hardware device to config: {}", config_path.display());

        // Update docker-compose.yml if it exists
        if let Err(e) = update_docker_compose_devices().await {
            warn!("Failed to update docker-compose.yml: {}", e);
            // Don't fail the entire operation if docker-compose update fails
        }
    } else {
        info!("Device already exists in config: {}", device_path);
    }

    Ok(AddHardwareDeviceResponse {
        success: true,
        message: format!("已添加设备 {} 到配置", device_name),
        device_path,
        device_type,
    })
}

/// Update docker-compose.yml with current hardware devices
async fn update_docker_compose_devices() -> Result<(), AppError> {
    use std::path::Path;

    // Find docker-compose.yml in project root
    let possible_paths = vec![
        "docker-compose.yml",
        "../docker-compose.yml",
        "../../docker-compose.yml",
    ];

    let compose_path = possible_paths.iter()
        .find(|p| Path::new(p).exists())
        .ok_or_else(|| AppError::Deployment("docker-compose.yml not found".to_string()))?;

    info!("Found docker-compose.yml at: {}", compose_path);

    // Load hardware devices
    let devices = load_hardware_devices().await?;

    if devices.is_empty() {
        info!("No devices to add to docker-compose.yml");
        return Ok(());
    }

    // Read docker-compose.yml
    let content = tokio::fs::read_to_string(compose_path).await
        .map_err(|e| AppError::Deployment(format!("Failed to read docker-compose.yml: {}", e)))?;

    // Parse YAML
    let mut yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| AppError::Deployment(format!("Failed to parse docker-compose.yml: {}", e)))?;

    // Navigate to services.frigate-config-tool
    if let Some(services) = yaml.get_mut("services") {
        if let Some(service) = services.get_mut("frigate-config-tool") {
            // Create or update devices array
            let device_mappings: Vec<serde_yaml::Value> = devices.iter()
                .map(|d| serde_yaml::Value::String(format!("{}:{}", d.device_path, d.device_path)))
                .collect();

            service.as_mapping_mut()
                .ok_or_else(|| AppError::Deployment("Invalid service structure".to_string()))?
                .insert(
                    serde_yaml::Value::String("devices".to_string()),
                    serde_yaml::Value::Sequence(device_mappings),
                );

            // Write back to file
            let updated_content = serde_yaml::to_string(&yaml)
                .map_err(|e| AppError::Deployment(format!("Failed to serialize YAML: {}", e)))?;

            tokio::fs::write(compose_path, updated_content).await
                .map_err(|e| AppError::Deployment(format!("Failed to write docker-compose.yml: {}", e)))?;

            info!("Updated docker-compose.yml with {} devices", devices.len());
        }
    }

    Ok(())
}

/// Validate hardware device existence
fn validate_device_path(device_path: &str) -> bool {
    std::path::Path::new(device_path).exists()
}

/// Load saved hardware devices from config
pub async fn load_hardware_devices() -> Result<Vec<HardwareDeviceConfig>, AppError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Deployment("Failed to get config directory".to_string()))?
        .join("frigate-config-tool");

    let config_path = config_dir.join("hardware_devices.json");

    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = tokio::fs::read_to_string(&config_path).await
        .map_err(|e| AppError::Deployment(format!("Failed to read hardware config: {}", e)))?;

    let devices: Vec<HardwareDeviceConfig> = serde_json::from_str(&content)
        .map_err(|e| AppError::Deployment(format!("Failed to parse hardware config: {}", e)))?;

    Ok(devices.into_iter().filter(|d| d.enabled).collect())
}

/// Get list of saved hardware devices
#[tauri::command]
pub async fn get_saved_hardware_devices(
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<HardwareDeviceConfig>, AppError> {
    load_hardware_devices().await
}

/// Validate all saved hardware devices
#[tauri::command]
pub async fn validate_hardware_devices(
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<DeviceValidationResponse, AppError> {
    info!("Validating hardware devices");

    let devices = load_hardware_devices().await?;
    let mut valid_devices = Vec::new();
    let mut invalid_devices = Vec::new();
    let mut warnings = Vec::new();

    for device in devices {
        if validate_device_path(&device.device_path) {
            valid_devices.push(DeviceValidationResult {
                device_path: device.device_path.clone(),
                device_name: device.device_name.clone(),
                device_type: device.device_type.clone(),
                exists: true,
                message: format!("✓ 设备 {} 存在", device.device_name),
            });
        } else {
            let warning = format!(
                "⚠️ 设备 {} ({}) 不存在",
                device.device_name,
                device.device_path
            );
            warnings.push(warning.clone());
            invalid_devices.push(DeviceValidationResult {
                device_path: device.device_path.clone(),
                device_name: device.device_name.clone(),
                device_type: device.device_type.clone(),
                exists: false,
                message: warning,
            });
        }
    }

    Ok(DeviceValidationResponse {
        total_devices: valid_devices.len() + invalid_devices.len(),
        valid_count: valid_devices.len(),
        invalid_count: invalid_devices.len(),
        valid_devices,
        invalid_devices,
        warnings,
    })
}

/// Remove hardware device from config
#[tauri::command]
pub async fn remove_hardware_device(
    device_path: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<RemoveHardwareDeviceResponse, AppError> {
    info!("Removing hardware device from config: {}", device_path);

    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Deployment("Failed to get config directory".to_string()))?
        .join("frigate-config-tool");

    let config_path = config_dir.join("hardware_devices.json");

    if !config_path.exists() {
        return Err(AppError::Deployment("No hardware devices configured".to_string()));
    }

    // Load existing devices
    let content = tokio::fs::read_to_string(&config_path).await
        .map_err(|e| AppError::Deployment(format!("Failed to read config: {}", e)))?;

    let mut devices: Vec<HardwareDeviceConfig> = serde_json::from_str(&content)
        .map_err(|e| AppError::Deployment(format!("Failed to parse config: {}", e)))?;

    // Remove the device
    let original_len = devices.len();
    devices.retain(|d| d.device_path != device_path);

    if devices.len() == original_len {
        return Err(AppError::Deployment(format!("Device not found: {}", device_path)));
    }

    // Save updated config
    let json = serde_json::to_string_pretty(&devices)
        .map_err(|e| AppError::Deployment(format!("Failed to serialize config: {}", e)))?;

    tokio::fs::write(&config_path, json).await
        .map_err(|e| AppError::Deployment(format!("Failed to write config: {}", e)))?;

    Ok(RemoveHardwareDeviceResponse {
        success: true,
        message: format!("已从配置中移除设备: {}", device_path),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HardwareDeviceConfig {
    pub device_path: String,
    pub device_type: String,
    pub device_name: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddHardwareDeviceResponse {
    pub success: bool,
    pub message: String,
    pub device_path: String,
    pub device_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveHardwareDeviceResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceValidationResult {
    pub device_path: String,
    pub device_name: String,
    pub device_type: String,
    pub exists: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceValidationResponse {
    pub total_devices: usize,
    pub valid_count: usize,
    pub invalid_count: usize,
    pub valid_devices: Vec<DeviceValidationResult>,
    pub invalid_devices: Vec<DeviceValidationResult>,
    pub warnings: Vec<String>,
}
