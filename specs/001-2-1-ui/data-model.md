# Data Model: Frigate Configuration Tool

**Feature**: 001-2-1-ui
**Date**: 2025-10-08
**Purpose**: Define core data structures, relationships, validation rules, and state transitions

## Overview

This document defines the data model for the Frigate Configuration Tool across four modules: UI, Agent, Configuration Engine, and Deployment. The model supports the full lifecycle from hardware detection through deployment and rollback.

---

## 1. Hardware Device

**Purpose**: Represents a detected hardware accelerator or video device from the host system.

### Schema

```typescript
interface HardwareDevice {
  // Identity
  id: string;                    // UUID generated on detection
  type: DeviceType;              // Enum: "gpu" | "tpu" | "camera" | "capture_card"
  name: string;                  // Human-readable name: "NVIDIA GeForce RTX 3060"
  device_path: string;           // System path: "/dev/video0", "COM3", "/dev/apex_0"

  // Capabilities
  capabilities: string[];        // ["h264_decode", "hevc_encode", "mjpeg"]
  driver: string | null;         // "nvidia", "intel-qsv", "v4l2", null if unknown

  // Platform Info
  platform: Platform;            // Enum: "linux" | "windows" | "darwin"
  architecture: Architecture;    // Enum: "x86_64" | "arm64" | "arm32"

  // Metadata
  vendor_id: string | null;      // PCI vendor ID or USB VID:PID
  detected_at: DateTime;         // ISO 8601 timestamp
  detection_source: string;      // "lsusb", "nvidia-smi", "system_profiler", etc.

  // State
  available: boolean;            // True if device currently accessible
  in_use: boolean;               // True if assigned to a camera config
  error: string | null;          // Error message if detection failed
}
```

### Validation Rules

- `id`: Must be valid UUIDv4
- `type`: Must be one of enum values
- `name`: 1-200 characters, non-empty
- `device_path`: Must start with "/" (Unix) or match Windows device pattern
- `capabilities`: Array of known capability strings (validated against whitelist)
- `detected_at`: Must be valid ISO 8601 timestamp
- `available`: If false, `error` must be populated

### Relationships

- **One HardwareDevice** → **Zero or Many CameraConfiguration** (a device can be assigned to multiple cameras)
- **Many HardwareDevice** → **One HardwareDetectionSession** (devices belong to a detection run)

### State Transitions

```
[Detected] --assign_to_camera--> [In Use]
[In Use] --unassign--> [Detected]
[Detected] --device_removed--> [Unavailable]
[Unavailable] --re-detect--> [Detected] (if device reappears)
```

### Example Instances

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "type": "tpu",
  "name": "Google Coral USB Accelerator",
  "device_path": "/dev/apex_0",
  "capabilities": ["tensorflow_lite"],
  "driver": "apex",
  "platform": "linux",
  "architecture": "x86_64",
  "vendor_id": "1a6e:089a",
  "detected_at": "2025-10-08T14:32:15Z",
  "detection_source": "lsusb",
  "available": true,
  "in_use": false,
  "error": null
}
```

---

## 2. Camera Configuration

**Purpose**: Represents a single camera's complete configuration settings for Frigate.

### Schema

```typescript
interface CameraConfiguration {
  // Identity
  id: string;                    // UUID for this camera config
  name: string;                  // Camera name: "front_door", "backyard"
  enabled: boolean;              // Whether camera is active

  // Connection
  rtsp_url: string;              // RTSP stream URL (may contain credentials)
  rtsp_url_display: string;      // RTSP URL with masked credentials for display

  // Video Settings
  resolution: Resolution;        // { width: number, height: number }
  fps: number;                   // Target FPS (1-60)

  // Hardware Assignment
  hardware_device_id: string | null;  // FK to HardwareDevice.id
  hwaccel: HWAccelType | null;   // "vaapi", "cuda", "qsv", "videotoolbox", null

  // Detection Settings (optional, may be manually configured)
  detect_enabled: boolean;
  detect_zones: Zone[] | null;   // Manually configured detection zones
  record_enabled: boolean;
  snapshots_enabled: boolean;

