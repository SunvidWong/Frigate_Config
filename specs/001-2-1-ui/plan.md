# Implementation Plan: Frigate Configuration Tool - Complete System

**Branch**: `001-2-1-ui` | **Date**: 2025-10-08 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-2-1-ui/spec.md`

**Note**: This plan follows the 7-phase implementation strategy provided by the user.

## Summary

Build a cross-platform visual configuration tool for Frigate NVR that eliminates the YAML configuration barrier through an iOS-style desktop application. The system consists of four independent modules: (1) UI for user interaction, (2) Agent for hardware detection, (3) Configuration Engine for YAML generation with conflict detection and rollback, and (4) Deployment Module for Docker orchestration with health checks. Technical approach leverages Tauri for lightweight cross-platform UI, Go for single-binary agent distribution, and a YAML library with comment preservation for non-destructive configuration merging.

## Technical Context

### UI Module
**Language/Version**: Rust 1.75+ (Tauri backend), TypeScript 5.0+ (React/Vue frontend)
**Primary Dependencies**: Tauri 1.5+, React 18 or Vue 3, shadcn/ui or similar iOS-style component library
**Testing**: cargo test (Rust), Vitest or Jest (frontend), Playwright (E2E UI testing)
**Target Platform**: Desktop application (Windows 10+, Linux with GTK 3+, macOS 11+)
**Performance Goals**: UI renders in <100ms, page transitions <50ms, real-time log streaming at 60fps
**Constraints**: <10MB application bundle, <200MB RAM usage at idle, offline-capable after initial setup

### Agent Module
**Language/Version**: Go 1.21+
**Primary Dependencies**: Standard library only (os/exec for system commands, encoding/json for output)
**Testing**: go test with table-driven tests, mock system command execution
**Target Platform**: Cross-compile to Linux (x86/ARM), Windows (x86/ARM), macOS (x86/ARM)
**Performance Goals**: Hardware detection completes in <5 seconds, JSON response generation <100ms
**Constraints**: Single binary <20MB, zero runtime dependencies, no elevated privileges required

### Configuration Engine
**Language/Version**: Rust 1.75+ (embedded in Tauri backend) or Go 1.21+ (separate service)
**Primary Dependencies**: serde_yaml + custom merge logic (Rust) or goccy/go-yaml (Go)
**Storage**: Local filesystem (YAML files), SQLite for backup history and metadata
**Testing**: Unit tests for merge logic, integration tests with sample configs, property-based testing for conflict detection
**Performance Goals**: YAML parse/generate <50ms for typical configs, conflict detection <100ms
**Constraints**: Must preserve comments and formatting, atomic file operations, backup retention configurable

### Deployment Module
**Language/Version**: Go 1.21+ or Rust 1.75+ (matches Configuration Engine choice)
**Primary Dependencies**: Docker CLI via os/exec, no Docker SDK (reduces dependencies)
**Testing**: Integration tests with Docker-in-Docker, mock Docker commands for unit tests
**Performance Goals**: Pre-deployment validation <2 seconds, health check polling every 5 seconds
**Constraints**: Must work with Docker installed via snap/brew/native, handle rootless Docker

**Project Type**: Desktop application with embedded backend services (modular architecture)
**Scale/Scope**: Single-user application, ~15-20 UI pages/views total, 4 independent modules with ~5-10K LOC each

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with Frigate Configuration Tool Constitution (v1.0.0):

- [x] **User First**: ✅ YES - Configuration Engine includes explicit conflict detection (FR-022, FR-023), user override choice (FR-024), automatic backups (FR-025), and rollback capability (FR-026). Non-destructive merging (FR-027) with explicit deletion markers (FR-028) ensures user configurations are never silently overwritten.

- [x] **Automation + Control**: ✅ YES - System auto-generates configurations (FR-020, FR-021) while preserving manual editability (FR-004 Manual Config page, FR-030 comment preservation). Users can switch between GUI and YAML editing at any time.

- [x] **Cross-Platform Support**: ✅ YES - Agent designed for Windows/Linux/macOS on x86/ARM (FR-014, FR-015). Tauri UI targets all desktop platforms. Test plan includes platform-specific validation (User Story 4). Go's cross-compilation ensures single codebase for all platforms.

- [x] **Modular Design**: ✅ YES - Four independent modules with clear boundaries: (1) UI communicates via Tauri IPC, (2) Agent exposes JSON API (FR-013), (3) Configuration Engine has file-based interface, (4) Deployment Module uses command execution interface. Each module independently testable (FR-044 through FR-048).

- [x] **Security & Permissions**: ✅ YES - Agent explicitly avoids driver scanning (FR-016), operates without elevation (FR-017). Deployment module requires explicit user action for Docker commands (FR-034 through FR-036 validation gates). Audit logging via timestamped backups (FR-025, FR-031) and deployment history.

- [x] **AI-Driven Testing**: ✅ YES - Test-first development enforced through acceptance scenarios in spec. Each module has dedicated test strategy (cargo test, go test, Playwright). Multi-round AI testing planned (Phase 6, FR-049). Coverage target >80% (SC-004). Tests block deployment on failure (FR-050).

- [x] **Open Source Community**: ✅ YES - Apache 2.0 license. Documentation phase (Phase 7) includes architecture.md, API docs, user guide. Public APIs (Agent JSON schema, Tauri commands) will have usage examples. quickstart.md generated in Phase 1.

**Violations Requiring Justification**: None - all principles satisfied.

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
frigate-config-tool/
├── src/                          # Tauri Rust backend
│   ├── main.rs                   # Tauri entry point
│   ├── commands/                 # Tauri command handlers (IPC)
│   │   ├── agent.rs              # Hardware detection commands
│   │   ├── config.rs             # Configuration engine commands
│   │   └── deploy.rs             # Deployment commands
│   ├── config_engine/            # Configuration Engine module
│   │   ├── parser.rs             # YAML parsing
│   │   ├── merger.rs             # Conflict detection & merging
│   │   ├── backup.rs             # Backup management
│   │   └── validator.rs          # Schema validation
│   └── deployment/               # Deployment Module
│       ├── docker.rs             # Docker command generation
│       ├── health.rs             # Health check logic
│       └── rollback.rs           # Rollback management
│
├── agent/                        # Go Agent Module (separate binary)
│   ├── cmd/
│   │   └── agent/
│   │       └── main.go           # Agent CLI entry point
│   ├── internal/
│   │   ├── detect/
│   │   │   ├── linux.go          # Linux hardware detection
│   │   │   ├── windows.go        # Windows hardware detection
│   │   │   ├── darwin.go         # macOS hardware detection
│   │   │   └── common.go         # Shared detection logic
│   │   └── schema/
│   │       └── hardware.go       # JSON schema definitions
│   └── go.mod
│
├── src-ui/                       # Frontend (React/Vue)
│   ├── src/
│   │   ├── pages/                # 6 main pages
│   │   │   ├── Hardware.tsx
│   │   │   ├── Cameras.tsx
│   │   │   ├── ManualConfig.tsx
│   │   │   ├── Deploy.tsx
│   │   │   ├── DiskMapping.tsx
│   │   │   └── Logs.tsx
│   │   ├── components/           # Shared UI components
│   │   ├── hooks/                # React hooks for Tauri IPC
│   │   ├── services/             # Frontend service layer
│   │   └── types/                # TypeScript types
│   ├── package.json
│   └── tsconfig.json
│
├── tests/
│   ├── contract/                 # Contract tests (Agent JSON API)
│   ├── integration/              # Cross-module integration tests
│   ├── e2e/                      # Playwright UI tests
│   └── fixtures/                 # Test data (sample YAML configs)
│
├── docs/                         # Generated documentation
│   ├── architecture.md
│   ├── api/                      # API documentation
│   └── user-guide/
│
├── Cargo.toml                    # Rust dependencies (Tauri)
├── tauri.conf.json               # Tauri configuration
└── .github/
    └── workflows/
        ├── test.yml              # CI: run tests
        ├── build.yml             # CI: build multi-platform
        └── release.yml           # CI: release automation
```

