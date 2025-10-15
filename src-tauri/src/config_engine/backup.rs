// Configuration snapshot management system
// T056: Configuration version control and snapshot implementation

use crate::config_engine::parser::*;
use crate::error::AppError;
use crate::models::configuration_snapshot::*;
use chrono::Utc;
use serde_yaml::Value;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Configuration snapshot manager
pub struct SnapshotManager {
    /// Base directory for storing snapshots
    snapshots_dir: PathBuf,
    /// Maximum number of snapshots to retain
    max_snapshots: usize,
    /// Enable automatic snapshot creation
    auto_snapshot: bool,
}

/// Snapshot creation options
#[derive(Debug, Clone)]
pub struct SnapshotOptions {
    /// Description for the snapshot
    pub description: Option<String>,
    /// Creation source (UI, template, manual, etc.)
    pub creation_source: CreationSource,
    /// Mark as backup (not a user-visible change)
    pub is_backup: bool,
    /// Reason for creating backup
    pub backup_reason: Option<String>,
    /// Tags for the snapshot
    pub tags: Vec<String>,
}

/// Snapshot restore options
#[derive(Debug, Clone)]
pub struct RestoreOptions {
    /// Whether to validate the snapshot before restoring
    pub validate: bool,
    /// Whether to create a backup before restoring
    pub backup_before_restore: bool,
    /// Reason for restoration
    pub reason: Option<String>,
    /// Whether to merge or replace current configuration
    pub merge_changes: bool,
}

/// Snapshot comparison result
#[derive(Debug, Clone)]
pub struct SnapshotComparison {
    /// Whether snapshots are identical
    pub identical: bool,
    /// List of differences
    pub differences: Vec<ConfigDifference>,
    /// Statistics about differences
    pub statistics: ComparisonStatistics,
}

/// Configuration difference
#[derive(Debug, Clone)]
pub struct ConfigDifference {
    /// Path to the changed configuration
    pub path: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Value in snapshot A
    pub value_a: Option<Value>,
    /// Value in snapshot B
    pub value_b: Option<Value>,
    /// Description of the difference
    pub description: String,
}

/// Types of configuration changes
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    /// Field was added
    Added,
    /// Field was removed
    Removed,
    /// Field was modified
    Modified,
    /// Value type changed
    TypeChanged,
    /// Array order changed
    Reordered,
}

/// Comparison statistics
#[derive(Debug, Clone)]
pub struct ComparisonStatistics {
    /// Total number of fields compared
    pub total_fields: usize,
    /// Number of differences found
    pub differences_count: usize,
    /// Number of additions
    pub additions: usize,
    /// Number of removals
    pub removals: usize,
    /// Number of modifications
    pub modifications: usize,
    /// Number of reorders
    pub reorders: usize,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new<P: AsRef<Path>>(snapshots_dir: P) -> Result<Self, AppError> {
        let manager = Self {
            snapshots_dir: snapshots_dir.as_ref().to_path_buf(),
            max_snapshots: 50, // Default: retain 50 snapshots
            auto_snapshot: true,
        };

        manager.initialize()?;
        Ok(manager)
    }

    /// Initialize snapshot directory
    fn initialize(&self) -> Result<(), AppError> {
        if !self.snapshots_dir.exists() {
            std::fs::create_dir_all(&self.snapshots_dir).map_err(|e| {
                AppError::Config(format!("Failed to create snapshots directory: {}", e))
            })?;
        }

        // Create subdirectories
        let subdirs = ["configs", "metadata", "backups"];
        for subdir in &subdirs {
            let path = self.snapshots_dir.join(subdir);
            if !path.exists() {
                std::fs::create_dir_all(&path).map_err(|e| {
                    AppError::Config(format!("Failed to create directory {}: {}", subdir, e))
                })?;
            }
        }

        info!(
            "Initialized snapshot directory: {}",
            self.snapshots_dir.display()
        );
        Ok(())
    }

