// Configuration merging engine with intelligent conflict detection and resolution
// T054: Configuration merger implementation

use crate::config_engine::parser::*;
use crate::error::AppError;
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use tracing::info;

/// Configuration merger result
#[derive(Debug, Clone)]
pub struct MergeResult {
    /// Merged configuration
    pub merged_config: FrigateConfig,
    /// List of detected conflicts
    pub conflicts: Vec<ConfigConflict>,
    /// List of manual edits that were preserved
    pub preserved_edits: Vec<PreservedEdit>,
    /// Merge warnings
    pub warnings: Vec<MergeWarning>,
    /// Merge statistics
    pub statistics: MergeStatistics,
}

/// Configuration conflict between UI config and manual edits
#[derive(Debug, Clone)]
pub struct ConfigConflict {
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Path to the conflicting configuration (e.g., "cameras.front_door.detect.fps")
    pub path: String,
    /// Value from UI configuration
    pub ui_value: Option<Value>,
    /// Value from manual configuration
    pub manual_value: Option<Value>,
    /// Line number in manual config where conflict occurs
    pub line_number: Option<usize>,
    /// Conflict severity
    pub severity: ConflictSeverity,
    /// Description of the conflict
    pub description: String,
    /// Suggested resolution
    pub suggested_resolution: Option<ConflictResolution>,
}

/// Types of conflicts
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    /// Same field has different values
    ValueMismatch,
    /// Field exists in one but not the other
    MissingField,
    /// Array elements differ
    ArrayDifference,
    /// Structural differences (e.g., array vs object)
    StructuralDifference,
    /// Manual edit conflicts with UI validation
    ValidationError,
    /// Duplicate configuration items
    DuplicateItems,
}

/// Conflict severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictSeverity {
    /// User must resolve
    Critical,
    /// Should be reviewed
    Warning,
    /// Informational only
    Info,
}

/// Conflict resolution suggestions
#[derive(Debug, Clone)]
pub struct ConflictResolution {
    /// Resolution type
    pub resolution_type: ResolutionType,
    /// Suggested value
    pub suggested_value: Option<Value>,
    /// Description of the resolution
    pub description: String,
    /// Whether this resolution is automatically applicable
    pub auto_applicable: bool,
}

/// Types of conflict resolution
#[derive(Debug, Clone)]
pub enum ResolutionType {
    /// Use UI value
    UseUiValue,
    /// Use manual value
    UseManualValue,
    /// Combine both values
    CombineValues,
    /// Delete field
    DeleteField,
    /// Manual review required
    ManualReview,
}

/// Preserved manual edit
#[derive(Debug, Clone)]
pub struct PreservedEdit {
    /// Path to the preserved configuration
    pub path: String,
    /// Preserved value
    pub value: Value,
    /// Reason for preservation
    pub reason: String,
    /// Line number in original config
    pub line_number: usize,
}

/// Merge warning
#[derive(Debug, Clone)]
pub struct MergeWarning {
    /// Warning message
    pub message: String,
    /// Warning context
    pub context: Option<String>,
    /// Suggested action
    pub suggested_action: Option<String>,
}

/// Merge statistics
#[derive(Debug, Clone)]
pub struct MergeStatistics {
    /// Total number of fields processed
    pub total_fields: usize,
    /// Number of conflicts detected
    pub conflicts_count: usize,
    /// Number of preserved manual edits
    pub preserved_edits_count: usize,
    /// Number of warnings generated
    pub warnings_count: usize,
    /// Number of automatic resolutions
    pub auto_resolutions_count: usize,
}

/// Configuration merger
pub struct ConfigMerger {
    preserve_manual_comments: bool,
    auto_resolve_conflicts: bool,
    prefer_ui_values: bool,
}

impl ConfigMerger {
    /// Create a new configuration merger
    pub fn new() -> Self {
        Self {
            preserve_manual_comments: true,
            auto_resolve_conflicts: false,
            prefer_ui_values: true,
        }
    }

