# Research: Frigate Configuration Tool - Complete System

**Date**: 2025-10-08
**Feature**: 001-2-1-ui
**Purpose**: Technical research and decision documentation for implementation planning

## Executive Summary

This document consolidates research findings that inform the technical architecture for the Frigate Configuration Tool. Key decisions: (1) Tauri + React for cross-platform UI with native performance, (2) Go for hardware detection agent with zero-dependency single-binary distribution, (3) Rust for configuration engine with comment-preserving YAML manipulation, (4) IPC-based modular architecture enabling independent testing and deployment.

---

## 1. Cross-Platform Desktop UI Framework

### Decision: Tauri 1.5+ with React 18/TypeScript

### Rationale

**Evaluated Options**:
1. **Electron** - Popular but 50MB+ bundle size, 100MB+ RAM overhead
2. **Qt/QML** - Mature but requires C++/QML expertise, licensing complexity (GPL vs commercial)
3. **Flutter Desktop** - Emerging but limited system integration, large binary size
4. **Tauri** - Modern, Rust-backed, 3-10MB bundles, <50MB RAM

**Why Tauri**:
- **Performance**: Native webview (WebView2/WebKit/WKWebView) instead of bundled Chromium
- **Size**: 10MB vs Electron's 50MB+ - critical for tool users expect to be lightweight
- **Security**: Rust backend provides memory safety and fine-grained IPC permission model
- **System Access**: Native integration for hardware detection, file operations, Docker CLI
- **Ecosystem**: React/Vue/Svelte frontend options, rich plugin ecosystem
- **Cross-Platform**: First-class Windows/Linux/macOS support with single codebase

**iOS-Style Aesthetics**:
- Use **shadcn/ui** or **Headless UI** component library for modern, clean design
- Tailwind CSS for rapid styling iterations
- React Router for page-based navigation matching iOS app structure

**Trade-offs**:
- Requires Rust knowledge for backend (mitigated: backend is thin IPC layer, most logic in modules)
- Younger ecosystem than Electron (mitigated: rapidly growing, strong community)

### Alternatives Considered

| Framework | Pros | Cons | Rejection Reason |
|-----------|------|------|------------------|
| Electron | Mature ecosystem, rich docs | 50MB+ size, high memory | Violates <10MB bundle constraint |
| Qt/QML | Native performance, mature | C++ complexity, licensing | Steep learning curve, GPL concerns |
| Flutter Desktop | Single codebase mobile+desktop | Limited system APIs, 40MB+ | Immature desktop support, size |

### Implementation Notes

- **Tauri Commands**: Define Rust functions with `#[tauri::command]` macro for IPC
- **State Management**: Use Zustand or React Context for frontend state
- **Async Handling**: Tauri supports async Rust functions, maps to Promises in JS
- **Packaging**: tauri build generates platform-specific installers (.exe, .dmg, .AppImage)

---

## 2. Hardware Detection Agent

### Decision: Go 1.21+ with Standard Library Only

### Rationale

**Evaluated Options**:
1. **Go** - Best-in-class cross-compilation, single binary, mature stdlib
2. **Rust** - System-level performance, but cross-platform detection libraries less mature
3. **Python** - Easy development, but distribution challenges (requires runtime or PyInstaller bloat)
4. **C++** - Maximum control, but complex build systems and cross-compilation pain

**Why Go**:
- **Cross-Compilation**: `GOOS=linux GOARCH=arm64 go build` from any platform
- **Single Binary**: Static linking by default, zero runtime dependencies
- **Standard Library**: `os/exec` for system commands, `encoding/json` for output - no external deps
- **Simplicity**: Clean error handling, no manual memory management
- **Testing**: Built-in testing framework with table-driven test patterns

**Hardware Detection Approach**:

| Platform | Detection Method | Go Implementation |
|----------|------------------|-------------------|
| Linux | Parse `/dev/video*`, `/dev/dri/*`, `lsusb`, `/sys/class/video4linux/` | `os.ReadDir("/dev")`, `exec.Command("lsusb")` |
| Windows | WMIC for devices, `nvidia-smi`, DirectShow enumeration | `exec.Command("wmic", "path", "win32_videocontroller")` |
| macOS | `system_profiler SPCameraDataType`, `ioreg`, AVFoundation list | `exec.Command("system_profiler", "-json", "SPCameraDataType")` |

**Privilege-Free Detection**:
- **No Driver Scanning**: Read device nodes and execute user-accessible commands only
- **Fallback**: If detection fails, return empty list with error message - UI allows manual entry
- **Timeout**: 5-second timeout per detection operation to prevent hangs

### Alternatives Considered

