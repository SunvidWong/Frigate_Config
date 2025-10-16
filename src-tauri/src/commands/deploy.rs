// T120-T124: Deployment-related Tauri commands
// Provides frontend interface for deployment execution, health checking, and rollback
// REQUIREMENT: FR-037, FR-038, FR-039, FR-040, FR-041, FR-042

use crate::deployment::executor::{
    execute_deployment, get_container_logs, get_deployment_status, stop_deployment,
    DeploymentRequest,
};
use crate::deployment::health::perform_comprehensive_health_check;
use crate::deployment::rollback::{
    execute_rollback, get_deployment_by_id, load_deployment_history, save_deployment_snapshot,
    DeploymentSnapshot, RollbackRequest,
};
use crate::error::AppError;
use crate::models::deployment_state::{DeploymentMethod, DeploymentStatus};
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
            info!(
                "Adding hardware device from config: {} ({})",
                hw_device.device_name, hw_device.device_path
            );

            // Validate device existence
            if !validate_device_path(&hw_device.device_path) {
                let warning = format!(
                    "⚠️ 设备 {} ({}) 不存在,但仍会添加到配置中。请确保设备在部署时可用。",
                    hw_device.device_name, hw_device.device_path
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
        volumes: request
            .volumes
            .into_iter()
            .map(|v| (v.host_path, v.container_path))
            .collect(),
        ports: request
            .ports
            .into_iter()
            .map(|p| (p.host_port, p.container_port))
            .collect(),
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
        checks: health_result
            .checks
            .into_iter()
            .map(|c| HealthCheckDetail {
                name: c.name,
                passed: c.passed,
                message: c.message,
                details: c.details,
            })
            .collect(),
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

    let deployments: Vec<DeploymentHistoryItem> = history
        .into_iter()
        .map(|snapshot| DeploymentHistoryItem {
            id: snapshot.id,
            container_id: snapshot.container_id,
            config_path: snapshot.config_path.to_string_lossy().to_string(),
            deployment_time: chrono::DateTime::<chrono::Utc>::from(snapshot.deployment_time)
                .to_rfc3339(),
            command: snapshot.command,
            status: format!("{:?}", snapshot.status),
        })
        .collect();

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

/// Internal function to add hardware device (no State required)
/// Returns AddHardwareDeviceResponse with diagnostic information
pub async fn add_device_internal(
    device_path: String,
    device_type: String,
    device_name: String,
) -> Result<AddHardwareDeviceResponse, AppError> {
    info!(
        "Adding hardware device to config: {} ({})",
        device_name, device_path
    );

    // Validate device path
    if device_path.is_empty() {
        return Err(AppError::Deployment(
            "Device path cannot be empty".to_string(),
        ));
    }

    // Get or create hardware config file path
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Deployment("Failed to get config directory".to_string()))?
        .join("frigate-config-tool");

    tokio::fs::create_dir_all(&config_dir)
        .await
        .map_err(|e| AppError::Deployment(format!("Failed to create config directory: {}", e)))?;

    let config_path = config_dir.join("hardware_devices.json");

    // Load existing devices or create new list
    let mut devices: Vec<HardwareDeviceConfig> = if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path)
            .await
            .map_err(|e| AppError::Deployment(format!("Failed to read config: {}", e)))?;
        serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
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

        tokio::fs::write(&config_path, json)
            .await
            .map_err(|e| AppError::Deployment(format!("Failed to write config: {}", e)))?;

        info!("Saved hardware device to config: {}", config_path.display());

        // Update docker-compose.yml and get diagnostic info
        info!("Attempting to update docker-compose.yml...");
        let update_result = update_docker_compose_devices().await.map_err(|e| {
            let err_msg = format!(
                "设备已保存到配置文件,但无法更新 docker-compose.yml: {}。请检查日志获取详细信息。",
                e
            );
            warn!("{}", err_msg);
            AppError::Deployment(err_msg)
        })?;

        info!("✓ Device added and docker-compose.yml updated successfully");

        Ok(AddHardwareDeviceResponse {
            success: true,
            message: format!("已添加设备 {} 到配置并更新 docker-compose.yml", device_name),
            device_path,
            device_type,
            docker_compose_updated: true,
            docker_compose_path: Some(update_result.compose_path),
            frigate_service_name: Some(update_result.service_name),
            warnings: update_result.warnings,
        })
    } else {
        info!("Device already exists in config: {}", device_path);
        // Still try to update docker-compose.yml in case it was not updated before
        info!("Attempting to update docker-compose.yml for existing device...");
        let update_result = update_docker_compose_devices().await.map_err(|e| {
            let err_msg = format!(
                "设备已在配置中,但无法更新 docker-compose.yml: {}。请检查日志获取详细信息。",
                e
            );
            warn!("{}", err_msg);
            AppError::Deployment(err_msg)
        })?;

        Ok(AddHardwareDeviceResponse {
            success: true,
            message: format!("设备 {} 已存在,已更新 docker-compose.yml", device_name),
            device_path,
            device_type,
            docker_compose_updated: true,
            docker_compose_path: Some(update_result.compose_path),
            frigate_service_name: Some(update_result.service_name),
            warnings: update_result.warnings,
        })
    }
}