    /// Merge UI configuration with manual configuration
    pub fn merge_configurations(
        &self,
        ui_config: &FrigateConfig,
        manual_config: &FrigateConfig,
    ) -> Result<MergeResult, AppError> {
        info!("Starting configuration merge between UI and manual config");

        let mut conflicts = Vec::new();
        let mut preserved_edits = Vec::new();
        let mut warnings = Vec::new();
        let mut auto_resolutions_count = 0;

        // Start with UI config as base
        let mut merged_yaml = ui_config.yaml.clone();

        // Merge global sections
        self.merge_global_sections(
            &mut merged_yaml,
            &ui_config.yaml,
            &manual_config.yaml,
            &mut conflicts,
            &mut preserved_edits,
            &mut warnings,
            &mut auto_resolutions_count,
        )?;

        // Merge cameras
        self.merge_cameras(
            &mut merged_yaml,
            &ui_config.yaml,
            &manual_config.yaml,
            &mut conflicts,
            &mut preserved_edits,
            &mut warnings,
            &mut auto_resolutions_count,
        )?;

        // Merge detectors
        self.merge_detectors(
            &mut merged_yaml,
            &ui_config.yaml,
            &manual_config.yaml,
            &mut conflicts,
            &mut preserved_edits,
            &mut warnings,
            &mut auto_resolutions_count,
        )?;

        // Preserve manual comments and formatting
        let merged_content = if self.preserve_manual_comments {
            self.preserve_comments_and_formatting(&merged_yaml, &manual_config.raw_content)?
        } else {
            serde_yaml::to_string(&merged_yaml)
                .map_err(|e| AppError::Config(format!("Failed to serialize merged config: {}", e)))?
        };

        // Re-parse the merged content to create FrigateConfig
        let parser = ConfigParser::new();
        let parse_result = parser.parse_content(
            merged_content,
            format!("merged_{}", ui_config.metadata.file_path),
        )?;

        let total_fields = ConfigMerger::count_total_fields(&merged_yaml);

        let merge_result = MergeResult {
            merged_config: parse_result.config,
            conflicts: conflicts.clone(),
            preserved_edits: preserved_edits.clone(),
            warnings: warnings.clone(),
            statistics: MergeStatistics {
                total_fields,
                conflicts_count: conflicts.len(),
                preserved_edits_count: preserved_edits.len(),
                warnings_count: warnings.len(),
                auto_resolutions_count,
            },
        };

        info!("Configuration merge completed: {} conflicts, {} preserved edits",
              merge_result.statistics.conflicts_count,
              merge_result.statistics.preserved_edits_count);

        Ok(merge_result)
    }

    /// Merge global configuration sections
    fn merge_global_sections(
        &self,
        merged_yaml: &mut Value,
        ui_yaml: &Value,
        manual_yaml: &Value,
        conflicts: &mut Vec<ConfigConflict>,
        preserved_edits: &mut Vec<PreservedEdit>,
        warnings: &mut Vec<MergeWarning>,
        auto_resolutions_count: &mut usize,
    ) -> Result<(), AppError> {
        let global_sections = ["mqtt", "database", "go2rtc", "ffmpeg", "logger"];

        for section in &global_sections {
            let path = *section;
            self.merge_yaml_section(
                merged_yaml,
                ui_yaml,
                manual_yaml,
                path,
                conflicts,
                preserved_edits,
                warnings,
                auto_resolutions_count,
            )?;
        }

        Ok(())
    }

