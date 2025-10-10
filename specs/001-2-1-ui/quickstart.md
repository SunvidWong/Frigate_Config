# Quickstart Guide: Frigate Configuration Tool

**Version**: 1.0.0
**Date**: 2025-10-08
**Target Audience**: Developers and early testers

## Overview

This quickstart guide walks you through building, running, and testing the Frigate Configuration Tool from source. For end-user installation instructions, see the User Guide (Phase 7).

---

## Prerequisites

### Required Software

#### All Platforms

- **Git** - Version control
- **Docker** - For Frigate deployment (not required for development/testing UI)

#### For Tauri UI + Rust Backend

- **Rust** 1.75 or later
  - Install: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Node.js** 18+ and npm/yarn
  - Install: https://nodejs.org/ or use nvm
- **System Dependencies** (platform-specific):
  - **Linux**: `sudo apt install libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`
  - **macOS**: Xcode Command Line Tools: `xcode-select --install`
  - **Windows**: Microsoft Visual Studio C++ Build Tools

#### For Go Agent

- **Go** 1.21 or later
  - Install: https://golang.org/dl/

### Recommended Tools

- **VS Code** with extensions: Rust Analyzer, Go, Tauri
- **Docker Desktop** (macOS/Windows) or Docker Engine (Linux)

---

## Project Setup

### 1. Clone Repository

```bash
git clone https://github.com/frigate-config-tool/frigate-config-tool.git
cd frigate-config-tool
```

### 2. Install Dependencies

#### Frontend Dependencies

```bash
cd src-ui
npm install
cd ..
```

#### Rust Dependencies

```bash
cargo fetch
```

#### Go Agent Dependencies

```bash
cd agent
go mod download
cd ..
```

---

## Development Workflow

### Option A: Run Complete Stack

This starts the Tauri app in development mode with hot reload.

```bash
# Terminal 1: Build agent first
cd agent
go build -o ../src-tauri/bin/agent cmd/agent/main.go
cd ..

# Terminal 2: Start Tauri dev mode (includes frontend + backend)
npm run tauri dev
```

The app opens in a window. Frontend changes auto-reload. Rust changes trigger recompilation.

### Option B: Run Components Separately

#### Run Agent Standalone

```bash
cd agent
go run cmd/agent/main.go detect
```

Output: JSON hardware detection results.

#### Run Frontend Only (without Tauri)

```bash
cd src-ui
npm run dev
```

Opens in browser at `http://localhost:5173`. Tauri commands will fail (mock them for pure frontend dev).

#### Test Rust Backend Only

```bash
cargo test
```

---

## Building for Production

### Build All Components

```bash
# 1. Build agent binaries for target platforms
cd agent
GOOS=linux GOARCH=amd64 go build -o ../src-tauri/bin/agent-linux-amd64 cmd/agent/main.go
GOOS=darwin GOARCH=arm64 go build -o ../src-tauri/bin/agent-darwin-arm64 cmd/agent/main.go
GOOS=windows GOARCH=amd64 go build -o ../src-tauri/bin/agent-windows-amd64.exe cmd/agent/main.go
cd ..

# 2. Build Tauri app
npm run tauri build
```

Output locations:
- **Linux**: `src-tauri/target/release/bundle/appimage/frigate-config-tool_1.0.0_amd64.AppImage`
- **macOS**: `src-tauri/target/release/bundle/dmg/Frigate Config Tool_1.0.0_aarch64.dmg`
- **Windows**: `src-tauri/target/release/bundle/msi/Frigate Config Tool_1.0.0_x64_en-US.msi`

---

## Quick Test Scenarios

### Test 1: Hardware Detection

**Goal**: Verify agent detects devices on your platform.

```bash
# Run agent
cd agent
go run cmd/agent/main.go detect | jq

# Expected: JSON output with devices array
# Verify: platform matches your OS, devices array has entries (or empty if no hardware)
```

### Test 2: Load Configuration

**Goal**: Verify configuration engine parses YAML.

**Setup**: Create `test-config.yml`:

```yaml
mqtt:
  enabled: true
  host: mqtt.local

cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://camera.local/stream
          roles:
            - detect
```