    /// Create a configuration snapshot
    pub async fn create_snapshot(
        &self,
        config: &FrigateConfig,
        options: SnapshotOptions,
    ) -> Result<ConfigurationSnapshot, AppError> {
        info!("Creating configuration snapshot: {:?}", options.description);

        let _timestamp = Utc::now();
        let version = self.get_next_version()?;

        // Generate YAML content
        let yaml_content = serde_yaml::to_string(&config.yaml)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {}", e)))?;

        // Calculate checksum
        let _checksum = ConfigurationSnapshot::calculate_checksum(&yaml_content);

        // Create snapshot ID
        let snapshot_id = format!("snapshot_{}", version);

        // Create snapshot
        let mut snapshot = ConfigurationSnapshot::new(
            snapshot_id,
            version as i32,
            yaml_content.clone(),
            options.creation_source,
        );

        // Mark as backup if specified
        if options.is_backup {
            snapshot.mark_as_backup(&options.backup_reason.unwrap_or_default());
        }

        // Metadata is set in the constructor

        // Save snapshot to disk
        let snapshot_path = self.save_snapshot(&snapshot, &yaml_content).await?;

        // Tags would need to be added to the model - skip for now

        // Cleanup old snapshots if necessary
        self.cleanup_old_snapshots().await?;

        info!(
            "Created snapshot {} at {}",
            snapshot.id,
            snapshot_path.display()
        );
        Ok(snapshot)
    }

    /// Load a configuration snapshot
    pub async fn load_snapshot(
        &self,
        snapshot_id: &str,
    ) -> Result<ConfigurationSnapshot, AppError> {
        info!("Loading snapshot: {}", snapshot_id);

        let snapshot_path = self.get_snapshot_path(snapshot_id)?;

        if !snapshot_path.exists() {
            return Err(AppError::Config(format!(
                "Snapshot {} not found",
                snapshot_id
            )));
        }

        // Read metadata file
        let metadata_path = self.get_metadata_path(snapshot_id)?;
        let metadata_content = tokio::fs::read_to_string(&metadata_path)
            .await
            .map_err(|e| AppError::Config(format!("Failed to read snapshot metadata: {}", e)))?;

        let mut snapshot: ConfigurationSnapshot = serde_json::from_str(&metadata_content)
            .map_err(|e| AppError::Config(format!("Failed to parse snapshot metadata: {}", e)))?;

        // Read YAML content
        let config_path = self.get_config_path(snapshot_id)?;
        let yaml_content = tokio::fs::read_to_string(&config_path)
            .await
            .map_err(|e| AppError::Config(format!("Failed to read config content: {}", e)))?;

        // Update snapshot with actual content
        snapshot.yaml_content = yaml_content;

        info!(
            "Loaded snapshot {} with {} bytes",
            snapshot_id,
            snapshot.yaml_content.len()
        );
        Ok(snapshot)
    }

    /// List all snapshots
    pub async fn list_snapshots(&self) -> Result<Vec<ConfigurationSnapshot>, AppError> {
        let mut snapshots = Vec::new();

        let entries = tokio::fs::read_dir(&self.snapshots_dir.join("metadata"))
            .await
            .map_err(|e| AppError::Config(format!("Failed to read snapshots directory: {}", e)))?;

        let mut entry_stream = entries;
        while let Some(entry) = entry_stream
            .next_entry()
            .await
            .map_err(|e| AppError::Config(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let snapshot_id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();

                match self.load_snapshot(snapshot_id).await {
                    Ok(snapshot) => snapshots.push(snapshot),
                    Err(e) => warn!("Failed to load snapshot {}: {}", snapshot_id, e),
                }
            }
        }

        // Sort by creation time (newest first)
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(snapshots)
    }

