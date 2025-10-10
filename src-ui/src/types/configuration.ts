// Type definitions for configuration management
// Shared types for UI components

// Configuration data structures
export interface ConfigurationData {
  file_path: string;
  content: string;
  version?: number;
  timestamp?: string;
}

// Conflict types
export interface Conflict {
  path: string;
  conflict_type: string;
  severity: string;
  description: string;
  ui_value: any;
  manual_value: any;
  suggested_resolution?: {
    resolution_type: string;
    description: string;
    auto_applicable: boolean;
  };
}

export interface ConflictResolution {
  path: string;
  resolutionType: string;
  description: string;
  suggestedValue?: any;
  autoApplicable: boolean;
}

// Preserved edits
export interface PreservedEdit {
  path: string;
  reason: string;
  line_number: number;
}

// Merge statistics
export interface ConflictStatistics {
  total_fields: number;
  conflicts_count: number;
  preserved_edits_count: number;
  warnings_count: number;
  auto_resolutions_count: number;
}

// Merge result
export interface MergeResult {
  merged_config: ConfigurationData;
  conflicts: Conflict[];
  preserved_edits: PreservedEdit[];
  statistics: ConflictStatistics;
  success: boolean;
}

// Snapshot/Backup types
export interface Snapshot {
  id: string;
  version: number;
  timestamp: string;
  created_at: string;
  trigger: string;
  description?: string;
  file_path: string;
  file_size: number;
  checksum: string;
  metadata?: SnapshotMetadata;
}

export interface SnapshotMetadata {
  frigate_version?: string;
  camera_count?: number;
  detector_type?: string;
  [key: string]: any;
}

// Validation types
export interface ValidationError {
  message: string;
  line_number?: number;
  field_path?: string;
  severity: 'error' | 'warning';
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationError[];
}

// Tauri command request types
export interface SaveConfigRequest {
  file_path: string;
  content: string;
  create_snapshot: boolean;
  snapshot_description?: string;
}

export interface MergeConfigRequest {
  ui_config: ConfigurationData;
  manual_config: ConfigurationData;
}

export interface ResolveConflictsRequest {
  ui_config: ConfigurationData;
  manual_config: ConfigurationData;
  resolutions: Array<{
    path: string;
    resolution_type: string;
    description: string;
    suggested_value: string | null;
    auto_applicable: boolean;
  }>;
}

export interface RestoreSnapshotRequest {
  snapshot_id: string;
  target_path: string;
  validate: boolean;
  backup_before_restore: boolean;
  reason: string;
  merge_changes: boolean;
}

// Rollback types
export interface RollbackPreview {
  config_content: string;
  metadata: {
    created_at: string;
    version: number;
    trigger: string;
    description?: string;
  };
}

export interface RollbackHistoryEntry {
  backup_id: string;
  action: string;
  timestamp: string;
  success: boolean;
  reason?: string;
}

// Comparison types
export interface ComparisonResult {
  differences: Array<{
    path: string;
    old_value: any;
    new_value: any;
    change_type: 'added' | 'removed' | 'modified';
  }>;
  statistics: {
    additions: number;
    deletions: number;
    modifications: number;
  };
}
