# Feature Specification: Frigate Configuration Tool - Complete System

**Feature Branch**: `001-2-1-ui`
**Created**: 2025-10-08
**Status**: Draft
**Input**: User description: "Complete modular system including UI, Agent, Configuration Engine, Deployment, and Testing modules for Frigate NVR configuration management"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Hardware Discovery and Camera Setup (Priority: P1)

A user wants to set up Frigate NVR on their system. They need to identify available hardware accelerators (e.g., Coral TPU, NVIDIA GPU) and configure cameras to use those accelerators for efficient video processing.

**Why this priority**: This is the core workflow that most users need immediately. Without hardware detection and camera configuration, the system provides no value. This forms the foundation for all other features.

**Independent Test**: Can be fully tested by launching the UI, navigating to Hardware page to view detected devices, then navigating to Cameras page to add a camera with selected hardware acceleration. Success is verified when the system displays detected hardware and allows camera configuration without requiring deployment.

**Acceptance Scenarios**:

1. **Given** the user launches the application, **When** they navigate to the Hardware page, **Then** the system displays all available hardware accelerators (GPUs, Coral TPUs) and video devices with their device paths
2. **Given** hardware has been detected, **When** the user navigates to the Cameras page and adds a new camera, **Then** they can select an available hardware accelerator from the detected devices
3. **Given** the user is configuring a camera, **When** they enter camera details (name, RTSP URL, resolution), **Then** the system validates the input and shows configuration preview
4. **Given** multiple cameras are configured, **When** the user assigns different accelerators to different cameras, **Then** the system shows which hardware is assigned to each camera

---

### User Story 2 - Configuration Management with Conflict Detection (Priority: P2)

A user has an existing Frigate configuration file with custom settings. They want to use the tool to add new cameras or update hardware settings without losing their manual tweaks (motion detection zones, notification settings, etc.).

**Why this priority**: This addresses the critical "User First" constitution principle. Users with existing configurations need confidence that the tool won't break their working setup. This prevents adoption barriers for experienced users.

**Independent Test**: Can be fully tested by loading an existing frigate.yml file with custom settings, making changes through the UI (adding a camera or changing hardware), and verifying that the tool detects conflicts, shows warnings, and allows the user to choose whether to keep existing values or apply new ones. Success is confirmed when original settings are preserved or explicitly overridden.

**Acceptance Scenarios**:

1. **Given** the user loads an existing frigate.yml file, **When** the tool attempts to apply a template that conflicts with existing settings, **Then** the system displays a warning showing the conflict and offers options to keep existing value or apply new value
2. **Given** a conflict is detected, **When** the user chooses to keep their existing configuration, **Then** the tool preserves the original value and marks it as user-protected
3. **Given** the user modifies a configuration through the UI, **When** they save changes, **Then** the system creates a timestamped backup of the previous configuration automatically
4. **Given** a configuration change causes problems, **When** the user initiates rollback, **Then** the system restores the previous configuration from backup and displays what was changed
5. **Given** the user manually edits the configuration file outside the tool, **When** they reload it in the tool, **Then** the system recognizes manual changes and preserves them

---

### User Story 3 - Safe Deployment with Validation (Priority: P3)

A user has configured their cameras and hardware settings. They want to deploy Frigate using Docker with confidence that the configuration is valid and that they can roll back if something goes wrong.

**Why this priority**: Deployment is the final step in the workflow. While critical, it depends on hardware discovery (P1) and configuration management (P2) being complete. Users need this to actually run Frigate, but it's only valuable after configuration is ready.

**Independent Test**: Can be fully tested by completing a configuration, clicking Deploy, and verifying that the system performs pre-deployment checks (validates YAML, checks Docker availability, verifies device paths), shows a deployment plan, executes deployment commands, monitors deployment status, and provides rollback capability. Success is confirmed when Frigate starts successfully or when rollback restores the previous state.

**Acceptance Scenarios**:

1. **Given** the user has completed their configuration, **When** they navigate to the Deploy page and click Deploy, **Then** the system runs pre-deployment validation checks (YAML syntax, Docker availability, device path accessibility)
2. **Given** validation passes, **When** deployment begins, **Then** the system displays real-time deployment progress and logs
3. **Given** deployment is in progress, **When** the system performs health checks, **Then** it verifies that Frigate container starts and responds to health endpoint
4. **Given** deployment completes, **When** health checks pass, **Then** the system displays success status with links to Frigate UI and log viewer
5. **Given** deployment fails or health checks fail, **When** the user clicks Rollback, **Then** the system stops the failed deployment, restores the previous configuration, and restarts the previous container (if one existed)
6. **Given** Frigate is running, **When** the user views the Logs page, **Then** they see real-time Frigate container logs with filtering capabilities