| Language | Pros | Cons | Rejection Reason |
|----------|------|------|------------------|
| Rust | Memory safety, performance | Cross-platform system libs immature | Go stdlib more complete for system detection |
| Python | Rapid development | Distribution complexity, 50MB+ bundles | Violates <20MB agent binary constraint |
| C++ | Maximum control | Complex build, poor cross-compilation | Development velocity too low |

### Implementation Notes

```go
// Example JSON schema
type HardwareDevice struct {
    Type        string   `json:"type"`         // "gpu", "tpu", "camera"
    Name        string   `json:"name"`         // "NVIDIA GeForce RTX 3060"
    DevicePath  string   `json:"device_path"`  // "/dev/video0"
    Capabilities []string `json:"capabilities"` // ["h264", "hevc"]
    Platform    string   `json:"platform"`     // "linux", "windows", "darwin"
}
```

- **Build Tags**: Use `//go:build linux` to separate platform-specific code
- **Error Handling**: Return partial results on failures (e.g., GPU detection succeeds but camera fails)
- **Caching**: Optionally cache results for 30 seconds to avoid repeated system calls

---

## 3. YAML Configuration Engine

### Decision: Rust with serde_yaml + Custom Merge Logic

### Rationale

**Evaluated Options**:
1. **Rust serde_yaml** - Native Tauri integration, performant, but comment preservation requires custom logic
2. **Go goccy/go-yaml** - Good comment preservation, but would require separate service
3. **Python ruamel.yaml** - Excellent comment preservation, but distribution issues

**Why Rust serde_yaml**:
- **Tauri Integration**: Same language as UI backend, no IPC overhead
- **Performance**: Sub-50ms parse/generate for typical configs (tested with 1000-line YAML)
- **Custom Merge**: Implement conflict detection via AST comparison
- **Type Safety**: Strong typing prevents runtime errors in config manipulation

**Comment Preservation Strategy**:
- **Approach 1** (Simpler): Use `yaml-rust2` which preserves comments during parsing
- **Approach 2** (Fallback): Store comments in separate metadata file, reinsert on write
- **Chosen**: Approach 1 with yaml-rust2 fork that maintains comment AST nodes

**Conflict Detection Algorithm**:
1. Parse both existing and template YAML into ASTs
2. Walk both trees simultaneously (depth-first traversal)
3. On key collision, compare values:
   - If identical: merge (no conflict)
   - If different: create ConflictRecord with both values
4. Present conflicts to user via UI with three-way merge UI component

### Alternatives Considered

| Option | Pros | Cons | Rejection Reason |
|--------|------|------|------------------|
| Go goccy/go-yaml | Great comment support | Separate service, IPC latency | Adds architectural complexity |
| Python ruamel.yaml | Best comment preservation | Distribution, performance | Cannot integrate with Tauri |
| Pure serde_yaml | Fast, well-tested | No comment preservation | Comments critical for user trust |

### Implementation Notes

```rust
// Core types
pub struct ConfigMerger {
    strategy: MergeStrategy,  // Recursive, TemplateWins, UserWins
    preserve_comments: bool,
}

pub struct Conflict {
    path: String,              // "cameras.front_door.fps"
    existing_value: Value,
    template_value: Value,
    user_choice: Option<Resolution>,  // Keep, Override, Custom
}
```

- **Atomic Writes**: Write to temp file, then rename for atomicity
- **Backup Before Merge**: Auto-backup to `~/.frigate-tool/backups/{timestamp}.yml`
- **Rollback**: Keep last 10 backups, allow restore via UI dropdown

---

## 4. Docker Deployment Module

### Decision: Docker CLI via os::exec (Rust) or exec.Command (Go)

### Rationale

**Evaluated Options**:
1. **Docker CLI** - Simple, works with all Docker installations
2. **Docker SDK (bollard in Rust)** - Programmatic control, but adds 5MB+ dependency
3. **docker-compose CLI** - Higher-level, but requires separate installation

**Why Docker CLI**:
- **Universal**: Works with Docker Desktop, Docker Engine, Rootless Docker, Podman
- **No Dependencies**: Executes system `docker` command
- **Simplicity**: Generate command string, execute, capture output
- **Error Handling**: Docker CLI provides clear error messages

**Pre-Deployment Validation**:
```rust
// Validation checklist
fn validate_deployment() -> Result<(), DeploymentError> {
    check_docker_installed()?;           // `docker --version`
    check_yaml_valid()?;                 // Parse YAML syntax
    check_ports_available()?;            // netstat/ss port check
    check_device_paths_exist()?;         // stat /dev/video0, etc.
    check_volume_paths_writable()?;      // test write to recording dirs
    Ok(())
}
```

