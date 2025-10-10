# Changelog

All notable changes to the Frigate Configuration Tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- API documentation generation
- Comprehensive user guide
- Official v1.0.0 release

## [0.9.0] - 2025-10-10

### Added - Phase 8: Disk & Volume Mapping
- **Disk Information Display** - Real-time disk space monitoring
- **Low Space Warnings** - Alerts when disk space < 10GB
- **Volume Mapping Configuration** - Visual volume selector
- **Path Validation** - Validate paths before deployment
- **Docker Integration** - Automatic volume mapping in deployments
- **Recommended Paths** - Smart path suggestions based on available space

### Added - Phase 9: Polish & Release
- **Architecture Documentation** - Complete system architecture diagrams
- **Release Automation** - GitHub Actions for multi-platform builds
- **Build Scripts** - Automated agent and Tauri build scripts
- **Test Coverage** - Verified >80% coverage on core modules
- **README Updates** - Comprehensive project documentation

## [0.8.0] - 2025-10-09

### Added - Phase 7: Manual Configuration Override
- **YAML Editor** - Syntax highlighting with Monaco Editor
- **Comment Preservation** - Maintains all YAML comments
- **Real-time Validation** - Live syntax checking
- **Manual Edit Tracking** - Marks manually edited fields
- **Two-way Sync** - YAML ↔ UI preview synchronization

## [0.7.0] - 2025-10-08

### Added - Phase 6: Cross-Platform Hardware Detection
- **ARM Architecture Support** - Full ARM64 detection
- **Hailo NPU Detection** - Support for Hailo AI accelerators on Linux
- **Fallback Logic** - Graceful handling of missing system tools
- **CPU-only Detection** - Performance warnings for systems without accelerators
- **Platform-specific Paths** - Correct path formatting per OS

## [0.6.0] - 2025-10-07

### Added - Phase 5: Safe Deployment with Validation
- **Pre-deployment Validation** - YAML, Docker, ports, volumes checks
- **Docker Run Support** - Generate and execute Docker run commands
- **Docker Compose Support** - Generate docker-compose.yml
- **Health Check Monitoring** - Automatic container health verification
- **Deployment Rollback** - One-click rollback on failure
- **Deployment History** - SQLite-backed deployment tracking
- **Container Logs** - Real-time log streaming and export

### Added - Phase 5: Deploy Page
- **Validation Results Display** - Clear validation feedback
- **Deployment Progress** - Step-by-step progress tracking
- **Health Check Status** - Visual health check indicators
- **Command Preview** - View generated Docker commands

## [0.5.0] - 2025-10-06

### Added - Phase 4: Configuration Management
- **YAML Parser** - Comment-preserving YAML parsing
- **Conflict Detection** - Automatic merge conflict detection
- **Non-destructive Merge** - Preserve user customizations
- **Automatic Backups** - Backup before every change
- **Rollback System** - One-click restore from backups
- **Backup Retention** - Keep last 10 backups automatically

### Added - Phase 4: Manual Config Page
- **Visual YAML Editor** - Syntax highlighting
- **Conflict Resolution UI** - Three-way merge interface
- **Backup List** - Browse and restore backups
- **Unsaved Changes Warning** - Prevent accidental data loss

## [0.4.0] - 2025-10-05

### Added - Phase 3: Hardware Detection & Camera Setup
- **Hardware Detection** - Automatic GPU/TPU/accelerator detection
  - Intel QuickSync (Linux, Windows)
  - NVIDIA CUDA/NVENC (Linux, Windows)
  - AMD VAAPI (Linux)
  - Apple VideoToolbox (macOS)
  - Coral TPU (Linux)
- **Cross-platform Agent** - Go-based detection agent
  - Linux support (x86_64, ARM64)
  - Windows support (x86_64)
  - macOS support (x86_64, ARM64)

### Added - Phase 3: Camera Configuration
- **Visual Camera Setup** - No YAML editing required
- **Hardware Assignment** - Assign accelerators to cameras
- **RTSP Configuration** - URL, resolution, FPS settings
- **Live YAML Preview** - See generated configuration
- **Camera Validation** - Input validation and error handling

## [0.3.0] - 2025-10-04

### Added - Phase 2: Foundational Infrastructure
- **SQLite Database** - Backup and deployment history storage
- **Data Models** - Core Rust data structures
  - HardwareDevice
  - CameraConfiguration
  - DeploymentState
  - VolumeMapping
  - ConflictResolution
- **Tauri IPC Framework** - Command layer for frontend-backend communication
- **Error Handling** - Comprehensive error types and handling
- **Logging Infrastructure** - Structured logging (Rust + Go)

## [0.2.0] - 2025-10-03

### Added - Phase 1: Project Setup
- **Project Structure** - Modular monorepo layout
- **Tauri + Rust Backend** - Core application framework
- **React + TypeScript Frontend** - Modern UI framework
- **Go Agent Module** - Hardware detection agent
- **Development Environment** - ESLint, Prettier, Clippy, rustfmt
- **CI/CD Scaffolding** - GitHub Actions workflows
- **Test Infrastructure** - Unit, integration, E2E test setup

## [0.1.0] - 2025-10-02

### Added
- Initial project scaffolding
- Project constitution and principles
- Feature specifications
- Implementation plan
- Task breakdown (205 tasks)

---

## Development Statistics

- **Total Tasks**: 205
- **Completed**: 202 (98.5%)
- **Test Coverage**: >80% on core modules
- **Unit Tests**: 101
- **Integration Tests**: 24 files
- **E2E Tests**: Full user flow coverage

## Architecture

- **Frontend**: React 18 + TypeScript + Vite
- **Backend**: Rust 1.75+ + Tauri 1.5+
- **Agent**: Go 1.21+
- **Database**: SQLite
- **Deployment**: Docker CLI

## Supported Platforms

- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)
- Windows (x86_64)

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)

[Unreleased]: https://github.com/YOUR_ORG/frigate-config/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/YOUR_ORG/frigate-config/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/YOUR_ORG/frigate-config/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/YOUR_ORG/frigate-config/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/YOUR_ORG/frigate-config/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/YOUR_ORG/frigate-config/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/YOUR_ORG/frigate-config/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/YOUR_ORG/frigate-config/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/YOUR_ORG/frigate-config/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/YOUR_ORG/frigate-config/releases/tag/v0.1.0