/// Add hardware device to deployment configuration
#[tauri::command]
pub async fn add_hardware_device_to_config(
    device_path: String,
    device_type: String,
    device_name: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<AddHardwareDeviceResponse, AppError> {
    // Call internal implementation which now returns full diagnostic info
    add_device_internal(device_path, device_type, device_name).await
}

/// 查找 docker-compose.yml 文件
///
/// 搜索顺序：
/// 1. 用户自定义路径（如果已配置）
/// 2. 当前目录的 docker-compose.yml
/// 3. 当前目录的 docker-compose.yaml
/// 4. 其他标准位置
async fn find_docker_compose_file() -> Result<String, AppError> {
    use std::path::Path;

    info!("=== 开始查找 docker-compose.yml 文件 ===");

    // 1. 优先检查用户自定义路径
    if let Ok(Some(custom_path)) = get_docker_compose_path_internal().await {
        info!("发现用户自定义路径: {}", custom_path);
        if Path::new(&custom_path).exists() {
            info!("✓ 使用用户自定义路径: {}", custom_path);
            return Ok(custom_path);
        } else {
            warn!("⚠️ 用户自定义路径不存在,将使用自动检测: {}", custom_path);
        }
    }

    // 2. 搜索标准位置
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Deployment("无法获取配置目录".to_string()))?
        .join("frigate-config-tool");

    let default_compose_path = config_dir.join("frigate-docker-compose.yml");

    let possible_paths = vec![
        // 当前目录 (最常见)
        "./docker-compose.yml".to_string(),
        "./docker-compose.yaml".to_string(),
        // 用户配置目录
        default_compose_path.to_string_lossy().to_string(),
        // 用户 HOME/frigate 目录
        format!(
            "{}/frigate/docker-compose.yml",
            std::env::var("HOME").unwrap_or_default()
        ),
        format!(
            "{}/frigate/docker-compose.yaml",
            std::env::var("HOME").unwrap_or_default()
        ),
        // 系统标准位置
        "/opt/frigate/docker-compose.yml".to_string(),
        "/opt/frigate/docker-compose.yaml".to_string(),
        "docker-compose.frigate.yml".to_string(),
    ];

    info!("在 {} 个位置搜索 docker-compose.yml", possible_paths.len());
    for (idx, path) in possible_paths.iter().enumerate() {
        let exists = Path::new(path).exists();
        info!("  [{}] {} - 存在: {}", idx + 1, path, exists);
        if exists {
            info!("✓ 找到 docker-compose.yml: {}", path);
            return Ok(path.clone());
        }
    }

    // 3. 如果都不存在,返回默认路径(后续会创建)
    warn!("未在任何位置找到 docker-compose.yml");
    info!("将使用默认路径: {}", default_compose_path.display());
    Ok(default_compose_path.to_string_lossy().to_string())
}