    /// Delete a snapshot
    pub async fn delete_snapshot(&self, snapshot_id: &str) -> Result<(), AppError> {
        info!("Deleting snapshot: {}", snapshot_id);

        let paths = [
            self.get_snapshot_path(snapshot_id)?,
            self.get_metadata_path(snapshot_id)?,
            self.get_config_path(snapshot_id)?,
        ];

        for path in &paths {
            if path.exists() {
                tokio::fs::remove_file(path).await.map_err(|e| {
                    AppError::Config(format!("Failed to delete {}: {}", path.display(), e))
                })?;
            }
        }

        info!("Deleted snapshot: {}", snapshot_id);
        Ok(())
    }

    /// Restore configuration from snapshot
    pub async fn restore_snapshot(
        &self,
        snapshot_id: &str,
        target_path: &Path,
        options: RestoreOptions,
    ) -> Result<RestoreResult, AppError> {
        info!(
            "Restoring from snapshot {} to {}",
            snapshot_id,
            target_path.display()
        );

        // Load the snapshot
        let snapshot = self.load_snapshot(snapshot_id).await?;

        // Validate snapshot if requested
        if options.validate {
            if let Err(e) = self.validate_snapshot(&snapshot).await {
                return Err(AppError::Config(format!(
                    "Snapshot validation failed: {}",
                    e
                )));
            }
        }

        // Create backup of current config if requested
        if options.backup_before_restore && target_path.exists() {
            let backup_reason = format!("Before restoring from snapshot {}", snapshot_id);
            self.create_backup_of_file(target_path, &backup_reason)
                .await?;
        }

        // Perform restoration
        let restore_result = if options.merge_changes {
            self.merge_restore_from_snapshot(&snapshot, target_path)
                .await?
        } else {
            self.replace_restore_from_snapshot(&snapshot, target_path)
                .await?
        };

        info!("Successfully restored from snapshot {}", snapshot_id);
        Ok(restore_result)
    }

    /// Compare two snapshots
    pub async fn compare_snapshots(
        &self,
        snapshot_a_id: &str,
        snapshot_b_id: &str,
    ) -> Result<SnapshotComparison, AppError> {
        info!(
            "Comparing snapshots {} and {}",
            snapshot_a_id, snapshot_b_id
        );

        let snapshot_a = self.load_snapshot(snapshot_a_id).await?;
        let snapshot_b = self.load_snapshot(snapshot_b_id).await?;

        // Parse YAML structures
        let yaml_a: Value = serde_yaml::from_str(&snapshot_a.yaml_content)
            .map_err(|e| AppError::Config(format!("Failed to parse snapshot A: {}", e)))?;
        let yaml_b: Value = serde_yaml::from_str(&snapshot_b.yaml_content)
            .map_err(|e| AppError::Config(format!("Failed to parse snapshot B: {}", e)))?;

        // Compare configurations
        let differences = self.compare_yaml_values(&yaml_a, &yaml_b, "");
        let statistics = self.calculate_comparison_statistics(&differences);
        let identical = differences.is_empty();

        Ok(SnapshotComparison {
            identical,
            differences,
            statistics,
        })
    }

    /// Get next available version number
    fn get_next_version(&self) -> Result<u32, AppError> {
        // For now, use timestamp as version
        // In a more sophisticated implementation, this would check existing snapshots
        Ok(Utc::now().timestamp() as u32)
    }

    /// Save snapshot to disk
    async fn save_snapshot(
        &self,
        snapshot: &ConfigurationSnapshot,
        yaml_content: &str,
    ) -> Result<PathBuf, AppError> {
        let snapshot_id = &snapshot.id;

        // Save YAML content
        let config_path = self.get_config_path(snapshot_id)?;
        tokio::fs::write(&config_path, yaml_content)
            .await
            .map_err(|e| AppError::Config(format!("Failed to write config: {}", e)))?;

        // Save metadata
        let metadata_path = self.get_metadata_path(snapshot_id)?;
        let metadata_json = serde_json::to_string(snapshot)
            .map_err(|e| AppError::Config(format!("Failed to serialize metadata: {}", e)))?;
        tokio::fs::write(&metadata_path, metadata_json)
            .await
            .map_err(|e| AppError::Config(format!("Failed to write metadata: {}", e)))?;

        Ok(config_path)
    }

