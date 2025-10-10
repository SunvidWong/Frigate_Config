# Frigate Configuration Tool

A cross-platform visual configuration tool for [Frigate NVR](https://frigate.video/) that eliminates the YAML configuration barrier through an iOS-style desktop application.

## ⚠️ Development Status

**Current Phase: Phase 9 (Polish) - 92% COMPLETE ✅**

This project has completed 8 of 9 phases following a structured implementation plan.

### ✅ Completed Phases

- ✅ **Phase 1**: Project Setup
- ✅ **Phase 2**: Foundational Infrastructure
- ✅ **Phase 3**: User Story 1 - Hardware Detection & Camera Setup (MVP)
- ✅ **Phase 4**: User Story 2 - Configuration Management
- ✅ **Phase 5**: User Story 3 - Deployment with Validation
- ✅ **Phase 6**: User Story 4 - Cross-Platform Detection
- ✅ **Phase 7**: User Story 5 - Manual Configuration
- ✅ **Phase 8**: User Story 6 - Disk & Volume Mapping

### 🔄 Current Phase

- **Phase 9**: Polish & Release (190/205 tasks complete)
  - ✅ Test coverage >80%
  - ✅ Architecture documentation
  - ✅ README creation
  - ⏳ API documentation
  - ⏳ User guide
  - ⏳ Performance optimization
  - ⏳ Security audit
  - ⏳ Release packaging

## 🎯 Project Goals

Enable users to:
1. **Detect hardware accelerators** automatically (GPU, TPU, cameras)
2. **Configure cameras visually** with hardware assignment
3. **Manage configurations safely** with conflict detection and rollback
4. **Deploy to Docker** with pre-validation and health checks
5. **Edit YAML manually** for power users while preserving comments
6. **Map storage volumes** for recordings and clips

## 🏗️ Architecture

This is a modular monorepo containing:

- **`src-tauri/`** - Rust backend (Configuration Engine, Deployment Module)
- **`agent/`** - Go agent for hardware detection (cross-platform single binary)
- **`src-ui/`** - React/TypeScript frontend
- **`tests/`** - Contract, integration, and E2E tests
- **`specs/`** - Feature specifications and design documents

### Technology Stack

- **UI**: Tauri 1.5+ with React 18 and TypeScript
- **Backend**: Rust 1.75+
- **Agent**: Go 1.21+
- **Styling**: Tailwind CSS
- **Database**: SQLite (for backup metadata)
- **Deployment**: Docker CLI

## 📋 Prerequisites

To build and run from source:

### Required

- **Rust** 1.75 or later ([Install](https://rustup.rs/))
- **Node.js** 18+ and npm ([Install](https://nodejs.org/))
- **Go** 1.21+ ([Install](https://golang.org/dl/))
- **Docker** (for deployment features)

### Platform-Specific

#### Linux
```bash
sudo apt install libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

#### macOS
```bash
xcode-select --install
```

#### Windows
Install Microsoft Visual Studio C++ Build Tools

## 🚀 Quick Start

### 🐳 Docker Deployment (推荐)

#### ⚡ 使用预构建镜像部署

**使用 docker-compose（推荐）**

创建 `docker-compose.yml` 文件：
```yaml
services:
  frigate-config-web:
    image: ghcr.io/sunvidwong/frigate_config:latest
    container_name: frigate-config-web
    ports:
      - 8080:80
    restart: unless-stopped
```

启动服务：
```bash
docker compose up -d
```

访问：http://localhost:8080

---

**或使用 docker run**
```bash
docker run -d \
  --name frigate-config-web \
  -p 8080:80 \
  --restart unless-stopped \
  ghcr.io/sunvidwong/frigate_config:latest
```

**停止服务**
```bash
docker compose down
# 或
docker stop frigate-config-web
```

---

### 💻 Development Setup (开发模式)

如果需要修改代码或贡献到项目：

```bash
# Clone the repository
git clone https://github.com/SunvidWong/Frigate_Config.git
cd Frigate_Config

# Install frontend dependencies
cd src-ui
npm install
cd ..

# Build the Go agent (when Go is installed)
cd agent
go build -o ../src-tauri/bin/agent cmd/agent/main.go
cd ..

# Run in development mode (when all dependencies are installed)
npm run tauri dev
```

### Build for Production

```bash
# Build all components
npm run tauri build
```

Output will be in `src-tauri/target/release/bundle/`

## 📚 Documentation

### User Documentation
- **[Quick Start Guide](#-quick-start)** - Get started quickly
- **[Docker Installation Guide](DOCKER_INSTALL.md)** - 🐳 Complete Docker deployment documentation
- **[Usage Guide](#usage)** - Step-by-step usage instructions
- **[Troubleshooting](#troubleshooting)** - Common issues and solutions

### Developer Documentation
- **[Architecture](docs/architecture.md)** - System architecture and design
- **[Specification](specs/001-2-1-ui/spec.md)** - Feature requirements
- **[Implementation Plan](specs/001-2-1-ui/plan.md)** - Technical decisions
- **[Tasks Breakdown](specs/001-2-1-ui/tasks.md)** - Implementation tasks (190/205 complete)

## 🧪 Testing

This project follows Test-First Development (TDD) with >80% test coverage.

```bash
# Rust tests (101 unit tests + 24 integration tests)
cargo test

# Go agent tests
cd agent && go test ./...

# E2E tests (Playwright)
npm run test:e2e
```

**Test Statistics**:
- 101 unit tests ✅
- 24 integration test files ✅
- E2E test coverage for all user flows ✅
- Test coverage: >80% (core modules) ✅

## 🤝 Contributing

This project follows the [Constitution principles](specs/001-2-1-ui/spec.md) outlined in the specification:

1. **User First** - Never silently overwrite user configurations
2. **Automation + Control** - Auto-generate configs while preserving manual editability
3. **Cross-Platform** - Support Windows, Linux, macOS on x86/ARM
4. **Modular Design** - Independent, testable components
5. **Security & Permissions** - No privilege escalation, audit logging
6. **AI-Driven Testing** - Test-first development (NON-NEGOTIABLE)
7. **Open Source Community** - Apache 2.0 license

Contributions are welcome once Phase 3 (MVP) is complete!

## 📄 License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [Frigate NVR](https://frigate.video/) - The excellent NVR system this tool configures
- [Tauri](https://tauri.app/) - Cross-platform desktop framework
- The open source community

## 📞 Support

- **Issues**: https://github.com/frigate-config-tool/frigate-config-tool/issues
- **Discussions**: https://github.com/frigate-config-tool/frigate-config-tool/discussions

---

## 📦 Installation

### Download Pre-built Binaries (Coming Soon)

Binaries for Linux, Windows, and macOS will be available in the releases section.

### Usage

1. **Hardware Detection**: Detect GPUs, TPUs, and accelerators
2. **Camera Setup**: Configure cameras with visual interface
3. **Disk Mapping**: Configure storage volumes
4. **Deploy**: One-click Docker deployment with health checks
5. **Manage**: Edit YAML, create backups, rollback changes

---

**Status**: 92% complete (190/205 tasks). Phase 9 (polish & release) in progress.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