  // Advanced Settings (from manual config)
  custom_ffmpeg_args: string | null;
  motion_config: object | null;  // Freeform JSON for advanced settings

  // Metadata
  created_at: DateTime;
  updated_at: DateTime;
  manually_edited: boolean;      // True if user edited YAML directly

  // Validation State
  validation_status: ValidationStatus;  // "valid" | "warning" | "error"
  validation_errors: string[];
}
```

### Validation Rules

- `name`: 1-50 characters, alphanumeric + underscore, must be unique
- `rtsp_url`: Must match RTSP URL pattern: `rtsp://[user:pass@]host[:port]/path`
- `resolution.width`: 320-3840 (4K), must be multiple of 2
- `resolution.height`: 240-2160 (4K), must be multiple of 2
- `fps`: 1-60, integer
- `hardware_device_id`: Must reference existing HardwareDevice if non-null
- `hwaccel`: Must be compatible with assigned hardware device
- `detect_zones`: If present, each zone must have valid polygon coordinates

### Relationships

- **Many CameraConfiguration** → **One HardwareDevice** (optional, via `hardware_device_id`)
- **One CameraConfiguration** → **One ConfigurationSnapshot** (embedded in full config)
- **One CameraConfiguration** → **Many DeploymentState** (cameras deployed across versions)

### State Transitions

```
[Draft] --validate--> [Valid]
[Valid] --assign_hardware--> [Ready for Deployment]
[Ready for Deployment] --deploy--> [Deployed]
[Deployed] --edit--> [Modified] --save--> [Valid]
[Valid] --validation_error--> [Error]
[Error] --fix--> [Valid]
```

### Example Instance

```json
{
  "id": "c4a7d9e2-8f3b-4b5e-9a1c-3d4f6e8b0a2c",
  "name": "front_door",
  "enabled": true,
  "rtsp_url": "rtsp://admin:password123@192.168.1.100:554/stream1",
  "rtsp_url_display": "rtsp://admin:***@192.168.1.100:554/stream1",
  "resolution": { "width": 1920, "height": 1080 },
  "fps": 15,
  "hardware_device_id": "550e8400-e29b-41d4-a716-446655440000",
  "hwaccel": "cuda",
  "detect_enabled": true,
  "detect_zones": null,
  "record_enabled": true,
  "snapshots_enabled": true,
  "custom_ffmpeg_args": null,
  "motion_config": null,
  "created_at": "2025-10-08T14:35:00Z",
  "updated_at": "2025-10-08T14:35:00Z",
  "manually_edited": false,
  "validation_status": "valid",
  "validation_errors": []
}
```

---

## 3. Configuration Snapshot

**Purpose**: Represents a complete Frigate configuration at a specific point in time, including all cameras and settings.

### Schema

```typescript
interface ConfigurationSnapshot {
  // Identity
  id: string;                    // UUID for this snapshot
  version: number;               // Sequential version number (1, 2, 3...)

  // Content
  yaml_content: string;          // Full YAML configuration text
  parsed_config: object;         // Parsed YAML as JSON object
  checksum: string;              // SHA-256 hash of yaml_content

  // Cameras
  cameras: CameraConfiguration[]; // All cameras in this config

  // Global Settings
  mqtt_config: object | null;
  detectors_config: object | null;
  database_config: object | null;

  // Metadata
  created_at: DateTime;
  created_by: CreationSource;    // "ui" | "template" | "manual_edit" | "rollback"
  description: string | null;    // User-provided description

  // Backup Info
  is_backup: boolean;            // True if auto-created backup
  backup_reason: string | null;  // "pre_deploy", "pre_merge", "manual"

  // Deployment
  deployed: boolean;             // True if this config was ever deployed
  deployed_at: DateTime | null;
  deployment_success: boolean | null;
}
```

### Validation Rules

- `yaml_content`: Must be valid YAML syntax
- `parsed_config`: Must conform to Frigate configuration schema
- `checksum`: Must be 64-character hex string (SHA-256)
- `cameras`: All cameras must have valid configurations
- `version`: Must be unique and sequential

### Relationships