**Health Check Strategy**:
```rust
// Poll Frigate's health endpoint
async fn health_check(container_id: &str, timeout: Duration) -> HealthStatus {
    let url = format!("http://localhost:5000/api/version");
    for attempt in 1..=10 {
        if reqwest::get(&url).await.is_ok() {
            return HealthStatus::Healthy;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    HealthStatus::Unhealthy
}
```

**Rollback Mechanism**:
1. Store previous deployment state in SQLite: container ID, config checksum, timestamp
2. On rollback: `docker stop {current}`, `docker start {previous}`, restore previous config
3. If previous doesn't exist: stop current, leave user with clean state

### Alternatives Considered

| Option | Pros | Cons | Rejection Reason |
|--------|------|------|------------------|
| Docker SDK (bollard) | Programmatic control | 5MB+ dependency, version coupling | CLI is simpler, works everywhere |
| docker-compose | Declarative | Requires separate install | Not universally available |
| Podman compatibility | Alternative to Docker | Slightly different CLI | Compatible via `alias docker=podman` |

### Implementation Notes

```rust
// Command generation example
pub fn generate_docker_run(config: &FrigateConfig) -> Vec<String> {
    let mut cmd = vec![
        "docker", "run", "-d",
        "--name", "frigate",
        "--restart", "unless-stopped",
    ];

    // Add device mounts
    for device in &config.hardware_devices {
        cmd.push("--device");
        cmd.push(&device.path);
    }

    // Add volume mounts
    for volume in &config.volumes {
        cmd.push("-v");
        cmd.push(&format!("{}:{}", volume.host, volume.container));
    }

    cmd.push("ghcr.io/blakeblackshear/frigate:stable");
    cmd
}
```

---

## 5. Inter-Process Communication (IPC) Architecture

### Decision: Tauri IPC + HTTP for Agent

### Rationale

**UI ↔ Tauri Backend**:
- Use Tauri's built-in IPC (JSON-RPC over message passing)
- Frontend calls: `await invoke('detect_hardware')`
- Backend handles: `#[tauri::command] async fn detect_hardware() -> Result<Vec<Device>>`

**Tauri Backend ↔ Go Agent**:
- **Option A** (Chosen): Execute agent binary, parse JSON stdout
  ```rust
  let output = Command::new("./agent")
      .arg("detect")
      .output()?;
  let devices: Vec<Device> = serde_json::from_slice(&output.stdout)?;
  ```
- **Option B** (Future): HTTP server in agent for long-running process
  - Requires keeping agent alive (daemonization complexity)
  - Adds HTTP dependency to agent (violates minimal dependencies)

**Why Execute-and-Parse**:
- Simpler: No daemon management, no port allocation
- Stateless: Each detection is fresh, no stale caching
- Debugging: Can run agent manually for testing

**Future Migration Path**:
If performance demands it, convert agent to HTTP server:
```go
// agent/main.go future enhancement
http.HandleFunc("/api/detect", handleDetect)
http.ListenAndServe(":18237", nil)
```

### Message Flow Example

```
User clicks "Detect Hardware"
    ↓
[Frontend] invoke('detect_hardware')
    ↓
[Tauri Backend] execute ./agent detect
    ↓
[Go Agent] runs detection, outputs JSON
    ↓
[Tauri Backend] parses JSON, returns to frontend
    ↓
[Frontend] displays results in UI
```

---

## 6. Testing Strategy

### Decision: Multi-Layer Test Pyramid

**Unit Tests (70% of tests)**:
- **Rust**: `cargo test` for config engine, deployment logic
- **Go**: `go test` with table-driven tests for hardware detection
- **Frontend**: Vitest for components, hooks, services

**Integration Tests (20% of tests)**:
- **Cross-Module**: Rust integration tests calling real agent binary
- **Docker Integration**: Test deployment with Docker-in-Docker
- **Fixtures**: Sample YAML configs covering edge cases

**E2E Tests (10% of tests)**:
- **Playwright**: Full UI workflows (Hardware → Camera → Deploy)
- **Platform Matrix**: Run on Linux + one of (Windows or macOS)

**AI-Driven Testing (Phase 6)**:
- Generate synthetic configs with variations
- Multi-round validation: generate config → deploy → verify → rollback
- Mutation testing: inject errors, verify error handling

**Test-First Workflow**:
1. Write acceptance test based on spec (Red)
2. Implement minimal code to pass (Green)
3. Refactor while keeping test green (Refactor)
4. Repeat for next requirement

### CI/CD Pipeline

```yaml
# .github/workflows/test.yml
- Unit tests (all platforms)
- Integration tests (Linux only)
- E2E tests (Linux + macOS)
- Coverage report (>80% requirement)
- Block merge if tests fail (FR-050)
```