    /// Merge camera configurations
    fn merge_cameras(
        &self,
        merged_yaml: &mut Value,
        ui_yaml: &Value,
        manual_yaml: &Value,
        conflicts: &mut Vec<ConfigConflict>,
        preserved_edits: &mut Vec<PreservedEdit>,
        warnings: &mut Vec<MergeWarning>,
        auto_resolutions_count: &mut usize,
    ) -> Result<(), AppError> {
        let ui_cameras = ui_yaml.get("cameras").and_then(|c| c.as_mapping());
        let manual_cameras = manual_yaml.get("cameras").and_then(|c| c.as_mapping());

        match (ui_cameras, manual_cameras) {
            (Some(ui_map), Some(manual_map)) => {
                // Both have cameras - merge them
                let mut merged_cameras = Mapping::new();

                // Collect all camera names
                let mut all_camera_names = std::collections::HashSet::new();
                for key in ui_map.keys() {
                    if let Some(name) = key.as_str() {
                        all_camera_names.insert(name.to_string());
                    }
                }
                for key in manual_map.keys() {
                    if let Some(name) = key.as_str() {
                        all_camera_names.insert(name.to_string());
                    }
                }

                for camera_name in all_camera_names {
                    let ui_camera = ui_map.get(&Value::String(camera_name.clone()));
                    let manual_camera = manual_map.get(&Value::String(camera_name.clone()));

                    match (ui_camera, manual_camera) {
                        (Some(ui_cam), Some(manual_cam)) => {
                            // Camera exists in both - merge
                                            let merged_camera = self.merge_camera_config(
                                &camera_name,
                                ui_cam,
                                manual_cam,
                                conflicts,
                                preserved_edits,
                                warnings,
                                auto_resolutions_count,
                            )?;
                            merged_cameras.insert(Value::String(camera_name), merged_camera);
                        }
                        (Some(ui_cam), None) => {
                            // Only in UI
                            merged_cameras.insert(Value::String(camera_name), ui_cam.clone());
                        }
                        (None, Some(manual_cam)) => {
                            // Only in manual - preserve
                            merged_cameras.insert(Value::String(camera_name.clone()), manual_cam.clone());
                            preserved_edits.push(PreservedEdit {
                                path: format!("cameras.{}", camera_name),
                                value: manual_cam.clone(),
                                reason: "Camera only exists in manual configuration".to_string(),
                                line_number: 0, // Would need to calculate from line mapping
                            });
                        }
                        _ => {}
                    }
                }

                if let Some(merged) = merged_yaml.as_mapping_mut() {
                    merged.insert(Value::String("cameras".to_string()), Value::Mapping(merged_cameras));
                }
            }
            (Some(ui_map), None) => {
                // Only UI has cameras - use as-is
                if let Some(merged) = merged_yaml.as_mapping_mut() {
                    merged.insert(Value::String("cameras".to_string()), Value::Mapping(ui_map.clone()));
                }
            }
            (None, Some(manual_map)) => {
                // Only manual has cameras - preserve all
                if let Some(merged) = merged_yaml.as_mapping_mut() {
                    merged.insert(Value::String("cameras".to_string()), Value::Mapping(manual_map.clone()));
                }
                preserved_edits.push(PreservedEdit {
                    path: "cameras".to_string(),
                    value: Value::Mapping(manual_map.clone()),
                    reason: "All cameras only exist in manual configuration".to_string(),
                    line_number: 0,
                });
            }
            _ => {} // Neither has cameras
        }

        Ok(())
    }

    /// Merge a single camera configuration
    fn merge_camera_config(
        &self,
        camera_name: &str,
        ui_camera: &Value,
        manual_camera: &Value,
        conflicts: &mut Vec<ConfigConflict>,
        preserved_edits: &mut Vec<PreservedEdit>,
        warnings: &mut Vec<MergeWarning>,
        auto_resolutions_count: &mut usize,
    ) -> Result<Value, AppError> {
        let ui_mapping = ui_camera.as_mapping()
            .ok_or_else(|| AppError::Config("UI camera config must be a mapping".to_string()))?;

        let manual_mapping = manual_camera.as_mapping()
            .ok_or_else(|| AppError::Config("Manual camera config must be a mapping".to_string()))?;

        let mut merged_mapping = ui_mapping.clone();

        // Merge camera-level fields
        let camera_fields = ["enabled", "best_image_timeout", "ffmpeg", "detect", "record", "snapshots", "objects", "motion"];

        for field in &camera_fields {
            let path = format!("cameras.{}.{}", camera_name, field);
            self.merge_yaml_field(
                &mut merged_mapping,
                ui_mapping,
                manual_mapping,
                field,
                &path,
                conflicts,
                preserved_edits,
                warnings,
                auto_resolutions_count,
            )?;
        }

        // Check for manual-only fields in camera
        for (key, value) in manual_mapping {
            if let Some(field_name) = key.as_str() {
                if !camera_fields.contains(&field_name) && !merged_mapping.contains_key(key) {
                    // Manual-only field
                    merged_mapping.insert(key.clone(), value.clone());
                    preserved_edits.push(PreservedEdit {
                        path: format!("cameras.{}.{}", camera_name, field_name),
                        value: value.clone(),
                        reason: "Field only exists in manual configuration".to_string(),
                        line_number: 0,
                    });
                }
            }
        }

        Ok(Value::Mapping(merged_mapping))
    }

