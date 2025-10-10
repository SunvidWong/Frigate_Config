<!--
==============================================================================
SYNC IMPACT REPORT
==============================================================================
Version Change: 1.1.0 → 1.3.0
Reason: Added Development Priorities + Principle VIII (Unattended Mode Support)

Sections Added:
- Development Priorities: Incremental delivery guidance and priority ordering (v1.2.0)
- Principle VIII: Unattended Mode Support - CI/CD integration requirements (v1.3.0)

Sections Modified:
- Core Principles: Expanded from 7 to 8 principles
- Version metadata updated to 1.3.0
- Last Amended date updated to 2025-10-10
- Version History updated with 1.2.0 and 1.3.0 entries

Principle Summary (8 core principles):
1. User First: Prevent configuration overwrites, explicit conflict warnings
2. Automation + Control: Generate configs with manual override capability
3. Cross-Platform Support: Windows/Linux/macOS, x86/ARM architectures
4. Modular Design: Independent UI/Agent/Config/Deployment modules
5. Security & Permissions: No host driver scanning, authorized deployments, traceable logs
6. AI-Driven Testing: Test-first development, multi-core automated validation
7. Open Source Community: Apache 2.0 license, community contributions welcome
8. Unattended Mode Support: CLI interfaces, non-interactive operation, CI/CD integration (NEW)

New Requirements from v1.3.0:
- CLI interfaces MUST be provided for all core operations
- Non-interactive mode MUST NOT require user input
- Exit codes MUST follow standard conventions (0=success, non-zero=failure)
- Structured logging (JSON preferred) MUST be supported
- Dry-run mode MUST be available to preview actions
- Docker image MUST be provided for containerized execution
- GitOps workflows MUST be supported
- Health check endpoints/commands MUST be available

New Requirements from v1.2.0:
- Incremental Delivery designated as NON-NEGOTIABLE
- MVP (Phase 1-5) must be complete before advanced features
- Production-critical features prioritized (disk mapping > manual overrides)
- Each phase must be independently testable and deployable
- Documentation must stay synchronized with phase completion

Templates Status:
⚠️  .specify/templates/plan-template.md - Constitution Check needs update to v1.3.0, add Principle VIII
⚠️  .specify/templates/spec-template.md - Should add CLI interface requirements section
⚠️  .specify/templates/tasks-template.md - Should add unattended mode testing tasks
✅ .specify/templates/agent-file-template.md - No changes needed (guidance template)
✅ .specify/templates/checklist-template.md - No changes needed (format template)
⚠️  README.md - Should reference Principle VIII and constitution v1.3.0
⚠️  docs/quickstart.md - May benefit from CLI usage examples

Follow-up TODOs:
- Update plan-template.md Constitution Check to v1.3.0 with 8 principles
- Add CLI interface requirements to spec-template.md
- Add unattended mode test category to tasks-template.md
- Document CLI commands in README.md
- Add CI/CD integration examples to docs/
- Verify all core operations have CLI equivalents
==============================================================================
-->

# Frigate Configuration Tool Constitution

## Core Principles

### I. User First (NON-NEGOTIABLE)

**Rule**: User configurations MUST be treated as sacred and immutable by automated systems.

- Template injections MUST NOT overwrite existing user configurations under any circumstances
- Configuration conflicts MUST trigger explicit warnings before any changes
- Users MUST retain full visibility and control over all configuration changes
- The system MUST default to preserving user intent when conflicts arise
- Rollback capabilities MUST be available for all configuration changes

**Rationale**: Users have invested time and expertise in their configurations. Breaking their working setups erodes trust and can cause operational failures. This principle ensures user confidence and safe automation.

### II. Automation + Control

**Rule**: The system MUST automatically generate valid configurations while preserving manual adjustment capabilities.

- Configuration files MUST be automatically generated based on detected hardware and user preferences
- Generated configurations MUST remain human-readable and manually editable
- Manual adjustments MUST be preserved across regeneration cycles
- The system MUST provide both GUI-driven and file-based configuration workflows
- Documentation MUST be generated alongside configuration files

**Rationale**: Automation lowers the barrier to entry, but power users need direct control. Supporting both workflows maximizes accessibility without limiting capability.

### III. Cross-Platform Support

**Rule**: The tool MUST support all major platforms and architectures without feature degradation.

- Supported operating systems: Windows, Linux, macOS (all currently maintained versions)
- Supported architectures: x86_64, ARM64, ARM32 (v7+)
- Hardware accelerator support: NVIDIA CUDA, Intel QSV, AMD VAAPI, Apple VideoToolbox, Coral TPU
- Platform-specific features MUST fail gracefully with clear guidance when unavailable
- Core functionality MUST work identically across all supported platforms