---

### User Story 4 - Cross-Platform Hardware Detection (Priority: P4)

A user runs the tool on different operating systems (Windows, Linux, macOS) and hardware architectures (x86, ARM). They expect the hardware detection to work correctly on their platform without special configuration.

**Why this priority**: This validates the cross-platform constitution principle but is less critical than core workflows. Users on each platform need it to work, but it's an enabler for P1 rather than a standalone user-facing feature.

**Independent Test**: Can be fully tested by running the agent module on each supported platform, calling the hardware detection API, and verifying that it returns correct JSON data for that platform's devices (e.g., /dev/video* on Linux, COM ports on Windows, AVFoundation devices on macOS). Success is platform-specific but follows the same test pattern.

**Acceptance Scenarios**:

1. **Given** the agent is running on Linux, **When** the UI requests hardware information, **Then** the agent returns JSON listing /dev/video*, /dev/dri/*, and USB Coral devices
2. **Given** the agent is running on Windows, **When** the UI requests hardware information, **Then** the agent returns JSON listing DirectShow devices, NVIDIA GPUs via nvidia-smi output, and USB devices
3. **Given** the agent is running on macOS, **When** the UI requests hardware information, **Then** the agent returns JSON listing AVFoundation devices and Apple Neural Engine (if available)
4. **Given** the agent runs on ARM architecture (e.g., Raspberry Pi), **When** hardware detection executes, **Then** it correctly identifies ARM-specific accelerators like Coral USB without attempting x86-specific detection
5. **Given** the user's system has no hardware accelerators, **When** hardware detection completes, **Then** the system shows CPU-only option and warns about performance implications

---

### User Story 5 - Manual Configuration Override (Priority: P5)

A power user wants to manually edit the YAML configuration for advanced settings not exposed in the UI (custom FFmpeg parameters, advanced motion detection algorithms, MQTT settings, etc.).

**Why this priority**: This supports the "Automation + Control" principle but is needed by a smaller subset of power users. Most users will use the UI exclusively. This is important for flexibility but not critical for MVP.

**Independent Test**: Can be fully tested by navigating to the Manual Config page, editing YAML directly in a text editor, saving changes, and verifying that the system validates syntax, preserves manual edits, and updates UI preview where applicable. Success is confirmed when manual edits persist and are respected by the UI.

**Acceptance Scenarios**:

1. **Given** the user navigates to Manual Config page, **When** they view the configuration, **Then** they see the full generated YAML with syntax highlighting
2. **Given** the user edits the YAML manually, **When** they save changes, **Then** the system validates YAML syntax and shows errors if invalid
3. **Given** the user adds advanced settings not in the UI (custom FFmpeg args), **When** they switch back to Hardware or Cameras pages, **Then** those manual settings are preserved and marked as "manually configured"
4. **Given** the user makes UI changes after manual edits, **When** the system regenerates configuration, **Then** it performs non-destructive merging that preserves manual additions unless explicitly overridden
5. **Given** the user wants to remove a field, **When** they add a deletion marker (per non-destructive merge rules), **Then** the system removes that field on next generation

---

### User Story 6 - Disk and Volume Mapping for Recordings (Priority: P6)

A user needs to configure where Frigate stores recordings and clips. They want to map host directories to container volumes and see available disk space to avoid running out of storage.

**Why this priority**: Essential for production use but depends on deployment infrastructure (P3). Users can initially test without persistent storage. Important for real deployments but not blocking for initial testing.

**Independent Test**: Can be fully tested by navigating to Disk Mapping page, viewing available disks and their free space, selecting mount paths for recordings/clips/cache, and verifying that deployment command includes correct volume mounts. Success is confirmed when Docker run/compose command shows proper volume mappings.

**Acceptance Scenarios**:

1. **Given** the user navigates to Disk Mapping page, **When** the page loads, **Then** the system displays available disks with total size, used space, and free space
2. **Given** the user selects a directory for recordings, **When** they specify the host path, **Then** the system validates that the path exists and is writable
3. **Given** the user configures multiple volume mappings (recordings, clips, cache), **When** they save the configuration, **Then** the system generates the correct Docker volume mount syntax (-v flags or compose volumes section)
4. **Given** the selected disk has low free space (<10GB), **When** the user tries to use it for recordings, **Then** the system displays a warning about potential storage issues
5. **Given** volume mappings are configured, **When** deployment occurs, **Then** the Frigate container mounts those directories and can write recordings to them

---

### Edge Cases

- What happens when hardware detection fails or times out (e.g., permission issues, missing drivers)? System should show graceful error message and allow manual device path entry.
- How does system handle corrupt or malformed existing YAML files? System should display syntax errors with line numbers and offer to fix common issues or start fresh.
- What happens when user attempts deployment without Docker installed or running? Pre-deployment check should detect this and show installation instructions.
- How does system handle device path changes between reboots (e.g., /dev/video0 becomes /dev/video1)? System should warn about potential device instability and suggest using device IDs when available.
- What happens when multiple users/processes modify the configuration file simultaneously? System should detect file changes and prompt to reload or merge.
- How does system handle Docker permission errors (user not in docker group on Linux)? Show clear error with remediation steps (add user to docker group).
- What happens when user tries to deploy with conflicting port bindings? Pre-deployment validation should detect port conflicts and suggest alternatives.
- How does system handle network timeouts when fetching Docker images? Show progress and allow retry with clear error messages.
- What happens when user loads a configuration generated by a newer version of Frigate? Warn about potential incompatibility and offer to upgrade or show documentation.

## Requirements *(mandatory)*

### Functional Requirements

#### UI Module Requirements

- **FR-001**: System MUST provide an iOS-style graphical interface accessible on the user's local machine
- **FR-002**: System MUST include a Hardware page that displays detected hardware accelerators and video devices
- **FR-003**: System MUST include a Cameras page that allows adding, editing, and removing camera configurations
- **FR-004**: System MUST include a Manual Config page that displays the full YAML configuration with syntax highlighting
- **FR-005**: System MUST include a Deploy Frigate page that shows deployment options, validation status, and deployment controls
- **FR-006**: System MUST include a Disk Mapping page that shows available storage and allows volume configuration
- **FR-007**: System MUST include a Logs page that displays real-time Frigate container logs
- **FR-008**: UI MUST communicate with the Agent module to retrieve hardware information
- **FR-009**: UI MUST communicate with the Configuration Engine to generate and validate YAML
- **FR-010**: UI MUST communicate with the Deployment Module to execute and monitor deployment

#### Agent Module Requirements

- **FR-011**: Agent MUST detect hardware accelerators available on the host system (GPUs, Coral TPUs, Intel QSV)
- **FR-012**: Agent MUST detect video input devices (cameras, capture cards) and return device paths
- **FR-013**: Agent MUST return hardware information in JSON format with a defined schema
- **FR-014**: Agent MUST run on Windows, Linux, and macOS operating systems
- **FR-015**: Agent MUST run on x86_64, ARM64, and ARM32 (v7+) architectures
- **FR-016**: Agent MUST NOT scan or interact with host drivers or kernel modules (Constitution Principle V)
- **FR-017**: Agent MUST return device node paths without requiring elevated privileges
- **FR-018**: Agent MUST handle detection failures gracefully and return partial results when possible
- **FR-019**: Agent MUST complete hardware detection within 5 seconds under normal conditions

#### Configuration Engine Requirements

- **FR-020**: Configuration Engine MUST generate valid Frigate YAML configuration from user inputs
- **FR-021**: Configuration Engine MUST support template injection for common configuration patterns
- **FR-022**: Configuration Engine MUST detect conflicts between template values and existing user configurations
- **FR-023**: Configuration Engine MUST display conflict warnings showing both the existing value and the new value
- **FR-024**: Configuration Engine MUST allow explicit user override of conflicting values
- **FR-025**: Configuration Engine MUST create automatic timestamped backups before making configuration changes
- **FR-026**: Configuration Engine MUST support rollback to any previous configuration backup
- **FR-027**: Configuration Engine MUST perform non-destructive deep merging of configuration changes
- **FR-028**: Configuration Engine MUST require explicit deletion markers to remove existing configuration fields
- **FR-029**: Configuration Engine MUST validate YAML syntax before saving
- **FR-030**: Configuration Engine MUST preserve user comments in manually edited sections where possible
- **FR-031**: Configuration Engine MUST maintain a history of configuration changes with timestamps and descriptions

#### Deployment Module Requirements

- **FR-032**: Deployment Module MUST support both Docker Compose and Docker Run deployment methods
- **FR-033**: Deployment Module MUST perform pre-deployment configuration validation (YAML syntax, required fields)
- **FR-034**: Deployment Module MUST verify Docker availability before attempting deployment
- **FR-035**: Deployment Module MUST verify that device paths exist and are accessible before deployment
- **FR-036**: Deployment Module MUST verify that specified ports are available before deployment
- **FR-037**: Deployment Module MUST generate appropriate Docker commands with device mounts and volume mappings
- **FR-038**: Deployment Module MUST execute deployment commands and capture output logs
- **FR-039**: Deployment Module MUST perform health checks after deployment to verify Frigate is running
- **FR-040**: Deployment Module MUST support rollback to previous deployment state if health checks fail
- **FR-041**: Deployment Module MUST allow manual rollback initiated by the user
- **FR-042**: Deployment Module MUST collect and display Frigate container logs in real-time
- **FR-043**: Deployment Module MUST handle hardware device mounts (GPU, Coral TPU) with proper permissions

#### Testing Requirements

- **FR-044**: System MUST include automated tests for template injection and merging logic
- **FR-045**: System MUST include automated tests for YAML generation and validation
- **FR-046**: System MUST include automated tests for deployment command generation
- **FR-047**: System MUST include automated tests for UI-Agent communication across platforms
- **FR-048**: System MUST include automated tests for cross-platform hardware detection
- **FR-049**: Automated test suite MUST run multi-round validation with different configurations
- **FR-050**: Any test failure MUST block deployment and display specific failure information
- **FR-051**: Test suite MUST validate configuration against Frigate's schema requirements

### Key Entities

- **Hardware Device**: Represents a detected hardware accelerator or video device. Attributes: device type (GPU, Coral, camera), device path, device name, capabilities, platform-specific identifiers.

- **Camera Configuration**: Represents a single camera's settings. Attributes: camera name, RTSP URL, resolution, assigned hardware accelerator, detection zones (if manually configured), recording settings.

- **Configuration Template**: Represents a reusable configuration pattern. Attributes: template name, template content (YAML fragment), applicable scenarios, required parameters.

- **Configuration Backup**: Represents a saved state of the configuration. Attributes: timestamp, configuration content (full YAML), change description, automatic vs manual backup flag.

- **Deployment State**: Represents the current deployment status. Attributes: deployment method (compose/run), container ID, deployment timestamp, health check status, previous state reference (for rollback).

- **Conflict Resolution**: Represents a detected configuration conflict. Attributes: configuration path, existing value, new value, user resolution choice, conflict detection timestamp.

- **Volume Mapping**: Represents a host-to-container path mapping. Attributes: host path, container path, mapping type (recordings/clips/cache), read-only flag, disk space availability.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can complete initial hardware detection and camera setup in under 5 minutes (from application launch to first camera configured)
- **SC-002**: Configuration conflict detection identifies 100% of overlapping values between templates and existing configs
- **SC-003**: Users can successfully roll back to previous configuration within 30 seconds of encountering issues
- **SC-004**: Automated test suite achieves >80% code coverage for core modules (Agent, Configuration Engine, Deployment)
- **SC-005**: Hardware detection works correctly on all supported platforms (Windows, Linux, macOS) with 95%+ accuracy
- **SC-006**: Pre-deployment validation catches 90%+ of common configuration errors before deployment attempt
- **SC-007**: System successfully deploys Frigate on a correctly configured system within 2 minutes
- **SC-008**: Zero instances of user configurations being overwritten without explicit user approval (100% compliance with User First principle)
- **SC-009**: Manual configuration edits made outside the UI are preserved in 100% of cases where valid YAML syntax is maintained
- **SC-010**: Users report 40%+ reduction in time required to set up Frigate compared to manual YAML editing (measured via user surveys)
- **SC-011**: System handles deployment failures gracefully with successful rollback in 95%+ of cases
- **SC-012**: Real-time log viewing displays container logs within 1 second of log generation
- **SC-013**: Cross-platform agent returns hardware information within 5 seconds on all supported platforms
- **SC-014**: Configuration backups are created automatically before 100% of potentially destructive operations

## Assumptions

- Users have Docker installed or are willing to install it (deployment module requires Docker)
- Users have basic familiarity with Frigate NVR concepts (cameras, hardware acceleration, RTSP streams)
- Network connectivity is available for pulling Docker images during deployment
- Users have read/write permissions to the directory where configuration files are stored
- Device paths remain relatively stable between detection and deployment (acknowledged edge case when they don't)
- Users running on Linux have appropriate permissions to access video devices and Docker socket
- YAML configuration format follows Frigate's documented schema (version compatibility is an edge case)
- iOS-style interface assumes desktop application with modern UI framework, not mobile iOS app
- Agent module can execute standard system commands for hardware detection (lsusb, nvidia-smi, etc.) without special privileges
- Users understand that manual configuration edits may require technical knowledge of Frigate's configuration schema