    /// Merge detector configurations
    fn merge_detectors(
        &self,
        merged_yaml: &mut Value,
        ui_yaml: &Value,
        manual_yaml: &Value,
        conflicts: &mut Vec<ConfigConflict>,
        preserved_edits: &mut Vec<PreservedEdit>,
        warnings: &mut Vec<MergeWarning>,
        auto_resolutions_count: &mut usize,
    ) -> Result<(), AppError> {
        let ui_detectors = ui_yaml.get("detectors").and_then(|d| d.as_mapping());
        let manual_detectors = manual_yaml.get("detectors").and_then(|d| d.as_mapping());

        match (ui_detectors, manual_detectors) {
            (Some(ui_map), Some(manual_map)) => {
                let mut merged_detectors = Mapping::new();

                // Collect all detector names
                let mut all_detector_names = std::collections::HashSet::new();
                for key in ui_map.keys() {
                    if let Some(name) = key.as_str() {
                        all_detector_names.insert(name.to_string());
                    }
                }
                for key in manual_map.keys() {
                    if let Some(name) = key.as_str() {
                        all_detector_names.insert(name.to_string());
                    }
                }

                for detector_name in all_detector_names {
                    let ui_detector = ui_map.get(&Value::String(detector_name.clone()));
                    let manual_detector = manual_map.get(&Value::String(detector_name.clone()));

                    match (ui_detector, manual_detector) {
                        (Some(ui_det), Some(manual_det)) => {
                                            let merged_detector = self.merge_detector_config(
                                &detector_name,
                                ui_det,
                                manual_det,
                                conflicts,
                                preserved_edits,
                                warnings,
                                auto_resolutions_count,
                            )?;
                            merged_detectors.insert(Value::String(detector_name), merged_detector);
                        }
                        (Some(ui_det), None) => {
                            merged_detectors.insert(Value::String(detector_name), ui_det.clone());
                        }
                        (None, Some(manual_det)) => {
                            merged_detectors.insert(Value::String(detector_name.clone()), manual_det.clone());
                            preserved_edits.push(PreservedEdit {
                                path: format!("detectors.{}", detector_name),
                                value: manual_det.clone(),
                                reason: "Detector only exists in manual configuration".to_string(),
                                line_number: 0,
                            });
                        }
                        _ => {}
                    }
                }

                if let Some(merged) = merged_yaml.as_mapping_mut() {
                    merged.insert(Value::String("detectors".to_string()), Value::Mapping(merged_detectors));
                }
            }
            (Some(ui_map), None) => {
                if let Some(merged) = merged_yaml.as_mapping_mut() {
                    merged.insert(Value::String("detectors".to_string()), Value::Mapping(ui_map.clone()));
                }
            }
            (None, Some(manual_map)) => {
                if let Some(merged) = merged_yaml.as_mapping_mut() {
                    merged.insert(Value::String("detectors".to_string()), Value::Mapping(manual_map.clone()));
                }
                preserved_edits.push(PreservedEdit {
                    path: "detectors".to_string(),
                    value: Value::Mapping(manual_map.clone()),
                    reason: "All detectors only exist in manual configuration".to_string(),
                    line_number: 0,
                });
            }
            _ => {}
        }

        Ok(())
    }

