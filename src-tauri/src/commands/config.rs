// Configuration management Tauri commands
// T058: Backend configuration management commands

use crate::config_engine::backup::*;
use crate::config_engine::merger::*;
use crate::config_engine::parser::*;
use crate::config_engine::merger::ConflictSeverity;
use crate::error::AppError;
use crate::models::configuration_snapshot::*;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Load configuration from file
#[tauri::command]
pub async fn load_config(
    file_path: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<ConfigurationData, AppError> {
    info!("Loading configuration from: {}", file_path);

    let parser = ConfigParser::new();
    let parse_result = parser.parse_file(&file_path).await
        .map_err(|e| AppError::Config(format!("Failed to parse config: {}", e)))?;

    let config = ConfigurationData {
        file_path: file_path.clone(),
        content: parse_result.config.raw_content,
        yaml: parse_result.config.yaml,
        cameras: parse_result.config.cameras,
        detectors: parse_result.config.detectors,
        global_config: parse_result.config.global_config,
        metadata: ConfigMetadataResponse {
            file_path: parse_result.config.metadata.file_path,
            file_size: parse_result.config.metadata.file_size,
            last_modified: parse_result.config.metadata.last_modified,
            has_manual_edits: parse_result.config.metadata.has_manual_edits,
            manual_edit_lines: parse_result.config.metadata.manual_edit_lines,
            comments_count: parse_result.config.metadata.comments.len(),
        },
        warnings: parse_result.warnings,
        errors: parse_result.errors,
    };

    Ok(config)
}

/// Save configuration to file
#[tauri::command]
pub async fn save_config(
    config_request: SaveConfigRequest,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<SaveConfigResponse, AppError> {
    info!("Saving configuration to: {}", config_request.file_path);

    // Create snapshot before saving if requested
    if config_request.create_snapshot {
        let snapshot_manager = SnapshotManager::default();
        let parser = ConfigParser::new();

        // Parse current content to create FrigateConfig
        let parse_result = parser.parse_content(
            config_request.content.clone(),
            config_request.file_path.clone(),
        )?;

        let snapshot_options = SnapshotOptions {
            description: Some(config_request.snapshot_description.unwrap_or_default()),
            creation_source: CreationSource::Ui,
            is_backup: false,
            backup_reason: None,
            tags: vec![],
        };

        match snapshot_manager.create_snapshot(&parse_result.config, snapshot_options).await {
            Ok(snapshot) => {
                info!("Created snapshot before saving: {}", snapshot.id);
            }
            Err(e) => {
                warn!("Failed to create snapshot: {}", e);
            }
        }
    }

    // Save the configuration
    tokio::fs::write(&config_request.file_path, &config_request.content).await
        .map_err(|e| AppError::Config(format!("Failed to save config: {}", e)))?;

    Ok(SaveConfigResponse {
        success: true,
        message: "Configuration saved successfully".to_string(),
        file_path: config_request.file_path,
        bytes_written: config_request.content.len(),
        snapshot_id: None, // Would be populated if snapshot was created
    })
}

/// Merge two configurations
#[tauri::command]
pub async fn merge_configurations(
    merge_request: MergeConfigurationsRequest,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<MergeResultResponse, AppError> {
    info!("Merging UI and manual configurations");

    let parser = ConfigParser::new();
    let merger = ConfigMerger::new();

    // Parse UI configuration
    let ui_parse_result = parser.parse_content(
        merge_request.ui_config.content,
        merge_request.ui_config.file_path,
    )?;

    // Parse manual configuration
    let manual_parse_result = parser.parse_content(
        merge_request.manual_config.content,
        merge_request.manual_config.file_path,
    )?;

    // Merge configurations
    let merge_result = merger.merge_configurations(&ui_parse_result.config, &manual_parse_result.config)?;

    // Convert to response format
    let response = MergeResultResponse {
        success: true,
        merged_config: MergedConfiguration {
            content: merge_result.merged_config.raw_content,
            yaml: merge_result.merged_config.yaml,
            cameras: merge_result.merged_config.cameras,
            detectors: merge_result.merged_config.detectors,
            global_config: merge_result.merged_config.global_config,
        },
        conflicts: merge_result.conflicts.into_iter().map(|c| ConfigConflictResponse {
            path: c.path,
            conflict_type: format!("{:?}", c.conflict_type),
            severity: format!("{:?}", c.severity),
            description: c.description,
            ui_value: c.ui_value,
            manual_value: c.manual_value,
            suggested_resolution: c.suggested_resolution.map(|r| ConflictResolutionResponse {
                resolution_type: format!("{:?}", r.resolution_type),
                description: r.description,
                auto_applicable: r.auto_applicable,
            }),
        }).collect(),
        preserved_edits: merge_result.preserved_edits.into_iter().map(|e| PreservedEditResponse {
            path: e.path,
            reason: e.reason,
            line_number: e.line_number,
        }).collect(),
        warnings: merge_result.warnings.into_iter().map(|w| MergeWarningResponse {
            message: w.message,
            context: w.context,
            suggested_action: w.suggested_action,
        }).collect(),
        statistics: MergeStatisticsResponse {
            total_fields: merge_result.statistics.total_fields,
            conflicts_count: merge_result.statistics.conflicts_count,
            preserved_edits_count: merge_result.statistics.preserved_edits_count,
            warnings_count: merge_result.statistics.warnings_count,
            auto_resolutions_count: merge_result.statistics.auto_resolutions_count,
        },
    };

    Ok(response)
}

/// Resolve configuration conflicts
#[tauri::command]
pub async fn resolve_conflicts(
    resolve_request: ResolveConflictsRequest,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<ResolveConflictsResponse, AppError> {
    info!("Resolving configuration conflicts");

    let parser = ConfigParser::new();
    let merger = ConfigMerger::new();

    // Parse configurations
    let ui_parse_result = parser.parse_content(
        resolve_request.ui_config.content,
        resolve_request.ui_config.file_path,
    )?;

    let manual_parse_result = parser.parse_content(
        resolve_request.manual_config.content,
        resolve_request.manual_config.file_path,
    )?;

    // Merge with conflict resolution
    let mut merge_result = merger.merge_configurations(&ui_parse_result.config, &manual_parse_result.config)?;

    // Apply resolutions
    let mut resolutions = std::collections::HashMap::new();
    let resolutions_ref = &resolve_request.resolutions;
    for resolution in resolutions_ref {
        let conflict_resolution = ConflictResolution {
            resolution_type: match resolution.resolution_type.as_str() {
                "UseUiValue" => ResolutionType::UseUiValue,
                "UseManualValue" => ResolutionType::UseManualValue,
                "CombineValues" => ResolutionType::CombineValues,
                "DeleteField" => ResolutionType::DeleteField,
                _ => ResolutionType::ManualReview,
            },
            suggested_value: resolution.suggested_value.as_ref().and_then(|v| serde_yaml::from_str(v).ok()),
            description: resolution.description.clone(),
            auto_applicable: resolution.auto_applicable,
        };
        resolutions.insert(resolution.path.clone(), conflict_resolution);
    }

    merger.resolve_conflicts(&mut merge_result, &resolutions)?;

    Ok(ResolveConflictsResponse {
        success: true,
        resolved_config: ResolvedConfiguration {
            content: merge_result.merged_config.raw_content,
            yaml: merge_result.merged_config.yaml,
            cameras: merge_result.merged_config.cameras,
            detectors: merge_result.merged_config.detectors,
            global_config: merge_result.merged_config.global_config,
        },
        applied_resolutions: resolve_request.resolutions.len(),
        remaining_conflicts: merge_result.conflicts.len(),
    })
}

/// Create configuration snapshot
#[tauri::command]
pub async fn create_snapshot(
    snapshot_request: CreateSnapshotRequest,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<CreateSnapshotResponse, AppError> {
    info!("Creating configuration snapshot: {:?}", snapshot_request.description);

    let snapshot_manager = SnapshotManager::default();
    let parser = ConfigParser::new();

    // Parse configuration
    let parse_result = parser.parse_content(
        snapshot_request.config_content,
        snapshot_request.file_path,
    )?;

    // Create snapshot options
    let snapshot_options = SnapshotOptions {
        description: snapshot_request.description,
        creation_source: match snapshot_request.creation_source.as_str() {
            "Ui" => CreationSource::Ui,
            "Template" => CreationSource::Template,
            "ManualEdit" => CreationSource::ManualEdit,
            "Rollback" => CreationSource::Rollback,
            _ => CreationSource::Ui,
        },
        is_backup: snapshot_request.is_backup,
        backup_reason: snapshot_request.backup_reason,
        tags: snapshot_request.tags,
    };

    // Create snapshot
    let snapshot = snapshot_manager.create_snapshot(&parse_result.config, snapshot_options).await?;

    Ok(CreateSnapshotResponse {
        success: true,
        snapshot_id: snapshot.id,
        version: snapshot.version,
        created_at: snapshot.created_at,
        checksum: snapshot.checksum,
        message: "Snapshot created successfully".to_string(),
    })
}

/// List configuration snapshots
#[tauri::command]
pub async fn list_snapshots(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<ListSnapshotsResponse, AppError> {
    info!("Listing configuration snapshots");

    let snapshot_manager = SnapshotManager::default();
    let snapshots = snapshot_manager.list_snapshots().await?;

    let snapshot_responses: Vec<SnapshotResponse> = snapshots.iter().map(|s| SnapshotResponse {
        id: s.id.clone(),
        version: s.version,
        created_at: s.created_at.clone(),
        created_by: format!("{:?}", s.created_by),
        description: s.description.clone(),
        is_backup: s.is_backup,
        backup_reason: s.backup_reason.clone(),
        deployed: s.deployed,
        deployed_at: s.deployed_at.clone(),
        deployment_success: s.deployment_success,
        checksum: s.checksum.clone(),
        size_bytes: s.yaml_content.len(),
    }).collect();

    let total_count = snapshot_responses.len();
    Ok(ListSnapshotsResponse {
        snapshots: snapshot_responses,
        total_count,
    })
}

/// Restore configuration from snapshot
#[tauri::command]
pub async fn restore_from_snapshot(
    restore_request: RestoreSnapshotRequest,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<RestoreSnapshotResponse, AppError> {
    info!("Restoring configuration from snapshot: {}", restore_request.snapshot_id);

    let snapshot_manager = SnapshotManager::default();

    // Create restore options
    let restore_options = RestoreOptions {
        validate: restore_request.validate,
        backup_before_restore: restore_request.backup_before_restore,
        reason: restore_request.reason,
        merge_changes: restore_request.merge_changes,
    };

    // Restore from snapshot
    let restore_result = snapshot_manager.restore_snapshot(
        &restore_request.snapshot_id,
        std::path::Path::new(&restore_request.target_path),
        restore_options,
    ).await?;

    Ok(RestoreSnapshotResponse {
        success: restore_result.success,
        message: restore_result.message,
        backup_path: restore_result.backup_path.map(|p| p.to_string_lossy().to_string()),
        changes_count: restore_result.changes_count,
        snapshot_id: restore_request.snapshot_id,
    })
}

/// Delete configuration snapshot
#[tauri::command]
pub async fn delete_snapshot(
    snapshot_id: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<DeleteSnapshotResponse, AppError> {
    info!("Deleting configuration snapshot: {}", snapshot_id);

    let snapshot_manager = SnapshotManager::default();
    snapshot_manager.delete_snapshot(&snapshot_id).await?;

    Ok(DeleteSnapshotResponse {
        success: true,
        message: format!("Snapshot {} deleted successfully", snapshot_id),
        snapshot_id,
    })
}

/// Compare two snapshots
#[tauri::command]
pub async fn compare_snapshots(
    compare_request: CompareSnapshotsRequest,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<CompareSnapshotsResponse, AppError> {
    info!("Comparing snapshots: {} and {}", compare_request.snapshot_a_id, compare_request.snapshot_b_id);

    let snapshot_manager = SnapshotManager::default();
    let comparison = snapshot_manager.compare_snapshots(
        &compare_request.snapshot_a_id,
        &compare_request.snapshot_b_id,
    ).await?;

    let differences_response: Vec<ConfigConflictResponse> = comparison.differences.into_iter().map(|d| ConfigConflictResponse {
        path: d.path,
        conflict_type: format!("{:?}", d.change_type),
        severity: format!("{:?}", ConflictSeverity::Warning), // Default severity
        description: d.description,
        ui_value: d.value_a,
        manual_value: d.value_b,
        suggested_resolution: None,
    }).collect();

    Ok(CompareSnapshotsResponse {
        identical: comparison.identical,
        differences: differences_response,
        statistics: ComparisonStatisticsResponse {
            total_fields: comparison.statistics.total_fields,
            differences_count: comparison.statistics.differences_count,
            additions: comparison.statistics.additions,
            removals: comparison.statistics.removals,
            modifications: comparison.statistics.modifications,
            reorders: comparison.statistics.reorders,
        },
    })
}

/// Get snapshot statistics
#[tauri::command]
pub async fn get_snapshot_statistics(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<SnapshotStatisticsResponse, AppError> {
    info!("Getting snapshot statistics");

    let snapshot_manager = SnapshotManager::default();
    let stats = snapshot_manager.get_statistics().await?;

    Ok(SnapshotStatisticsResponse {
        total_snapshots: stats.total_snapshots,
        total_size_bytes: stats.total_size_bytes,
        oldest_snapshot: stats.oldest_snapshot,
        newest_snapshot: stats.newest_snapshot,
        backup_snapshots: stats.backup_snapshots,
        regular_snapshots: stats.regular_snapshots,
    })
}

/// Validate configuration
#[tauri::command]
pub async fn validate_config(
    content: String,
    file_path: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<ValidationResponse, AppError> {
    info!("Validating configuration: {}", file_path);

    let parser = ConfigParser::new();
    let parse_result = parser.parse_content(content, file_path)?;

    Ok(ValidationResponse {
        valid: parse_result.errors.is_empty(),
        errors: parse_result.errors.into_iter().map(|e| ValidationError {
            message: e.message,
            line_number: e.line_number,
            error_type: format!("{:?}", e.error_type),
        }).collect(),
        warnings: parse_result.warnings.into_iter().map(|w| ValidationWarning {
            message: w.message,
            line_number: w.line_number,
            suggestion: w.suggestion,
        }).collect(),
        cameras_count: parse_result.config.cameras.len(),
        detectors_count: parse_result.config.detectors.len(),
    })
}

// Request/Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationData {
    pub file_path: String,
    pub content: String,
    pub yaml: serde_yaml::Value,
    pub cameras: std::collections::HashMap<String, crate::config_engine::parser::CameraConfig>,
    pub detectors: std::collections::HashMap<String, crate::config_engine::parser::DetectorConfig>,
    pub global_config: crate::config_engine::parser::GlobalConfig,
    pub metadata: ConfigMetadataResponse,
    pub warnings: Vec<crate::config_engine::parser::ParseWarning>,
    pub errors: Vec<crate::config_engine::parser::ParseError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigMetadataResponse {
    pub file_path: String,
    pub file_size: u64,
    pub last_modified: String,
    pub has_manual_edits: bool,
    pub manual_edit_lines: Vec<usize>,
    pub comments_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveConfigRequest {
    pub file_path: String,
    pub content: String,
    pub create_snapshot: bool,
    pub snapshot_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveConfigResponse {
    pub success: bool,
    pub message: String,
    pub file_path: String,
    pub bytes_written: usize,
    pub snapshot_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeConfigurationsRequest {
    pub ui_config: ConfigurationInput,
    pub manual_config: ConfigurationInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationInput {
    pub file_path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeResultResponse {
    pub success: bool,
    pub merged_config: MergedConfiguration,
    pub conflicts: Vec<ConfigConflictResponse>,
    pub preserved_edits: Vec<PreservedEditResponse>,
    pub warnings: Vec<MergeWarningResponse>,
    pub statistics: MergeStatisticsResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergedConfiguration {
    pub content: String,
    pub yaml: serde_yaml::Value,
    pub cameras: std::collections::HashMap<String, crate::config_engine::parser::CameraConfig>,
    pub detectors: std::collections::HashMap<String, crate::config_engine::parser::DetectorConfig>,
    pub global_config: crate::config_engine::parser::GlobalConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigConflictResponse {
    pub path: String,
    pub conflict_type: String,
    pub severity: String,
    pub description: String,
    pub ui_value: Option<serde_yaml::Value>,
    pub manual_value: Option<serde_yaml::Value>,
    pub suggested_resolution: Option<ConflictResolutionResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictResolutionResponse {
    pub resolution_type: String,
    pub description: String,
    pub auto_applicable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreservedEditResponse {
    pub path: String,
    pub reason: String,
    pub line_number: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeWarningResponse {
    pub message: String,
    pub context: Option<String>,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeStatisticsResponse {
    pub total_fields: usize,
    pub conflicts_count: usize,
    pub preserved_edits_count: usize,
    pub warnings_count: usize,
    pub auto_resolutions_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveConflictsRequest {
    pub ui_config: ConfigurationInput,
    pub manual_config: ConfigurationInput,
    pub resolutions: Vec<ConflictResolutionRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictResolutionRequest {
    pub path: String,
    pub resolution_type: String,
    pub description: String,
    pub suggested_value: Option<String>,
    pub auto_applicable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveConflictsResponse {
    pub success: bool,
    pub resolved_config: ResolvedConfiguration,
    pub applied_resolutions: usize,
    pub remaining_conflicts: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvedConfiguration {
    pub content: String,
    pub yaml: serde_yaml::Value,
    pub cameras: std::collections::HashMap<String, crate::config_engine::parser::CameraConfig>,
    pub detectors: std::collections::HashMap<String, crate::config_engine::parser::DetectorConfig>,
    pub global_config: crate::config_engine::parser::GlobalConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSnapshotRequest {
    pub config_content: String,
    pub file_path: String,
    pub description: Option<String>,
    pub creation_source: String,
    pub is_backup: bool,
    pub backup_reason: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSnapshotResponse {
    pub success: bool,
    pub snapshot_id: String,
    pub version: i32,
    pub created_at: String,
    pub checksum: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListSnapshotsResponse {
    pub snapshots: Vec<SnapshotResponse>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotResponse {
    pub id: String,
    pub version: i32,
    pub created_at: String,
    pub created_by: String,
    pub description: Option<String>,
    pub is_backup: bool,
    pub backup_reason: Option<String>,
    pub deployed: bool,
    pub deployed_at: Option<String>,
    pub deployment_success: Option<bool>,
    pub checksum: String,
    pub size_bytes: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreSnapshotRequest {
    pub snapshot_id: String,
    pub target_path: String,
    pub validate: bool,
    pub backup_before_restore: bool,
    pub reason: Option<String>,
    pub merge_changes: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreSnapshotResponse {
    pub success: bool,
    pub message: String,
    pub backup_path: Option<String>,
    pub changes_count: usize,
    pub snapshot_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteSnapshotResponse {
    pub success: bool,
    pub message: String,
    pub snapshot_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompareSnapshotsRequest {
    pub snapshot_a_id: String,
    pub snapshot_b_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompareSnapshotsResponse {
    pub identical: bool,
    pub differences: Vec<ConfigConflictResponse>,
    pub statistics: ComparisonStatisticsResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComparisonStatisticsResponse {
    pub total_fields: usize,
    pub differences_count: usize,
    pub additions: usize,
    pub removals: usize,
    pub modifications: usize,
    pub reorders: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotStatisticsResponse {
    pub total_snapshots: usize,
    pub total_size_bytes: usize,
    pub oldest_snapshot: Option<String>,
    pub newest_snapshot: Option<String>,
    pub backup_snapshots: usize,
    pub regular_snapshots: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub cameras_count: usize,
    pub detectors_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub message: String,
    pub line_number: Option<usize>,
    pub error_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub message: String,
    pub line_number: Option<usize>,
    pub suggestion: Option<String>,
}
