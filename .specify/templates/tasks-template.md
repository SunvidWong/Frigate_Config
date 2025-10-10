---
description: "Task list template for feature implementation"
---

# Tasks: [FEATURE NAME]

**Input**: Design documents from `/specs/[###-feature-name]/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Per Constitution Principle VI (Test-First Development - NON-NEGOTIABLE), tests MUST be written BEFORE implementation for ALL features. Tests are MANDATORY, not optional.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story. Each story follows Red-Green-Refactor: write tests → verify failure → implement → verify pass.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- **Single project**: `src/`, `tests/` at repository root
- **Web app**: `backend/src/`, `frontend/src/`
- **Mobile**: `api/src/`, `ios/src/` or `android/src/`
- Paths shown below assume single project - adjust based on plan.md structure

<!-- 
  ============================================================================
  IMPORTANT: The tasks below are SAMPLE TASKS for illustration purposes only.
  
  The /speckit.tasks command MUST replace these with actual tasks based on:
  - User stories from spec.md (with their priorities P1, P2, P3...)
  - Feature requirements from plan.md
  - Entities from data-model.md
  - Endpoints from contracts/
  
  Tasks MUST be organized by user story so each story can be:
  - Implemented independently
  - Tested independently
  - Delivered as an MVP increment
  
  DO NOT keep these sample tasks in the generated tasks.md file.
  ============================================================================
-->

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create project structure per implementation plan
- [ ] T002 Initialize [language] project with [framework] dependencies
- [ ] T003 [P] Configure linting and formatting tools

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

Examples of foundational tasks (adjust based on your project):

- [ ] T004 Setup database schema and migrations framework
- [ ] T005 [P] Implement authentication/authorization framework
- [ ] T006 [P] Setup API routing and middleware structure
- [ ] T007 Create base models/entities that all stories depend on
- [ ] T008 Configure error handling and logging infrastructure
- [ ] T009 Setup environment configuration management

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - [Title] (Priority: P1) 🎯 MVP

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 1 (MANDATORY per Constitution Principle VI) ⚠️

**NOTE: Write these tests FIRST, ensure they FAIL before implementation**

**Constitution Requirement**: Test-first development is NON-NEGOTIABLE (Principle VI).
Tests MUST be written, run, and FAIL before any implementation code is written.

#### Core Functionality Tests

- [ ] T010 [P] [US1] Contract test for [endpoint] in tests/contract/test_[name].py
- [ ] T011 [P] [US1] Integration test for [user journey] in tests/integration/test_[name].py
- [ ] T012 [P] [US1] Unit tests for core functionality in tests/unit/test_[name].py

#### Unattended Mode Tests (if CLI interface provided - Constitution Principle VIII)

<!--
  If this user story provides CLI commands for core operations (per CLI-001 through CLI-010 in spec.md),
  add tests to verify unattended mode compliance. Delete this section if no CLI interface is provided.
-->

- [ ] T013 [P] [US1] Test CLI accepts configuration via arguments/files/env vars (CLI-001)
- [ ] T014 [P] [US1] Test CLI non-interactive mode (no user input required) (CLI-002)
- [ ] T015 [P] [US1] Test CLI exit codes (0 for success, non-zero for failures) (CLI-003)
- [ ] T016 [P] [US1] Test CLI structured output (JSON format) (CLI-004)
- [ ] T017 [P] [US1] Test CLI dry-run mode (no actual changes made) (CLI-005)
- [ ] T018 [P] [US1] Test CLI input validation (fail fast with clear errors) (CLI-006)
- [ ] T019 [P] [US1] Test CLI silent/quiet mode (minimal output) (CLI-007)

**Checkpoint - Tests Written**: Verify all tests written and failing before proceeding to implementation

### Implementation for User Story 1

- [ ] T020 [P] [US1] Create [Entity1] model in src/models/[entity1].py
- [ ] T021 [P] [US1] Create [Entity2] model in src/models/[entity2].py
- [ ] T022 [US1] Implement [Service] in src/services/[service].py (depends on T020, T021)
- [ ] T023 [US1] Implement [endpoint/feature] in src/[location]/[file].py
- [ ] T024 [US1] Add validation and error handling (Constitution Principle V: input validation required)
- [ ] T025 [US1] Add structured logging for user story 1 operations (Constitution Principle V: audit trail required)
- [ ] T026 [US1] Verify tests now pass (Red-Green-Refactor completion)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - [Title] (Priority: P2)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 2 (MANDATORY per Constitution Principle VI) ⚠️

#### Core Functionality Tests

- [ ] T027 [P] [US2] Contract test for [endpoint] in tests/contract/test_[name].py
- [ ] T028 [P] [US2] Integration test for [user journey] in tests/integration/test_[name].py
- [ ] T029 [P] [US2] Unit tests for core functionality in tests/unit/test_[name].py

#### Unattended Mode Tests (if CLI interface provided - Constitution Principle VIII)

- [ ] T030 [P] [US2] Test CLI accepts configuration via arguments/files/env vars (CLI-001)
- [ ] T031 [P] [US2] Test CLI non-interactive mode (CLI-002)
- [ ] T032 [P] [US2] Test CLI exit codes (CLI-003)
- [ ] T033 [P] [US2] Test CLI structured output (JSON format) (CLI-004)
- [ ] T034 [P] [US2] Test CLI dry-run mode (CLI-005)
- [ ] T035 [P] [US2] Test CLI input validation (CLI-006)
- [ ] T036 [P] [US2] Test CLI silent/quiet mode (CLI-007)

**Checkpoint - Tests Written**: Verify all tests written and failing before proceeding to implementation

### Implementation for User Story 2

- [ ] T037 [P] [US2] Create [Entity] model in src/models/[entity].py
- [ ] T038 [US2] Implement [Service] in src/services/[service].py
- [ ] T039 [US2] Implement [endpoint/feature] in src/[location]/[file].py
- [ ] T040 [US2] Add validation and error handling (Constitution Principle V)
- [ ] T041 [US2] Add structured logging (Constitution Principle V)
- [ ] T042 [US2] Integrate with User Story 1 components (if needed)
- [ ] T043 [US2] Verify tests now pass

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - [Title] (Priority: P3)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 3 (MANDATORY per Constitution Principle VI) ⚠️

#### Core Functionality Tests

- [ ] T044 [P] [US3] Contract test for [endpoint] in tests/contract/test_[name].py
- [ ] T045 [P] [US3] Integration test for [user journey] in tests/integration/test_[name].py
- [ ] T046 [P] [US3] Unit tests for core functionality in tests/unit/test_[name].py

#### Unattended Mode Tests (if CLI interface provided - Constitution Principle VIII)

- [ ] T047 [P] [US3] Test CLI accepts configuration via arguments/files/env vars (CLI-001)
- [ ] T048 [P] [US3] Test CLI non-interactive mode (CLI-002)
- [ ] T049 [P] [US3] Test CLI exit codes (CLI-003)
- [ ] T050 [P] [US3] Test CLI structured output (JSON format) (CLI-004)
- [ ] T051 [P] [US3] Test CLI dry-run mode (CLI-005)
- [ ] T052 [P] [US3] Test CLI input validation (CLI-006)
- [ ] T053 [P] [US3] Test CLI silent/quiet mode (CLI-007)

**Checkpoint - Tests Written**: Verify all tests written and failing before proceeding to implementation

### Implementation for User Story 3

- [ ] T054 [P] [US3] Create [Entity] model in src/models/[entity].py
- [ ] T055 [US3] Implement [Service] in src/services/[service].py
- [ ] T056 [US3] Implement [endpoint/feature] in src/[location]/[file].py
- [ ] T057 [US3] Add validation and error handling (Constitution Principle V)
- [ ] T058 [US3] Add structured logging (Constitution Principle V)
- [ ] T059 [US3] Verify tests now pass

**Checkpoint**: All user stories should now be independently functional

---

[Add more user story phases as needed, following the same pattern]

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

### Test Coverage & Quality

- [ ] TXXX Verify test coverage >80% for core modules (Constitution Principle VI requirement)
- [ ] TXXX Multi-platform validation (Constitution Principle III: test on 2+ platforms)
- [ ] TXXX Security audit: input validation, audit logging (Constitution Principle V)

### Unattended Mode & CI/CD Integration (Constitution Principle VIII)

<!--
  If ANY user story provides CLI interfaces, verify full compliance with Principle VIII.
  Delete this section if the feature is purely internal with no CLI commands.
-->

- [ ] TXXX Verify all core operations have CLI equivalents (detect, configure, validate, deploy)
- [ ] TXXX Verify CLI commands support non-interactive mode (--non-interactive or equivalent)
- [ ] TXXX Verify CLI exit codes follow standards (0=success, non-zero=failure)
- [ ] TXXX Verify structured logging output (JSON format support)
- [ ] TXXX Verify dry-run mode availability (--dry-run flags)
- [ ] TXXX Test CI/CD integration (run in Docker container, GitOps workflow)
- [ ] TXXX Document environment variables for configuration
- [ ] TXXX Verify health check endpoints/commands available

### Documentation & Release

- [ ] TXXX [P] Documentation updates in docs/ (Constitution Principle VII: public APIs need examples)
- [ ] TXXX [P] Add CLI usage examples to README.md
- [ ] TXXX [P] Add CI/CD integration examples to docs/
- [ ] TXXX Code cleanup and refactoring
- [ ] TXXX Performance optimization across all stories
- [ ] TXXX Run quickstart.md validation
- [ ] TXXX Verify Constitution compliance across all implemented features

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - May integrate with US1 but should be independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - May integrate with US1/US2 but should be independently testable

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together (if tests requested):
Task: "Contract test for [endpoint] in tests/contract/test_[name].py"
Task: "Integration test for [user journey] in tests/integration/test_[name].py"

# Launch all models for User Story 1 together:
Task: "Create [Entity1] model in src/models/[entity1].py"
Task: "Create [Entity2] model in src/models/[entity2].py"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
   - Developer C: User Story 3
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence


