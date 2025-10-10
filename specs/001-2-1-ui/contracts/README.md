# API Contracts Overview

**Feature**: 001-2-1-ui - Frigate Configuration Tool
**Date**: 2025-10-08

## Contract Files

1. **`agent-api.json`** - OpenAPI 3.0 schema for Agent JSON output
2. **`tauri-commands.md`** - Tauri IPC command specifications

## Architecture

```
┌─────────────┐         Tauri IPC          ┌──────────────┐
│   Frontend  │◄────────────────────────────┤ Tauri Rust   │
│ (React/TS)  │   invoke('command', {})    │   Backend    │
└─────────────┘                             └──────────────┘
                                                    │
                                                    │ Execute binary
                                                    │ Parse JSON stdout
                                                    ▼
                                            ┌──────────────┐
                                            │  Go Agent    │
                                            │   Binary     │
                                            └──────────────┘
                                                    │
                                                    │ Detect hardware
                                                    │ Output JSON
                                                    ▼
                                            { "devices": [...] }
```

## Communication Patterns

### Pattern 1: Frontend → Tauri Command → Response

Used for: Configuration operations, deployment, file dialogs

```typescript
// Frontend
const snapshot = await invoke<ConfigurationSnapshot>('load_configuration', {
  filePath: '/path/to/frigate.yml'
});

// Tauri backend handles request, returns result
```

### Pattern 2: Tauri → Agent Binary → JSON Response

Used for: Hardware detection

```rust
// Tauri Rust
let output = Command::new("./agent")
    .arg("detect")
    .output()?;

let response: HardwareDetectionResponse = serde_json::from_slice(&output.stdout)?;
```

### Pattern 3: Tauri → Event Emission → Frontend Listener

Used for: Log streaming, long-running operations

```rust
// Tauri Rust
app_handle.emit_all("container-log", log_line)?;
```

```typescript
// Frontend
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<string>('container-log', (event) => {
  appendLog(event.payload);
});
```

## Data Flow Examples

### Example 1: Hardware Detection Flow

```
1. User clicks "Detect Hardware" button
2. Frontend: invoke('detect_hardware')
3. Tauri: Execute ./agent detect
4. Agent: Runs lsusb, nvidia-smi, etc.
5. Agent: Outputs JSON to stdout
6. Tauri: Parses JSON
7. Tauri: Returns HardwareDetectionResponse
8. Frontend: Updates UI with device list
```

### Example 2: Configuration Merge with Conflicts

```
1. User selects template to apply
2. Frontend: invoke('merge_template', { currentYaml, templateYaml })
3. Tauri: Parse both YAMLs
4. Tauri: Run conflict detection algorithm
5. Tauri: Return MergeResult with conflicts array
6. Frontend: Display conflict resolution UI
7. User resolves conflicts
8. Frontend: invoke('resolve_conflicts', { resolutions })
9. Tauri: Apply resolutions, return merged YAML
10. Frontend: Update config preview
```

### Example 3: Deployment with Health Checks

```
1. User clicks "Deploy"
2. Frontend: invoke('validate_deployment', { ... })
3. Tauri: Run validation checks
4. Tauri: Return ValidationResult
5. Frontend: Show validation results, confirm deployment
6. Frontend: invoke('execute_deployment', { command, snapshotId })
7. Tauri: Execute docker run command
8. Tauri: Create DeploymentState record
9. Tauri: Start health check polling (background)
10. Tauri: Emit health-check-update events
11. Frontend: Listen to events, update UI
12. After 10 attempts: Tauri returns final DeploymentState
```

## Error Handling Strategy

### Frontend Error Display

```typescript
async function detectHardware() {
  try {
    setLoading(true);
    const devices = await invoke<HardwareDetectionResponse>('detect_hardware');
    setDevices(devices.devices);

    // Show non-fatal errors as warnings
    if (devices.errors.length > 0) {
      devices.errors.forEach(err => {
        toast.warning(`${err.source}: ${err.message}`);
      });
    }
  } catch (error) {
    // Fatal error
    toast.error(`Hardware detection failed: ${error}`);
  } finally {
    setLoading(false);
  }
}
```

### Backend Error Messages

Follow pattern: `"{Operation} failed: {specific reason}"`

Examples:
- ✅ Good: `"Docker deployment failed: container port 5000 already in use"`
- ❌ Bad: `"Error"`, `"Failed"`, `"Exception occurred"`

## Validation Rules

### Agent JSON Output

- Must be valid JSON
- Must match `HardwareDetectionResponse` schema
- Devices array can be empty (no error if no devices found)
- Errors array documents non-fatal issues

### Tauri Command Inputs

- File paths must be absolute
- YAML content must be valid YAML syntax (validated before processing)
- UUIDs must be valid UUIDv4 format
- Enum values must match defined enum strings

### Tauri Command Outputs

- All async operations return `Result<T, String>`
- Timestamps in ISO 8601 format
- File sizes in bytes (convert to human-readable in frontend)
- Errors are descriptive strings, not codes

## Testing

### Agent Contract Tests

```bash
cd agent
go test ./internal/detect -v
```

Verify:
- JSON output matches schema
- Platform-specific detection works
- Timeout protection (5s max)
- Partial results on detection failures

### Tauri Command Tests

```bash
cargo test --test integration
```

Verify:
- Request validation
- Response schema compliance
- Error handling for known failure modes
- Idempotency (commands can be safely retried)

### End-to-End Contract Tests

```typescript
// tests/contract/agent-to-tauri.spec.ts
test('agent detection integrates with tauri command', async () => {
  const response = await invoke<HardwareDetectionResponse>('detect_hardware');

  expect(response.devices).toBeInstanceOf(Array);
  expect(response.platform).toMatch(/^(linux|windows|darwin)$/);
  expect(response.detected_at).toMatch(/^\d{4}-\d{2}-\d{2}T/); // ISO 8601
});
```

## Security Considerations

### Input Validation

- **File Paths**: Validate no path traversal (../)
- **YAML Content**: Limit size to 10MB, depth to 50 levels
- **Docker Commands**: Escape special characters, no command injection
- **Device Paths**: Whitelist allowed patterns

### Agent Security

- Agent runs with user permissions (no sudo)
- Timeout protection prevents hanging
- No network access (standalone binary)
- Read-only system inspection

### IPC Security

- Tauri CSP prevents XSS
- Commands exposed explicitly (no dynamic invocation)
- Sensitive data (credentials) masked in logs

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Hardware detection | <5s | Includes agent spawn + JSON parse |
| Load configuration | <100ms | Parse 1000-line YAML |
| Merge template | <200ms | Including conflict detection |
| Validate deployment | <2s | Docker check + path validation |
| Generate docker command | <50ms | String generation only |

## Versioning

Contracts follow semantic versioning:

- **MAJOR**: Breaking changes (remove command, change response schema)
- **MINOR**: Add new command, add optional field
- **PATCH**: Bug fixes, error message improvements

Current version: **1.0.0**

## Future Enhancements

Potential additions for v2.0:

1. **Agent HTTP Mode**: Long-running agent with REST API (replace CLI invocation)
2. **Streaming Responses**: Large config files streamed instead of loaded at once
3. **Batch Operations**: Multiple camera configs in single command
4. **Remote Deployment**: Deploy to remote Docker hosts via SSH
5. **GraphQL**: Replace multiple Tauri commands with unified GraphQL query layer

## References

- [Tauri IPC Documentation](https://tauri.app/v1/guides/features/command)
- [OpenAPI 3.0 Specification](https://swagger.io/specification/)
- [Frigate Configuration Schema](https://docs.frigate.video/configuration/)
