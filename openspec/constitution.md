# Frigate Config Tool 项目开发宪章

**版本**: 1.0
**生效日期**: 2025-10-16
**适用范围**: 所有 OpenSpec 提案和开发任务

---

## 核心开发原则

本项目的所有开发活动必须严格遵守以下核心原则：

### 1. 🇨🇳 全程中文交互

**原则说明**：
- 所有代码注释必须使用中文
- 所有提交信息（commit message）使用中文
- 所有日志输出使用中文
- 所有用户可见的错误消息使用中文
- 所有文档（包括 README、API 文档、用户指南）使用中文
- AI 与用户的所有交互使用中文

**示例**：
```rust
// ✅ 正确：使用中文注释
/// 查找 docker-compose.yml 文件
///
/// 搜索顺序：
/// 1. 当前目录
/// 2. 用户配置路径
/// 3. 默认系统路径
pub fn find_docker_compose_file() -> Result<PathBuf, AppError> {
    // 实现代码
}

// ❌ 错误：使用英文注释
/// Find docker-compose.yml file
pub fn find_docker_compose_file() -> Result<PathBuf, AppError> {
    // Implementation
}
```

**提交信息示例**：
```bash
# ✅ 正确
git commit -m "feat: 添加智能路径搜索功能

- 实现 find_docker_compose_file 函数
- 支持 .yml 和 .yaml 两种扩展名
- 添加详细日志输出

相关 Issue: #123"

# ❌ 错误
git commit -m "feat: add smart path search feature"
```

---

### 2. 🚀 一次性连续开发完成

**原则说明**：
- 用户只需执行一次命令（如 `/openspec:apply <proposal-name>`）
- AI 自动连续完成所有 Phase 的开发任务
- 中途不停顿、不等待用户确认、不需要人工干预
- 按顺序自动执行：代码开发 → 编译检查 → 测试验证 → 格式化 → 提交
- 直到所有功能开发测试完成并提交代码

**执行流程**：
```
用户执行: /openspec:apply fix-hardware-device-insertion-complete
    ↓
AI 自动执行 Phase 0
    ↓
AI 自动执行 Phase 2
    ↓
AI 自动执行 Phase 3
    ↓
AI 自动执行 Phase 4
    ↓
AI 自动运行最终集成测试
    ↓
AI 自动提交所有代码
    ↓
完成 ✅
```

**禁止行为**：
- ❌ 开发到一半停下来问用户"是否继续？"
- ❌ 完成一个函数后等待用户确认再继续
- ❌ 测试失败后停止，应自动修复并重试
- ❌ 需要用户手动运行命令（除非是初始触发命令）

**允许行为**：
- ✅ 遇到错误时自动尝试修复（最多重试3次）
- ✅ 自动运行所有必需的命令（cargo test, npm test等）
- ✅ 自动提交代码到 git
- ✅ 在最后输出完整的执行报告

---

### 3. ✅ 不写无用代码

**原则说明**：
- 只实现项目功能必需的代码
- 不做过度设计（YAGNI - You Aren't Gonna Need It）
- 不添加"可能以后会用到"的功能
- 保持代码简洁明了

**示例**：
```rust
// ✅ 正确：简单直接
pub fn find_docker_compose_file() -> Result<PathBuf, AppError> {
    let paths = vec!["./docker-compose.yml", "./docker-compose.yaml"];
    for path in paths {
        if Path::new(path).exists() {
            return Ok(PathBuf::from(path));
        }
    }
    Err(AppError::NotFound("未找到 docker-compose 文件".to_string()))
}

// ❌ 错误：过度设计
pub struct DockerComposeFileFinder {
    search_paths: Vec<PathBuf>,
    cache: HashMap<String, PathBuf>,
    retry_strategy: RetryStrategy,
    logger: Arc<Mutex<Logger>>,
}
impl DockerComposeFileFinder {
    // 大量不必要的抽象和配置
}
```

---

### 4. ✅ 全自动无人值守

**原则说明**：
- 所有重复性任务必须自动化
- 使用脚本和自动化工具
- 编译、测试、格式化、提交全自动完成

**必须自动执行的任务**：
- `cargo fmt` - 代码格式化
- `cargo clippy` - 静态分析
- `cargo test` - 单元测试
- `npm run lint` - 前端代码检查
- `npm test` - 前端测试
- `git add` + `git commit` - 代码提交

**示例执行序列**：
```bash
# AI 自动执行以下命令序列（用户无需手动运行）
cargo fmt
cargo clippy --fix --allow-dirty
cargo test --lib
npm run lint -- --fix
npm test
git add .
git commit -m "feat: 实现 XXX 功能"
```

---

### 5. ✅ 充分测试再提交

**原则说明**：
- 每个功能完成后必须通过完整测试
- 所有测试通过后才能提交代码
- 测试失败必须修复，不能跳过

**测试层级**：
1. **单元测试** - 测试单个函数/方法
2. **集成测试** - 测试模块间交互
3. **E2E 测试** - 测试完整用户流程（可选）

**最低测试要求**：
- Rust 后端：`cargo test --lib` 全部通过
- TypeScript 前端：`npm test` 全部通过（如有测试）
- 无 clippy 警告
- 无 ESLint 错误

---

## 代码质量标准

### Rust 代码

**必须满足**：
- ✅ `cargo clippy` 零警告
- ✅ `cargo test` 全部通过
- ✅ `cargo fmt` 已格式化
- ✅ 所有公共 API 有中文文档注释
- ✅ 错误处理使用 `Result<T, AppError>`，禁止 `unwrap()` 和 `panic!()`（测试代码除外）

