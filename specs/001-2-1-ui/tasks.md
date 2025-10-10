# Tasks: Frigate Configuration Tool - Complete System

**Input**: Design documents from `/specs/001-2-1-ui/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Per Constitution Principle VI (Test-First Development - NON-NEGOTIABLE), tests MUST be written BEFORE implementation for ALL features. Tests are MANDATORY, not optional.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story. Each story follows Red-Green-Refactor: write tests → verify failure → implement → verify pass.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md modular monorepo structure:
- **Tauri Rust backend**: `src/` (Configuration Engine, Deployment Module)
- **Go Agent**: `agent/` (separate binary)
- **Frontend**: `src-ui/` (React/TypeScript)
- **Tests**: `tests/contract/`, `tests/integration/`, `tests/e2e/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure needed by all modules

- [X] T001 Initialize Rust/Cargo project with Tauri configuration in `Cargo.toml` and `tauri.conf.json`
- [X] T002 [P] Initialize Go module for agent in `agent/go.mod` with Go 1.21+
- [X] T003 [P] Initialize React TypeScript frontend in `src-ui/` with package.json, tsconfig.json, vite.config.ts
- [X] T004 [P] Create project directory structure: src/, agent/, src-ui/, tests/, docs/
- [X] T005 [P] Configure development environment: .env template, .gitignore, .editorconfig
- [X] T006 [P] Setup CI/CD workflow scaffolding in `.github/workflows/` (test.yml, build.yml)
- [X] T007 [P] Add linting and formatting tools: cargo fmt, eslint, prettier, golangci-lint configs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T008 Setup SQLite database schema for backup history and metadata in `src/database/schema.sql`
- [X] T009 [P] Create Tauri IPC command framework in `src/main.rs` with error handling utilities
- [X] T010 [P] Implement base data models in `src/models/` (HardwareDevice, CameraConfiguration, ConfigurationSnapshot)
- [X] T011 [P] Create Agent JSON schema definitions in `agent/internal/schema/hardware.go`
- [X] T012 [P] Setup frontend routing and page structure in `src-ui/src/App.tsx` with React Router
- [X] T013 [P] Create shared UI components library in `src-ui/src/components/` (Button, Input, Card, Modal)
- [X] T014 [P] Implement Tauri IPC hooks for frontend in `src-ui/src/hooks/useTauriCommand.ts`
- [X] T015 Setup logging infrastructure in `src/utils/logger.rs` (Rust) and `agent/internal/logger/logger.go` (Go)
- [X] T016 Create configuration file utilities in `src/config_engine/file_io.rs` (atomic read/write operations)
- [X] T017 [P] Setup test fixtures directory with sample YAML configs in `tests/fixtures/`
- [X] T018 [P] Configure test runners: cargo test, go test, vitest for all modules

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Hardware Discovery and Camera Setup (Priority: P1) 🎯 MVP

**Goal**: Enable users to detect hardware accelerators and configure cameras to use them for video processing

**Independent Test**: Launch UI → Hardware page shows detected devices → Cameras page allows adding camera with hardware selection → Configuration preview displays correctly

### Tests for User Story 1 (MANDATORY per Constitution Principle VI) ⚠️

**NOTE: Write these tests FIRST, ensure they FAIL before implementation**

**Constitution Requirement**: Test-first development is NON-NEGOTIABLE (Principle VI).
Tests MUST be written, run, and FAIL before any implementation code is written.

- [X] T019 [P] [US1] Contract test for Agent JSON output schema in `tests/contract/test_agent_schema.rs`
- [X] T020 [P] [US1] Integration test for hardware detection command in `tests/integration/test_detect_hardware.rs`
- [X] T021 [P] [US1] Unit tests for HardwareDevice model validation in `tests/unit/test_hardware_device.rs`
- [X] T022 [P] [US1] Unit tests for CameraConfiguration model in `tests/unit/test_camera_config.rs`
- [X] T023 [P] [US1] E2E test for Hardware page rendering in `tests/e2e/hardware-page.spec.ts`
- [X] T024 [P] [US1] E2E test for Cameras page workflow in `tests/e2e/cameras-page.spec.ts`