    /// Get snapshot file path
    fn get_snapshot_path(&self, snapshot_id: &str) -> Result<PathBuf, AppError> {
        Ok(self
            .snapshots_dir
            .join("configs")
            .join(format!("{}.yaml", snapshot_id)))
    }

    /// Get metadata file path
    fn get_metadata_path(&self, snapshot_id: &str) -> Result<PathBuf, AppError> {
        Ok(self
            .snapshots_dir
            .join("metadata")
            .join(format!("{}.json", snapshot_id)))
    }

    /// Get config file path
    fn get_config_path(&self, snapshot_id: &str) -> Result<PathBuf, AppError> {
        Ok(self
            .snapshots_dir
            .join("configs")
            .join(format!("{}.yaml", snapshot_id)))
    }

    /// Cleanup old snapshots
    async fn cleanup_old_snapshots(&self) -> Result<(), AppError> {
        let snapshots = self.list_snapshots().await?;

        if snapshots.len() > self.max_snapshots {
            // Sort by creation time (oldest first)
            let mut sorted_snapshots = snapshots;
            sorted_snapshots.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            // Remove oldest snapshots
            let to_remove = sorted_snapshots.len() - self.max_snapshots;
            for snapshot in sorted_snapshots.iter().take(to_remove) {
                if !snapshot.is_backup {
                    self.delete_snapshot(&snapshot.id).await?;
                    info!("Removed old snapshot: {}", snapshot.id);
                }
            }
        }

        Ok(())
    }

    /// Validate a snapshot
    async fn validate_snapshot(&self, snapshot: &ConfigurationSnapshot) -> Result<(), AppError> {
        // Verify checksum
        if !snapshot.verify_checksum() {
            return Err(AppError::Config(
                "Snapshot checksum verification failed".to_string(),
            ));
        }

        // Validate YAML structure
        let _: Value = serde_yaml::from_str(&snapshot.yaml_content)
            .map_err(|e| AppError::Config(format!("Invalid YAML in snapshot: {}", e)))?;

        // Validate Frigate-specific structure
        let parser = ConfigParser::new();
        let _ =
            parser.parse_content(snapshot.yaml_content.clone(), "validation.yaml".to_string())?;

        Ok(())
    }

    /// Create backup of existing file
    async fn create_backup_of_file(
        &self,
        file_path: &Path,
        reason: &str,
    ) -> Result<PathBuf, AppError> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("backup_{}.yaml", timestamp);
        let backup_path = self.snapshots_dir.join("backups").join(backup_name);

        tokio::fs::copy(file_path, &backup_path)
            .await
            .map_err(|e| AppError::Config(format!("Failed to create backup: {}", e)))?;

        // Create backup metadata
        let backup_info = serde_json::json!({
            "original_path": file_path,
            "backup_path": backup_path,
            "reason": reason,
            "created_at": Utc::now().to_rfc3339(),
            "size": backup_path.metadata().ok().map(|m| m.len()).unwrap_or(0)
        });

        let metadata_path = backup_path.with_extension("json");
        tokio::fs::write(&metadata_path, backup_info.to_string())
            .await
            .map_err(|e| AppError::Config(format!("Failed to write backup metadata: {}", e)))?;

