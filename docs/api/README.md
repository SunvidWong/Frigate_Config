# API Documentation

## Tauri Commands Reference

This document describes all available Tauri commands that can be invoked from the frontend.

## Table of Contents

- [Hardware Detection](#hardware-detection)
- [Configuration Management](#configuration-management)
- [Deployment](#deployment)
- [Disk & Volume Management](#disk--volume-management)

---

## Hardware Detection

### `detect_hardware`

Executes the hardware detection agent and returns detected devices.

**Parameters**: None

**Returns**: `HardwareDevice[]`

```typescript
interface HardwareDevice {
  id: string;
  device_type: string;
  name: string;
  vendor: string;
  driver: string | null;
  path: string;
  properties: Record<string, string>;
}
```

**Example**:
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const devices = await invoke<HardwareDevice[]>('detect_hardware');
console.log(`Found ${devices.length} devices`);
```

**Errors**:
- `AgentExecutionFailed` - Agent binary failed to execute
- `JsonParseError` - Invalid JSON output from agent

---

### `get_device_details`

Get detailed information about a specific device.

**Parameters**:
- `device_id: string` - Device identifier

**Returns**: `HardwareDevice`

**Example**:
```typescript
const device = await invoke<HardwareDevice>('get_device_details', {
  deviceId: 'intel-qsv-0'
});
```

---

## Configuration Management

### `load_configuration`

Load existing Frigate configuration from file.

**Parameters**:
- `path: string` - Path to config file

**Returns**: `ConfigurationData`

```typescript
interface ConfigurationData {
  yaml: string;
  cameras: CameraConfig[];
  detectors: Record<string, DetectorConfig>;
  mqtt: MqttConfig | null;
}
```

**Example**:
```typescript
const config = await invoke<ConfigurationData>('load_configuration', {
  path: '/etc/frigate/config.yml'
});
```

---

### `save_configuration`

Save configuration with automatic backup.

**Parameters**:
- `path: string` - Output path
- `content: string` - YAML content
- `create_backup: boolean` - Create backup before save (default: true)

**Returns**: `SaveResult`

```typescript
interface SaveResult {
  success: boolean;
  backup_path: string | null;
  conflicts: Conflict[] | null;
}
```

**Example**:
```typescript
const result = await invoke<SaveResult>('save_configuration', {
  path: '/etc/frigate/config.yml',
  content: yamlContent,
  createBackup: true
});

if (result.conflicts) {
  // Handle conflicts
}
```

---

### `merge_template`

Merge UI-generated config with existing configuration.

**Parameters**:
- `existing_path: string` - Path to existing config
- `template: string` - UI-generated YAML
- `strategy: 'ui_priority' | 'manual_priority'` - Merge strategy

**Returns**: `MergeResult`

```typescript
interface MergeResult {
  merged_yaml: string;
  conflicts: Conflict[];
  has_conflicts: boolean;
}

interface Conflict {
  path: string;
  manual_value: string;
  ui_value: string;
  description: string;
}
```

**Example**:
```typescript
const result = await invoke<MergeResult>('merge_template', {
  existingPath: '/etc/frigate/config.yml',
  template: uiGeneratedYaml,
  strategy: 'manual_priority'
});
```

---

### `resolve_conflicts`

Resolve detected conflicts with user choices.

**Parameters**:
- `path: string` - Config file path
- `resolutions: ConflictResolution[]` - User's conflict resolutions

**Returns**: `string` (resolved YAML)

```typescript
interface ConflictResolution {
  path: string;
  choice: 'manual' | 'ui' | 'custom';
  custom_value?: string;
}
```

---

### `list_backups`

List all configuration backups.

**Parameters**:
- `limit: number | null` - Max number of backups to return

**Returns**: `Backup[]`

```typescript
interface Backup {
  id: string;
  timestamp: string;
  file_path: string;
  size_bytes: number;
  description: string | null;
}
```

---

### `restore_backup`

Restore configuration from backup.

**Parameters**:
- `backup_id: string` - Backup identifier
- `target_path: string` - Where to restore

**Returns**: `RestoreResult`

```typescript
interface RestoreResult {
  success: boolean;
  restored_path: string;
}
```

---

## Deployment

### `validate_config`

Validate Frigate configuration before deployment.

**Parameters**:
- `config_path: string` - Path to config file

**Returns**: `ValidationResult`

```typescript
interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
  checks: ValidationCheck[];
}

interface ValidationCheck {
  name: string;
  passed: boolean;
  message: string;
}
```

---

### `deploy_frigate`

Deploy Frigate container.

**Parameters**:
- `config: DeploymentConfig`

```typescript
interface DeploymentConfig {
  config_path: string;
  method: 'DockerRun' | 'DockerCompose';
  devices: string[];
  volumes: VolumeMapping[];
  ports: PortMapping[];
  environment: Record<string, string>;
}
```

**Returns**: `DeploymentResponse`

```typescript
interface DeploymentResponse {
  success: boolean;
  container_id: string | null;
  command: string;
  stdout: string;
  stderr: string;
  exit_code: number | null;
  deployment_time: string;
}
```

---

### `check_deployment_health`

Check health of deployed Frigate container.

**Parameters**:
- `container_id: string`

**Returns**: `HealthCheckResponse`

```typescript
interface HealthCheckResponse {
  container_id: string;
  status: 'healthy' | 'unhealthy' | 'starting';
  checks: HealthCheck[];
  response_time_ms: number;
  timestamp: string;
}

interface HealthCheck {
  name: string;
  passed: boolean;
  message: string;
  details: string | null;
}
```

---

### `rollback_deployment`

Rollback to previous deployment.

**Parameters**:
- `reason: string` - Rollback reason
- `target_snapshot_id: string | null` - Specific snapshot to rollback to
- `preserve_data: boolean` - Keep volumes/data
- `create_backup: boolean` - Backup current state

**Returns**: `RollbackResult`

```typescript
interface RollbackResult {
  success: boolean;
  previous_container_id: string;
  restored_container_id: string | null;
  backup_path: string | null;
}
```

---

### `list_deployment_history`

Get deployment history.

**Parameters**:
- `limit: number | null`

**Returns**: `DeploymentHistory`

```typescript
interface DeploymentHistory {
  deployments: DeploymentRecord[];
  total_count: number;
}

interface DeploymentRecord {
  id: string;
  container_id: string;
  config_path: string;
  deployment_time: string;
  command: string;
  status: string;
}
```

---

## Disk & Volume Management

### `get_disk_info_command`

Get disk information for a path.

**Parameters**:
- `path: string` - Directory path to check

**Returns**: `DiskInfoResponse`

```typescript
interface DiskInfoResponse {
  path: string;
  total_bytes: number;
  used_bytes: number;
  free_bytes: number;
  available_bytes: number;
  total_formatted: string;
  used_formatted: string;
  free_formatted: string;
  available_formatted: string;
  mount_point: string;
  filesystem: string;
  usage_percent: number;
  is_low_space: boolean;
}
```

---

### `validate_volume_path_command`

Validate a path for volume mounting.

**Parameters**:
- `path: string` - Path to validate

**Returns**: `VolumeValidationResponse`

```typescript
interface VolumeValidationResponse {
  valid: boolean;
  errors: string[];
  warnings: string[];
  disk_info: DiskInfoResponse | null;
}
```

---

### `create_volume_mapping`

Create a volume mapping configuration.

**Parameters**:
- `host_path: string` - Host directory path
- `container_path: string` - Container mount path
- `mapping_type: 'recordings' | 'clips' | 'cache' | 'config' | 'custom'`
- `read_only: boolean`
- `description: string | null`

**Returns**: `VolumeMapping`

```typescript
interface VolumeMapping {
  host_path: string;
  container_path: string;
  mapping_type: string;
  read_only: boolean;
  description: string | null;
}
```

---

### `get_default_volume_paths`

Get default volume paths for current platform.

**Parameters**: None

**Returns**: `DefaultVolumePaths`

```typescript
interface DefaultVolumePaths {
  config: string;
  recordings: string;
  clips: string;
  cache: string;
}
```

---

### `get_recommended_paths`

Get recommended paths based on available disk space.

**Parameters**: None

**Returns**: `RecommendedPath[]`

```typescript
interface RecommendedPath {
  path: string;
  total_space: string;
  free_space: string;
  usage_percent: number;
  recommended_for: string[]; // ['recordings', 'clips', etc.]
}
```

---

## Error Handling

All commands may throw the following error types:

### `AppError`

```typescript
type AppError =
  | { type: 'Validation', message: string }
  | { type: 'FileSystem', message: string }
  | { type: 'Agent', message: string }
  | { type: 'Docker', message: string }
  | { type: 'Disk', message: string }
  | { type: 'Database', message: string }
  | { type: 'Configuration', message: string };
```

### Error Handling Example

```typescript
import { invoke } from '@tauri-apps/api/tauri';

try {
  const result = await invoke('deploy_frigate', { config });
  console.log('Deployment successful:', result);
} catch (error) {
  if (error.type === 'Docker') {
    console.error('Docker error:', error.message);
    // Show user-friendly Docker error
  } else {
    console.error('Unexpected error:', error);
  }
}
```

---

## Event Listeners

### Container Logs

Subscribe to real-time container logs:

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('container-log', (event) => {
  console.log('Log:', event.payload);
});

// Later: unlisten()
```

### Deployment Status

```typescript
await listen('deployment-status', (event) => {
  const { status, message } = event.payload;
  console.log(`Deployment ${status}: ${message}`);
});
```

---

## Rate Limits & Best Practices

1. **Hardware Detection**: Cache results for 5 minutes
2. **Health Checks**: Poll every 5 seconds during deployment
3. **Disk Info**: Update when user changes path
4. **Backups**: Auto-backup before every save
5. **Error Handling**: Always wrap commands in try-catch

---

## See Also

- [Architecture Documentation](../architecture.md)
- [Frontend Hooks](../../src-ui/src/hooks/useTauriCommand.ts)
- [Backend Commands](../../src-tauri/src/commands/)

🤖 Generated with [Claude Code](https://claude.com/claude-code)
