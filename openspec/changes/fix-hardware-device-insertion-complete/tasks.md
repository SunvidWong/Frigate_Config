# Implementation Tasks: 彻底修复硬件设备插入功能

**Change ID**: fix-hardware-device-insertion-complete
**Created**: 2025-10-16
**Status**: 待实施

## 开发原则

**必须遵守的原则:**
1. 🇨🇳 **全程中文交互** - 所有代码注释、提交信息、日志输出、文档、错误消息全部使用中文
2. 🚀 **一次性连续开发完成** - 执行一次命令后，AI 自动连续完成所有 Phase 的开发任务，中途不停顿、不等待用户确认、不需要人工干预，直到所有功能开发测试完成并提交
3. ✅ **不写无用代码** - 只实现项目功能必需的代码，不做过度设计
4. ✅ **全自动无人值守** - 使用脚本和自动化工具，所有编译、测试、格式化、提交全自动完成
5. ✅ **测试优先** - 每个功能完成后必须完整测试通过才能提交

**执行模式:**
- 用户只需执行一次命令（如 `/openspec:apply fix-hardware-device-insertion-complete`）
- AI 自动按顺序完成 Phase 0 → Phase 2 → Phase 3 → Phase 4 → 最终集成
- 每个 Phase 完成后自动运行测试验证
- 所有测试通过后自动提交代码
- 无需用户中途确认或手动操作

## Phase 0: 项目错误全面排查和修复 (1-2小时) 🔥 最高优先级

**目标**: 确保代码质量基线,修复所有现存的编译警告、测试失败和代码质量问题

