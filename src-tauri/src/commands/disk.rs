// T180: Disk and volume mapping Tauri commands
// Provides Tauri commands for disk information and volume validation

use crate::deployment::disk::{get_disk_info, validate_volume_path, DiskInfo};
use crate::error::AppError;
use crate::models::volume_mapping::{VolumeMapping, VolumeMappingType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::command;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfoResponse {
    pub path: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub total_formatted: String,
    pub used_formatted: String,
    pub free_formatted: String,
    pub available_formatted: String,
    pub mount_point: String,
    pub filesystem: String,
    pub usage_percent: f64,
    pub is_low_space: bool,
}

impl From<DiskInfo> for DiskInfoResponse {
    fn from(info: DiskInfo) -> Self {
        let is_low_space = info.is_low_space();

        Self {
            path: info.mount_point.display().to_string(),
            total_bytes: info.total,
            used_bytes: info.used,
            free_bytes: info.free,
            available_bytes: info.free, // Use free as available
            total_formatted: info.total_formatted.clone(),
            used_formatted: info.used_formatted.clone(),
            free_formatted: info.free_formatted.clone(),
            available_formatted: info.free_formatted.clone(),
            mount_point: info.mount_point.display().to_string(),
            filesystem: info
                .filesystem_type
                .unwrap_or_else(|| "unknown".to_string()),
            usage_percent: info.usage_percent,
            is_low_space,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeValidationResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub disk_info: Option<DiskInfoResponse>,
}

/// Get disk information for a given path
#[command]
pub async fn get_disk_info_command(path: String) -> Result<DiskInfoResponse, AppError> {
    info!("Getting disk info for path: {}", path);

    let path_buf = PathBuf::from(&path);

    let disk_info = get_disk_info(&path_buf)
        .map_err(|e| AppError::Disk(format!("Failed to get disk info: {}", e)))?;

    Ok(disk_info.into())
}

/// Validate a volume path for mounting
#[command]
pub async fn validate_volume_path_command(
    path: String,
) -> Result<VolumeValidationResponse, AppError> {
    info!("Validating volume path: {}", path);

    let path_buf = PathBuf::from(&path);

    // Get disk info (if possible)
    let disk_info = get_disk_info(&path_buf).ok().map(|info| info.into());

    // Validate path
    match validate_volume_path(&path_buf) {
        Ok(_) => Ok(VolumeValidationResponse {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            disk_info,
        }),
        Err(errors) => {
            // Separate warnings from errors
            let (warnings, errors): (Vec<_>, Vec<_>) =
                errors.into_iter().partition(|e| e.contains("Warning:"));

            let is_valid = errors.is_empty();

            Ok(VolumeValidationResponse {
                valid: is_valid,
                errors,
                warnings,
                disk_info,
            })
        }
    }
}

/// Create a volume mapping
#[command]
pub async fn create_volume_mapping(
    host_path: String,
    container_path: String,
    mapping_type: String,
    read_only: bool,
    description: Option<String>,
) -> Result<VolumeMapping, AppError> {
    info!(
        "Creating volume mapping: {} -> {} ({}, read_only: {})",
        host_path, container_path, mapping_type, read_only
    );

    let host_path_buf = PathBuf::from(&host_path);

    // Parse mapping type
    let mapping_type_enum = match mapping_type.as_str() {
        "recordings" => VolumeMappingType::Recordings,
        "clips" => VolumeMappingType::Clips,
        "cache" => VolumeMappingType::Cache,
        "config" => VolumeMappingType::Config,
        "custom" => VolumeMappingType::Custom,
        _ => {
            return Err(AppError::Validation(format!(
                "Invalid mapping type: {}",
                mapping_type
            )))
        }
    };

    let mut mapping = VolumeMapping::new(host_path_buf, container_path, mapping_type_enum);

    if read_only {
        mapping = mapping.read_only();
    }

    if let Some(desc) = description {
        mapping = mapping.with_description(desc);
    }

    // Validate the mapping
    mapping.validate().map_err(AppError::Validation)?;

    Ok(mapping)
}

/// Get default volume paths based on platform
#[command]
pub async fn get_default_volume_paths() -> Result<DefaultVolumePaths, AppError> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| AppError::Disk("Could not determine home directory".to_string()))?;

    let frigate_dir = home_dir.join("frigate");

    Ok(DefaultVolumePaths {
        config: frigate_dir.join("config").display().to_string(),
        recordings: frigate_dir.join("recordings").display().to_string(),
        clips: frigate_dir.join("clips").display().to_string(),
        cache: frigate_dir.join("cache").display().to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultVolumePaths {
    pub config: String,
    pub recordings: String,
    pub clips: String,
    pub cache: String,
}

/// Get recommended paths for Frigate directories
#[command]
pub async fn get_recommended_paths() -> Result<Vec<RecommendedPath>, AppError> {
    let mut recommendations = Vec::new();

    let home_dir = dirs::home_dir()
        .ok_or_else(|| AppError::Disk("Could not determine home directory".to_string()))?;

    // Check common locations
    let common_paths = vec![
        home_dir.join("frigate"),
        PathBuf::from("/mnt"),
        PathBuf::from("/media"),
        PathBuf::from("/data"),
    ];

    for path in common_paths {
        if path.exists() {
            if let Ok(disk_info) = get_disk_info(&path) {
                recommendations.push(RecommendedPath {
                    path: path.display().to_string(),
                    total_space: disk_info.total_formatted.clone(),
                    free_space: disk_info.free_formatted.clone(),
                    usage_percent: disk_info.usage_percent,
                    recommended_for: determine_recommendation(&disk_info),
                });
            }
        }
    }

    Ok(recommendations)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendedPath {
    pub path: String,
    pub total_space: String,
    pub free_space: String,
    pub usage_percent: f64,
    pub recommended_for: Vec<String>,
}

fn determine_recommendation(disk_info: &DiskInfo) -> Vec<String> {
    let mut recommendations = Vec::new();

    const GB: u64 = 1024 * 1024 * 1024;

    // Config files need very little space
    recommendations.push("config".to_string());

    // Cache needs moderate space
    if disk_info.free > 10 * GB {
        recommendations.push("cache".to_string());
    }

    // Clips need moderate space
    if disk_info.free > 20 * GB {
        recommendations.push("clips".to_string());
    }

    // Recordings need lots of space
    if disk_info.free > 100 * GB {
        recommendations.push("recordings".to_string());
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_disk_info_command() {
        let current_dir = std::env::current_dir().unwrap();
        let result = get_disk_info_command(current_dir.display().to_string()).await;

        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(info.total_bytes > 0);
        assert!(info.free_bytes > 0);
        assert!(!info.total_formatted.is_empty());
    }

    #[tokio::test]
    async fn test_validate_volume_path_command_valid() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let result = validate_volume_path_command(temp_dir.path().display().to_string()).await;

        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(validation.valid);
        assert!(validation.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_volume_path_command_invalid() {
        let result = validate_volume_path_command("/nonexistent/path".to_string()).await;

        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(!validation.valid);
        assert!(!validation.errors.is_empty());
    }

    #[tokio::test]
    async fn test_create_volume_mapping() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let result = create_volume_mapping(
            temp_dir.path().display().to_string(),
            "/config".to_string(),
            "config".to_string(),
            false,
            Some("Test mapping".to_string()),
        )
        .await;

        assert!(result.is_ok());

        let mapping = result.unwrap();
        assert_eq!(mapping.container_path, "/config");
        assert!(!mapping.read_only);
        assert_eq!(mapping.description, Some("Test mapping".to_string()));
    }

    #[tokio::test]
    async fn test_get_default_volume_paths() {
        let result = get_default_volume_paths().await;

        assert!(result.is_ok());

        let paths = result.unwrap();
        assert!(!paths.config.is_empty());
        assert!(!paths.recordings.is_empty());
        assert!(!paths.clips.is_empty());
        assert!(!paths.cache.is_empty());
    }

    #[tokio::test]
    async fn test_get_recommended_paths() {
        let result = get_recommended_paths().await;

        assert!(result.is_ok());

        let recommendations = result.unwrap();
        // Should have at least home directory recommendation if ~/frigate exists
        // On some systems, this might be empty if none of the paths exist
        // So we just verify the function succeeds
        println!("Found {} recommendations", recommendations.len());
    }
}