**Rationale**: Frigate runs on diverse hardware. Users should not be locked into specific platforms, and the configuration tool must match Frigate's platform flexibility.

### IV. Modular Design

**Rule**: System components MUST be independently deployable, testable, and maintainable.

Core modules (each with clear boundaries):
- **UI Module**: Visual configuration builder, cross-platform GUI framework
- **Agent Module**: System detection, deployment orchestration, service management
- **Configuration Engine**: YAML generation, validation, templating, conflict resolution
- **Deployment Module**: Docker orchestration, service lifecycle, rollback management

Requirements for all modules:
- MUST have well-defined interfaces and contracts
- MUST be independently testable without full system integration
- MUST expose CLI interfaces for automation
- MUST log operations with structured, parseable output
- MUST handle failures gracefully with actionable error messages

**Rationale**: Modularity enables parallel development, easier testing, and selective deployment. Users may only need certain modules (e.g., CLI-only users skip UI).

### V. Security & Permissions

**Rule**: The system MUST operate within strict security boundaries and never perform privileged operations without explicit authorization.

Security requirements:
- MUST NOT scan host system drivers or kernel modules
- MUST NOT access filesystem locations outside designated configuration directories
- MUST request explicit user authorization before any deployment actions
- MUST log all system-modifying operations with timestamps, user identity, and actions taken
- MUST validate all configuration inputs to prevent injection attacks
- MUST use least-privilege principles for all operations
- MUST encrypt sensitive data (API keys, credentials) at rest and in transit

Audit requirements:
- All logs MUST be traceable to specific user actions
- Configuration changes MUST be versioned with authorship metadata
- Deployment operations MUST generate audit trails suitable for compliance review

**Rationale**: Configuration tools have elevated privileges. Strict security controls prevent abuse, protect user systems, and maintain trust. Auditability supports troubleshooting and compliance.

### VI. AI-Driven Testing (Test-First Development - NON-NEGOTIABLE)

**Rule**: Tests MUST be written before implementation, and automated validation MUST replace manual review wherever possible.

Test-First Development (TDD) requirements:
- Unit tests MUST be written and MUST FAIL before implementation begins
- Integration tests MUST be defined before cross-module work starts
- Test coverage MUST be measured and maintained above 80% for core modules
- Every bug fix MUST include a regression test before the fix is implemented

AI-Driven Testing requirements:
- Multi-core parallel test execution MUST be supported for fast feedback
- Automated validation MUST run on every commit (pre-commit hooks, CI/CD)
- Multi-round testing strategies MUST validate configurations across platform matrices
- Generated configurations MUST undergo automated schema validation
- Hardware accelerator configurations MUST be validated against known-good templates

Testing categories (all mandatory for new features):
- **Contract tests**: Verify module interfaces remain stable
- **Integration tests**: Validate end-to-end workflows (UI → config → deployment)
- **Platform tests**: Ensure cross-platform compatibility
- **Regression tests**: Prevent re-introduction of fixed bugs

**Rationale**: Manual testing is slow, inconsistent, and incomplete. Test-first development catches issues early and documents expected behavior. AI-driven automation enables comprehensive validation at scale, critical for supporting diverse hardware/platform combinations.

### VII. Open Source Community

**Rule**: The project MUST operate transparently under an open source license with clear contribution guidelines.

- Licensed under Apache License 2.0
- Source code MUST be publicly available on GitHub
- Contributions MUST be welcome from all community members
- Contribution guidelines MUST be documented and enforced consistently
- Code of conduct MUST be established and enforced
- Security vulnerabilities MUST have a responsible disclosure process
- Release notes MUST be published with every version
- Breaking changes MUST follow semantic versioning (MAJOR version bump)

Community engagement requirements:
- Issues MUST be triaged within 7 days
- Pull requests MUST receive initial review within 14 days
- Documentation MUST be maintained in sync with code changes
- Community discussions MUST be facilitated via GitHub Discussions or equivalent

**Rationale**: Open source development leverages community expertise, ensures transparency, and prevents vendor lock-in. Clear processes ensure maintainability and encourage contributions.

### VIII. Unattended Mode Support

**Rule**: The system MUST support fully automated, non-interactive operation for CI/CD integration and scripted deployments.

Unattended mode requirements:
- MUST provide CLI interfaces for all core operations (detect, configure, validate, deploy)
- MUST accept configuration via files, environment variables, or command-line arguments
- MUST NOT require interactive user input when running in non-interactive mode
- MUST exit with appropriate status codes (0 for success, non-zero for failures)
- MUST output structured, parseable logs (JSON format preferred) for automation consumption
- MUST support dry-run mode to preview actions without execution
- MUST validate inputs before execution and fail fast with clear error messages
- MUST support configuration presets for common deployment scenarios