**Structure Decision**: Modular monorepo structure with three main components:

1. **Tauri Backend (src/)**: Rust-based backend handles Configuration Engine and Deployment Module. Chosen for tight integration with UI via IPC and native system access.

2. **Go Agent (agent/)**: Standalone binary for hardware detection. Separate from main app for easier cross-compilation and potential future use as standalone CLI tool.

3. **Frontend (src-ui/)**: Modern web-based UI using React/TypeScript. Leverages Tauri's webview for cross-platform rendering with native performance.

This structure supports independent module development (Constitution Principle IV) while maintaining simplicity through a single repository.

## Complexity Tracking

*No violations identified - Constitution Check passed completely.*

However, noting architectural complexity for transparency:

| Design Choice | Justification | Simpler Alternative Considered |
|---------------|---------------|-------------------------------|
| Two languages (Rust + Go) | Go provides superior cross-compilation for agent binary; Rust integrates natively with Tauri for UI backend | Pure Rust: Rejected due to less mature cross-platform system detection libraries compared to Go's stdlib |
| Separate Agent binary | Enables independent distribution, easier testing, and potential standalone CLI use | Embedded in Tauri: Rejected because cross-compiling Go from Rust build is complex; separate binary is cleaner |
| SQLite for backup metadata | Provides queryable history, atomic transactions, and version tracking | JSON files: Rejected due to lack of ACID guarantees for backup operations and difficulty querying history |