    /// Merge a single detector configuration
    fn merge_detector_config(
        &self,
        detector_name: &str,
        ui_detector: &Value,
        manual_detector: &Value,
        conflicts: &mut Vec<ConfigConflict>,
        preserved_edits: &mut Vec<PreservedEdit>,
        warnings: &mut Vec<MergeWarning>,
        auto_resolutions_count: &mut usize,
    ) -> Result<Value, AppError> {
        let ui_mapping = ui_detector.as_mapping()
            .ok_or_else(|| AppError::Config("UI detector config must be a mapping".to_string()))?;

        let manual_mapping = manual_detector.as_mapping()
            .ok_or_else(|| AppError::Config("Manual detector config must be a mapping".to_string()))?;

        let mut merged_mapping = ui_mapping.clone();

        // Merge detector fields
        let detector_fields = ["type", "model", "model_path", "labelmap_path", "input_tensor", "input_pixel_format", "device"];

        for field in &detector_fields {
            let path = format!("detectors.{}.{}", detector_name, field);
            self.merge_yaml_field(
                &mut merged_mapping,
                ui_mapping,
                manual_mapping,
                field,
                &path,
                conflicts,
                preserved_edits,
                warnings,
                auto_resolutions_count,
            )?;
        }

        // Check for manual-only fields
        for (key, value) in manual_mapping {
            if let Some(field_name) = key.as_str() {
                if !detector_fields.contains(&field_name) && !merged_mapping.contains_key(key) {
                    merged_mapping.insert(key.clone(), value.clone());
                    preserved_edits.push(PreservedEdit {
                        path: format!("detectors.{}.{}", detector_name, field_name),
                        value: value.clone(),
                        reason: "Field only exists in manual configuration".to_string(),
                        line_number: 0,
                    });
                }
            }
        }

        Ok(Value::Mapping(merged_mapping))
    }

    /// Merge a YAML field between UI and manual configurations
    fn merge_yaml_field(
        &self,
        merged_mapping: &mut Mapping,
        ui_mapping: &Mapping,
        manual_mapping: &Mapping,
        field: &str,
        path: &str,
        conflicts: &mut Vec<ConfigConflict>,
        preserved_edits: &mut Vec<PreservedEdit>,
        warnings: &mut Vec<MergeWarning>,
        auto_resolutions_count: &mut usize,
    ) -> Result<(), AppError> {
        let field_key = Value::String(field.to_string());
        let ui_value = ui_mapping.get(&field_key);
        let manual_value = manual_mapping.get(&field_key);

        match (ui_value, manual_value) {
            (Some(ui_val), Some(manual_val)) => {
                // Field exists in both - check for conflicts
                if ui_val != manual_val {
                    // Conflict detected
                    let conflict = self.create_field_conflict(
                        field,
                        path,
                        Some(ui_val.clone()),
                        Some(manual_val.clone()),
                    );

                    conflicts.push(conflict.clone());

                    // Auto-resolve if possible
                    if self.auto_resolve_conflicts {
                        let resolution = self.auto_resolve_conflict(&conflict);
                        if let Some(resolved_value) = &resolution.suggested_value {
                            merged_mapping.insert(field_key, resolved_value.clone());
                            *auto_resolutions_count += 1;
                        }
                    } else if self.prefer_ui_values {
                        merged_mapping.insert(field_key, ui_val.clone());
                    } else {
                        // Keep UI value but note the conflict
                        merged_mapping.insert(field_key, ui_val.clone());
                    }
                } else {
                    // Same value - use it
                    merged_mapping.insert(field_key, ui_val.clone());
                }
            }
            (Some(ui_val), None) => {
                // Only in UI - use it
                merged_mapping.insert(field_key, ui_val.clone());
            }
            (None, Some(manual_val)) => {
                // Only in manual - preserve
                merged_mapping.insert(field_key, manual_val.clone());
                preserved_edits.push(PreservedEdit {
                    path: path.to_string(),
                    value: manual_val.clone(),
                    reason: format!("Field '{}' only exists in manual configuration", field),
                    line_number: 0,
                });
            }
            _ => {} // Neither has the field
        }

        Ok(())
    }

