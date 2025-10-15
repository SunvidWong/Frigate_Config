# 硬件设备插入功能修复 - 实施计划

**提案**: fix-hardware-device-insertion
**当前阶段**: Phase 1 - 诊断增强 ✅ 已完成
**下一阶段**: Phase 2 - 根据用户反馈决定

---

## 📋 执行清单

### ✅ 已完成 (Phase 1)

- [x] **根本原因分析**
  - 识别了 5 个关键问题
  - 创建了完整的 OpenSpec 规范文档

- [x] **代码实施**
  - 添加了 80+ 行详细日志
  - 修复了错误处理逻辑
  - 优化了文件路径搜索
  - 所有错误都有中文说明

- [x] **文档创建**
  - OpenSpec 规范: `openspec/specs/fix-hardware-device-insertion.md`
  - Change Proposal: `openspec/changes/001-fix-hardware-device-insertion-phase1.md`
  - 用户执行指南: `EXECUTION-GUIDE.md`

- [x] **代码提交**
  - Commit 60b374a: Phase 1 代码实施
  - Commit 5a02e98: 文档补充

### ⏳ 待执行 (用户操作)

#### 步骤 1: 拉取最新代码
```bash
git pull origin 001-2-1-ui
```

**验证**:
```bash
git log --oneline -1
# 应该显示: 5a02e98 docs: 添加 OpenSpec change proposal 和执行指南
```

#### 步骤 2: 重新构建应用
```bash
# 开发模式(推荐)
npm run tauri dev

# 或生产模式
npm run tauri build
```

**预期时间**: 2-5 分钟

**验证**:
- 开发模式: 应用自动启动
- 生产模式: 在 `src-tauri/target/release/` 找到可执行文件

#### 步骤 3: 测试硬件添加功能

**测试用例 1: 基本功能测试**
1. 打开应用
2. 导航到"硬件检测"页面
3. 选择一个预设硬件(如 Intel GPU)
4. 点击"添加到配置"
5. 观察结果

**预期结果**:
- 成功: 看到成功消息,docker-compose.yml 被更新
- 失败: 看到**清晰的错误消息**(而不是静默失败)

**测试用例 2: 日志验证**
1. 查看终端输出(开发模式)或日志文件
2. 应该看到以 `===` 开始的详细日志
3. 每一步操作都有记录

**测试用例 3: 错误场景**
1. 删除或移动 docker-compose.yml
2. 再次尝试添加硬件
3. 应该看到明确的错误消息告诉你文件在哪里找不到

#### 步骤 4: 收集反馈

**成功场景**:
- [ ] 硬件成功添加到 docker-compose.yml
- [ ] 日志显示所有步骤都成功
- [ ] 文件内容正确(有 devices 或 deploy.resources 配置)

**失败场景**:
- [ ] 看到清晰的错误消息(中文)
- [ ] 错误消息说明了具体问题
- [ ] 日志显示了详细的诊断信息

**需要收集**:
1. 成功/失败的截图
2. 完整的日志输出(特别是 `===` 之间的部分)
3. docker-compose.yml 的位置和内容(前 50 行)
4. 遇到的任何错误消息

---

## 🔄 决策树

### 场景 A: Phase 1 解决了问题 ✅

**标志**:
- 硬件成功添加
- docker-compose.yml 正确更新
- 没有错误

**下一步**:
1. 归档 Phase 1 change
2. 更新 OpenSpec 规范状态为 "已解决"
3. 关闭相关 issue
4. 🎉 完成!

### 场景 B: 错误 - 找不到 docker-compose.yml ⚠️

**标志**:
```
❌ Failed to read docker-compose.yml from './docker-compose.yml': No such file or directory
```

**诊断**:
- 日志会显示搜索的所有路径
- 用户的文件可能在其他位置

**解决方案**: 实施 Phase 2
- 添加自定义路径配置功能
- 创建新的 Tauri 命令: `set_docker_compose_path`
- 添加前端 UI 让用户选择文件

**工作量**: 2-3 小时

### 场景 C: 错误 - 找不到 'frigate' 服务 ⚠️

**标志**:
```
❌ No 'frigate' service found in docker-compose.yml
```

**诊断**:
- 日志会列出找到的所有服务
- 用户的服务可能叫其他名字(如 frigate-nvr)

**解决方案**: 实施 Phase 3
- 实现智能服务查找函数
- 支持多种服务名称模式
- 添加服务名称配置选项

**工作量**: 1-2 小时

### 场景 D: 错误 - YAML 解析失败 ⚠️

**标志**:
```
❌ Failed to parse docker-compose.yml: ...
```

**诊断**:
- 用户的 docker-compose.yml 可能有语法错误
- 或者使用了不兼容的 YAML 特性

**解决方案**: 提供修复建议
- 分析具体的 YAML 错误
- 提供修复后的 docker-compose.yml
- 或者实现更宽松的 YAML 解析