CI/CD integration requirements:
- MUST provide Docker image for containerized execution
- MUST document environment variables for configuration
- MUST support GitOps workflows (configuration as code)
- MUST enable silent mode (minimal/no output) for clean logs
- MUST provide health check endpoints or commands for monitoring

**Rationale**: Production deployments require automation. Manual intervention creates bottlenecks, inconsistencies, and prevents scaling. Unattended mode enables integration with existing DevOps pipelines, continuous deployment, and infrastructure-as-code workflows.

## Security & Permissions Requirements

- **Principle of Least Privilege**: Every component MUST request only the minimum permissions required for its function
- **No Privileged Discovery**: System detection MUST work without root/administrator privileges
- **Explicit Consent**: Deployment actions (Docker operations, service restarts) MUST require user confirmation
- **Audit Trail**: Every privileged operation MUST generate a structured log entry with timestamp, user, action, and result
- **Input Validation**: All user inputs (file paths, YAML content, CLI arguments) MUST be sanitized and validated

## Quality Assurance Standards

- **Constitution Compliance**: All features MUST be reviewed against this constitution before merge
- **Multi-Platform Validation**: New features MUST be tested on at least 2 platforms (Linux + one other)
- **Performance Baselines**: Configuration generation MUST complete in <2 seconds for typical setups
- **Error Handling**: All errors MUST provide actionable guidance (never "Error: unknown")
- **Documentation**: Public APIs and CLI commands MUST have usage examples

## Development Process

**Language Preference (NON-NEGOTIABLE)**:
- ALL interaction with AI assistants and development tools MUST be conducted in Chinese (中文)
- All code, comments, documentation, and user-facing content MUST remain in English
- Technical discussions, planning, status updates, and commit messages MUST be in Chinese
- Error messages and debugging output shown to developers MAY be in Chinese
- Only end-user documentation and UI text MUST be in English (or localized as appropriate)

## Governance

**Amendment Procedure**:
- Proposed amendments MUST be documented with rationale and impact analysis
- Amendments MUST be approved via pull request review (requires 2+ maintainer approvals)
- MAJOR amendments (removing/redefining principles) MUST include migration guidance
- MINOR amendments (new principles) MUST increment version accordingly

**Versioning Policy**:
- Constitution version follows semantic versioning (MAJOR.MINOR.PATCH)
- MAJOR: Backward-incompatible governance changes, principle removals/redefinitions
- MINOR: New principles added, materially expanded guidance
- PATCH: Clarifications, wording improvements, non-semantic changes

**Compliance Review**:
- All feature specifications MUST include a Constitution Check section
- All pull requests MUST verify compliance with relevant principles
- Complexity violations MUST be justified in the implementation plan
- Quarterly audits MUST review adherence to test-first and security principles

## Development Priorities

**Incremental Delivery (NON-NEGOTIABLE)**:
- MVP features (Phase 1-5) MUST be fully functional and tested before advanced features
- Each phase MUST be independently testable and deployable
- Production-critical features (deployment, rollback, disk mapping) MUST take priority over convenience features
- Documentation MUST be updated in sync with each phase completion
- Regression tests MUST prevent breaking existing functionality when adding new features

**Priority Ordering for Post-MVP Development**:
1. **Critical Path**: Features required for production deployment (disk mapping, health monitoring)
2. **Platform Validation**: Cross-platform testing and ARM support verification
3. **Advanced Features**: Manual configuration overrides, advanced YAML editing
4. **Polish**: Documentation, performance optimization, release builds

**Version**: 1.3.0 | **Ratified**: 2025-10-08 | **Last Amended**: 2025-10-10

---

## Version History

### 1.3.0 (2025-10-10)
- **MINOR**: Added Principle VIII - Unattended Mode Support
- Mandated CLI interfaces for all core operations (detect, configure, validate, deploy)
- Required non-interactive mode with proper exit codes and structured logging
- Established CI/CD integration requirements (Docker, GitOps, health checks)
- Defined dry-run mode and configuration presets as mandatory features
- Expanded from 7 to 8 core principles

### 1.2.0 (2025-10-10)
- **MINOR**: Added Development Priorities section for post-MVP guidance
- Established incremental delivery as NON-NEGOTIABLE principle
- Defined priority ordering for Phase 6-9 implementation
- Mandated independent testing and deployment for each phase
- Required documentation synchronization with phase completion

### 1.1.0 (2025-10-09)
- **MINOR**: Enhanced language preference requirement to NON-NEGOTIABLE status
- Mandated Chinese language for ALL AI/development tool interactions
- Clarified English-only requirement for end-user content
- Upgraded from suggestion to strict requirement for Chinese in technical discussions

### 1.0.0 (2025-10-08)
- Initial constitution creation
- Established 7 core principles
- Defined security, testing, and governance requirements