    /// Merge a YAML section
    fn merge_yaml_section(
        &self,
        merged_yaml: &mut Value,
        ui_yaml: &Value,
        manual_yaml: &Value,
        section_path: &str,
        conflicts: &mut Vec<ConfigConflict>,
        preserved_edits: &mut Vec<PreservedEdit>,
        warnings: &mut Vec<MergeWarning>,
        auto_resolutions_count: &mut usize,
    ) -> Result<(), AppError> {
        let ui_section = ui_yaml.get(section_path);
        let manual_section = manual_yaml.get(section_path);

        match (ui_section, manual_section) {
            (Some(ui_val), Some(manual_val)) => {
                if ui_val != manual_val {
                    // Conflict in section
                    let conflict = self.create_section_conflict(
                        section_path,
                        Some(ui_val.clone()),
                        Some(manual_val.clone()),
                    );
                    conflicts.push(conflict);

                    // Default to UI value if preference is set
                    if self.prefer_ui_values {
                        if let Some(merged) = merged_yaml.as_mapping_mut() {
                            merged.insert(Value::String(section_path.to_string()), ui_val.clone());
                        }
                    }
                } else {
                    // Same section - use it
                    if let Some(merged) = merged_yaml.as_mapping_mut() {
                        merged.insert(Value::String(section_path.to_string()), ui_val.clone());
                    }
                }
            }
            (Some(ui_val), None) => {
                // Only in UI
                if let Some(merged) = merged_yaml.as_mapping_mut() {
                    merged.insert(Value::String(section_path.to_string()), ui_val.clone());
                }
            }
            (None, Some(manual_val)) => {
                // Only in manual - preserve
                if let Some(merged) = merged_yaml.as_mapping_mut() {
                    merged.insert(Value::String(section_path.to_string()), manual_val.clone());
                }
                preserved_edits.push(PreservedEdit {
                    path: section_path.to_string(),
                    value: manual_val.clone(),
                    reason: format!("Section '{}' only exists in manual configuration", section_path),
                    line_number: 0,
                });
            }
            _ => {} // Neither has the section
        }

        Ok(())
    }

    /// Create a field conflict
    fn create_field_conflict(
        &self,
        field: &str,
        path: &str,
        ui_value: Option<Value>,
        manual_value: Option<Value>,
    ) -> ConfigConflict {
        let conflict_type = match (&ui_value, &manual_value) {
            (Some(_), Some(_)) => ConflictType::ValueMismatch,
            (Some(_), None) | (None, Some(_)) => ConflictType::MissingField,
            _ => ConflictType::StructuralDifference,
        };

        let severity = self.determine_conflict_severity(&conflict_type, field, path);

        let description = match &conflict_type {
            ConflictType::ValueMismatch => {
                format!("Field '{}' has different values in UI and manual configuration", field)
            }
            ConflictType::MissingField => {
                format!("Field '{}' exists in one configuration but not the other", field)
            }
            ConflictType::StructuralDifference => {
                format!("Field '{}' has structural differences between configurations", field)
            }
            _ => format!("Conflict in field '{}'", field),
        };

        let suggested_resolution = if self.auto_resolve_conflicts {
            self.auto_resolve_field_conflict(&conflict_type, &ui_value, &manual_value)
        } else {
            None
        };

        ConfigConflict {
            conflict_type,
            path: path.to_string(),
            ui_value,
            manual_value,
            line_number: None,
            severity,
            description,
            suggested_resolution,
        }
    }