**工作量**: 1-2 小时

### 场景 E: 其他错误 ⚠️

**处理流程**:
1. 分析日志确定根本原因
2. 创建新的 change proposal
3. 实施针对性修复
4. 重新测试

---

## 📊 Phase 2-4 准备状态

### Phase 2: 路径发现改进 (已规划)

**触发条件**: 场景 B (找不到 docker-compose.yml)

**已准备的内容**:
- ✅ 设计文档在 OpenSpec 规范中
- ✅ 代码框架已规划
- ⏳ 等待用户反馈确认需要

**实施步骤**(如需要):
1. 添加 Tauri 命令
   ```rust
   #[tauri::command]
   pub async fn set_docker_compose_path(path: String) -> Result<(), AppError>

   #[tauri::command]
   pub async fn get_docker_compose_path() -> Result<Option<String>, AppError>
   ```

2. 前端添加路径选择 UI
   ```typescript
   <Button onClick={handleSelectPath}>
     选择 docker-compose.yml 文件
   </Button>
   ```

3. 测试所有路径场景

### Phase 3: 服务名称灵活性 (已规划)

**触发条件**: 场景 C (找不到 frigate 服务)

**已准备的内容**:
- ✅ 智能查找算法已设计
- ✅ 多种匹配策略已规划
- ⏳ 等待用户反馈确认需要

**实施步骤**(如需要):
1. 实现 `find_frigate_service()` 函数
2. 测试不同的服务名称
3. 添加服务名称配置选项

### Phase 4: 前端优化 (已规划)

**触发条件**: Phase 2 和 3 完成后

**已准备的内容**:
- ✅ UI 设计已规划
- ✅ 用户体验流程已设计
- ⏳ 等待前面阶段完成

---

## 🧪 测试脚本

### 自动化测试(如需要)

```bash
#!/bin/bash
# test-hardware-insertion.sh

echo "=== 硬件插入功能测试 ==="

# 1. 检查 docker-compose.yml 是否存在
if [ -f "./docker-compose.yml" ]; then
    echo "✓ Found docker-compose.yml"
else
    echo "✗ docker-compose.yml not found"
fi

# 2. 检查是否包含 frigate 服务
if grep -q "frigate:" "./docker-compose.yml" 2>/dev/null; then
    echo "✓ Found frigate service"
else
    echo "✗ frigate service not found"
fi

# 3. 检查硬件设备配置
if [ -f "$HOME/.config/frigate-config-tool/hardware_devices.json" ]; then
    echo "✓ Found hardware devices config"
    echo "Devices:"
    cat "$HOME/.config/frigate-config-tool/hardware_devices.json" | jq -r '.[] | "  - \(.device_name) (\(.device_type)): \(.device_path)"'
else
    echo "✗ No hardware devices configured"
fi

# 4. 检查 docker-compose.yml 中的 devices
if grep -q "devices:" "./docker-compose.yml" 2>/dev/null; then
    echo "✓ Found devices section in docker-compose.yml"
    echo "Devices:"
    grep -A 5 "devices:" "./docker-compose.yml" | grep "- /" | sed 's/^/  /'
else
    echo "⚠ No devices section in docker-compose.yml"
fi

echo ""
echo "=== 测试完成 ==="
```

使用方法:
```bash
chmod +x test-hardware-insertion.sh
./test-hardware-insertion.sh
```

---

## 📞 联系和反馈

### 提供反馈时请包含:

1. **测试环境**
   - 操作系统: macOS / Linux / Windows
   - 构建模式: dev / production
   - Rust 版本: `rustc --version`
   - Node 版本: `node --version`

2. **测试结果**
   - 成功 / 失败
   - 错误消息(如有)
   - 截图(如有)

3. **日志文件**
   - 完整的日志输出
   - 特别是 `=== Starting...` 到 `=== completed ===` 之间的部分

4. **配置文件**
   - docker-compose.yml 的位置
   - docker-compose.yml 的内容(前 50 行)
   - 使用的服务名称

5. **期望行为** (如果与实际不符)

### 反馈渠道

- **GitHub Issue**: (如果有的话)
- **直接交流**: 把上述信息发给我
- **日志文件**: 可以附件形式提供

---

## ✅ 最终验收标准

Phase 1 被认为成功,当且仅当:

- [x] 代码编译无错误
- [ ] 用户能看到详细的日志输出
- [ ] 如果操作失败,用户能看到清晰的错误消息
- [ ] 错误消息包含足够的信息用于排查
- [ ] 日志能帮助开发者快速定位问题

如果以上标准全部满足,则:
- ✅ Phase 1 归档为"完成"
- 🎯 问题的根本原因已识别
- 🔄 准备根据实际情况实施 Phase 2-4

---

**当前状态**: ⏳ 等待用户测试反馈

**最后更新**: 2025-10-16

**Commit**: 5a02e98