**推荐遵守**：
- 函数参数不超过5个（超过使用 struct 封装）
- 单个函数不超过50行
- 单个文件不超过500行

### TypeScript 代码

**必须满足**：
- ✅ `npm run lint` 零错误
- ✅ `npm test` 全部通过（如有测试）
- ✅ 所有导出的函数/组件有 JSDoc 中文注释
- ✅ 使用 TypeScript 类型，避免 `any`

**推荐遵守**：
- React 组件不超过200行
- 复杂组件拆分为子组件
- 使用自定义 Hook 封装逻辑

---

## Git 提交规范

### Commit Message 格式

```
<类型>: <简短描述>

<详细说明>

<关联信息>
```

**类型（必需）**：
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构（不是新功能也不是修复）
- `test`: 添加或修改测试
- `chore`: 构建过程或辅助工具的变动

**示例**：
```bash
feat: 添加智能 docker-compose.yml 路径搜索

- 实现 find_docker_compose_file 函数
- 支持 .yml 和 .yaml 两种扩展名
- 添加用户自定义路径配置
- 添加详细的中文日志输出

相关 Issue: #123
相关 PR: #124
```

### 提交频率

- ✅ 每完成一个 Phase 提交一次
- ✅ 每修复一个独立的 Bug 提交一次
- ❌ 不要积累大量修改后一次性提交
- ❌ 不要提交未完成的功能（除非使用 feature branch）

---

## OpenSpec 提案规范

### 提案结构

每个提案必须包含：
1. **proposal.md** - 提案主文档
   - Why（为什么）- 问题描述
   - What Changes（变更内容）- 解决方案
   - Implementation Order（实施顺序）- 包含本宪章的核心原则
   - Success Criteria（成功标准）- 验收标准

2. **tasks.md** - 详细任务清单
   - 开发原则（引用本宪章）
   - 按 Phase 分解的具体任务
   - 每个任务有明确的验收标准

### 提案命名

- 使用小写字母和连字符
- 描述性命名
- 示例：`fix-hardware-device-insertion-complete`

---

## 质量检查清单

每个提案完成前必须通过以下检查：

### Phase 0 质量基线
- [ ] `cargo clippy --all-targets --all-features` 零警告
- [ ] `cargo test --lib` 全部通过
- [ ] `cargo fmt` 已执行
- [ ] `npm run lint` 零错误
- [ ] `npm test` 全部通过（如有测试）
- [ ] 错误处理完整性检查通过

### 功能完成检查
- [ ] 所有 tasks.md 中的任务已完成
- [ ] 所有成功标准（Success Criteria）已满足
- [ ] 所有单元测试通过
- [ ] 集成测试通过
- [ ] 中文注释和文档完整

### 提交前检查
- [ ] 代码已格式化
- [ ] 提交信息遵循规范
- [ ] 无未追踪的文件
- [ ] 无调试代码（console.log, println! 等）

---

## 异常处理

### 如果测试失败

1. **自动重试**（最多3次）
   - 修复明显的错误
   - 重新运行测试

2. **如果仍然失败**
   - 输出详细错误信息
   - 标记失败的测试
   - 继续完成其他任务

3. **最后报告**
   - 汇总所有失败项
   - 提供修复建议

### 如果编译失败

1. **分析错误信息**
2. **自动修复语法错误**
3. **重新编译**
4. **最多重试3次**

### 如果遇到不确定的设计决策

1. **优先选择简单方案**
2. **遵循现有代码风格**
3. **添加 TODO 注释标记待改进项**
4. **在提交信息中说明决策理由**

---

## 文档要求

### 代码文档

**Rust**：
```rust
/// 查找 docker-compose.yml 文件
///
/// # 参数
/// - `base_path` - 搜索起始路径
///
/// # 返回值
/// - `Ok(PathBuf)` - 找到的文件路径
/// - `Err(AppError)` - 未找到文件或访问错误
///
/// # 示例
/// ```
/// let path = find_docker_compose_file()?;
/// println!("找到文件: {}", path.display());
/// ```
pub fn find_docker_compose_file(base_path: &Path) -> Result<PathBuf, AppError> {
    // 实现
}
```

**TypeScript**：
```typescript
/**
 * 查找 docker-compose.yml 文件
 *
 * @param basePath - 搜索起始路径
 * @returns 找到的文件路径
 * @throws AppError 未找到文件或访问错误
 *
 * @example
 * ```typescript
 * const path = await findDockerComposeFile('/path/to/search');
 * console.log(`找到文件: ${path}`);
 * ```
 */
export async function findDockerComposeFile(basePath: string): Promise<string> {
  // 实现
}
```

### 用户文档

所有面向用户的文档必须：
- 使用简单明了的中文
- 包含实际的使用示例
- 提供截图（如果是 UI 功能）
- 包含故障排除章节

---

## 版本控制

### 分支策略

- `main` - 主分支，始终保持可发布状态
- `feature/*` - 功能分支（如 `feature/smart-path-search`）
- `fix/*` - Bug 修复分支（如 `fix/hardware-insertion`）

### 合并要求

- 所有 PR 必须通过 CI 检查
- 代码审查通过
- 所有对话和注释使用中文

---

## 持续改进

本宪章会根据项目实践持续更新。如果发现某些原则不合理或需要补充，应：

1. 在项目讨论中提出
2. 达成共识后更新本文档
3. 通知所有开发者
4. 更新相关提案和文档

---

**最后更新**: 2025-10-16
**维护者**: Frigate Config Tool Team
**生效范围**: 所有 OpenSpec 提案和开发活动