    /// Create a section conflict
    fn create_section_conflict(
        &self,
        section_path: &str,
        ui_value: Option<Value>,
        manual_value: Option<Value>,
    ) -> ConfigConflict {
        let conflict_type = ConflictType::StructuralDifference;
        let severity = ConflictSeverity::Warning;

        let description = format!("Section '{}' has different configurations", section_path);

        ConfigConflict {
            conflict_type,
            path: section_path.to_string(),
            ui_value,
            manual_value,
            line_number: None,
            severity,
            description,
            suggested_resolution: None,
        }
    }

    /// Determine conflict severity based on field and type
    fn determine_conflict_severity(&self, conflict_type: &ConflictType, field: &str, path: &str) -> ConflictSeverity {
        match conflict_type {
            ConflictType::ValueMismatch => {
                // Critical fields that should not have conflicts
                if field == "enabled" || path.contains("detect.enabled") || path.contains("record.enabled") {
                    ConflictSeverity::Critical
                } else if path.contains("ffmpeg") || path.contains("model_path") {
                    ConflictSeverity::Warning
                } else {
                    ConflictSeverity::Info
                }
            }
            ConflictType::MissingField => {
                if field == "enabled" || field == "ffmpeg" {
                    ConflictSeverity::Critical
                } else {
                    ConflictSeverity::Warning
                }
            }
            ConflictType::StructuralDifference => ConflictSeverity::Warning,
            ConflictType::ValidationError => ConflictSeverity::Critical,
            ConflictType::ArrayDifference => ConflictSeverity::Warning,
            ConflictType::DuplicateItems => ConflictSeverity::Critical,
        }
    }