- **One ConfigurationSnapshot** → **Many CameraConfiguration** (embedded)
- **One ConfigurationSnapshot** → **Many ConflictResolution** (conflicts discovered during merge)
- **One ConfigurationSnapshot** → **Zero or One DeploymentState** (if deployed)

### State Transitions

```
[Draft] --validate--> [Valid]
[Valid] --create_backup--> [Backed Up]
[Backed Up] --deploy--> [Deployed]
[Deployed] --rollback--> [Restored] (creates new snapshot)
[Valid] --merge_template--> [Conflict Detected] --resolve--> [Merged]
```

### Example Instance

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "version": 3,
  "yaml_content": "mqtt:\n  enabled: true\n  host: mqtt.local\ncameras:\n  front_door:\n    ...",
  "parsed_config": { "mqtt": { "enabled": true, "host": "mqtt.local" }, "cameras": {...} },
  "checksum": "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8",
  "cameras": [ /* array of CameraConfiguration */ ],
  "mqtt_config": { "enabled": true, "host": "mqtt.local" },
  "detectors_config": null,
  "database_config": null,
  "created_at": "2025-10-08T14:40:00Z",
  "created_by": "ui",
  "description": "Added front door camera with Coral TPU",
  "is_backup": true,
  "backup_reason": "pre_deploy",
  "deployed": true,
  "deployed_at": "2025-10-08T14:45:00Z",
  "deployment_success": true
}
```

---

## 4. Conflict Resolution

**Purpose**: Represents a detected conflict between existing user configuration and a template being merged.

### Schema

```typescript
interface ConflictResolution {
  // Identity
  id: string;                    // UUID for this conflict
  snapshot_id: string;           // FK to ConfigurationSnapshot being merged

  // Conflict Details
  path: string;                  // YAML path: "cameras.front_door.fps"
  existing_value: any;           // User's current value
  template_value: any;           // Template's proposed value
  value_type: string;            // "number", "string", "boolean", "object", "array"

  // Resolution
  resolution: Resolution | null; // "keep_existing" | "use_template" | "custom" | null (pending)
  custom_value: any | null;      // If resolution="custom", user-provided value
  resolved_at: DateTime | null;

  // Context
  detected_at: DateTime;
  severity: Severity;            // "warning" | "error" | "info"
  auto_resolvable: boolean;      // True if safe default exists
  suggested_resolution: Resolution | null;

  // User Decision
  user_notes: string | null;     // User-provided explanation for choice
}
```

### Validation Rules

- `path`: Must be valid YAML path notation
- `existing_value` and `template_value`: Must be serializable JSON
- `resolution`: If not null, must be enum value
- `custom_value`: If `resolution="custom"`, must be non-null
- `resolved_at`: Must be set if resolution is not null

### Relationships

- **Many ConflictResolution** → **One ConfigurationSnapshot**
- **One ConflictResolution** → **Zero or One AuditLogEntry** (logged when resolved)

### State Transitions

```
[Detected] --auto_resolve--> [Resolved] (if auto_resolvable=true)
[Detected] --user_keeps_existing--> [Resolved]
[Detected] --user_accepts_template--> [Resolved]
[Detected] --user_provides_custom--> [Resolved]
[Resolved] --undo--> [Detected] (allow re-resolution)
```

### Example Instance

```json
{
  "id": "f1e2d3c4-b5a6-9788-0fed-cba987654321",
  "snapshot_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "path": "cameras.front_door.fps",
  "existing_value": 20,
  "template_value": 15,
  "value_type": "number",
  "resolution": "keep_existing",
  "custom_value": null,
  "resolved_at": "2025-10-08T14:41:30Z",
  "detected_at": "2025-10-08T14:41:00Z",
  "severity": "warning",
  "auto_resolvable": false,
  "suggested_resolution": "keep_existing",
  "user_notes": "I want higher FPS for this camera"
}
```

---

## 5. Deployment State

**Purpose**: Represents the current or historical deployment state of Frigate.

### Schema

```typescript
interface DeploymentState {
  // Identity
  id: string;                    // UUID for this deployment
  snapshot_id: string;           // FK to ConfigurationSnapshot deployed