/// 查找 Frigate 服务
///
/// 搜索策略：
/// 1. 精确匹配 "frigate" 服务名
/// 2. 模糊匹配包含 "frigate" 的服务名
/// 3. 通过镜像名称识别 (包含 "frigate" 字符串)
///
/// 返回: (服务名称, 服务配置的可变引用)
fn find_frigate_service(
    services: &mut serde_yaml::Mapping,
) -> Result<(String, &mut serde_yaml::Value), AppError> {
    info!("=== 查找 Frigate 服务 ===");
    info!("可用服务数量: {}", services.len());

    // 列出所有服务名
    let service_names: Vec<String> = services
        .keys()
        .filter_map(|k| k.as_str().map(|s| s.to_string()))
        .collect();
    info!("所有服务: {:?}", service_names);

    // 第一遍：查找目标服务名(不可变借用)
    let mut target_service_name: Option<String> = None;

    // 策略 1: 精确匹配 "frigate"
    if services.contains_key(serde_yaml::Value::String("frigate".to_string())) {
        info!("✓ 找到精确匹配的服务: frigate");
        target_service_name = Some("frigate".to_string());
    }

    // 策略 2: 模糊匹配包含 "frigate" 的服务名
    if target_service_name.is_none() {
        for key in services.keys() {
            if let Some(service_name) = key.as_str() {
                if service_name.to_lowercase().contains("frigate") {
                    info!("✓ 找到模糊匹配的服务: {}", service_name);
                    target_service_name = Some(service_name.to_string());
                    break;
                }
            }
        }
    }

    // 策略 3: 通过镜像名称识别
    if target_service_name.is_none() {
        info!("尝试通过镜像名称识别 Frigate 服务...");
        for (key, value) in services.iter() {
            if let Some(service_name) = key.as_str() {
                if let Some(service_map) = value.as_mapping() {
                    if let Some(image) =
                        service_map.get(serde_yaml::Value::String("image".to_string()))
                    {
                        if let Some(image_str) = image.as_str() {
                            info!("  服务 '{}' 使用镜像: {}", service_name, image_str);
                            if image_str.to_lowercase().contains("frigate") {
                                info!("✓ 通过镜像识别到 Frigate 服务: {}", service_name);
                                target_service_name = Some(service_name.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // 第二遍：获取目标服务的可变引用
    if let Some(name) = target_service_name {
        if let Some(service) = services.get_mut(serde_yaml::Value::String(name.clone())) {
            return Ok((name, service));
        }
    }

    // 如果都找不到,返回错误并列出所有服务
    let err_msg = format!(
        "未找到 Frigate 服务。请确保 docker-compose.yml 中有包含 'frigate' 字样的服务名,或使用 Frigate 镜像。\n当前服务列表: {:?}",
        service_names
    );
    warn!("{}", err_msg);
    Err(AppError::Deployment(err_msg))
}

/// Docker Compose 更新结果
#[derive(Debug)]
struct DockerComposeUpdateResult {
    pub compose_path: String,
    pub service_name: String,
    pub warnings: Vec<String>,
}

/// 更新 docker-compose.yml 中的硬件设备配置
async fn update_docker_compose_devices() -> Result<DockerComposeUpdateResult, AppError> {
    use std::path::Path;

    info!("=== 开始更新 docker-compose.yml ===");

    // 使用新的智能路径查找
    let compose_path = find_docker_compose_file().await?;

    info!("✓ 选定的 docker-compose.yml 路径: {}", compose_path);
    info!("✓ 文件存在: {}", Path::new(&compose_path).exists());

    // If file doesn't exist, create from template
    if !Path::new(&compose_path).exists() {
        info!("Creating new docker-compose.yml from template");
        let template = get_frigate_compose_template();
        tokio::fs::write(&compose_path, template)
            .await
            .map_err(|e| {
                AppError::Deployment(format!("Failed to create docker-compose.yml: {}", e))
            })?;
    }

    // Load hardware devices
    let devices = load_hardware_devices().await?;

    info!("Loaded {} hardware devices from config", devices.len());
    for (idx, device) in devices.iter().enumerate() {
        info!(
            "  [{}] {} ({}) - path: {}, enabled: {}",
            idx + 1,
            device.device_name,
            device.device_type,
            device.device_path,
            device.enabled
        );
    }

    if devices.is_empty() {
        warn!("No devices to add to docker-compose.yml - returning early");
        return Ok(DockerComposeUpdateResult {
            compose_path,
            service_name: "unknown".to_string(),
            warnings: vec!["没有设备需要添加到 docker-compose.yml".to_string()],
        });
    }

    // Read docker-compose.yml
    info!("Reading docker-compose.yml from: {}", compose_path);
    let content = tokio::fs::read_to_string(&compose_path)
        .await
        .map_err(|e| {
            let err_msg = format!(
                "Failed to read docker-compose.yml from '{}': {}",
                compose_path, e
            );
            warn!("{}", err_msg);
            AppError::Deployment(err_msg)
        })?;

    info!(
        "✓ Successfully read {} bytes from docker-compose.yml",
        content.len()
    );
    info!(
        "First 200 chars: {}",
        &content[..content.len().min(200)].replace('\n', "\\n")
    );

    // Parse YAML
    info!("Parsing YAML content...");
    let mut yaml: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| {
        let err_msg = format!(
            "Failed to parse docker-compose.yml: {}. Content preview: {}",
            e,
            &content[..content.len().min(500)]
        );
        warn!("{}", err_msg);
        AppError::Deployment(err_msg)
    })?;

    info!("✓ YAML parsed successfully");

    // Log YAML structure
    if let Some(services) = yaml.get("services").and_then(|s| s.as_mapping()) {
        info!("Found {} services in docker-compose.yml:", services.len());
        for (key, _) in services {
            info!("  - Service: {:?}", key.as_str().unwrap_or("<unknown>"));
        }
    } else {
        warn!("No 'services' key found in docker-compose.yml!");
        return Err(AppError::Deployment(
            "docker-compose.yml does not contain 'services' key".to_string(),
        ));
    }

    // 使用智能服务查找替代硬编码的 "frigate" 查找
    let (service_name, service_map) = if let Some(services_value) = yaml.get_mut("services") {
        let services = services_value
            .as_mapping_mut()
            .ok_or_else(|| AppError::Deployment("services 不是有效的映射结构".to_string()))?;

        // 调用智能查找函数
        let (name, service_val) = find_frigate_service(services)?;
        info!("✓ 找到 Frigate 服务: '{}'", name);

        let service_map = service_val
            .as_mapping_mut()
            .ok_or_else(|| AppError::Deployment("服务配置不是有效的映射结构".to_string()))?;

        (name, service_map)
    } else {
        return Err(AppError::Deployment(
            "docker-compose.yml 根级别缺少 'services' 键".to_string(),
        ));
    };

    // Categorize devices by type
    // IMPORTANT: NVIDIA GPUs should NOT use device mapping, only deploy.resources
    // Following Frigate 2025 official documentation
    let mut standard_device_mappings = Vec::new();
    let mut has_nvidia_gpu = false;
    let mut requires_privileged = false;

    for device in &devices {
        // Check for NVIDIA GPU by device_name OR special device_path marker
        let is_nvidia = device.device_name.contains("NVIDIA")
            || device.device_path == "nvidia-gpu-runtime"
            || device.device_path.starts_with("nvidia-");

        let is_hailo = device.device_name.contains("Hailo") || device.device_path.contains("hailo");

        match device.device_type.as_str() {
            "gpu" if is_nvidia => {
                // NVIDIA GPUs - DO NOT add to devices mapping
                // NVIDIA uses deploy.resources.reservations only
                has_nvidia_gpu = true;
                info!("Detected NVIDIA GPU ({}), will use deploy.resources.reservations (no device mapping)", device.device_name);
            }
            "tpu" if is_hailo => {
                // Hailo devices need device mapping AND privileged mode
                standard_device_mappings.push(serde_yaml::Value::String(format!(
                    "{}:{}",
                    device.device_path, device.device_path
                )));
                requires_privileged = true;
                info!(
                    "Detected Hailo TPU ({}), added device mapping and will enable privileged mode",
                    device.device_name
                );
            }
            _ => {
                // Other devices use standard device mapping
                // Skip special marker paths
                if !device.device_path.starts_with("nvidia-")
                    && !device.device_path.contains("runtime")
                {
                    standard_device_mappings.push(serde_yaml::Value::String(format!(
                        "{}:{}",
                        device.device_path, device.device_path
                    )));
                }
            }
        }
    }

    // Add standard devices mapping for non-NVIDIA devices
    if !standard_device_mappings.is_empty() {
        let device_count = standard_device_mappings.len();
        service_map.insert(
            serde_yaml::Value::String("devices".to_string()),
            serde_yaml::Value::Sequence(standard_device_mappings),
        );
        info!(
            "Added {} device mappings to docker-compose.yml",
            device_count
        );
    }

    // For NVIDIA, add deploy.resources.reservations (Frigate 2025 official format)
    // This is the ONLY way to use NVIDIA GPUs with Frigate
    if has_nvidia_gpu {
        // Create deploy.resources.reservations structure
        let mut deploy_map = serde_yaml::Mapping::new();
        let mut resources_map = serde_yaml::Mapping::new();
        let mut reservations_map = serde_yaml::Mapping::new();

        // Create devices array for GPU reservation
        let mut gpu_devices = Vec::new();
        let mut gpu_device_map = serde_yaml::Mapping::new();
        gpu_device_map.insert(
            serde_yaml::Value::String("driver".to_string()),
            serde_yaml::Value::String("nvidia".to_string()),
        );
        gpu_device_map.insert(
            serde_yaml::Value::String("count".to_string()),
            serde_yaml::Value::Number(1.into()),
        );
        gpu_device_map.insert(
            serde_yaml::Value::String("capabilities".to_string()),
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("gpu".to_string())]),
        );
        gpu_devices.push(serde_yaml::Value::Mapping(gpu_device_map));

        reservations_map.insert(
            serde_yaml::Value::String("devices".to_string()),
            serde_yaml::Value::Sequence(gpu_devices),
        );
        resources_map.insert(
            serde_yaml::Value::String("reservations".to_string()),
            serde_yaml::Value::Mapping(reservations_map),
        );
        deploy_map.insert(
            serde_yaml::Value::String("resources".to_string()),
            serde_yaml::Value::Mapping(resources_map),
        );

        service_map.insert(
            serde_yaml::Value::String("deploy".to_string()),
            serde_yaml::Value::Mapping(deploy_map),
        );

        info!("Added NVIDIA GPU configuration with deploy.resources.reservations (official Frigate 2025 format)");
    }

    // Set privileged mode for Hailo and other devices that need it
    if requires_privileged {
        service_map.insert(
            serde_yaml::Value::String("privileged".to_string()),
            serde_yaml::Value::Bool(true),
        );
        info!("Enabled privileged mode for hardware requiring it");
    }

    // Write back to file
    info!("Serializing updated YAML...");
    let updated_content = serde_yaml::to_string(&yaml).map_err(|e| {
        let err_msg = format!("Failed to serialize YAML: {}", e);
        warn!("{}", err_msg);
        AppError::Deployment(err_msg)
    })?;

    info!(
        "✓ Generated {} bytes of YAML content",
        updated_content.len()
    );
    info!(
        "Preview (first 500 chars): {}",
        &updated_content[..updated_content.len().min(500)].replace('\n', "\\n")
    );

    info!("Writing updated content to: {}", compose_path);
    tokio::fs::write(&compose_path, &updated_content)
        .await
        .map_err(|e| {
            let err_msg = format!(
                "Failed to write docker-compose.yml to '{}': {}",
                compose_path, e
            );
            warn!("{}", err_msg);
            AppError::Deployment(err_msg)
        })?;

    info!("✓ Successfully wrote docker-compose.yml");
    info!(
        "✓ Updated '{}' service in docker-compose.yml with {} devices",
        service_name,
        devices.len()
    );
    info!("=== docker-compose.yml update completed successfully ===");

    Ok(DockerComposeUpdateResult {
        compose_path,
        service_name,
        warnings: Vec::new(),
    })
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

    let content = tokio::fs::read_to_string(&config_path)
        .await
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
                device.device_name, device.device_path
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
        return Err(AppError::Deployment(
            "No hardware devices configured".to_string(),
        ));
    }

    // Load existing devices
    let content = tokio::fs::read_to_string(&config_path)
        .await
        .map_err(|e| AppError::Deployment(format!("Failed to read config: {}", e)))?;

    let mut devices: Vec<HardwareDeviceConfig> = serde_json::from_str(&content)
        .map_err(|e| AppError::Deployment(format!("Failed to parse config: {}", e)))?;

    // Remove the device
    let original_len = devices.len();
    devices.retain(|d| d.device_path != device_path);

    if devices.len() == original_len {
        return Err(AppError::Deployment(format!(
            "Device not found: {}",
            device_path
        )));
    }

    // Save updated config
    let json = serde_json::to_string_pretty(&devices)
        .map_err(|e| AppError::Deployment(format!("Failed to serialize config: {}", e)))?;

    tokio::fs::write(&config_path, json)
        .await
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
    pub docker_compose_updated: bool,
    pub docker_compose_path: Option<String>,
    pub frigate_service_name: Option<String>,
    pub warnings: Vec<String>,
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

// ========== PCI Device Scanning ==========

/// Scan for PCI devices using lspci command
#[tauri::command]
pub async fn scan_pci_devices(
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<PciDeviceList, AppError> {
    info!("Scanning PCI devices");

    let pci_devices = scan_host_pci_devices().await?;
    let total_count = pci_devices.len();

    Ok(PciDeviceList {
        devices: pci_devices,
        total_count,
    })
}

/// Internal function to scan PCI devices
pub async fn scan_host_pci_devices() -> Result<Vec<PciDeviceInfo>, AppError> {
    use tokio::process::Command;

    // Run lspci command to list all PCI devices
    let output = Command::new("lspci")
        .arg("-vmm") // Machine-readable format
        .arg("-nn") // Show numeric IDs
        .output()
        .await
        .map_err(|e| {
            AppError::Deployment(format!(
                "Failed to run lspci: {}. Please ensure lspci is installed.",
                e
            ))
        })?;

    if !output.status.success() {
        return Err(AppError::Deployment(format!(
            "lspci command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let devices = parse_lspci_output(&output_str)?;

    Ok(devices)
}

/// Parse lspci -vmm output
fn parse_lspci_output(output: &str) -> Result<Vec<PciDeviceInfo>, AppError> {
    let mut devices = Vec::new();
    let mut current_device: Option<PciDeviceInfo> = None;

    for line in output.lines() {
        if line.is_empty() {
            // End of device entry
            if let Some(device) = current_device.take() {
                devices.push(device);
            }
            continue;
        }

        // Parse key: value format
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().to_string();

            match key {
                "Slot" => {
                    // Start new device
                    current_device = Some(PciDeviceInfo {
                        slot: value.clone(),
                        class: String::new(),
                        vendor: String::new(),
                        device: String::new(),
                        subsystem: None,
                        revision: None,
                        device_path: format!("/sys/bus/pci/devices/{}", value),
                        is_gpu: false,
                        is_nvidia: false,
                        is_amd: false,
                        is_intel: false,
                        recommended_device_path: None,
                    });
                }
                "Class" => {
                    if let Some(ref mut dev) = current_device {
                        // Check if it's a GPU (VGA or 3D controller)
                        dev.is_gpu = value.contains("VGA")
                            || value.contains("3D controller")
                            || value.contains("Display");
                        dev.class = value;
                    }
                }
                "Vendor" => {
                    if let Some(ref mut dev) = current_device {
                        dev.is_nvidia = value.to_lowercase().contains("nvidia");
                        dev.is_amd = value.to_lowercase().contains("amd")
                            || value.to_lowercase().contains("advanced micro");
                        dev.is_intel = value.to_lowercase().contains("intel");
                        dev.vendor = value;
                    }
                }
                "Device" => {
                    if let Some(ref mut dev) = current_device {
                        dev.device = value;
                    }
                }
                "SVendor" | "SDevice" => {
                    if let Some(ref mut dev) = current_device {
                        dev.subsystem = Some(value);
                    }
                }
                "Rev" => {
                    if let Some(ref mut dev) = current_device {
                        dev.revision = Some(value);
                    }
                }
                _ => {}
            }
        }
    }

    // Don't forget the last device
    if let Some(device) = current_device {
        devices.push(device);
    }

    // Add recommended device paths for GPUs
    for device in &mut devices {
        if device.is_gpu {
            device.recommended_device_path = determine_recommended_device_path(device);
        }
    }

    Ok(devices)
}

/// Determine the recommended device path based on vendor
/// IMPORTANT: NVIDIA GPUs should NOT use device mapping in Frigate 2025
fn determine_recommended_device_path(device: &PciDeviceInfo) -> Option<String> {
    if device.is_nvidia {
        // NVIDIA GPUs - Frigate 2025 uses deploy.resources, NOT device mapping
        // Return a special marker to indicate this is NVIDIA
        Some("🔧 NVIDIA GPU 使用 deploy.resources.reservations,无需设备映射".to_string())
    } else if device.is_amd || device.is_intel {
        // AMD and Intel use DRI render nodes
        let mut dri_devices = Vec::new();

        if let Ok(entries) = std::fs::read_dir("/dev/dri") {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("renderD") || name.starts_with("card") {
                    dri_devices.push(format!("/dev/dri/{}", name));
                }
            }
        }

        if !dri_devices.is_empty() {
            Some(dri_devices.join(", "))
        } else {
            Some("/dev/dri/renderD128, /dev/dri/card0".to_string())
        }
    } else {
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PciDeviceList {
    pub devices: Vec<PciDeviceInfo>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PciDeviceInfo {
    pub slot: String,
    pub class: String,
    pub vendor: String,
    pub device: String,
    pub subsystem: Option<String>,
    pub revision: Option<String>,
    pub device_path: String,
    pub is_gpu: bool,
    pub is_nvidia: bool,
    pub is_amd: bool,
    pub is_intel: bool,
    pub recommended_device_path: Option<String>,
}

/// Get Frigate official docker-compose.yml template
/// Based on https://docs.frigate.video/frigate/installation
fn get_frigate_compose_template() -> String {
    r#"# Frigate NVR Docker Compose Configuration
# Official documentation: https://docs.frigate.video/frigate/installation
# Generated by Frigate Configuration Tool

services:
  frigate:
    container_name: frigate
    privileged: true # this may not be necessary for all setups
    restart: unless-stopped
    stop_grace_period: 30s # allow enough time to shut down the various services
    image: ghcr.io/blakeblackshear/frigate:stable
    shm_size: "512mb" # update for your cameras based on calculation above
    devices:
      # Add your hardware acceleration devices here
      # Example: /dev/dri/renderD128:/dev/dri/renderD128 for Intel hwaccel
      # Example: /dev/nvidia0:/dev/nvidia0 for NVIDIA GPU
      # Example: /dev/apex_0:/dev/apex_0 for Google Coral PCIe
      # The Frigate Config Tool will automatically add devices here
    volumes:
      - /etc/localtime:/etc/localtime:ro
      - /path/to/your/config:/config  # Change this to your config path
      - /path/to/your/storage:/media/frigate  # Change this to your storage path
      - type: tmpfs # Optional: 1GB of memory, reduces SSD/SD Card wear
        target: /tmp/cache
        tmpfs:
          size: 1000000000
    ports:
      - "8971:8971"
      # - "5000:5000" # Internal unauthenticated access. Expose carefully.
      - "8554:8554" # RTSP feeds
      - "8555:8555/tcp" # WebRTC over tcp
      - "8555:8555/udp" # WebRTC over udp
    environment:
      FRIGATE_RTSP_PASSWORD: "password"  # Change this to a secure password
"#
    .to_string()
}

// ========== Docker Compose Path Management ==========

/// 内部函数：获取用户自定义的 docker-compose.yml 路径
async fn get_docker_compose_path_internal() -> Result<Option<String>, AppError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Deployment("无法获取配置目录".to_string()))?
        .join("frigate-config-tool");

    let path_file = config_dir.join("docker_compose_path.txt");

    if !path_file.exists() {
        return Ok(None);
    }

    let content = tokio::fs::read_to_string(&path_file)
        .await
        .map_err(|e| AppError::Deployment(format!("读取路径配置失败: {}", e)))?;

    let path = content.trim().to_string();
    if path.is_empty() {
        Ok(None)
    } else {
        Ok(Some(path))
    }
}

/// 设置用户自定义的 docker-compose.yml 路径
/// 如果传入空字符串,则删除自定义配置,恢复自动检测模式
#[tauri::command]
pub async fn set_docker_compose_path(
    path: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<SetDockerComposePathResponse, AppError> {
    info!("设置 docker-compose.yml 路径: {}", path);

    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Deployment("无法获取配置目录".to_string()))?
        .join("frigate-config-tool");

    tokio::fs::create_dir_all(&config_dir)
        .await
        .map_err(|e| AppError::Deployment(format!("创建配置目录失败: {}", e)))?;

    let path_file = config_dir.join("docker_compose_path.txt");

    // 如果路径为空,表示重置为自动检测模式
    if path.is_empty() {
        // 删除配置文件
        if path_file.exists() {
            tokio::fs::remove_file(&path_file)
                .await
                .map_err(|e| AppError::Deployment(format!("删除路径配置失败: {}", e)))?;
        }
        info!("✓ 已重置为自动检测模式");
        return Ok(SetDockerComposePathResponse {
            success: true,
            message: "已重置为自动检测模式".to_string(),
            path: String::new(),
        });
    }

    // 验证非空路径存在
    if !std::path::Path::new(&path).exists() {
        warn!("⚠️ 路径不存在,但仍会保存配置: {}", path);
        // 不返回错误,允许用户先设置路径后创建文件
    }

    // 保存路径到配置文件
    tokio::fs::write(&path_file, &path)
        .await
        .map_err(|e| AppError::Deployment(format!("保存路径配置失败: {}", e)))?;

    info!("✓ 已保存 docker-compose.yml 路径配置");

    Ok(SetDockerComposePathResponse {
        success: true,
        message: format!("已设置 docker-compose.yml 路径: {}", path),
        path,
    })
}

/// 获取当前配置的 docker-compose.yml 路径
#[tauri::command]
pub async fn get_docker_compose_path(
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<GetDockerComposePathResponse, AppError> {
    info!("获取 docker-compose.yml 路径配置");

    let custom_path = get_docker_compose_path_internal().await?;

    Ok(GetDockerComposePathResponse {
        path: custom_path.clone(),
        is_custom: custom_path.is_some(),
        message: if let Some(ref p) = custom_path {
            format!("使用自定义路径: {}", p)
        } else {
            "使用自动检测路径".to_string()
        },
    })
}

/// 响应：设置 docker-compose.yml 路径
#[derive(Debug, Serialize, Deserialize)]
pub struct SetDockerComposePathResponse {
    pub success: bool,
    pub message: String,
    pub path: String,
}

/// 响应：获取 docker-compose.yml 路径
#[derive(Debug, Serialize, Deserialize)]
pub struct GetDockerComposePathResponse {
    pub path: Option<String>,
    pub is_custom: bool,
    pub message: String,
}

// ========== Docker Compose Content Management ==========

/// 保存 docker-compose.yml 内容到文件
#[tauri::command]
pub async fn save_docker_compose_content(
    path: String,
    content: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<SaveDockerComposeContentResponse, AppError> {
    info!("保存 docker-compose.yml 内容到: {}", path);

    // 确保父目录存在
    if let Some(parent) = std::path::Path::new(&path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| AppError::Deployment(format!("创建目录失败: {}", e)))?;
    }

    // 写入文件
    tokio::fs::write(&path, &content)
        .await
        .map_err(|e| AppError::Deployment(format!("写入文件失败: {}", e)))?;

    info!("✓ 成功保存 docker-compose.yml ({} 字节)", content.len());

    Ok(SaveDockerComposeContentResponse {
        success: true,
        message: format!("已保存到: {}", path),
        path,
        bytes_written: content.len(),
    })
}

/// 响应：保存 docker-compose.yml 内容
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveDockerComposeContentResponse {
    pub success: bool,
    pub message: String,
    pub path: String,
    pub bytes_written: usize,
}