**Test via Tauri** (after running `npm run tauri dev`):

1. Launch app
2. Click "Load Configuration"
3. Select `test-config.yml`
4. Verify: Configuration loads, cameras tab shows "front_door"

### Test 3: Conflict Detection

**Goal**: Verify merge logic detects conflicts.

**Setup**: Use existing config from Test 2.

**Test**:

1. In app, apply a template that sets `mqtt.host` to a different value
2. Click "Merge Template"
3. Verify: Conflict dialog appears showing existing value vs template value
4. Choose resolution (keep/override)
5. Verify: Merged config reflects your choice

### Test 4: Deployment Validation

**Goal**: Verify pre-deployment checks work.

**Prerequisites**: Docker installed and running.

**Test**:

1. Load a valid config
2. Navigate to "Deploy Frigate" page
3. Click "Validate"
4. Verify checks:
   - ✅ Docker available
   - ✅ YAML syntax valid
   - ✅ Ports free (5000, 8554)
   - ⚠ Device paths (may warn if no hardware)

---

## Common Issues & Solutions

### Issue: `agent: command not found`

**Cause**: Agent binary not built or not in expected location.

**Solution**:
```bash
cd agent
go build -o ../src-tauri/bin/agent cmd/agent/main.go
```

### Issue: `webkit2gtk` not found (Linux)

**Cause**: Missing system dependencies.

**Solution**:
```bash
sudo apt install libwebkit2gtk-4.0-dev libgtk-3-dev
```

### Issue: Frontend shows "Failed to invoke command"

**Cause**: Tauri backend not running or command not registered.

**Solution**:
- Ensure `npm run tauri dev` is running (not just `npm run dev`)
- Check Rust console for compilation errors

### Issue: Docker validation fails

**Cause**: Docker not running or permission issues (Linux).

**Solution**:
- Start Docker: `sudo systemctl start docker` (Linux) or open Docker Desktop (macOS/Windows)
- Add user to docker group (Linux): `sudo usermod -aG docker $USER`, then logout/login

### Issue: Hardware detection returns empty array

**Cause**: May be expected if no GPU/TPU/cameras connected, or permission issues.

**Solution**:
- Verify hardware is connected
- On Linux, check permissions: `ls -l /dev/video*`
- Add user to video group: `sudo usermod -aG video $USER`

---

## Directory Structure Reference

```
frigate-config-tool/
├── agent/                    # Go agent module
│   ├── cmd/agent/main.go     # CLI entry point
│   ├── internal/detect/      # Platform-specific detection
│   └── go.mod
│
├── src/                      # Tauri Rust backend
│   ├── main.rs               # Tauri setup
│   ├── commands/             # IPC command handlers
│   ├── config_engine/        # YAML manipulation
│   └── deployment/           # Docker operations
│
├── src-ui/                   # React frontend
│   ├── src/
│   │   ├── pages/            # UI pages
│   │   ├── components/       # Shared components
│   │   └── hooks/            # Tauri IPC hooks
│   └── package.json
│
├── src-tauri/                # Tauri configuration
│   ├── tauri.conf.json       # Tauri settings
│   ├── Cargo.toml            # Rust dependencies
│   └── bin/                  # Agent binaries (built)
│
└── tests/                    # Integration & E2E tests
```

---

## Running Tests

### Agent Tests (Go)

```bash
cd agent
go test ./... -v
```

Tests: Hardware detection logic, JSON schema validation, timeout handling.

### Rust Backend Tests

```bash
cargo test
```

Tests: Configuration engine, merge logic, conflict detection, deployment command generation.

### Frontend Tests

```bash
cd src-ui
npm test
```

Tests: Component rendering, hooks, service layer.

### Integration Tests

```bash
cargo test --test integration
```

Tests: Tauri commands end-to-end, agent invocation, file operations.

### E2E Tests (requires built app)

```bash
cd tests/e2e
npm install
npx playwright test
```

Tests: Full user workflows (hardware detection → config → deploy).

---

## Configuration Files

### Development Configuration

**`.env` (create in project root)**:

```env
RUST_LOG=info
TAURI_DEV_WATCHER=true
```

**`tauri.conf.json`** (modify for dev):

```json
{
  "build": {
    "devPath": "http://localhost:5173",
    "beforeDevCommand": "cd src-ui && npm run dev"
  }
}
```

### Test Configuration

**`tests/fixtures/sample-config.yml`**: Sample Frigate configs for testing.

---

## Next Steps

1. **Review Architecture**: Read `docs/architecture.md` (Phase 7 deliverable)
2. **API Reference**: Study `specs/001-2-1-ui/contracts/` for command specifications
3. **Data Model**: Review `specs/001-2-1-ui/data-model.md` for entity relationships
4. **Contribute**: Follow `CONTRIBUTING.md` (Phase 7 deliverable)

---

## Development Tools

### Recommended VS Code Extensions

- **Rust Analyzer** - Rust IntelliSense
- **Go** - Go language support
- **Tauri** - Tauri-specific commands
- **ES Lint** - Frontend linting
- **Prettier** - Code formatting

### Debug Configuration

**`.vscode/launch.json`** (for debugging Rust):

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tauri",
      "cargo": {
        "args": ["build", "--bin", "frigate-config-tool"]
      },
      "program": "${cargo:program}"
    }
  ]
}
```

### Useful Commands

```bash
# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy

# Format Go code
cd agent && go fmt ./...

# Lint Go code
cd agent && golangci-lint run

# Format frontend code
cd src-ui && npm run format

# Lint frontend code
cd src-ui && npm run lint

# Clean build artifacts
cargo clean
cd src-ui && rm -rf node_modules dist
```

---

## Performance Profiling

### Profile Agent Performance

```bash
cd agent
go test -bench=. -cpuprofile=cpu.prof ./internal/detect/
go tool pprof -http=:8080 cpu.prof
```

### Profile Tauri Backend

```bash
cargo build --release --features profiling
valgrind --tool=callgrind ./target/release/frigate-config-tool
```

### Profile Frontend

Use Chrome DevTools Performance tab:
1. Run `npm run tauri dev`
2. Open DevTools (F12)
3. Performance → Record
4. Perform actions (hardware detection, config load)
5. Stop recording, analyze flame graph

---

## CI/CD Preview

### GitHub Actions Workflow (Phase 7)

The project will include CI/CD workflows for:

- **test.yml**: Run tests on every PR (Linux/macOS/Windows matrix)
- **build.yml**: Build binaries for releases
- **release.yml**: Publish GitHub releases with installers

To test locally before Phase 7:

```bash
# Install act (GitHub Actions local runner)
brew install act  # macOS
# or: curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Run test workflow
act pull_request
```

---

## Troubleshooting Resources

- **Tauri Docs**: https://tauri.app/v1/guides/
- **Go Cross-Compilation**: https://golang.org/doc/install/source#environment
- **Rust Cargo Book**: https://doc.rust-lang.org/cargo/
- **GitHub Discussions**: https://github.com/frigate-config-tool/discussions

---

## Quick Reference Card

| Task | Command |
|------|---------|
| Start dev server | `npm run tauri dev` |
| Build agent | `cd agent && go build -o ../src-tauri/bin/agent cmd/agent/main.go` |
| Run tests | `cargo test && cd agent && go test ./... && cd ../src-ui && npm test` |
| Format code | `cargo fmt && cd agent && go fmt ./... && cd ../src-ui && npm run format` |
| Build release | `npm run tauri build` |
| Clean build | `cargo clean && rm -rf src-ui/node_modules src-ui/dist` |

---

## Support

- **Issues**: https://github.com/frigate-config-tool/issues
- **Discussions**: https://github.com/frigate-config-tool/discussions
- **Slack**: #frigate-config-tool (Frigate community Slack)

---

**Ready to start?** Run:

```bash
git clone https://github.com/frigate-config-tool/frigate-config-tool.git
cd frigate-config-tool
cd agent && go build -o ../src-tauri/bin/agent cmd/agent/main.go && cd ..
cd src-ui && npm install && cd ..
npm run tauri dev
```

**Happy coding! 🚀**