  // Deployment Info
  deployment_method: DeploymentMethod;  // "docker_run" | "docker_compose"
  container_id: string | null;   // Docker container ID
  command: string;               // Full Docker command executed

  // Timing
  started_at: DateTime;
  completed_at: DateTime | null;
  duration_ms: number | null;

  // Status
  status: DeploymentStatus;      // "pending" | "running" | "success" | "failed" | "rolled_back"
  exit_code: number | null;      // Docker command exit code

  // Health Checks
  health_check_status: HealthStatus | null;  // "healthy" | "unhealthy" | "pending" | null
  health_check_attempts: number;
  health_check_last_attempt: DateTime | null;

  // Logs
  stdout_log: string;            // Captured stdout
  stderr_log: string;            // Captured stderr
  frigate_logs: string | null;   // Streamed Frigate container logs

  // Rollback
  previous_state_id: string | null;  // FK to previous DeploymentState for rollback
  rolled_back: boolean;
  rollback_reason: string | null;

  // Volume Mappings
  volume_mappings: VolumeMapping[];

  // Metadata
  deployed_by: string;           // User identifier or "system"
  deployment_notes: string | null;
}
```

### Validation Rules

- `deployment_method`: Must be enum value
- `container_id`: If status="success", must be non-null
- `command`: Non-empty string
- `started_at`: Required
- `completed_at`: If status="success" or "failed", must be non-null
- `status`: Must progress logically (pending → running → success/failed)
- `previous_state_id`: If rolled_back=true, must be non-null

### Relationships

- **One DeploymentState** → **One ConfigurationSnapshot** (via `snapshot_id`)
- **One DeploymentState** → **Zero or One DeploymentState** (via `previous_state_id`, rollback chain)
- **One DeploymentState** → **Many VolumeMapping**

### State Transitions

```
[Pending] --start_deploy--> [Running]
[Running] --health_check_pass--> [Success]
[Running] --health_check_fail--> [Failed]
[Running] --timeout--> [Failed]
[Success] --rollback_initiated--> [Rolled Back]
[Failed] --auto_rollback--> [Rolled Back]
```

### Example Instance

```json
{
  "id": "d1e2f3a4-b5c6-7d8e-9f0a-1b2c3d4e5f6a",
  "snapshot_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "deployment_method": "docker_run",
  "container_id": "abc123def456",
  "command": "docker run -d --name frigate --device /dev/apex_0 -v /media/nvr:/media/frigate ghcr.io/blakeblackshear/frigate:stable",
  "started_at": "2025-10-08T14:45:00Z",
  "completed_at": "2025-10-08T14:45:15Z",
  "duration_ms": 15000,
  "status": "success",
  "exit_code": 0,
  "health_check_status": "healthy",
  "health_check_attempts": 3,
  "health_check_last_attempt": "2025-10-08T14:45:45Z",
  "stdout_log": "Container started successfully...",
  "stderr_log": "",
  "frigate_logs": "INFO: Starting Frigate...",
  "previous_state_id": null,
  "rolled_back": false,
  "rollback_reason": null,
  "volume_mappings": [ /* array */ ],
  "deployed_by": "user",
  "deployment_notes": "Initial deployment with Coral TPU"
}
```

---

## 6. Volume Mapping

**Purpose**: Represents a host-to-container directory mapping for Docker volumes.

### Schema

```typescript
interface VolumeMapping {
  // Identity
  id: string;                    // UUID for this mapping
  deployment_state_id: string;   // FK to DeploymentState

  // Paths
  host_path: string;             // Absolute path on host: "/media/nvr/recordings"
  container_path: string;        // Path inside container: "/media/frigate/recordings"

  // Type
  mapping_type: MappingType;     // "recordings" | "clips" | "cache" | "config" | "custom"
  read_only: boolean;            // True if mounted read-only

  // Storage Info
  disk_total_bytes: number | null;
  disk_used_bytes: number | null;
  disk_free_bytes: number | null;
  disk_info_updated_at: DateTime | null;

  // Validation
  path_exists: boolean;
  path_writable: boolean;
  validation_error: string | null;