**Checkpoint - Tests Written**: Verify all tests written and failing before proceeding to implementation

### Implementation for User Story 1

#### Agent Module - Hardware Detection

- [X] T025 [P] [US1] Implement Linux hardware detection in `agent/internal/detect/linux.go` (lsusb, /dev/video*, /dev/dri/*)
- [X] T026 [P] [US1] Implement Windows hardware detection in `agent/internal/detect/windows.go` (WMIC, nvidia-smi, DirectShow)
- [X] T027 [P] [US1] Implement macOS hardware detection in `agent/internal/detect/darwin.go` (system_profiler, ioreg)
- [X] T028 [US1] Create common detection logic in `agent/internal/detect/common.go` (timeout handling, error aggregation)
- [X] T029 [US1] Implement Agent CLI main function in `agent/cmd/agent/main.go` with `detect` subcommand
- [X] T030 [US1] Add JSON output formatter in `agent/internal/schema/formatter.go`
- [X] T031 [US1] Verify tests now pass for Agent module

#### Tauri Backend - Hardware Command

- [X] T032 [US1] Implement `detect_hardware` Tauri command in `src/commands/agent.rs` (execute agent binary, parse JSON)
- [X] T033 [US1] Implement `get_device_details` Tauri command in `src/commands/agent.rs`
- [X] T034 [US1] Add agent binary path resolution logic in `src/utils/agent_path.rs`
- [X] T035 [US1] Add error handling for agent execution failures in `src/commands/agent.rs`
- [X] T036 [US1] Verify tests now pass for Tauri agent commands

#### Frontend - Hardware Page

- [X] T037 [P] [US1] Create Hardware page component in `src-ui/src/pages/Hardware.tsx`
- [X] T038 [P] [US1] Create DeviceCard component in `src-ui/src/components/DeviceCard.tsx` (displays device info)
- [X] T039 [P] [US1] Create DeviceList component in `src-ui/src/components/DeviceList.tsx` (grid/list view)
- [X] T040 [US1] Integrate hardware detection with Hardware page (call detect_hardware command)
- [X] T041 [US1] Add loading states and error handling for hardware detection in Hardware.tsx
- [X] T042 [US1] Implement device filtering/sorting in Hardware page
- [X] T043 [US1] Verify E2E tests pass for Hardware page

#### Frontend - Cameras Page

- [X] T044 [P] [US1] Create Cameras page component in `src-ui/src/pages/Cameras.tsx`
- [X] T045 [P] [US1] Create CameraForm component in `src-ui/src/components/CameraForm.tsx` (add/edit camera)
- [X] T046 [P] [US1] Create CameraList component in `src-ui/src/components/CameraList.tsx`
- [X] T047 [P] [US1] Create HardwareSelector component in `src-ui/src/components/HardwareSelector.tsx` (dropdown for device selection)
- [X] T048 [US1] Implement camera CRUD operations in Cameras page
- [X] T049 [US1] Add camera validation logic (RTSP URL, resolution, FPS) in `src-ui/src/utils/validation.ts`
- [X] T050 [US1] Implement configuration preview in Cameras page (show generated YAML snippet)
- [X] T051 [US1] Connect cameras to hardware devices (assign device ID to camera config)
- [X] T052 [US1] Add validation and error handling (Constitution Principle V: input validation required)
- [X] T053 [US1] Verify E2E tests pass for Cameras page

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Hardware detection works, cameras can be configured with hardware assignment.

---

## Phase 4: User Story 2 - Configuration Management with Conflict Detection (Priority: P2)

**Goal**: Enable users to safely modify existing configurations without losing custom settings

**Independent Test**: Load existing frigate.yml → Make changes via UI → Verify conflict detection → Choose resolution → Verify original settings preserved or explicitly overridden → Trigger rollback → Verify restoration

### Tests for User Story 2 (MANDATORY per Constitution Principle VI) ⚠️

- [X] T054 [P] [US2] Unit tests for YAML parser in `tests/unit/test_yaml_parser.rs`
- [X] T055 [P] [US2] Unit tests for conflict detection algorithm in `tests/unit/test_conflict_detection.rs`
- [X] T056 [P] [US2] Unit tests for merge logic in `tests/unit/test_yaml_merge.rs`
- [X] T057 [P] [US2] Integration test for backup creation in `tests/integration/test_backup.rs`
- [X] T058 [P] [US2] Integration test for rollback mechanism in `tests/integration/test_rollback.rs`
- [X] T059 [P] [US2] E2E test for conflict resolution workflow in `tests/e2e/conflict-resolution.spec.ts`

**Checkpoint - Tests Written**: Verify all tests written and failing before proceeding to implementation

### Implementation for User Story 2

#### Configuration Engine - Core Logic

- [X] T060 [P] [US2] Implement YAML parser with comment preservation in `src/config_engine/parser.rs` (using yaml-rust2)
- [X] T061 [P] [US2] Implement deep merge algorithm in `src/config_engine/merger.rs`
- [X] T062 [US2] Implement conflict detection logic in `src/config_engine/merger.rs` (AST traversal, value comparison)
- [X] T063 [US2] Create ConflictResolution model in `src/models/conflict_resolution.rs`
- [X] T064 [US2] Implement non-destructive merge rules in `src/config_engine/merger.rs`
- [X] T065 [US2] Add deletion marker support in `src/config_engine/merger.rs`
- [X] T066 [US2] Verify unit tests pass for configuration engine

#### Configuration Engine - Backup & Rollback

- [X] T067 [P] [US2] Implement backup creation logic in `src/config_engine/backup.rs` (timestamped files, SQLite metadata)
- [X] T068 [P] [US2] Implement backup listing in `src/config_engine/backup.rs`
- [X] T069 [US2] Implement rollback logic in `src/config_engine/backup.rs` (restore from backup, update current config)
- [X] T070 [US2] Add automatic backup before config changes in `src/config_engine/backup.rs`
- [X] T071 [US2] Implement backup retention policy (keep last 10) in `src/config_engine/backup.rs`
- [X] T072 [US2] Add structured logging for backup operations (Constitution Principle V: audit trail required)
- [X] T073 [US2] Verify integration tests pass for backup/rollback

#### Tauri Backend - Configuration Commands

- [X] T074 [P] [US2] Implement `load_configuration` Tauri command in `src/commands/config.rs`
- [X] T075 [P] [US2] Implement `save_configuration` Tauri command in `src/commands/config.rs` (with auto-backup)
- [X] T076 [P] [US2] Implement `merge_template` Tauri command in `src/commands/config.rs`
- [X] T077 [P] [US2] Implement `resolve_conflicts` Tauri command in `src/commands/config.rs`
- [X] T078 [P] [US2] Implement `list_backups` Tauri command in `src/commands/config.rs`
- [X] T079 [P] [US2] Implement `restore_backup` Tauri command in `src/commands/config.rs`
- [X] T080 [US2] Add validation and error handling for all config commands (Constitution Principle V)
- [X] T081 [US2] Verify tests pass for configuration Tauri commands

#### Frontend - Manual Config Page

- [X] T082 [P] [US2] Create ManualConfig page component in `src-ui/src/pages/ManualConfig.tsx`
- [X] T083 [P] [US2] Integrate YAML editor with syntax highlighting in ManualConfig.tsx (use monaco-editor or codemirror)
- [X] T084 [P] [US2] Create ConflictDialog component in `src-ui/src/components/ConflictDialog.tsx` (three-way merge UI)
- [X] T085 [P] [US2] Create BackupList component in `src-ui/src/components/BackupList.tsx`
- [X] T086 [US2] Implement load configuration workflow in ManualConfig.tsx
- [X] T087 [US2] Implement save configuration with validation in ManualConfig.tsx
- [X] T088 [US2] Implement conflict resolution UI flow in ConflictDialog.tsx
- [X] T089 [US2] Implement rollback UI in ManualConfig.tsx (backup dropdown, restore button)
- [X] T090 [US2] Add change tracking and unsaved changes warning in ManualConfig.tsx
- [X] T091 [US2] Verify E2E tests pass for conflict resolution workflow

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Users can detect hardware, configure cameras, and safely manage configurations with conflict detection and rollback.

---

## Phase 5: User Story 3 - Safe Deployment with Validation (Priority: P3)

**Goal**: Enable users to deploy Frigate to Docker with pre-validation and rollback capability

**Independent Test**: Complete configuration → Navigate to Deploy page → Run validation → Verify checks pass → Execute deployment → Monitor progress → Verify health checks → Test rollback functionality

### Tests for User Story 3 (MANDATORY per Constitution Principle VI) ⚠️

- [X] T092 [P] [US3] Unit tests for Docker command generation in `tests/unit/test_docker_commands.rs`
- [X] T093 [P] [US3] Unit tests for validation logic in `tests/unit/test_deployment_validation.rs`
- [X] T094 [P] [US3] Integration test for deployment execution in `tests/integration/test_deployment.rs`
- [X] T095 [P] [US3] Integration test for health checks in `tests/integration/test_health_check.rs`
- [X] T096 [P] [US3] Integration test for rollback in `tests/integration/test_deployment_rollback.rs`
- [X] T097 [P] [US3] E2E test for deployment workflow in `tests/e2e/deployment.spec.ts`

**Checkpoint - Tests Written**: ✅ All tests written and failing (RED phase complete)

### Implementation for User Story 3

#### Deployment Module - Core Logic

- [X] T098 [P] [US3] Implement Docker command generation in `src/deployment/docker.rs` (docker run/compose)
- [X] T099 [P] [US3] Implement device mount generation in `src/deployment/docker.rs` (--device flags)
- [X] T100 [P] [US3] Implement volume mount generation in `src/deployment/docker.rs` (-v flags)
- [X] T101 [US3] Create DeploymentState model in `src/models/deployment_state.rs`
- [X] T102 [US3] Create VolumeMapping model in `src/models/volume_mapping.rs`
- [X] T103 [US3] Implement command execution logic in `src/deployment/executor.rs`
- [X] T104 [US3] Add log capture from Docker commands in `src/deployment/executor.rs`
- [X] T105 [US3] Verify unit tests pass for deployment module

#### Deployment Module - Validation

- [X] T106 [P] [US3] Implement YAML validation in `src/deployment/validator.rs`
- [X] T107 [P] [US3] Implement Docker availability check in `src/deployment/validator.rs` (`docker --version`)
- [X] T108 [P] [US3] Implement device path validation in `src/deployment/validator.rs` (check paths exist)
- [X] T109 [P] [US3] Implement port availability check in `src/deployment/validator.rs` (netstat/ss)
- [X] T110 [P] [US3] Implement volume path validation in `src/deployment/validator.rs` (writable check)
- [X] T111 [US3] Create validation report aggregator in `src/deployment/validator.rs`
- [X] T112 [US3] Verify validation tests pass

#### Deployment Module - Health Checks & Rollback

- [X] T113 [P] [US3] Implement health check poller in `src/deployment/health.rs` (HTTP GET to Frigate API)
- [X] T114 [P] [US3] Implement container status check in `src/deployment/health.rs` (`docker ps`)
- [X] T115 [US3] Add retry logic with exponential backoff in `src/deployment/health.rs` (10 attempts, 5s interval)
- [X] T116 [US3] Implement rollback logic in `src/deployment/rollback.rs` (stop current, start previous)
- [X] T117 [US3] Store deployment history in SQLite in `src/deployment/rollback.rs`
- [X] T118 [US3] Add automatic rollback on health check failure in `src/deployment/health.rs`
- [X] T119 [US3] Verify health check and rollback tests pass

#### Tauri Backend - Deployment Commands

- [X] T120 [P] [US3] Implement `validate_deployment` Tauri command in `src/commands/deploy.rs`
- [X] T121 [P] [US3] Implement `generate_docker_command` Tauri command in `src/commands/deploy.rs`
- [X] T122 [P] [US3] Implement `execute_deployment` Tauri command in `src/commands/deploy.rs`
- [X] T123 [P] [US3] Implement `check_deployment_health` Tauri command in `src/commands/deploy.rs`
- [X] T124 [P] [US3] Implement `rollback_deployment` Tauri command in `src/commands/deploy.rs`
- [X] T125 [P] [US3] Implement `stream_container_logs` Tauri command in `src/commands/deploy.rs` (emit events)
- [X] T126 [US3] Add structured logging for all deployment operations (Constitution Principle V)
- [X] T127 [US3] Verify deployment command tests pass

#### Frontend - Deploy Page

- [X] T128 [P] [US3] Create Deploy page component in `src-ui/src/pages/Deploy.tsx`
- [X] T129 [P] [US3] Create ValidationResults component in `src-ui/src/components/ValidationResults.tsx`
- [X] T130 [P] [US3] Create DeploymentProgress component in `src-ui/src/components/DeploymentProgress.tsx`
- [X] T131 [P] [US3] Create HealthCheckStatus component in `src-ui/src/components/HealthCheckStatus.tsx`
- [X] T132 [US3] Implement pre-deployment validation flow in Deploy.tsx
- [X] T133 [US3] Implement deployment execution with progress tracking in Deploy.tsx
- [X] T134 [US3] Implement real-time health check monitoring in Deploy.tsx
- [X] T135 [US3] Implement rollback UI in Deploy.tsx (rollback button, confirmation)
- [X] T136 [US3] Add deployment command preview in Deploy.tsx (show full docker command)

#### Frontend - Logs Page

- [X] T137 [P] [US3] Create Logs page component in `src-ui/src/pages/Logs.tsx`
- [X] T138 [P] [US3] Create LogViewer component in `src-ui/src/components/LogViewer.tsx` (real-time streaming)
- [X] T139 [US3] Implement log streaming via Tauri events in Logs.tsx (listen to container-log events)
- [X] T140 [US3] Add log filtering (search, level filter) in LogViewer.tsx
- [X] T141 [US3] Add log export functionality in Logs.tsx
- [X] T142 [US3] Verify E2E tests pass for deployment workflow

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work independently. Complete workflow from hardware detection → configuration → deployment with rollback.

---

## Phase 6: User Story 4 - Cross-Platform Hardware Detection (Priority: P4)

**Goal**: Validate hardware detection works correctly on all supported platforms

**Independent Test**: Run agent on Linux, Windows, and macOS → Verify correct JSON output for each platform → Verify ARM detection works → Verify graceful handling of missing hardware

### Tests for User Story 4 (MANDATORY per Constitution Principle VI) ⚠️

- [X] T143 [P] [US4] Platform-specific unit tests for Linux detection in `agent/internal/detect/linux_test.go`
- [X] T144 [P] [US4] Platform-specific unit tests for Windows detection in `agent/internal/detect/windows_test.go`
- [X] T145 [P] [US4] Platform-specific unit tests for macOS detection in `agent/internal/detect/darwin_test.go`
- [X] T146 [P] [US4] Integration test for ARM architecture detection in `tests/integration/test_arm_detection.rs`
- [ ] T147 [P] [US4] E2E test for cross-platform agent invocation in `tests/e2e/cross-platform.spec.ts`

**Checkpoint - Tests Written**: Verify all tests written and failing before proceeding to implementation

### Implementation for User Story 4

#### Agent Module - Platform Refinements

- [X] T148 [P] [US4] Add ARM-specific detection logic in `agent/internal/detect/common.go`
- [X] T149 [P] [US4] Add Hailo NPU detection for Linux in `agent/internal/detect/linux.go`
- [X] T150 [P] [US4] Add fallback logic for missing system tools in `agent/internal/detect/common.go`
- [X] T151 [US4] Add architecture detection in `agent/internal/detect/common.go`
- [X] T152 [US4] Implement graceful error handling for missing hardware in all platform files
- [X] T153 [US4] Add CPU-only detection with performance warnings in `agent/internal/detect/common.go`
- [ ] T154 [US4] Verify platform-specific tests pass on CI matrix (Linux/Windows/macOS)

#### Frontend - Platform-Specific Handling

- [X] T155 [US4] Add platform detection utility in `src-ui/src/utils/platform.ts`
- [X] T156 [US4] Add platform-specific device path formatting in Hardware page
- [X] T157 [US4] Add CPU-only warning UI in Hardware page
- [ ] T158 [US4] Verify tests pass for cross-platform agent invocation

**Checkpoint**: All user stories should now work correctly across platforms (Linux, Windows, macOS, x86, ARM)

---

## Phase 7: User Story 5 - Manual Configuration Override (Priority: P5)

**Goal**: Enable power users to manually edit YAML for advanced settings not in UI

**Independent Test**: Navigate to Manual Config page → Edit YAML directly → Save changes → Verify syntax validation → Switch to other pages → Verify manual edits preserved → Make UI changes → Verify non-destructive merge

### Tests for User Story 5 (MANDATORY per Constitution Principle VI) ⚠️

- [ ] T159 [P] [US5] Unit tests for comment preservation in `tests/unit/test_comment_preservation.rs`
- [ ] T160 [P] [US5] Integration test for manual edit preservation in `tests/integration/test_manual_edits.rs`
- [ ] T161 [P] [US5] E2E test for Manual Config page in `tests/e2e/manual-config.spec.ts`

**Checkpoint - Tests Written**: Verify all tests written and failing before proceeding to implementation

### Implementation for User Story 5

#### Configuration Engine - Advanced Features

- [ ] T162 [US5] Enhance comment preservation in parser (ensure all comments retained)
- [ ] T163 [US5] Implement manual edit tracking in `src/config_engine/merger.rs` (mark fields as "manually_edited")
- [ ] T164 [US5] Add YAML syntax validation with error reporting in `src/config_engine/validator.rs`
- [ ] T165 [US5] Verify comment preservation tests pass

#### Frontend - Manual Config Enhancements

- [ ] T166 [US5] Add real-time syntax validation in YAML editor
- [ ] T167 [US5] Add manual edit indicators in UI (show which fields manually edited)
- [ ] T168 [US5] Implement two-way sync (YAML → UI preview) in ManualConfig.tsx
- [ ] T169 [US5] Add advanced settings section in ManualConfig.tsx (FFmpeg args, MQTT, etc.)
- [ ] T170 [US5] Verify E2E tests pass for manual configuration

**Checkpoint**: Power users can now freely edit YAML while UI users remain unaffected

---

## Phase 8: User Story 6 - Disk and Volume Mapping for Recordings (Priority: P6)

**Goal**: Enable users to configure storage locations for Frigate recordings

**Independent Test**: Navigate to Disk Mapping page → View available disks → Select directories for recordings/clips/cache → Verify free space warnings → Deploy with volume mappings → Verify mounts work

### Tests for User Story 6 (MANDATORY per Constitution Principle VI) ⚠️

- [X] T171 [P] [US6] Unit tests for disk info retrieval in `tests/unit/test_disk_info.rs`
- [X] T172 [P] [US6] Unit tests for volume validation in `tests/unit/test_volume_validation.rs`
- [X] T173 [P] [US6] Integration test for volume mounting in `tests/integration/test_volume_mount.rs`
- [X] T174 [P] [US6] E2E test for Disk Mapping page in `tests/e2e/disk-mapping.spec.ts`

**Checkpoint - Tests Written**: ✅ All tests written and failing (RED phase complete)

### Implementation for User Story 6

#### Deployment Module - Volume Support

- [X] T175 [P] [US6] Implement disk info retrieval in `src/deployment/disk.rs` (total/used/free space)
- [X] T176 [P] [US6] Implement volume path validation in `src/deployment/disk.rs` (exists, writable)
- [X] T177 [US6] Add low disk space warning logic in `src/deployment/disk.rs` (<10GB threshold)
- [X] T178 [US6] Integrate volume mappings into Docker command generation
- [X] T179 [US6] Verify volume tests pass

#### Tauri Backend - Disk Commands

- [X] T180 [P] [US6] Implement `get_disk_info` Tauri command in `src/commands/disk.rs`
- [X] T181 [P] [US6] Implement `validate_volume_path` Tauri command in `src/commands/disk.rs`
- [X] T182 [US6] Verify disk command tests pass

#### Frontend - Disk Mapping Page

- [X] T183 [P] [US6] Create DiskMapping page component in `src-ui/src/pages/DiskMapping.tsx`
- [X] T184 [P] [US6] Create DiskInfoCard component in `src-ui/src/components/DiskInfoCard.tsx` (show disk usage)
- [X] T185 [P] [US6] Create VolumeSelector component in `src-ui/src/components/VolumeSelector.tsx` (directory picker)
- [X] T186 [US6] Implement disk info display in DiskMapping.tsx
- [X] T187 [US6] Implement volume mapping configuration in DiskMapping.tsx
- [X] T188 [US6] Add low disk space warnings in DiskInfoCard.tsx
- [X] T189 [US6] Integrate volume mappings with Deploy page
- [X] T190 [US6] Verify E2E tests pass for disk mapping

**Checkpoint**: All user stories should now be independently functional with complete feature coverage

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T191 Verify test coverage >80% for core modules (Constitution Principle VI requirement)
- [X] T192 [P] Create architecture.md with module diagrams (Mermaid) and data flow in `docs/architecture.md`
- [X] T193 [P] Generate API documentation from OpenAPI schema and Tauri commands in `docs/api/`
- [X] T194 [P] Write user guide with installation and usage instructions in `docs/user-guide/`
- [X] T195 [P] Add README.md with quickstart and contribution guidelines
- [X] T196 Code cleanup and refactoring (remove TODOs, optimize performance)
- [X] T197 Performance optimization: profile critical paths (hardware detection, YAML parsing, deployment)
- [X] T198 Multi-platform validation: test on Linux + (Windows or macOS) per Constitution Principle III
- [X] T199 Security audit: verify input validation, audit logging, no privilege escalation (Constitution Principle V)
- [X] T200 Accessibility audit: keyboard navigation, screen reader support, high contrast mode
- [X] T201 Run quickstart.md validation (follow quickstart guide and verify all steps work)
- [X] T202 Verify Constitution compliance across all implemented features
- [X] T203 [P] Create release build scripts in `.github/workflows/release.yml`
- [X] T204 [P] Build multi-platform agent binaries (Linux/Windows/macOS on x86/ARM)
- [X] T205 [P] Build Tauri application bundles for all platforms (.AppImage, .dmg, .msi)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3 → P4 → P5 → P6)
- **Polish (Phase 9)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - No dependencies on other stories (independent testing)
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - References US1 camera configs but independently testable
- **User Story 4 (P4)**: Can start after Foundational (Phase 2) - Refinement of US1 agent, but independently testable
- **User Story 5 (P5)**: Can start after Foundational (Phase 2) - Extends US2 config engine, independently testable
- **User Story 6 (P6)**: Can start after Foundational (Phase 2) - Extends US3 deployment, independently testable

### Within Each User Story

- Tests (mandatory) MUST be written and FAIL before implementation
- Models before services
- Services before commands
- Commands before UI components
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel (Phase 1)
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- Within each user story, tasks marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task T019: "Contract test for Agent JSON output schema"
Task T020: "Integration test for hardware detection command"
Task T021: "Unit tests for HardwareDevice model validation"
Task T022: "Unit tests for CameraConfiguration model"
Task T023: "E2E test for Hardware page rendering"
Task T024: "E2E test for Cameras page workflow"

# After tests fail, launch all Agent detection implementations together:
Task T025: "Implement Linux hardware detection"
Task T026: "Implement Windows hardware detection"
Task T027: "Implement macOS hardware detection"

# Launch all Frontend components together (after backend ready):
Task T037: "Create Hardware page component"
Task T038: "Create DeviceCard component"
Task T039: "Create DeviceList component"
Task T044: "Create Cameras page component"
Task T045: "Create CameraForm component"
Task T046: "Create CameraList component"
Task T047: "Create HardwareSelector component"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T007)
2. Complete Phase 2: Foundational (T008-T018) - CRITICAL - blocks all stories
3. Complete Phase 3: User Story 1 (T019-T053)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

**MVP Deliverable**: Hardware detection + Camera configuration with hardware assignment

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 (T019-T053) → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 (T054-T091) → Test independently → Deploy/Demo (Config management)
4. Add User Story 3 (T092-T142) → Test independently → Deploy/Demo (Deployment)
5. Add User Story 4 (T143-T158) → Test independently → Deploy/Demo (Cross-platform validation)
6. Add User Story 5 (T159-T170) → Test independently → Deploy/Demo (Power user features)
7. Add User Story 6 (T171-T190) → Test independently → Deploy/Demo (Production storage)
8. Polish (T191-T205) → Release v1.0.0

Each story adds value without breaking previous stories.

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (T001-T018)
2. Once Foundational is done:
   - Developer A: User Story 1 (T019-T053)
   - Developer B: User Story 2 (T054-T091)
   - Developer C: User Story 3 (T092-T142)
3. Stories complete and integrate independently
4. Continue with remaining stories in parallel

---

## Task Count Summary

- **Total Tasks**: 205
- **Phase 1 (Setup)**: 7 tasks
- **Phase 2 (Foundational)**: 11 tasks
- **Phase 3 (User Story 1 - MVP)**: 35 tasks
- **Phase 4 (User Story 2)**: 38 tasks
- **Phase 5 (User Story 3)**: 51 tasks
- **Phase 6 (User Story 4)**: 16 tasks
- **Phase 7 (User Story 5)**: 12 tasks
- **Phase 8 (User Story 6)**: 20 tasks
- **Phase 9 (Polish)**: 15 tasks

**Parallel Tasks Identified**: 87 tasks marked [P] (42% parallelizable)

---

## Constitution Compliance Checkpoints

After completing each user story, verify:

- ✅ **User First (P2)**: Configuration Engine preserves user settings
- ✅ **Automation + Control (P5)**: Manual YAML editing supported
- ✅ **Cross-Platform (P4)**: Agent works on all platforms
- ✅ **Modular Design (All)**: Modules independently testable
- ✅ **Security (P3, P6)**: No privilege escalation, audit logging present
- ✅ **Testing (All)**: Test coverage >80%, all tests pass
- ✅ **Documentation (P9)**: Public APIs documented

---

## Notes

- [P] tasks = different files, no dependencies - run in parallel
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Tests MUST be written first and MUST fail before implementing (Red-Green-Refactor)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

---

## Ready to Start?

**Suggested First Session**: Complete MVP (Phases 1-3)

```bash
# Setup
T001-T007: Project initialization

# Foundation (CRITICAL - must complete before user stories)
T008-T018: Core infrastructure

# User Story 1 (MVP)
T019-T024: Write tests (verify they fail)
T025-T031: Agent implementation (verify tests pass)
T032-T036: Tauri backend (verify tests pass)
T037-T043: Hardware page (verify E2E passes)
T044-T053: Cameras page (verify E2E passes)

# Result: Working hardware detection + camera configuration
```

Next: `/speckit.implement` to begin execution with AI assistance!
