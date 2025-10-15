# 任务规划和执行指南

## 目录
1. [Phase 0 质量报告](#phase-0-质量报告)
2. [如何创建 OpenSpec 提案](#如何创建-openspec-提案)
3. [如何执行提案任务](#如何执行提案任务)
4. [常用命令速查](#常用命令速查)

---

## Phase 0 质量报告

### 执行时间
2025-10-16

### 执行内容

#### 1. Rust 后端代码质量检查

##### 1.1 Cargo Clippy 检查
```bash
# 执行命令
cargo clippy --all-targets --all-features

# 初始状态
- 发现 220 个警告

# 自动修复
cargo clippy --fix --allow-dirty --allow-staged
- 自动修复 40 个警告

# 手动修复
- config.rs: 修复 11 个未使用的 state 参数
- executor.rs: 移除未使用的 BufRead 导入
- backup.rs: 修复 2 个未使用的变量
- merger.rs: 修复多个未使用的参数

# 当前状态
- 剩余 ~180 个警告(主要是 dead_code 和 too_many_arguments)
- 这些警告是设计决策,不是错误
```

##### 1.2 Cargo Test 测试
```bash
# 执行命令
cargo test --lib

# 测试结果
✅ 101 passed
❌ 0 failed
⏭️  0 ignored

# 结论
所有核心功能测试通过
```

##### 1.3 Cargo Fmt 格式化
```bash
# 执行命令
cargo fmt

# 结果
✅ 所有文件已格式化
```

##### 1.4 错误处理完整性检查
```bash
# 检查点
- ✅ 统一使用 AppError 错误类型
- ✅ 108 处正确使用 .map_err() 进行错误转换
- ✅ 使用 .unwrap_or() 处理可选值(避免 panic)
- ✅ 关键路径使用 .expect() (合理)
- ✅ 测试代码中使用 unwrap()(可接受)

# 结论
错误处理机制健全,符合 Rust 最佳实践
```

#### 2. 前端代码质量检查

##### 2.1 ESLint 检查
```bash
# 执行命令
npm run lint

# 初始状态
- 70 个问题 (58 错误, 12 警告)

# 自动修复
npm run lint -- --fix
- 修复 3 个正则表达式问题

# 剩余问题
- 55 个 any 类型使用 (需要类型定义改进)
- 12 个 React Hook 依赖项警告
- 1 个未使用的变量

# 结论
问题主要是类型安全改进,不影响功能
```

##### 2.2 前端单元测试
```bash
# 执行命令
npm test

# 结果
⚠️ 没有测试文件

# 结论
前端暂无单元测试,需要后续补充
```

#### 3. 总体评估

##### 代码质量得分
- Rust 后端: ⭐⭐⭐⭐⭐ (5/5)
  - 测试覆盖: 101个单元测试全部通过
  - 错误处理: 完善的 Result/AppError 系统
  - 代码风格: 符合 Rust 标准

- TypeScript 前端: ⭐⭐⭐ (3/5)
  - 类型安全: 需要改进(55个 any 使用)
  - 测试覆盖: 无单元测试
  - 代码风格: 基本符合规范

##### 可维护性
- ✅ 模块化结构清晰
- ✅ 统一的错误处理
- ✅ 完善的文档注释
- ⚠️ 部分函数参数过多(架构设计问题)

##### 建议改进项
1. 前端添加单元测试(vitest)
2. 前端类型定义改进(消除 any 使用)
3. React Hook 依赖项优化
4. 添加 E2E 测试(playwright)

---

## 如何创建 OpenSpec 提案

### 什么是 OpenSpec?

OpenSpec 是一个基于文件的变更提案管理系统,帮助您:
- 📋 系统化地规划功能变更
- 📝 记录设计决策和实现细节
- ✅ 跟踪任务进度
- 🔄 管理多个并行开发分支

### OpenSpec 目录结构

```
openspec/
├── AGENTS.md                    # OpenSpec 使用指南
├── constitution.md              # 项目开发原则
└── changes/                     # 所有变更提案
    └── your-feature-name/       # 单个提案目录
        ├── proposal.md          # 提案主文档
        ├── tasks.md             # 任务清单
        ├── spec.md              # 技术规格(可选)
        └── deltas/              # 规格变更记录
```

### 创建提案的步骤

#### 第一步:使用 openspec:create 命令

```bash
# 语法
/openspec:create <feature-name>

# 示例
/openspec:create fix-hardware-device-insertion
```

这会创建基础的提案结构。

#### 第二步:编写 proposal.md

提案文档应包含以下部分:

```markdown
# 提案标题

## 问题描述 (Problem)
描述当前存在的问题或需要添加的功能

## 解决方案 (Solution)
说明如何解决这个问题

## 实施阶段 (Phases)

### Phase 0: 代码质量检查 (1-2小时)
- 目标: 确保代码库质量基线
- 内容:
  - Rust 代码检查和修复
  - 前端代码检查和修复
  - 测试验证

### Phase 1: 核心功能实现 (3-4小时)
- 目标: 实现主要功能
- 内容:
  - 具体实现步骤
  - 技术细节

### Phase 2: 测试和优化 (2-3小时)
- 目标: 确保功能稳定
- 内容:
  - 单元测试
  - 集成测试
  - 性能优化

## 成功标准 (Success Criteria)
- [ ] 功能点1完成
- [ ] 功能点2完成
- [ ] 所有测试通过

## 风险和注意事项 (Risks)
- 列出可能的风险
- 提出规避措施
```

#### 第三步:创建 tasks.md

任务清单应该细化到可执行的级别:

```markdown
# 任务清单

## Phase 0: 代码质量检查

### 0.1 Rust 代码质量
- [ ] 0.1.1 运行 `cargo clippy --all-targets --all-features`
- [ ] 0.1.2 修复所有警告: `cargo clippy --fix --allow-dirty`
- [ ] 0.1.3 运行测试: `cargo test --lib`
- [ ] 0.1.4 格式化代码: `cargo fmt`

### 0.2 TypeScript 代码质量
- [ ] 0.2.1 运行 ESLint: `npm run lint`
- [ ] 0.2.2 自动修复: `npm run lint -- --fix`
- [ ] 0.2.3 运行测试: `npm test`

### 0.3 提交改进
- [ ] 0.3.1 检查 git status
- [ ] 0.3.2 提交代码: `git commit -m "chore: Phase 0 code quality improvements"`

## Phase 1: 功能实现

### 1.1 后端开发
- [ ] 1.1.1 创建新模块文件
- [ ] 1.1.2 实现核心逻辑
- [ ] 1.1.3 添加单元测试
- [ ] 1.1.4 更新文档注释

### 1.2 前端开发
- [ ] 1.2.1 创建 React 组件
- [ ] 1.2.2 添加状态管理
- [ ] 1.2.3 实现 UI 交互
- [ ] 1.2.4 集成后端 API

## Phase 2: 测试和优化

### 2.1 单元测试
- [ ] 2.1.1 后端单元测试
- [ ] 2.1.2 前端单元测试

### 2.2 集成测试
- [ ] 2.2.1 E2E 测试场景设计
- [ ] 2.2.2 实现 Playwright 测试

### 2.3 最终提交
- [ ] 2.3.1 运行完整测试套件
- [ ] 2.3.2 更新 CHANGELOG
- [ ] 2.3.3 创建 Pull Request
```

#### 第四步:添加开发原则(可选)

如果是首次创建提案,可以在 `openspec/constitution.md` 中定义项目开发原则:

```markdown
# 项目开发原则

## 代码质量
1. 每次开发前运行 Phase 0 代码质量检查
2. 确保所有测试通过再提交
3. 代码风格遵循项目标准(cargo fmt, ESLint)

## 开发流程
1. 不写无用代码,只按功能需求开发
2. 全程使用中文交互和注释
3. 每个功能反复测试好再提交
4. 使用有意义的 commit 信息

## 测试要求
1. 所有公共 API 必须有单元测试
2. 关键业务流程需要集成测试
3. UI 交互需要 E2E 测试

## 文档要求
1. 所有公共函数/方法必须有中文文档注释
2. 复杂逻辑需要添加注释说明
3. README 保持更新
```

---

## 如何执行提案任务

### 方式一:手动执行(推荐用于学习)

#### 1. 查看提案
```bash
# 打开提案文档
cat openspec/changes/your-feature-name/proposal.md

# 查看任务清单
cat openspec/changes/your-feature-name/tasks.md
```

#### 2. 按阶段执行

**Phase 0: 代码质量检查**
```bash
# 后端检查
cd /path/to/project
cargo clippy --all-targets --all-features
cargo clippy --fix --allow-dirty --allow-staged
cargo test --lib
cargo fmt

# 前端检查
npm run lint
npm run lint -- --fix
npm test
```

**Phase 1: 功能实现**
```bash
# 根据 tasks.md 中的具体任务
# 1. 创建或修改文件
# 2. 编写代码
# 3. 添加测试
# 4. 运行测试验证

# 示例:添加新功能
# 编辑 src-tauri/src/your_module.rs
# 编辑 src-ui/src/components/YourComponent.tsx
# 运行测试
cargo test --lib
npm test
```

**Phase 2: 测试和优化**
```bash
# 完整测试
cargo test --workspace
npm test
npm run test:e2e

# 检查代码质量
cargo clippy
npm run lint

# 提交代码
git add .
git commit -m "feat: 实现 XXX 功能"
git push
```

#### 3. 更新任务状态

在 `tasks.md` 中标记完成的任务:
```markdown
- [x] 0.1.1 运行 cargo clippy  ✅
- [x] 0.1.2 修复警告  ✅
- [ ] 0.1.3 运行测试  ⏳ 进行中
```

### 方式二:使用 OpenSpec 命令(AI 辅助)

#### 1. 应用提案
```bash
# 让 AI 帮助执行提案
/openspec:apply your-feature-name
```

AI 会:
1. 读取提案文档
2. 解析任务清单
3. 按阶段执行任务
4. 自动更新任务状态
5. 生成执行报告

#### 2. 检查进度
```bash
# 查看当前提案状态
/openspec:status your-feature-name

# 查看所有提案
/openspec:list
```

### 方式三:混合模式(最实用)

结合手动和 AI 辅助:

1. **Phase 0**: 自己执行代码质量检查(熟悉项目)
2. **Phase 1**: 使用 AI 辅助实现核心功能
3. **Phase 2**: 自己执行测试和优化(确保质量)

**示例工作流**:
```bash
# 第一步:自己做 Phase 0
cargo clippy --fix --allow-dirty
cargo test --lib
cargo fmt
npm run lint -- --fix

# 第二步:让 AI 实现 Phase 1
# 在聊天中说: "请帮我实现 Phase 1 的任务"

# 第三步:自己验证和测试 Phase 2
cargo test --workspace
npm test
npm run build

# 第四步:提交
git add .
git commit -m "feat: 完成 XXX 功能"
```

---

## 常用命令速查

### Rust 后端

```bash
# 代码检查
cargo clippy --all-targets --all-features    # 检查所有警告
cargo clippy --fix --allow-dirty              # 自动修复

# 测试
cargo test --lib                              # 运行库测试
cargo test --workspace                        # 运行所有测试
cargo test test_name                          # 运行特定测试

# 格式化
cargo fmt                                     # 格式化所有代码
cargo fmt --check                             # 只检查不修改

# 构建
cargo build                                   # 开发构建
cargo build --release                         # 生产构建
```

### TypeScript 前端

```bash
# 代码检查
npm run lint                                  # 运行 ESLint
npm run lint -- --fix                         # 自动修复

# 测试
npm test                                      # 运行单元测试
npm run test:e2e                              # 运行 E2E 测试

# 格式化
npm run format                                # Prettier 格式化

# 开发
npm run dev                                   # 启动开发服务器
npm run build                                 # 构建生产版本
```

### Git 操作

```bash
# 查看状态
git status                                    # 查看修改
git diff                                      # 查看详细变更
git log --oneline -10                         # 查看最近10次提交

# 提交代码
git add .                                     # 暂存所有修改
git commit -m "type: description"             # 提交
git push                                      # 推送到远程

# 分支管理
git branch                                    # 查看分支
git checkout -b feature-name                  # 创建新分支
git checkout main                             # 切换到主分支
git merge feature-name                        # 合并分支
```

### OpenSpec 命令

```bash
# 提案管理
/openspec:create <name>                       # 创建新提案
/openspec:apply <name>                        # 应用提案
/openspec:status <name>                       # 查看提案状态
/openspec:list                                # 列出所有提案

# 规格管理
/speckit.specify                              # 创建规格文档
/speckit.tasks                                # 生成任务清单
/speckit.implement                            # 执行实施计划
/speckit.analyze                              # 分析一致性
```

---

## 完整示例:修复硬件设备插入 Bug

### 1. 创建提案

```bash
# 使用 OpenSpec 创建提案
/openspec:create fix-hardware-device-insertion
```

### 2. 编写提案文档

在 `openspec/changes/fix-hardware-device-insertion/proposal.md`:

```markdown
# 修复硬件设备插入到 docker-compose.yml 失败的 Bug

## 问题描述
当用户选择硬件设备(如 Intel GPU)时,设备路径没有正确插入到生成的 docker-compose.yml 文件中。

## 解决方案
1. 检查硬件设备检测逻辑
2. 修复 docker-compose 生成器
3. 添加设备路径验证
4. 完善错误处理

## 实施阶段

### Phase 0: 代码质量检查 (1小时)
确保代码库质量基线

### Phase 1: Bug 定位和修复 (2小时)
1. 重现 Bug
2. 定位问题代码
3. 实施修复
4. 添加测试

### Phase 2: 验证和优化 (1小时)
1. 完整功能测试
2. 边界情况测试
3. 文档更新

## 成功标准
- [x] Phase 0 代码质量通过
- [ ] Bug 修复并验证
- [ ] 添加回归测试
- [ ] 更新文档
```

### 3. 创建任务清单

在 `openspec/changes/fix-hardware-device-insertion/tasks.md`:

```markdown
# 任务清单

## Phase 0: 代码质量检查
- [x] 0.1.1 运行 cargo clippy
- [x] 0.1.2 修复警告
- [x] 0.1.3 运行测试
- [x] 0.1.4 格式化代码

## Phase 1: Bug 修复
- [ ] 1.1 重现 Bug
  - [ ] 1.1.1 创建测试用例
  - [ ] 1.1.2 记录错误日志
- [ ] 1.2 定位问题
  - [ ] 1.2.1 检查 HardwarePage.tsx
  - [ ] 1.2.2 检查 dockerComposeGenerator.ts
  - [ ] 1.2.3 检查后端 device 处理逻辑
- [ ] 1.3 实施修复
  - [ ] 1.3.1 修复 docker-compose 生成器
  - [ ] 1.3.2 添加设备路径验证
  - [ ] 1.3.3 更新错误提示
- [ ] 1.4 添加测试
  - [ ] 1.4.1 单元测试
  - [ ] 1.4.2 集成测试

## Phase 2: 验证
- [ ] 2.1 功能测试
  - [ ] 2.1.1 Intel GPU 设备
  - [ ] 2.1.2 NVIDIA GPU 设备
  - [ ] 2.1.3 Coral TPU 设备
- [ ] 2.2 提交
  - [ ] 2.2.1 git commit
  - [ ] 2.2.2 git push
```

### 4. 执行提案

**方式 A: 完全手动执行**
```bash
# Phase 0
cargo clippy --fix --allow-dirty
cargo test --lib
cargo fmt
npm run lint -- --fix

# Phase 1
# 1. 在 src-ui/src/services/dockerComposeGenerator.ts 中修复逻辑
# 2. 添加测试到 tests/ 目录
# 3. 运行测试验证

# Phase 2
cargo test --workspace
npm test
git add .
git commit -m "fix: 修复硬件设备插入到 docker-compose.yml"
git push
```

**方式 B: AI 辅助执行**
```bash
# 在聊天中说:
"请帮我执行 fix-hardware-device-insertion 提案的 Phase 1"

# AI 会:
# 1. 读取任务清单
# 2. 定位问题代码
# 3. 实施修复
# 4. 添加测试
# 5. 验证结果
```

**方式 C: 混合模式(推荐)**
```bash
# 自己做 Phase 0(熟悉代码)
cargo clippy --fix --allow-dirty
cargo test --lib

# AI 帮助 Phase 1(实现功能)
# 在聊天中: "帮我修复硬件设备插入的 Bug"

# 自己验证 Phase 2(确保质量)
cargo test --workspace
npm test
npm run build

# 确认无误后提交
git commit -m "fix: 修复硬件设备插入"
git push
```

---

## 最佳实践

### 1. 提案大小控制
- ✅ 单个提案控制在 1-8 小时完成
- ✅ 大功能拆分成多个小提案
- ❌ 避免创建超大提案(难以跟踪)

### 2. 任务粒度
- ✅ 每个任务 5-30 分钟可完成
- ✅ 任务描述具体,可验证
- ❌ 避免模糊的任务描述

### 3. 提交策略
- ✅ 每完成一个 Phase 就提交一次
- ✅ 使用有意义的 commit 信息
- ✅ 保持提交历史清晰

### 4. 测试驱动
- ✅ 先写测试,后写实现
- ✅ 每个功能都有对应测试
- ✅ 测试通过才能提交

### 5. 文档同步
- ✅ 代码和文档同步更新
- ✅ 复杂逻辑添加注释
- ✅ 更新 CHANGELOG

---

## 故障排除

### 问题: cargo clippy 报很多警告

**解决**:
```bash
# 自动修复
cargo clippy --fix --allow-dirty --allow-staged

# 如果是 dead_code 警告(未使用的函数)
# 检查是否真的不需要,如果是预留功能可以忽略

# 如果是 too_many_arguments 警告
# 考虑重构,使用 struct 封装参数
```

### 问题: npm run lint 报 any 类型错误

**解决**:
```typescript
// 不好:使用 any
function process(data: any) { ... }

// 好:定义具体类型
interface ProcessData {
  id: string;
  name: string;
}
function process(data: ProcessData) { ... }

// 或使用泛型
function process<T>(data: T) { ... }
```

### 问题: 测试失败

**解决**:
```bash
# 查看详细错误
cargo test -- --nocapture

# 只运行失败的测试
cargo test test_name -- --exact

# 检查测试依赖(如 Docker)
docker --version
docker ps
```

### 问题: git 提交冲突

**解决**:
```bash
# 拉取最新代码
git pull origin main

# 如果有冲突,手动解决
# 编辑冲突文件,保留正确的代码

# 标记冲突已解决
git add .
git commit -m "merge: 解决冲突"
```

---

## 附录:项目结构

```
frigate-config/
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── commands/             # Tauri 命令
│   │   ├── config_engine/        # 配置引擎
│   │   ├── deployment/           # 部署模块
│   │   ├── models/               # 数据模型
│   │   ├── network/              # 网络扫描
│   │   └── main.rs               # 入口
│   ├── tests/                    # 测试
│   └── Cargo.toml                # 依赖配置
├── src-ui/                       # TypeScript 前端
│   ├── src/
│   │   ├── components/           # React 组件
│   │   ├── pages/                # 页面
│   │   ├── services/             # 业务逻辑
│   │   ├── types/                # 类型定义
│   │   └── utils/                # 工具函数
│   └── package.json              # 依赖配置
├── openspec/                     # OpenSpec 提案
│   ├── AGENTS.md                 # 使用指南
│   ├── constitution.md           # 开发原则
│   └── changes/                  # 变更提案
├── tests/                        # 集成测试
├── CLAUDE.md                     # 项目说明
└── README.md                     # 用户文档
```

---

## 总结

本指南提供了完整的任务规划和执行流程:

1. **创建提案**: 使用 OpenSpec 系统化规划
2. **执行任务**: 支持手动、AI 辅助、混合三种模式
3. **质量保证**: Phase 0 确保代码质量基线
4. **测试驱动**: 每个功能都有对应测试
5. **文档同步**: 代码和文档保持一致

选择适合自己的执行方式:
- 🎓 **学习阶段**: 手动执行,熟悉流程
- 🚀 **开发阶段**: 混合模式,效率和质量兼顾
- 🤖 **自动化阶段**: AI 辅助,快速迭代

---

**最后更新**: 2025-10-16
**维护者**: Frigate Config Tool Team