  // Metadata
  created_at: DateTime;
}
```

### Validation Rules

- `host_path`: Must be absolute path, must exist if `path_exists=true`
- `container_path`: Must be absolute path
- `mapping_type`: Must be enum value
- `disk_free_bytes`: If <10GB and mapping_type="recordings", generate warning
- `path_writable`: Must be true if `read_only=false`

### Relationships

- **Many VolumeMapping** → **One DeploymentState**

### Example Instance

```json
{
  "id": "v1m2a3p4-5d6f-7e8a-9b0c-1d2e3f4a5b6c",
  "deployment_state_id": "d1e2f3a4-b5c6-7d8e-9f0a-1b2c3d4e5f6a",
  "host_path": "/media/nvr/recordings",
  "container_path": "/media/frigate/recordings",
  "mapping_type": "recordings",
  "read_only": false,
  "disk_total_bytes": 2000000000000,
  "disk_used_bytes": 500000000000,
  "disk_free_bytes": 1500000000000,
  "disk_info_updated_at": "2025-10-08T14:44:50Z",
  "path_exists": true,
  "path_writable": true,
  "validation_error": null,
  "created_at": "2025-10-08T14:44:50Z"
}
```

---

## 7. Configuration Template

**Purpose**: Represents a reusable configuration pattern that can be injected into user configs.

### Schema

```typescript
interface ConfigurationTemplate {
  // Identity
  id: string;                    // UUID for this template
  name: string;                  // Template name: "coral_tpu_setup", "nvidia_gpu_h264"
  category: TemplateCategory;    // "hardware" | "cameras" | "detectors" | "complete"

  // Content
  yaml_fragment: string;         // YAML snippet to inject
  parsed_fragment: object;       // Parsed YAML as JSON

  // Metadata
  description: string;           // Human-readable description
  author: string;                // Template creator
  version: string;               // Semantic version: "1.0.0"

  // Applicability
  required_hardware: string[];   // ["coral_tpu"] - devices needed
  compatible_platforms: Platform[];
  tags: string[];                // ["beginner", "performance", "coral"]

  // Usage
  usage_count: number;           // Times this template was applied
  last_used_at: DateTime | null;

  // Validation
  validated: boolean;            // True if template passes Frigate schema
  validation_errors: string[];

  created_at: DateTime;
  updated_at: DateTime;
}
```

### Validation Rules

- `name`: 1-100 characters, unique
- `yaml_fragment`: Must be valid YAML
- `parsed_fragment`: Must be valid partial Frigate config
- `version`: Must follow semver: `X.Y.Z`
- `required_hardware`: Array of known hardware type strings

### Relationships

- **One ConfigurationTemplate** → **Many ConfigurationSnapshot** (template applied to snapshots)

### Example Instance

```json
{
  "id": "t1e2m3p4-5l6a-7t8e-9a0b-1c2d3e4f5a6b",
  "name": "coral_tpu_basic",
  "category": "hardware",
  "yaml_fragment": "detectors:\n  coral:\n    type: edgetpu\n    device: usb",
  "parsed_fragment": { "detectors": { "coral": { "type": "edgetpu", "device": "usb" } } },
  "description": "Basic Coral TPU detector configuration for USB accelerator",
  "author": "community",
  "version": "1.0.0",
  "required_hardware": ["coral_tpu"],
  "compatible_platforms": ["linux"],
  "tags": ["beginner", "coral", "detector"],
  "usage_count": 42,
  "last_used_at": "2025-10-08T14:30:00Z",
  "validated": true,
  "validation_errors": [],
  "created_at": "2025-09-01T10:00:00Z",
  "updated_at": "2025-09-01T10:00:00Z"
}
```

---

## 8. Audit Log Entry

**Purpose**: Records all significant user actions and system events for compliance and debugging.

### Schema

```typescript
interface AuditLogEntry {
  // Identity
  id: string;                    // UUID for this log entry

  // Event Info
  event_type: EventType;         // "config_change" | "deployment" | "rollback" | "conflict_resolved"
  event_action: string;          // Specific action: "camera_added", "deploy_started"

  // Context
  user_id: string;               // User identifier
  timestamp: DateTime;