    /// Auto-resolve a field conflict
    fn auto_resolve_field_conflict(
        &self,
        conflict_type: &ConflictType,
        ui_value: &Option<Value>,
        manual_value: &Option<Value>,
    ) -> Option<ConflictResolution> {
        match conflict_type {
            ConflictType::ValueMismatch => {
                if self.prefer_ui_values {
                    Some(ConflictResolution {
                        resolution_type: ResolutionType::UseUiValue,
                        suggested_value: ui_value.clone(),
                        description: "Using UI-generated value".to_string(),
                        auto_applicable: true,
                    })
                } else {
                    Some(ConflictResolution {
                        resolution_type: ResolutionType::UseManualValue,
                        suggested_value: manual_value.clone(),
                        description: "Preserving manual configuration".to_string(),
                        auto_applicable: true,
                    })
                }
            }
            ConflictType::MissingField => {
                if ui_value.is_some() {
                    Some(ConflictResolution {
                        resolution_type: ResolutionType::UseUiValue,
                        suggested_value: ui_value.clone(),
                        description: "Adding field from UI configuration".to_string(),
                        auto_applicable: true,
                    })
                } else if manual_value.is_some() {
                    Some(ConflictResolution {
                        resolution_type: ResolutionType::UseManualValue,
                        suggested_value: manual_value.clone(),
                        description: "Preserving manual field".to_string(),
                        auto_applicable: true,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Auto-resolve a conflict
    fn auto_resolve_conflict(&self, conflict: &ConfigConflict) -> ConflictResolution {
        conflict.suggested_resolution.clone().unwrap_or(ConflictResolution {
            resolution_type: ResolutionType::ManualReview,
            suggested_value: None,
            description: "Manual review required".to_string(),
            auto_applicable: false,
        })
    }

    /// Preserve comments and formatting from manual config
    fn preserve_comments_and_formatting(
        &self,
        merged_yaml: &Value,
        manual_content: &str,
    ) -> Result<String, AppError> {
        // For now, just serialize the merged YAML
        // In a full implementation, we would preserve comments from the manual config
        serde_yaml::to_string(merged_yaml)
            .map_err(|e| AppError::Config(format!("Failed to serialize merged config: {}", e)))
    }

    /// Count total fields in a YAML value
    pub fn count_total_fields(yaml: &Value) -> usize {
        fn count_value(value: &Value) -> usize {
            match value {
                Value::Mapping(mapping) => {
                    mapping.iter().map(|(k, v)| count_value(k) + count_value(v)).sum()
                }
                Value::Sequence(sequence) => {
                    sequence.iter().map(count_value).sum()
                }
                _ => 1,
            }
        }
        count_value(&yaml)
    }

    /// Resolve conflicts automatically based on preferences
    pub fn resolve_conflicts(
        &self,
        merge_result: &mut MergeResult,
        resolutions: &HashMap<String, ConflictResolution>,
    ) -> Result<(), AppError> {
        info!("Resolving {} conflicts", merge_result.conflicts.len());

        let mut auto_resolutions_count = 0;

        for conflict in &mut merge_result.conflicts {
            if let Some(resolution) = resolutions.get(&conflict.path) {
                if resolution.auto_applicable {
                    // Apply the resolution
                    if let Some(suggested_value) = &resolution.suggested_value {
                        // Update the merged config with the resolved value
                        self.apply_resolution(&mut merge_result.merged_config, &conflict.path, suggested_value)?;
                        auto_resolutions_count += 1;
                    }
                }
            }
        }

        merge_result.statistics.auto_resolutions_count += auto_resolutions_count;
        info!("Auto-resolved {} conflicts", auto_resolutions_count);

        Ok(())
    }

    /// Apply a conflict resolution to the merged configuration
    fn apply_resolution(
        &self,
        config: &mut FrigateConfig,
        path: &str,
        value: &Value,
    ) -> Result<(), AppError> {
        // Parse the path and update the YAML structure
        let path_parts: Vec<&str> = path.split('.').collect();

        let mut current = &mut config.yaml;

        for (i, part) in path_parts.iter().enumerate() {
            if i == path_parts.len() - 1 {
                // Final part - set the value
                if let Some(mapping) = current.as_mapping_mut() {
                    mapping.insert(Value::String(part.to_string()), value.clone());
                }
            } else {
                // Navigate deeper
                current = match current.as_mapping_mut() {
                    Some(mapping) => {
                        mapping.entry(Value::String(part.to_string()))
                            .or_insert_with(|| Value::Mapping(Mapping::new()))
                    }
                    None => return Err(AppError::Config(format!("Invalid path: {}", path))),
                };
            }
        }

        Ok(())
    }

    /// Generate a preview of what the merged configuration would look like
    pub fn generate_merge_preview(
        &self,
        ui_config: &FrigateConfig,
        manual_config: &FrigateConfig,
    ) -> Result<MergePreview, AppError> {
        let merge_result = self.merge_configurations(ui_config, manual_config)?;

        Ok(MergePreview {
            summary: self.generate_summary(&merge_result),
            conflicts_preview: self.generate_conflicts_preview(&merge_result.conflicts),
            preserved_edits_preview: self.generate_preserved_edits_preview(&merge_result.preserved_edits),
            statistics: merge_result.statistics,
        })
    }

    /// Generate merge summary
    fn generate_summary(&self, merge_result: &MergeResult) -> String {
        format!(
            "Configuration merge complete. Found {} conflicts and preserved {} manual edits.",
            merge_result.conflicts.len(),
            merge_result.preserved_edits.len()
        )
    }

    /// Generate conflicts preview
    fn generate_conflicts_preview(&self, conflicts: &[ConfigConflict]) -> Vec<String> {
        conflicts.iter().map(|conflict| {
            format!("{}: {}", conflict.path, conflict.description)
        }).collect()
    }

    /// Generate preserved edits preview
    fn generate_preserved_edits_preview(&self, preserved_edits: &[PreservedEdit]) -> Vec<String> {
        preserved_edits.iter().map(|edit| {
            format!("{}: {}", edit.path, edit.reason)
        }).collect()
    }
}

/// Preview of merge results
#[derive(Debug, Clone)]
pub struct MergePreview {
    pub summary: String,
    pub conflicts_preview: Vec<String>,
    pub preserved_edits_preview: Vec<String>,
    pub statistics: MergeStatistics,
}

impl Default for ConfigMerger {
    fn default() -> Self {
        Self::new()
    }
}
