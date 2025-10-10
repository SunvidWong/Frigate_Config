// Configuration API service
// Provides interface to Tauri configuration management commands

import { invoke } from '@tauri-apps/api/tauri'

export interface ConfigurationData {
  cameras: Record<string, any>
  detectors: Record<string, any>
  global_config: Record<string, any>
  raw_yaml: string
  file_path: string
  parsed_at: string
  warnings: ParseWarning[]
  errors: ParseError[]
}

export interface ParseWarning {
  line: number
  message: string
  code: string
}

export interface ParseError {
  line: number
  message: string
  code: string
  error_type: string
}

export interface SaveConfigRequest {
  config_data: ConfigurationData
  target_path?: string
  create_backup?: boolean
  validate_before_save?: boolean
}

export interface SaveConfigResponse {
  success: boolean
  saved_path: string
  backup_path?: string
  validation_errors: ValidationError[]
  warnings: string[]
}

export interface ValidationError {
  field: string
  message: string
  severity: 'error' | 'warning'
}

export interface MergeConfigurationsRequest {
  base_config: string
  incoming_config: string
  auto_resolve?: boolean
  resolution_strategy?: 'manual' | 'auto_prefer_base' | 'auto_prefer_incoming'
}

export interface MergeResultResponse {
  merged_config: string
  conflicts: ConfigConflict[]
  warnings: string[]
  auto_resolved: number
  manual_resolution_required: number
}

export interface ConfigConflict {
  path: string
  base_value: any
  incoming_value: any
  conflict_type: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  suggestion?: string
  description: string
}

export interface SnapshotRequest {
  config_data: ConfigurationData
  description?: string
  is_backup?: boolean
  backup_reason?: string
}

export interface ConfigurationSnapshot {
  id: string
  version: number
  yaml_content: string
  checksum: string
  created_at: string
  created_by: string
  description?: string
  is_backup: boolean
  backup_reason?: string
  deployed: boolean
  deployed_at?: string
  deployment_success?: boolean
}

export interface SnapshotListResponse {
  snapshots: ConfigurationSnapshot[]
  total_count: number
}

export interface RestoreSnapshotRequest {
  snapshot_id: string
  target_path: string
  create_backup_before_restore?: boolean
}

export interface CompareSnapshotsResponse {
  differences: ConfigDifference[]
  summary: {
    total_changes: number
    additions: number
    modifications: number
    deletions: number
  }
}

export interface ConfigDifference {
  path: string
  type: 'added' | 'modified' | 'deleted'
  old_value?: any
  new_value?: any
  section: string
}

export interface SnapshotStatisticsResponse {
  total_snapshots: number
  backup_snapshots: number
  manual_snapshots: number
  oldest_snapshot: string
  newest_snapshot: string
  total_size_bytes: number
  average_snapshots_per_day: number
}

export interface ValidationResult {
  valid: boolean
  errors: ValidationError[]
  warnings: string[]
  config_type: string
  schema_version: string
}

export interface ConflictResolutionRequest {
  merge_result_id: string
  conflicts: ResolvedConflict[]
}

export interface ResolvedConflict {
  path: string
  resolution: 'use_base' | 'use_incoming' | 'use_custom'
  custom_value?: any
}

export interface ConflictResolutionResponse {
  success: boolean
  resolved_config: string
  resolved_conflicts: number
  remaining_conflicts: number
}

class ConfigService {
  // Load configuration from file
  async loadConfig(filePath: string): Promise<ConfigurationData> {
    return await invoke('load_config', { filePath })
  }

  // Save configuration to file
  async saveConfig(request: SaveConfigRequest): Promise<SaveConfigResponse> {
    return await invoke('save_config', { configRequest: request })
  }

  // Merge two configurations
  async mergeConfigurations(request: MergeConfigurationsRequest): Promise<MergeResultResponse> {
    return await invoke('merge_configurations', { mergeRequest: request })
  }

  // Resolve conflicts in merged configuration
  async resolveConflicts(request: ConflictResolutionRequest): Promise<ConflictResolutionResponse> {
    return await invoke('resolve_conflicts', { resolutionRequest: request })
  }

  // Create a snapshot
  async createSnapshot(request: SnapshotRequest): Promise<ConfigurationSnapshot> {
    return await invoke('create_snapshot', { snapshotRequest: request })
  }

  // List all snapshots
  async listSnapshots(): Promise<SnapshotListResponse> {
    return await invoke('list_snapshots')
  }

  // Restore from snapshot
  async restoreSnapshot(request: RestoreSnapshotRequest): Promise<SaveConfigResponse> {
    return await invoke('restore_snapshot', { restoreRequest: request })
  }

  // Delete a snapshot
  async deleteSnapshot(snapshotId: string): Promise<boolean> {
    return await invoke('delete_snapshot', { snapshotId })
  }

  // Compare two snapshots
  async compareSnapshots(snapshotId1: string, snapshotId2: string): Promise<CompareSnapshotsResponse> {
    return await invoke('compare_snapshots', { snapshotId1, snapshotId2 })
  }

  // Get snapshot statistics
  async getSnapshotStatistics(): Promise<SnapshotStatisticsResponse> {
    return await invoke('get_snapshot_statistics')
  }

  // Validate configuration
  async validateConfig(configYaml: string): Promise<ValidationResult> {
    return await invoke('validate_config', { configYaml })
  }
}

export const configService = new ConfigService()
export default configService