  // References
  related_snapshot_id: string | null;
  related_deployment_id: string | null;
  related_conflict_id: string | null;

  // Details
  before_state: object | null;   // Snapshot of state before action
  after_state: object | null;    // Snapshot of state after action
  diff: string | null;           // Diff representation for configs

  // Outcome
  success: boolean;
  error_message: string | null;

  // Metadata
  client_info: string;           // "Frigate Config Tool v1.0.0 (Tauri/Linux)"
}
```

### Validation Rules

- `event_type`: Must be enum value
- `timestamp`: ISO 8601 format
- `success`: Must be boolean
- If `success=false`, `error_message` must be non-null

### Relationships

- **Many AuditLogEntry** → **One ConfigurationSnapshot** (optional)
- **Many AuditLogEntry** → **One DeploymentState** (optional)
- **Many AuditLogEntry** → **One ConflictResolution** (optional)

### Example Instance

```json
{
  "id": "a1u2d3i4-5t6l-7o8g-9e0n-1t2r3y4e5f6g",
  "event_type": "deployment",
  "event_action": "deploy_started",
  "user_id": "user_001",
  "timestamp": "2025-10-08T14:45:00Z",
  "related_snapshot_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "related_deployment_id": "d1e2f3a4-b5c6-7d8e-9f0a-1b2c3d4e5f6a",
  "related_conflict_id": null,
  "before_state": { "container_running": false },
  "after_state": { "container_running": true, "container_id": "abc123def456" },
  "diff": null,
  "success": true,
  "error_message": null,
  "client_info": "Frigate Config Tool v1.0.0 (Tauri/Linux x86_64)"
}
```

---

## Enums

### DeviceType
```typescript
enum DeviceType {
  GPU = "gpu",
  TPU = "tpu",
  CAMERA = "camera",
  CAPTURE_CARD = "capture_card"
}
```

### Platform
```typescript
enum Platform {
  LINUX = "linux",
  WINDOWS = "windows",
  DARWIN = "darwin"
}
```

### Architecture
```typescript
enum Architecture {
  X86_64 = "x86_64",
  ARM64 = "arm64",
  ARM32 = "arm32"
}
```

### HWAccelType
```typescript
enum HWAccelType {
  VAAPI = "vaapi",
  CUDA = "cuda",
  QSV = "qsv",
  VIDEOTOOLBOX = "videotoolbox"
}
```

### ValidationStatus
```typescript
enum ValidationStatus {
  VALID = "valid",
  WARNING = "warning",
  ERROR = "error"
}
```

### Resolution
```typescript
enum Resolution {
  KEEP_EXISTING = "keep_existing",
  USE_TEMPLATE = "use_template",
  CUSTOM = "custom"
}
```

### Severity
```typescript
enum Severity {
  INFO = "info",
  WARNING = "warning",
  ERROR = "error"
}
```

### DeploymentMethod
```typescript
enum DeploymentMethod {
  DOCKER_RUN = "docker_run",
  DOCKER_COMPOSE = "docker_compose"
}
```

### DeploymentStatus
```typescript
enum DeploymentStatus {
  PENDING = "pending",
  RUNNING = "running",
  SUCCESS = "success",
  FAILED = "failed",
  ROLLED_BACK = "rolled_back"
}
```

### HealthStatus
```typescript
enum HealthStatus {
  HEALTHY = "healthy",
  UNHEALTHY = "unhealthy",
  PENDING = "pending"
}
```

### MappingType
```typescript
enum MappingType {
  RECORDINGS = "recordings",
  CLIPS = "clips",
  CACHE = "cache",
  CONFIG = "config",
  CUSTOM = "custom"
}
```

### TemplateCategory
```typescript
enum TemplateCategory {
  HARDWARE = "hardware",
  CAMERAS = "cameras",
  DETECTORS = "detectors",
  COMPLETE = "complete"
}
```

### EventType
```typescript
enum EventType {
  CONFIG_CHANGE = "config_change",
  DEPLOYMENT = "deployment",
  ROLLBACK = "rollback",
  CONFLICT_RESOLVED = "conflict_resolved",
  HARDWARE_DETECTED = "hardware_detected"
}
```

---

## Storage Implementation

### SQLite Schema (for metadata)

```sql
-- Configuration snapshots
CREATE TABLE config_snapshots (
  id TEXT PRIMARY KEY,
  version INTEGER NOT NULL UNIQUE,
  yaml_content TEXT NOT NULL,
  checksum TEXT NOT NULL,
  created_at TEXT NOT NULL,
  created_by TEXT NOT NULL,
  description TEXT,
  is_backup BOOLEAN NOT NULL,
  deployed BOOLEAN NOT NULL DEFAULT 0
);

-- Deployment states
CREATE TABLE deployment_states (
  id TEXT PRIMARY KEY,
  snapshot_id TEXT NOT NULL,
  container_id TEXT,
  status TEXT NOT NULL,
  started_at TEXT NOT NULL,
  completed_at TEXT,
  FOREIGN KEY (snapshot_id) REFERENCES config_snapshots(id)
);

-- Audit log
CREATE TABLE audit_log (
  id TEXT PRIMARY KEY,
  event_type TEXT NOT NULL,
  timestamp TEXT NOT NULL,
  user_id TEXT NOT NULL,
  success BOOLEAN NOT NULL,
  details TEXT  -- JSON blob
);

CREATE INDEX idx_audit_timestamp ON audit_log(timestamp);
CREATE INDEX idx_deployment_snapshot ON deployment_states(snapshot_id);
```

### File System Layout

```
~/.frigate-config-tool/
├── database.sqlite          # Metadata storage
├── configs/                 # YAML configuration files
│   ├── current.yml          # Active configuration
│   └── backups/
│       ├── 2025-10-08T14-40-00_v2.yml
│       └── 2025-10-08T14-45-00_v3.yml
├── logs/
│   ├── audit.log
│   └── application.log
└── templates/               # User-created templates
    └── my_coral_setup.yml
```

---

## Relationships Diagram

```
HardwareDevice (1) ----< (M) CameraConfiguration
CameraConfiguration (M) ---> (1) ConfigurationSnapshot
ConfigurationSnapshot (1) ----< (M) ConflictResolution
ConfigurationSnapshot (1) ----< (1) DeploymentState
DeploymentState (1) ----< (M) VolumeMapping
DeploymentState (1) ----< (M) AuditLogEntry
ConfigurationTemplate (1) ----< (M) ConfigurationSnapshot [via usage]
```

---

## Data Flow Examples

### Example 1: Hardware Detection → Camera Assignment

```
1. User clicks "Detect Hardware"
2. System creates HardwareDevice records (status: detected)
3. User navigates to Cameras page
4. User creates CameraConfiguration, selects HardwareDevice from dropdown
5. CameraConfiguration.hardware_device_id set
6. HardwareDevice.in_use set to true
```

### Example 2: Configuration Merge with Conflict

```
1. User loads existing config → ConfigurationSnapshot (version: 5)
2. User applies template → Template
3. System detects conflict → ConflictResolution (resolution: null)
4. UI shows conflict dialog
5. User chooses "keep_existing"
6. ConflictResolution updated (resolution: "keep_existing")
7. System creates new ConfigurationSnapshot (version: 6) with resolved values
8. AuditLogEntry created (event: "conflict_resolved")
```

### Example 3: Deployment → Health Check → Rollback

```
1. User clicks Deploy → DeploymentState (status: pending)
2. System validates config → if fail, DeploymentState.status = failed
3. System executes docker run → DeploymentState.status = running
4. System performs health checks → if fail after 50s, initiate rollback
5. Rollback: create new DeploymentState with previous_state_id
6. Stop current container, start previous container
7. Update DeploymentState.status = rolled_back
8. AuditLogEntry created (event: "rollback")
```

---

## Conclusion

This data model provides:
- **Traceability**: Audit log tracks all actions
- **Rollback**: Snapshot history enables time-travel
- **Conflict Resolution**: Explicit conflict tracking prevents silent overwrites
- **Validation**: Multi-level validation (device, camera, config, deployment)
- **Flexibility**: Supports both UI-driven and manual workflows

Next: Generate API contracts from this model.
