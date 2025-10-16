# Frigate Configuration Tool - Project Conventions

## Project Overview

Frigate Configuration Tool is a cross-platform visual configuration tool for [Frigate NVR](https://frigate.video/) that eliminates the YAML configuration barrier through an iOS-style desktop application.

**Version:** 0.1.0
**License:** Apache 2.0
**Repository:** https://github.com/frigate-config-tool/frigate-config-tool

## Technology Stack

### Backend
- **Rust** 1.75+ (Tauri backend)
- **Tauri** 1.5+ (Desktop application framework)
- **Go** 1.21+ (Agent for hardware detection)

### Frontend
- **TypeScript** 5.0+
- **React** 18 or **Vue** 3
- **shadcn/ui** or similar iOS-style component library

### Infrastructure
- **Docker** (deployment)
- **Node.js** 18+ (build tooling)

## Project Structure

```
frigate-config/
├── src/                    # CLI source (if any)
├── src-tauri/             # Tauri backend (Rust)
├── src-ui/                # Frontend source (React/Vue)
├── agent/                 # Go agent for hardware detection
├── tests/                 # Integration tests
├── docs/                  # Documentation
├── specs/                 # Legacy specifications
├── openspec/              # OpenSpec specs and changes
│   ├── project.md         # This file
│   ├── specs/             # Current specifications
│   └── changes/           # Change proposals
├── scripts/               # Build and deployment scripts
├── Cargo.toml             # Rust workspace configuration
├── package.json           # Node.js scripts
└── docker-compose.yml     # Docker deployment configuration
```

## Core Principles (Constitution)

1. **User First** - Never silently overwrite user configurations
2. **Automation + Control** - Auto-generate configs while preserving manual editability
3. **Cross-Platform** - Support Windows, Linux, macOS on x86/ARM
4. **Modular Design** - Independent, testable components
5. **Security & Permissions** - No privilege escalation, audit logging
6. **AI-Driven Testing** - Test-first development (NON-NEGOTIABLE)
7. **Open Source Community** - Apache 2.0 license

## Coding Standards

### Rust
- Edition: 2021
- MSRV (Minimum Supported Rust Version): 1.75
- Follow standard Rust conventions
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Command: `cargo test` for testing

### TypeScript
- Version: 5.0+
- Use strict mode
- Follow project ESLint configuration
- Command: `npm run lint` for linting
- Command: `npm test` for testing

### Go
- Version: 1.21+
- Follow standard Go conventions
- Use `go fmt` for formatting
- Command: `go test ./...` for testing

## Testing Strategy

### Test-First Development
- Write tests BEFORE implementation (NON-NEGOTIABLE)
- All new features require tests
- Maintain high test coverage

### Test Types
1. **Unit Tests** - Test individual functions/modules
2. **Integration Tests** - Test component interactions
3. **E2E Tests** - Test full user workflows (Playwright)

### Running Tests
```bash
# All tests
npm test

# Rust tests only
cargo test

# Go tests only
cd agent && go test ./...

# Frontend tests only
cd src-ui && npm test

# E2E tests
npm run test:e2e
npm run test:e2e:ui      # with UI
npm run test:e2e:debug   # debug mode
```

## Build and Development

### Development
```bash
npm run dev              # Start development server
```

### Build
```bash
npm run build            # Build production version
cargo build --release    # Build Rust backend
```

### Formatting
```bash
npm run format           # Format all code
cargo fmt                # Format Rust
cd agent && go fmt ./... # Format Go
```

### Linting
```bash
npm run lint             # Lint all code
cargo clippy             # Lint Rust
```

## Deployment

### Docker Deployment (Recommended)
```bash
# Using docker-compose
docker compose up -d

# Using docker run
docker run -d \
  --name frigate-config-tool \
  -p 15000:15000 \
  -e FRIGATE_HTTP_MODE=true \
  -e RUST_LOG=info \
  --restart unless-stopped \
  ghcr.io/sunvidwong/frigate_config:latest
```

Access at: http://localhost:15000

### Environment Variables
- `FRIGATE_HTTP_MODE=true` - Enable HTTP server mode
- `RUST_LOG=info` - Set log level (trace, debug, info, warn, error)

## Capability Naming Conventions

### Change IDs
- Use kebab-case
- Verb-led prefixes: `add-`, `update-`, `remove-`, `refactor-`
- Examples: `add-camera-discovery`, `update-yaml-parser`, `remove-legacy-api`

### Capability IDs
- Use verb-noun format
- Single purpose per capability
- Examples: `camera-detection`, `config-validation`, `docker-deployment`

## Code Review Guidelines

1. **Functionality** - Does it work as intended?
2. **Tests** - Are there sufficient tests?
3. **Documentation** - Is it well documented?
4. **Performance** - Are there performance concerns?
5. **Security** - Are there security risks?
6. **Constitution** - Does it follow core principles?

## Git Workflow

### Branch Naming
- Feature branches: `feature/<change-id>`
- Bug fixes: `fix/<issue-description>`
- Documentation: `docs/<description>`

### Commit Messages
- Use conventional commits format
- Examples:
  - `feat: add camera discovery feature`
  - `fix: resolve YAML parsing error`
  - `docs: update installation guide`
  - `test: add tests for config validation`

## Documentation Standards

### Code Documentation
- Rust: Use doc comments (`///`) for public APIs
- TypeScript: Use JSDoc comments
- Go: Use Go doc comments

### User Documentation
- Keep README.md updated
- Maintain separate guides in `docs/`
- Include examples and screenshots

## Security Guidelines

1. **No Privilege Escalation** - Never require root/admin
2. **Audit Logging** - Log all configuration changes
3. **Input Validation** - Validate all user inputs
4. **Secure Defaults** - Use secure defaults for all configs
5. **Dependency Management** - Keep dependencies updated

## Performance Guidelines

1. **Simplicity First** - Default to simple solutions
2. **Measure Before Optimizing** - Use profiling data
3. **Complexity Triggers**:
   - Performance data showing bottlenecks
   - Concrete scale requirements (>1000 users, >100MB data)
   - Multiple proven use cases

## Common Tasks

### Adding a New Feature
1. Check existing specs: `openspec list --specs`
2. Create change proposal: `openspec/changes/<change-id>/`
3. Write proposal.md, tasks.md, and spec deltas
4. Validate: `openspec validate <change-id> --strict`
5. Get approval before implementation
6. Implement with tests (test-first!)
7. Archive after deployment: `openspec archive <change-id>`

### Bug Fixes
- If restoring spec behavior: fix directly (no proposal needed)
- If changing behavior: create change proposal

### Updating Dependencies
- Non-breaking updates: update directly
- Breaking changes: create change proposal

## Resources

- [Frigate NVR Documentation](https://frigate.video/)
- [Tauri Documentation](https://tauri.app/)
- [OpenSpec Documentation](https://openspec.dev/)
- [Project Issues](https://github.com/frigate-config-tool/frigate-config-tool/issues)
- [Project Discussions](https://github.com/frigate-config-tool/frigate-config-tool/discussions)

## Questions and Support

- **Issues**: Report bugs and feature requests on GitHub Issues
- **Discussions**: Ask questions on GitHub Discussions
- **Contributing**: See CONTRIBUTING.md (once Phase 3/MVP is complete)

---

Last updated: 2025-10-15
