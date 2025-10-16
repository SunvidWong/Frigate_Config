// 系统状态检查命令

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// 已保存的硬件设备数量
    pub hardware_devices: usize,
    /// docker-compose.yml 是否存在
    pub docker_compose_exists: bool,
    /// 配置文件是否有效
    pub config_valid: bool,
    /// Docker 是否运行中
    pub docker_running: bool,
}

/// 检查系统各组件状态
#[tauri::command]
pub async fn check_system_status() -> Result<SystemStatus, AppError> {
    info!("检查系统状态");

    // 1. 检查硬件设备数量
    let hardware_devices = count_saved_devices().await.unwrap_or(0);

    // 2. 检查 docker-compose.yml 是否存在
    let docker_compose_exists = check_docker_compose_exists().await.unwrap_or(false);

    // 3. 检查配置文件有效性
    let config_valid = check_config_validity().await.unwrap_or(false);

    // 4. 检查 Docker 是否运行中
    let docker_running = check_docker_running().unwrap_or(false);

    Ok(SystemStatus {
        hardware_devices,
        docker_compose_exists,
        config_valid,
        docker_running,
    })
}

/// 统计已保存的硬件设备数量
async fn count_saved_devices() -> Result<usize, AppError> {
    // 查找常见的 docker-compose.yml 路径
    let paths = vec![
        "./docker-compose.yml",
        "/opt/frigate/docker-compose.yml",
        "/etc/frigate/docker-compose.yml",
    ];

    for path_str in paths {
        if Path::new(path_str).exists() {
            let content = std::fs::read_to_string(path_str)?;
            // 简单统计 devices: 部分的设备数量
            let device_count = content.lines()
                .filter(|line| line.trim().starts_with("- /dev/"))
                .count();
            return Ok(device_count);
        }
    }

    Ok(0)
}

/// 检查 docker-compose.yml 是否存在
async fn check_docker_compose_exists() -> Result<bool, AppError> {
    let paths = vec![
        "./docker-compose.yml",
        "/opt/frigate/docker-compose.yml",
        "/etc/frigate/docker-compose.yml",
    ];

    for path_str in paths {
        if Path::new(path_str).exists() {
            return Ok(true);
        }
    }

    Ok(false)
}

/// 检查配置文件有效性
async fn check_config_validity() -> Result<bool, AppError> {
    let config_paths = vec![
        "/etc/frigate/config.yml",
        "/config/config.yml",
        "./config.yml",
    ];

    for path_str in config_paths {
        if Path::new(path_str).exists() {
            let content = std::fs::read_to_string(path_str)?;
            // 基本检查：是否包含必要的键
            let has_cameras = content.contains("cameras:");
            let has_mqtt = content.contains("mqtt:");
            return Ok(has_cameras && has_mqtt);
        }
    }

    Ok(false)
}

/// 检查 Docker 是否运行中
fn check_docker_running() -> Result<bool, AppError> {
    let output = Command::new("docker")
        .arg("info")
        .output();

    match output {
        Ok(result) => Ok(result.status.success()),
        Err(_) => Ok(false),
    }
}