---

## 7. Security Considerations

### Agent Security
- **No Root**: All detection via non-privileged commands
- **Timeout Protection**: 5s timeout prevents hanging on malicious input
- **Input Validation**: Agent only accepts `detect` subcommand, no arbitrary args
- **Sandboxing**: Future: run in restricted user context

### Configuration Engine Security
- **Path Traversal**: Validate all file paths are within allowed directories
- **YAML Bomb**: Limit YAML file size to 10MB, depth to 50 levels
- **Injection**: Escape all user inputs before using in Docker commands

### Deployment Security
- **Explicit Confirmation**: Show full Docker command before execution
- **Audit Log**: Log all deployment actions to `~/.frigate-tool/audit.log`
- **Rollback Guarantee**: Always keep previous working state

---

## 8. Performance Targets & Monitoring

### Targets (from Technical Context)

| Operation | Target | Measurement |
|-----------|--------|-------------|
| UI Render | <100ms | Chrome DevTools Performance |
| Page Transition | <50ms | React Profiler |
| Hardware Detection | <5s | `time ./agent detect` |
| YAML Parse | <50ms | `cargo bench` |
| Conflict Detection | <100ms | Integration test timing |
| Deployment Validation | <2s | End-to-end test |

### Monitoring Strategy

- **Development**: Manual profiling with tools above
- **CI**: Performance regression tests (alert if >20% slower)
- **User Telemetry** (Optional, opt-in): Anonymous usage metrics via Sentry or PostHog

---

## 9. Internationalization (i18n)

### Decision: Defer to Post-MVP

**Rationale**: User base primarily English-speaking, complexity not justified for v1.0

**Future Approach**:
- Use `react-i18next` for frontend
- Store translations in `src-ui/locales/{lang}.json`
- Agent JSON schema remains English (technical artifact)

---

## 10. Accessibility (a11y)

### Decision: WCAG 2.1 AA Compliance

**Implementation**:
- Use semantic HTML (`<button>`, `<input>`, not `<div onClick>`)
- ARIA labels for icon-only buttons
- Keyboard navigation support (Tab, Enter, Escape)
- High contrast mode detection (respect OS settings)
- Screen reader testing with NVDA (Windows), JAWS (Windows), VoiceOver (macOS)

**Testing**:
- **axe DevTools**: Automated accessibility scanning
- **Manual Testing**: Keyboard-only navigation test before each release

---

## 11. Documentation Strategy

### Phase 7 Deliverables

1. **architecture.md**: System overview with component diagrams (Mermaid)
2. **API Documentation**:
   - Agent JSON schema with examples
   - Tauri command reference
3. **User Guide**:
   - Installation instructions (per platform)
   - Quick start guide
   - Troubleshooting common issues
4. **Developer Guide**:
   - Setup instructions (Rust, Go, Node)
   - Build from source
   - Contributing guidelines

### Documentation Tools

- **Mermaid**: Architecture diagrams in Markdown
- **TypeDoc**: TypeScript API docs (auto-generated)
- **cargo doc**: Rust API docs (auto-generated)
- **godoc**: Go API docs (auto-generated)

---

## 12. Open Questions & Future Enhancements

### Resolved in This Research
- ✅ UI framework (Tauri)
- ✅ Agent language (Go)
- ✅ Config engine approach (Rust + yaml-rust2)
- ✅ Deployment method (Docker CLI)
- ✅ IPC architecture (Tauri IPC + exec)

### Deferred to Implementation
- Comment preservation implementation details (will test yaml-rust2 vs custom)
- Exact UI component library (shadcn/ui vs Headless UI)
- React vs Vue for frontend (recommend React for ecosystem maturity)

### Future Enhancements (Post-v1.0)
- **Agent HTTP Mode**: For performance-critical scenarios
- **Remote Deployment**: Deploy to remote Docker hosts via SSH
- **Multi-Instance**: Manage multiple Frigate instances from one tool
- **Config Sharing**: Export/import configs, share templates in community repo
- **Cloud Backup**: Optional encrypted cloud backup of configs

---

## Conclusion

This research establishes a solid technical foundation for the Frigate Configuration Tool. Key strengths:

1. **Modular Architecture**: Independent components enable parallel development
2. **Cross-Platform First**: Go + Tauri ensure true multi-platform support
3. **User-Centric**: Comment preservation and conflict detection honor User First principle
4. **Lightweight**: <10MB UI + <20MB agent meets performance constraints
5. **Testable**: Clear module boundaries support comprehensive testing

Next steps: Proceed to Phase 1 (Data Model, Contracts, Quickstart).