        info!(
            "Created backup at {} for reason: {}",
            backup_path.display(),
            reason
        );
        Ok(backup_path)
    }

    /// Perform replace restoration from snapshot
    async fn replace_restore_from_snapshot(
        &self,
        snapshot: &ConfigurationSnapshot,
        target_path: &Path,
    ) -> Result<RestoreResult, AppError> {
        // Write snapshot content to target file
        tokio::fs::write(target_path, &snapshot.yaml_content)
            .await
            .map_err(|e| AppError::Config(format!("Failed to restore config: {}", e)))?;

        Ok(RestoreResult {
            success: true,
            message: format!("Configuration restored from snapshot {}", snapshot.id),
            backup_path: None,
            changes_count: self.count_config_fields(&snapshot.yaml_content),
        })
    }

    /// Perform merge restoration from snapshot
    async fn merge_restore_from_snapshot(
        &self,
        snapshot: &ConfigurationSnapshot,
        target_path: &Path,
    ) -> Result<RestoreResult, AppError> {
        // For now, implement as replace (merge would require more complex logic)
        self.replace_restore_from_snapshot(snapshot, target_path)
            .await
    }

    /// Compare two YAML values recursively
    fn compare_yaml_values(
        &self,
        value_a: &Value,
        value_b: &Value,
        current_path: &str,
    ) -> Vec<ConfigDifference> {
        let mut differences = Vec::new();

        match (value_a, value_b) {
            (Value::Mapping(map_a), Value::Mapping(map_b)) => {
                // Compare mappings
                let mut all_keys = std::collections::HashSet::new();
                for key in map_a.keys() {
                    if let Some(key_str) = key.as_str() {
                        all_keys.insert(key_str.to_string());
                    }
                }
                for key in map_b.keys() {
                    if let Some(key_str) = key.as_str() {
                        all_keys.insert(key_str.to_string());
                    }
                }

                for key in all_keys {
                    let key_path = if current_path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", current_path, key)
                    };

                    let key_value = Value::String(key.clone());
                    let val_a = map_a.get(&key_value);
                    let val_b = map_b.get(&key_value);

                    match (val_a, val_b) {
                        (Some(a), Some(b)) => {
                            if a != b {
                                if self.is_different_type(a, b) {
                                    differences.push(ConfigDifference {
                                        path: key_path.clone(),
                                        change_type: ChangeType::TypeChanged,
                                        value_a: Some(a.clone()),
                                        value_b: Some(b.clone()),
                                        description: format!("Field '{}' changed type", key_path),
                                    });
                                } else {
                                    differences.push(ConfigDifference {
                                        path: key_path.clone(),
                                        change_type: ChangeType::Modified,
                                        value_a: Some(a.clone()),
                                        value_b: Some(b.clone()),
                                        description: format!("Field '{}' modified", key_path),
                                    });
                                }
                            }
                            // Recursively compare nested structures
                            differences.extend(self.compare_yaml_values(a, b, &key_path));
                        }
                        (Some(_), None) => {
                            differences.push(ConfigDifference {
                                path: key_path.clone(),
                                change_type: ChangeType::Removed,
                                value_a: val_a.cloned(),
                                value_b: None,
                                description: format!("Field '{}' removed", key_path),
                            });
                        }
                        (None, Some(_)) => {
                            differences.push(ConfigDifference {
                                path: key_path.clone(),
                                change_type: ChangeType::Added,
                                value_a: None,
                                value_b: val_b.cloned(),
                                description: format!("Field '{}' added", key_path),
                            });
                        }
                        _ => {}
                    }
                }
            }
            (Value::Sequence(seq_a), Value::Sequence(seq_b)) => {
                // Compare sequences
                if seq_a != seq_b {
                    if seq_a.len() != seq_b.len() {
                        differences.push(ConfigDifference {
                            path: current_path.to_string(),
                            change_type: ChangeType::Modified,
                            value_a: Some(value_a.clone()),
                            value_b: Some(value_b.clone()),
                            description: format!(
                                "Array '{}' length changed from {} to {}",
                                current_path,
                                seq_a.len(),
                                seq_b.len()
                            ),
                        });
                    } else {
                        differences.push(ConfigDifference {
                            path: current_path.to_string(),
                            change_type: ChangeType::Modified,
                            value_a: Some(value_a.clone()),
                            value_b: Some(value_b.clone()),
                            description: format!("Array '{}' content changed", current_path),
                        });
                    }
                }
            }
            (a, b) if a != b => {
                differences.push(ConfigDifference {
                    path: current_path.to_string(),
                    change_type: ChangeType::Modified,
                    value_a: Some(a.clone()),
                    value_b: Some(b.clone()),
                    description: format!("Value '{}' changed", current_path),
                });
            }
            _ => {} // Values are equal
        }

        differences
    }

    /// Check if two values have different types
    fn is_different_type(&self, a: &Value, b: &Value) -> bool {
        use serde_yaml::Value::*;
        !matches!(
            (a, b),
            (Mapping(_), Mapping(_)) | (Sequence(_), Sequence(_))
        )
    }

    /// Calculate comparison statistics
    fn calculate_comparison_statistics(
        &self,
        differences: &[ConfigDifference],
    ) -> ComparisonStatistics {
        let mut stats = ComparisonStatistics {
            total_fields: 0, // Would need to be calculated from original YAML
            differences_count: differences.len(),
            additions: 0,
            removals: 0,
            modifications: 0,
            reorders: 0,
        };

        for diff in differences {
            match diff.change_type {
                ChangeType::Added => stats.additions += 1,
                ChangeType::Removed => stats.removals += 1,
                ChangeType::Modified => stats.modifications += 1,
                ChangeType::Reordered => stats.reorders += 1,
                _ => {}
            }
        }

        stats
    }

    /// Count fields in configuration
    fn count_config_fields(&self, yaml_content: &str) -> usize {
        let yaml: Value = serde_yaml::from_str(yaml_content).unwrap_or(Value::Null);
        self.count_value_fields(&yaml)
    }

    /// Count fields in YAML value
    fn count_value_fields(&self, value: &Value) -> usize {
        match value {
            Value::Mapping(mapping) => mapping
                .iter()
                .map(|(k, v)| self.count_value_fields(k) + self.count_value_fields(v))
                .sum(),
            Value::Sequence(sequence) => sequence.iter().map(|v| self.count_value_fields(v)).sum(),
            _ => 1,
        }
    }

    /// Get snapshot statistics
    pub async fn get_statistics(&self) -> Result<SnapshotStatistics, AppError> {
        let snapshots = self.list_snapshots().await?;

        let total_size = snapshots
            .iter()
            .map(|s| s.yaml_content.len())
            .sum::<usize>();

        let oldest = snapshots.iter().min_by_key(|s| s.created_at.clone());
        let newest = snapshots.iter().max_by_key(|s| s.created_at.clone());

        Ok(SnapshotStatistics {
            total_snapshots: snapshots.len(),
            total_size_bytes: total_size,
            oldest_snapshot: oldest.map(|s| s.created_at.clone()),
            newest_snapshot: newest.map(|s| s.created_at.clone()),
            backup_snapshots: snapshots.iter().filter(|s| s.is_backup).count(),
            regular_snapshots: snapshots.iter().filter(|s| !s.is_backup).count(),
        })
    }

    /// Set maximum snapshots to retain
    pub fn set_max_snapshots(&mut self, max: usize) {
        self.max_snapshots = max;
    }

    /// Enable/disable automatic snapshot creation
    pub fn set_auto_snapshot(&mut self, enabled: bool) {
        self.auto_snapshot = enabled;
    }

    /// Get snapshots directory path
    pub fn snapshots_dir(&self) -> &Path {
        &self.snapshots_dir
    }
}

/// Restore operation result
#[derive(Debug, Clone)]
pub struct RestoreResult {
    pub success: bool,
    pub message: String,
    pub backup_path: Option<PathBuf>,
    pub changes_count: usize,
}

/// Snapshot statistics
#[derive(Debug, Clone)]
pub struct SnapshotStatistics {
    pub total_snapshots: usize,
    pub total_size_bytes: usize,
    pub oldest_snapshot: Option<String>,
    pub newest_snapshot: Option<String>,
    pub backup_snapshots: usize,
    pub regular_snapshots: usize,
}

impl Default for SnapshotManager {
    fn default() -> Self {
        // Use default snapshots directory
        let snapshots_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("snapshots");

        Self::new(snapshots_dir).unwrap_or_else(|_| Self {
            snapshots_dir: PathBuf::from("snapshots"),
            max_snapshots: 50,
            auto_snapshot: true,
        })
    }
}
