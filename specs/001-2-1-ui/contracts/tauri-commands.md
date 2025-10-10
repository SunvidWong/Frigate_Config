# Tauri IPC Commands Contract

**Version**: 1.0.0
**Date**: 2025-10-08
**Purpose**: Define all Tauri commands exposed from Rust backend to TypeScript frontend

## Overview

Tauri commands are defined in Rust with `#[tauri::command]` attribute and invoked from frontend via `invoke('command_name', { args })`. All commands return `Result<T, String>` where `String` is the error message.

---

## Hardware Detection Commands

### `detect_hardware`

Invokes the Agent binary and returns detected hardware devices.

**Frontend Invocation**:
```typescript
const devices = await invoke<HardwareDetectionResponse>('detect_hardware');
```

**Rust Signature**:
```rust
#[tauri::command]
async fn detect_hardware() -> Result<HardwareDetectionResponse, String>
```

**Response**: `HardwareDetectionResponse` (see `agent-api.json` schema)

**Errors**:
- `"Agent binary not found"` - Agent executable missing
- `"Agent execution failed: {error}"` - Agent process error
- `"Invalid JSON output from agent"` - Agent returned malformed JSON
- `"Agent timeout after 10 seconds"` - Detection took too long

---

### `get_device_details`

Retrieves detailed information about a specific device path.

**Frontend Invocation**:
```typescript
const details = await invoke<DeviceDetails>('get_device_details', {
  devicePath: '/dev/video0'
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn get_device_details(device_path: String) -> Result<DeviceDetails, String>
```

**Request**:
```typescript
interface GetDeviceDetailsRequest {
  devicePath: string;  // Device path to query
}
```

**Response**:
```typescript
interface DeviceDetails {
  device_path: string;
  exists: boolean;
  readable: boolean;
  writable: boolean;
  device_type: string | null;  // "char", "block", null
  permissions: string;          // "0660", etc.
  owner: string;
  group: string;
}
```

**Errors**:
- `"Invalid device path"` - Path validation failed
- `"Permission denied"` - Cannot stat device

---

## Configuration Engine Commands

### `load_configuration`

Loads a Frigate YAML configuration file.

**Frontend Invocation**:
```typescript
const snapshot = await invoke<ConfigurationSnapshot>('load_configuration', {
  filePath: '/home/user/.config/frigate.yml'
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn load_configuration(file_path: String) -> Result<ConfigurationSnapshot, String>
```

**Request**:
```typescript
interface LoadConfigurationRequest {
  filePath: string;  // Absolute path to YAML file
}
```

**Response**: `ConfigurationSnapshot` (see `data-model.md`)

**Errors**:
- `"File not found"` - File doesn't exist
- `"Invalid YAML syntax: {error}"` - Parse error with line number
- `"Permission denied"` - Cannot read file

---

### `save_configuration`

Saves a configuration to a YAML file with automatic backup.

**Frontend Invocation**:
```typescript
await invoke('save_configuration', {
  filePath: '/home/user/.config/frigate.yml',
  yamlContent: '...',
  createBackup: true
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn save_configuration(
    file_path: String,
    yaml_content: String,
    create_backup: bool
) -> Result<SaveConfigurationResponse, String>
```

**Request**:
```typescript
interface SaveConfigurationRequest {
  filePath: string;
  yamlContent: string;
  createBackup: boolean;
}
```

**Response**:
```typescript
interface SaveConfigurationResponse {
  saved: boolean;
  backup_path: string | null;  // Path to backup file if created
  checksum: string;             // SHA-256 of saved content
}
```

**Errors**:
- `"Invalid YAML syntax"` - Content validation failed
- `"Permission denied"` - Cannot write file
- `"Backup creation failed: {error}"` - Backup failed but save succeeded

---

### `merge_template`

Merges a template into existing configuration with conflict detection.

**Frontend Invocation**:
```typescript
const result = await invoke<MergeResult>('merge_template', {
  currentYaml: '...',
  templateYaml: '...',
  strategy: 'recursive'
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn merge_template(
    current_yaml: String,
    template_yaml: String,
    strategy: String
) -> Result<MergeResult, String>
```

**Request**:
```typescript
interface MergeTemplateRequest {
  currentYaml: string;
  templateYaml: string;
  strategy: 'recursive' | 'template_wins' | 'user_wins';
}
```

**Response**:
```typescript
interface MergeResult {
  merged_yaml: string | null;      // null if conflicts need resolution
  conflicts: ConflictResolution[]; // Empty if no conflicts
  auto_merged: boolean;            // True if merged without conflicts
}
```

**Errors**:
- `"Invalid current YAML"` - Current config parse error
- `"Invalid template YAML"` - Template parse error

---

