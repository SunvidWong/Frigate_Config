// Camera discovery commands

use crate::error::AppError;
use crate::network::{
    get_all_network_interfaces, get_local_ip, guess_network_range, scan_network,
    DiscoveredCamera, NetworkInterface, ScanConfig,
};
use serde::{Deserialize, Serialize};

/// 获取所有网络接口信息
#[tauri::command]
pub async fn get_network_interfaces() -> Result<Vec<NetworkInterface>, AppError> {
    Ok(get_all_network_interfaces())
}

/// Get local network IP
#[tauri::command]
pub async fn get_local_network_ip() -> Result<String, AppError> {
    get_local_ip().ok_or_else(|| AppError::Network("Failed to get local IP".to_string()))
}

/// Guess network range based on local IP
#[tauri::command]
pub async fn guess_network_range_command() -> Result<String, AppError> {
    Ok(guess_network_range())
}

/// Scan network for cameras
#[tauri::command]
pub async fn scan_for_cameras(
    network_range: Option<String>,
    ports: Option<Vec<u16>>,
    timeout_ms: Option<u64>,
) -> Result<Vec<DiscoveredCamera>, AppError> {
    let config = ScanConfig {
        network_range: network_range.unwrap_or_else(guess_network_range),
        ports: ports.unwrap_or_else(|| vec![554, 80, 8000, 8080, 8554, 8888]),
        timeout_ms: timeout_ms.unwrap_or(1000),
        concurrency: 50,
    };

    scan_network(config).await.map_err(AppError::Network)
}

/// Scan multiple network ranges for cameras
#[tauri::command]
pub async fn scan_multiple_networks(
    network_ranges: Vec<String>,
    ports: Option<Vec<u16>>,
    timeout_ms: Option<u64>,
) -> Result<Vec<DiscoveredCamera>, AppError> {
    let ports = ports.unwrap_or_else(|| vec![554, 80, 8000, 8080, 8554, 8888]);
    let timeout = timeout_ms.unwrap_or(1000);

    let mut all_cameras = Vec::new();

    for range in network_ranges {
        let config = ScanConfig {
            network_range: range,
            ports: ports.clone(),
            timeout_ms: timeout,
            concurrency: 50,
        };

        match scan_network(config).await {
            Ok(mut cameras) => {
                all_cameras.append(&mut cameras);
            }
            Err(e) => {
                eprintln!("Failed to scan network: {}", e);
                // Continue with other ranges even if one fails
            }
        }
    }

    Ok(all_cameras)
}

/// Quick scan with default settings
#[tauri::command]
pub async fn quick_scan_cameras() -> Result<Vec<DiscoveredCamera>, AppError> {
    let config = ScanConfig::default();
    scan_network(config).await.map_err(AppError::Network)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_local_ip() {
        let result = get_local_network_ip().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_guess_network_range() {
        let result = guess_network_range_command().await;
        assert!(result.is_ok());
        let range = result.unwrap();
        assert!(range.contains("/24"));
    }
}