### 0.1 Rust 代码质量检查和修复
- [ ] 0.1.1 运行 cargo clippy 并修复所有警告
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```
  - 修复未使用的导入 (unused imports)
  - 修复未使用的变量 (unused variables)
  - 修复未使用的函数 (dead code)
  - 替换 `unwrap()` 为适当的错误处理
  - 文件: 所有 Rust 文件

- [ ] 0.1.2 运行 cargo test 并确保所有测试通过
  ```bash
  cargo test --all-features
  ```
  - 修复失败的测试
  - 更新过时的测试断言
  - 确保测试覆盖主要功能
  - 文件: `tests/` 和 `src-tauri/tests/`

- [ ] 0.1.3 运行 cargo fmt 格式化所有代码
  ```bash
  cargo fmt --all
  ```
  - 确保代码风格一致

- [ ] 0.1.4 检查错误处理完整性
  - 审查 `src-tauri/src/commands/deploy.rs` 的所有错误路径
  - 确保没有 `panic!`, `expect()` 或 `unwrap()` 在生产代码中
  - 所有错误都应该返回 `Result<T, AppError>`
  - 文件: `src-tauri/src/commands/deploy.rs`

### 0.2 TypeScript/前端代码质量检查
- [ ] 0.2.1 运行 npm run lint 并修复所有警告
  ```bash
  npm run lint
  ```
  - 修复 ESLint 警告
  - 修复未使用的导入和变量
  - 修复类型错误
  - 文件: `src-ui/src/**/*.tsx` 和 `*.ts`

- [ ] 0.2.2 运行前端测试并确保通过
  ```bash
  npm test
  ```
  - 修复失败的单元测试
  - 更新测试快照(如需要)
  - 文件: `src-ui/src/**/*.test.tsx`

- [ ] 0.2.3 检查前端错误处理
  - 确保所有 API 调用都有 try-catch
  - 确保所有 Promise 都有错误处理
  - 检查 console.error 的使用是否适当
  - 文件: `src-ui/src/pages/HardwarePage.tsx`

### 0.3 集成测试验证
- [ ] 0.3.1 运行端到端测试
  ```bash
  npm run test:e2e
  ```
  - 修复失败的 E2E 测试
  - 确保硬件添加流程测试通过
  - 文件: `tests/e2e/hardware-page.spec.ts`

- [ ] 0.3.2 手动测试关键流程
  - 测试添加 Intel GPU
  - 测试添加 NVIDIA GPU
  - 测试添加 Coral TPU
  - 测试添加 Hailo TPU
  - 验证 docker-compose.yml 更新成功

### 0.4 文档和注释改进
- [ ] 0.4.1 添加/更新函数注释 (中文)
  - `update_docker_compose_devices()` - 说明函数职责和参数
  - `add_device_internal()` - 说明添加设备的完整流程
  - 其他公共函数
  - 文件: `src-tauri/src/commands/deploy.rs`

- [ ] 0.4.2 更新错误消息为中文
  - 确保所有用户可见的错误消息使用中文
  - 日志消息使用中文
  - 文件: `src-tauri/src/commands/deploy.rs`

### 0.5 验收检查
- [ ] 0.5.1 所有质量检查通过
  - ✅ `cargo clippy` 零警告
  - ✅ `cargo test` 全部通过
  - ✅ `npm run lint` 零警告
  - ✅ `npm test` 全部通过
  - ✅ `npm run test:e2e` 全部通过

- [ ] 0.5.2 生成质量报告
  ```bash
  # 创建质量检查脚本
  ./scripts/quality-check.sh
  ```
  - 记录所有检查结果
  - 创建修复前后对比报告
  - 文件: `docs/quality-report-phase0.md` (新建)

- [ ] 0.5.3 提交 Phase 0 改进
  ```bash
  git add .
  git commit -m "chore: 修复所有代码质量问题和测试

  - 修复所有 Clippy 警告
  - 修复所有 ESLint 警告
  - 确保所有测试通过
  - 改进错误处理和日志输出
  - 所有消息和注释改为中文

  Phase 0 - 项目质量基线建立完成"
  ```

## Phase 2: 路径发现改进 (2-3小时)

### 2.1 Backend - 路径管理功能
- [ ] 2.1.1 实现 `find_docker_compose_file()` 函数
  - 搜索顺序: `./docker-compose.yml` → `./docker-compose.yaml` → 其他标准路径
  - 支持 .yml 和 .yaml 两种扩展名
  - 添加详细的日志输出每个搜索路径
  - 文件: `src-tauri/src/commands/deploy.rs`

- [ ] 2.1.2 实现 `set_docker_compose_path()` Tauri 命令
  - 验证文件路径存在性
  - 保存到 `~/.config/frigate-config-tool/docker_compose_path.txt`
  - 返回成功/失败响应
  - 文件: `src-tauri/src/commands/deploy.rs`

- [ ] 2.1.3 实现 `get_docker_compose_path()` Tauri 命令
  - 读取保存的自定义路径
  - 如果未配置,返回 None
  - 文件: `src-tauri/src/commands/deploy.rs`

- [ ] 2.1.4 更新 `update_docker_compose_devices()` 函数
  - 优先使用用户自定义路径
  - 如果无自定义路径,使用 `find_docker_compose_file()`
  - 在日志中明确显示使用的路径来源(自定义/自动检测)
  - 文件: `src-tauri/src/commands/deploy.rs`

- [ ] 2.1.5 注册新的 Tauri 命令
  - 在 `main.rs` 中添加 `set_docker_compose_path` 和 `get_docker_compose_path`
  - 文件: `src-tauri/src/main.rs`

### 2.2 Frontend - 路径配置 UI
- [ ] 2.2.1 添加路径配置组件到 HardwarePage
  - 显示当前使用的路径(自定义或自动检测)
  - 添加"选择文件"按钮
  - 添加"重置为自动检测"按钮
  - 文件: `src-ui/src/pages/HardwarePage.tsx`

- [ ] 2.2.2 实现文件选择逻辑
  - 调用 `set_docker_compose_path()` 保存选择的路径
  - 显示成功/失败提示
  - 刷新显示的当前路径
  - 文件: `src-ui/src/pages/HardwarePage.tsx`

- [ ] 2.2.3 实现路径重置逻辑
  - 删除保存的自定义路径文件
  - 恢复自动检测模式
  - 更新 UI 显示
  - 文件: `src-ui/src/pages/HardwarePage.tsx`

### 2.3 Testing - Phase 2
- [ ] 2.3.1 单元测试
  - 测试 `find_docker_compose_file()` 的搜索逻辑
  - 测试路径保存和读取功能
  - 文件: `src-tauri/tests/test_docker_compose_path.rs` (新建)

- [ ] 2.3.2 集成测试
  - 测试自定义路径场景
  - 测试自动检测场景
  - 测试路径重置场景
  - 文件: `tests/integration/test_docker_compose_path.rs` (新建)

- [ ] 2.3.3 手动测试
  - 验证 UI 交互流程
  - 验证路径选择和显示
  - 检查日志输出是否清晰

## Phase 3: 服务名称灵活性 (1-2小时)

### 3.1 Backend - 智能服务查找
- [ ] 3.1.1 实现 `find_frigate_service()` 函数
  - 步骤 1: 精确匹配 "frigate" 服务
  - 步骤 2: 模糊匹配包含 "frigate" 的服务名
  - 步骤 3: 查找使用 Frigate 镜像的服务
  - 返回找到的服务名称或 None
  - 添加详细日志记录查找过程
  - 文件: `src-tauri/src/commands/deploy.rs`

- [ ] 3.1.2 更新 `update_docker_compose_devices()` 使用新函数
  - 替换硬编码的 `services.get_mut("frigate")`
  - 使用 `find_frigate_service()` 动态查找
  - 在日志中显示找到的服务名称
  - 更新错误消息,列出所有可用的服务
  - 文件: `src-tauri/src/commands/deploy.rs`

### 3.2 Testing - Phase 3
- [ ] 3.2.1 单元测试
  - 测试精确匹配场景
  - 测试模糊匹配场景(frigate-nvr, frigate-main)
  - 测试通过镜像识别场景
  - 测试未找到服务的场景
  - 文件: `src-tauri/tests/test_frigate_service_detection.rs` (新建)

- [ ] 3.2.2 集成测试
  - 使用不同服务名称的 docker-compose.yml 文件
  - 验证硬件设备成功插入
  - 文件: `tests/integration/test_service_name_flexibility.rs` (新建)

- [ ] 3.2.3 创建测试 fixture
  - `docker-compose-frigate-nvr.yml` - 服务名为 frigate-nvr
  - `docker-compose-custom.yml` - 服务名为 my-frigate
  - `docker-compose-no-frigate.yml` - 不包含 Frigate 服务
  - 目录: `tests/fixtures/`

## Phase 4: 用户体验优化 (2-3小时)

### 4.1 Backend - 扩展响应数据
- [ ] 4.1.1 扩展 `AddHardwareDeviceResponse` 结构
  - 添加 `docker_compose_updated: bool`
  - 添加 `docker_compose_path: Option<String>`
  - 添加 `frigate_service_name: Option<String>`
  - 添加 `warnings: Vec<String>`
  - 文件: `src-tauri/src/commands/deploy.rs`

- [ ] 4.1.2 更新 `add_hardware_device_to_config()` 返回详细信息
  - 捕获 docker-compose.yml 更新结果
  - 收集更新过程中的警告信息
  - 返回完整的诊断数据
  - 文件: `src-tauri/src/commands/deploy.rs`

- [ ] 4.1.3 实现 `view_docker_compose_file()` Tauri 命令
  - 获取当前使用的 docker-compose.yml 路径
  - 读取文件内容
  - 返回路径和内容
  - 文件: `src-tauri/src/commands/deploy.rs`

### 4.2 Frontend - 改进提示和可视化
- [ ] 4.2.1 更新硬件添加成功/失败提示
  - 显示详细的成功信息(设备路径、compose 路径、服务名)
  - 显示分类的失败信息(配置保存失败 vs compose 更新失败)
  - 显示警告信息(如果有)
  - 文件: `src-ui/src/pages/HardwarePage.tsx`

- [ ] 4.2.2 添加"查看 docker-compose.yml"功能
  - 添加按钮/链接
  - 实现模态框显示文件内容
  - 高亮显示 Frigate 服务和 devices 部分
  - 文件: `src-ui/src/pages/HardwarePage.tsx`

- [ ] 4.2.3 添加"手动修复"引导流程
  - 如果自动插入失败,显示手动修复步骤
  - 提供复制设备配置代码的功能
  - 链接到文档或故障排除指南
  - 文件: `src-ui/src/pages/HardwarePage.tsx`

- [ ] 4.2.4 改进路径显示组件
  - 使用图标区分自动检测和自定义路径
  - 显示文件是否存在的状态指示器
  - 添加"打开文件所在目录"功能(如果平台支持)
  - 文件: `src-ui/src/pages/HardwarePage.tsx`

### 4.3 Documentation
- [ ] 4.3.1 更新用户指南
  - 添加"配置 docker-compose.yml 路径"章节
  - 添加截图和操作步骤
  - 文件: `docs/user-guide/hardware-configuration.md` (新建或更新)

- [ ] 4.3.2 更新故障排除指南
  - 添加"硬件设备插入失败"章节
  - 列出常见问题和解决方案
  - 文件: `docs/troubleshooting/hardware-insertion-failures.md` (新建)

- [ ] 4.3.3 更新 API 文档
  - 记录所有新增的 Tauri 命令
  - 记录扩展的响应结构
  - 文件: `docs/api/tauri-commands.md` (更新)

### 4.4 Testing - Phase 4
- [ ] 4.4.1 端到端测试
  - 测试完整的添加硬件流程
  - 测试路径配置和查看功能
  - 测试错误场景和手动修复引导
  - 文件: `tests/e2e/hardware-insertion-complete.spec.ts` (新建)

- [ ] 4.4.2 用户验收测试
  - 邀请用户测试新功能
  - 收集反馈和改进建议
  - 记录测试结果

## Final Integration & Release

### 5.1 代码质量检查
- [ ] 5.1.1 运行 Rust linter 和格式化
  ```bash
  cargo fmt
  cargo clippy --all-targets --all-features
  ```

- [ ] 5.1.2 运行 TypeScript linter
  ```bash
  npm run lint
  ```

- [ ] 5.1.3 运行所有测试
  ```bash
  cargo test
  npm test
  npm run test:e2e
  ```

### 5.2 验证和审核
- [ ] 5.2.1 验证所有 acceptance criteria
  - 参考 proposal.md 中的成功标准
  - 逐项验证功能性、可用性、可靠性

- [ ] 5.2.2 代码审查
  - 检查代码质量和可维护性
  - 验证错误处理的完整性
  - 检查日志输出的有用性

- [ ] 5.2.3 性能测试
  - 测试路径搜索性能
  - 测试 YAML 解析和写入性能
  - 确保无明显性能退化

### 5.3 文档和归档
- [ ] 5.3.1 更新 CHANGELOG.md
  - 记录所有新功能和改进
  - 记录修复的 bug

- [ ] 5.3.2 更新 README.md(如需要)
  - 添加新功能说明
  - 更新截图

- [ ] 5.3.3 准备 release notes
  - 总结关键改进
  - 提供升级指南(如需要)

### 5.4 部署和监控
- [ ] 5.4.1 创建 Git 分支和 commits
  - Phase 2: `git checkout -b fix/docker-compose-path-config`
  - Phase 3: `git checkout -b fix/frigate-service-detection`
  - Phase 4: `git checkout -b feat/hardware-insertion-ux`

- [ ] 5.4.2 提交 Pull Requests
  - 每个 Phase 单独提交 PR
  - 包含详细的描述和测试证据
  - 关联相关 issue

- [ ] 5.4.3 合并和发布
  - 合并到 main 分支
  - 创建 release tag
  - 发布新版本

- [ ] 5.4.4 归档 OpenSpec change
  - 运行 `openspec archive fix-hardware-device-insertion-complete`
  - 验证规范已更新
  - 移动提案到 archive 目录

## Checklist Summary

**Phase 2**: 13 tasks
**Phase 3**: 6 tasks
**Phase 4**: 14 tasks
**Final**: 13 tasks

**总计**: 46 tasks

---

**开始前**: 确保 Phase 1 (001-fix-hardware-device-insertion-phase1) 已完成并合并
**完成后**: 使用 `openspec validate fix-hardware-device-insertion-complete --strict` 验证