### `resolve_conflicts`

Applies user choices to resolve configuration conflicts.

**Frontend Invocation**:
```typescript
const resolved = await invoke<string>('resolve_conflicts', {
  baseYaml: '...',
  conflicts: [...],
  resolutions: [...]
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn resolve_conflicts(
    base_yaml: String,
    conflicts: Vec<ConflictResolution>,
    resolutions: Vec<UserResolution>
) -> Result<String, String>  // Returns merged YAML
```

**Request**:
```typescript
interface ResolveConflictsRequest {
  baseYaml: string;
  conflicts: ConflictResolution[];
  resolutions: UserResolution[];
}

interface UserResolution {
  conflict_id: string;
  resolution: 'keep_existing' | 'use_template' | 'custom';
  custom_value?: any;
}
```

**Response**: `string` (merged YAML content)

**Errors**:
- `"Conflict ID not found: {id}"` - Invalid conflict reference
- `"Custom value validation failed"` - Invalid custom value

---

### `list_backups`

Lists available configuration backups.

**Frontend Invocation**:
```typescript
const backups = await invoke<ConfigurationBackup[]>('list_backups');
```

**Rust Signature**:
```rust
#[tauri::command]
async fn list_backups() -> Result<Vec<ConfigurationBackup>, String>
```

**Response**:
```typescript
interface ConfigurationBackup {
  id: string;
  version: number;
  file_path: string;
  created_at: string;  // ISO 8601
  description: string | null;
  size_bytes: number;
}
```

---

### `restore_backup`

Restores a configuration from backup.

**Frontend Invocation**:
```typescript
const snapshot = await invoke<ConfigurationSnapshot>('restore_backup', {
  backupId: 'abc-123'
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn restore_backup(backup_id: String) -> Result<ConfigurationSnapshot, String>
```

**Errors**:
- `"Backup not found"` - Invalid backup ID
- `"Backup file corrupted"` - Checksum mismatch

---

## Deployment Commands

### `validate_deployment`

Runs pre-deployment checks.

**Frontend Invocation**:
```typescript
const result = await invoke<ValidationResult>('validate_deployment', {
  yamlContent: '...',
  volumeMappings: [...],
  devicePaths: [...]
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn validate_deployment(
    yaml_content: String,
    volume_mappings: Vec<VolumeMapping>,
    device_paths: Vec<String>
) -> Result<ValidationResult, String>
```

**Request**:
```typescript
interface ValidateDeploymentRequest {
  yamlContent: string;
  volumeMappings: VolumeMapping[];
  devicePaths: string[];
}
```

**Response**:
```typescript
interface ValidationResult {
  valid: boolean;
  checks: ValidationCheck[];
}

interface ValidationCheck {
  name: string;             // "docker_available", "yaml_valid", "ports_free"
  passed: boolean;
  message: string;
  severity: 'error' | 'warning' | 'info';
}
```

---

### `generate_docker_command`

Generates Docker run/compose command.

**Frontend Invocation**:
```typescript
const command = await invoke<DockerCommand>('generate_docker_command', {
  method: 'docker_run',
  configPath: '/path/to/frigate.yml',
  volumeMappings: [...],
  devicePaths: [...],
  ports: { http: 5000, rtsp: 8554 }
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn generate_docker_command(
    method: String,
    config_path: String,
    volume_mappings: Vec<VolumeMapping>,
    device_paths: Vec<String>,
    ports: PortMapping
) -> Result<DockerCommand, String>
```

**Request**:
```typescript
interface GenerateDockerCommandRequest {
  method: 'docker_run' | 'docker_compose';
  configPath: string;
  volumeMappings: VolumeMapping[];
  devicePaths: string[];
  ports: PortMapping;
}

interface PortMapping {
  http: number;
  rtsp: number;
  webrtc?: number;
}
```

**Response**:
```typescript
interface DockerCommand {
  method: 'docker_run' | 'docker_compose';
  command: string;               // For docker_run
  compose_content: string | null; // For docker_compose
  working_dir: string | null;     // For docker_compose
}
```

---

### `execute_deployment`

Executes a deployment.

**Frontend Invocation**:
```typescript
const state = await invoke<DeploymentState>('execute_deployment', {
  command: 'docker run ...',
  snapshotId: 'abc-123'
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn execute_deployment(
    command: String,
    snapshot_id: String
) -> Result<DeploymentState, String>
```

**Request**:
```typescript
interface ExecuteDeploymentRequest {
  command: string;
  snapshotId: string;
}
```

**Response**: `DeploymentState` (see `data-model.md`)

**Errors**:
- `"Docker not available"` - Docker command not found
- `"Deployment failed: {error}"` - Docker command failed

---

### `check_deployment_health`

Checks health status of deployed Frigate instance.

**Frontend Invocation**:
```typescript
const health = await invoke<HealthCheckResult>('check_deployment_health', {
  deploymentId: 'dep-123'
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn check_deployment_health(deployment_id: String) -> Result<HealthCheckResult, String>
```

**Response**:
```typescript
interface HealthCheckResult {
  deployment_id: string;
  container_id: string;
  container_running: boolean;
  health_endpoint_reachable: boolean;
  frigate_version: string | null;
  status: 'healthy' | 'unhealthy' | 'starting';
  last_check_at: string;  // ISO 8601
}
```

---

### `rollback_deployment`

Rolls back to previous deployment state.

**Frontend Invocation**:
```typescript
const state = await invoke<DeploymentState>('rollback_deployment', {
  currentDeploymentId: 'dep-123',
  targetDeploymentId: 'dep-100'
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn rollback_deployment(
    current_deployment_id: String,
    target_deployment_id: String
) -> Result<DeploymentState, String>
```

**Errors**:
- `"Target deployment not found"` - Invalid target ID
- `"Rollback failed: {error}"` - Docker operation failed

---

### `stream_container_logs`

Streams logs from Docker container (uses Tauri events).

**Frontend Invocation**:
```typescript
// Start streaming
await invoke('stream_container_logs', {
  containerId: 'abc123',
  follow: true
});

// Listen to events
listen<string>('container-log', (event) => {
  console.log(event.payload);  // Log line
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn stream_container_logs(
    container_id: String,
    follow: bool,
    app_handle: tauri::AppHandle
) -> Result<(), String>
```

**Events Emitted**: `container-log` with string payload (each log line)

---

## Disk and Volume Commands

### `get_disk_info`

Retrieves disk space information for a path.

**Frontend Invocation**:
```typescript
const info = await invoke<DiskInfo>('get_disk_info', {
  path: '/media/nvr'
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn get_disk_info(path: String) -> Result<DiskInfo, String>
```

**Response**:
```typescript
interface DiskInfo {
  path: string;
  total_bytes: number;
  used_bytes: number;
  free_bytes: number;
  available_bytes: number;  // May differ from free due to reserved space
  filesystem: string;
  mount_point: string;
}
```

---

### `validate_volume_path`

Validates a path for use as Docker volume.

**Frontend Invocation**:
```typescript
const result = await invoke<VolumeValidationResult>('validate_volume_path', {
  path: '/media/recordings'
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn validate_volume_path(path: String) -> Result<VolumeValidationResult, String>
```

**Response**:
```typescript
interface VolumeValidationResult {
  valid: boolean;
  exists: boolean;
  writable: boolean;
  sufficient_space: boolean;  // >10GB free
  errors: string[];
  warnings: string[];
}
```

---

## Utility Commands

### `open_file_dialog`

Opens native file picker dialog.

**Frontend Invocation**:
```typescript
const filePath = await invoke<string | null>('open_file_dialog', {
  title: 'Select Frigate Configuration',
  filters: [{ name: 'YAML', extensions: ['yml', 'yaml'] }],
  directory: false
});
```

**Rust Signature**:
```rust
#[tauri::command]
async fn open_file_dialog(
    title: String,
    filters: Vec<FileFilter>,
    directory: bool
) -> Result<Option<String>, String>
```

**Response**: `string | null` (selected file path or null if cancelled)

---

### `get_app_version`

Returns application version info.

**Frontend Invocation**:
```typescript
const version = await invoke<VersionInfo>('get_app_version');
```

**Response**:
```typescript
interface VersionInfo {
  version: string;      // "1.0.0"
  build_date: string;   // ISO 8601
  platform: string;     // "linux", "windows", "darwin"
  architecture: string; // "x86_64", "arm64"
}
```

---

## Error Handling

All commands return `Result<T, String>`. Frontend should handle errors:

```typescript
try {
  const devices = await invoke<HardwareDetectionResponse>('detect_hardware');
  // Handle success
} catch (error) {
  // error is string
  console.error('Hardware detection failed:', error);
  // Show user-friendly error in UI
}
```

---

## Testing

### Contract Tests

Each command should have contract tests verifying:
1. Request validation (invalid inputs rejected)
2. Response schema matches contract
3. Error handling (known error conditions return proper messages)

### Example Test (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_hardware_returns_valid_response() {
        let result = detect_hardware().await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.platform, "linux"); // On Linux CI
        // Verify schema
    }

    #[tokio::test]
    async fn test_load_configuration_nonexistent_file() {
        let result = load_configuration("/nonexistent.yml".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File not found"));
    }
}
```

---

## Versioning

API version follows Tauri app version. Breaking changes require MAJOR version bump per constitution requirement.
