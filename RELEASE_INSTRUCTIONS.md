# 🚀 Release Instructions - v1.0.0

## ✅ 当前状态

- [X] 所有 205 个任务完成
- [X] 所有测试通过 (101 unit + 24 integration)
- [X] 代码已提交
- [X] v1.0.0 标签已创建
- [ ] 推送到远程仓库
- [ ] GitHub Actions 自动构建

---

## 📋 发布步骤

### 步骤 1: 推送代码和标签

```bash
# 推送当前分支
git push origin 001-2-1-ui

# 推送 v1.0.0 标签 (触发 GitHub Actions)
git push origin v1.0.0
```

**这将自动触发**:
- `.github/workflows/release.yml`
- 构建所有平台的 Agent 二进制
- 构建所有平台的 Tauri 应用
- 生成校验和
- 创建 GitHub Release (草稿)

---

### 步骤 2: GitHub Actions 构建 (自动)

GitHub Actions 将自动构建:

#### Agent 二进制
- `agent-linux-amd64`
- `agent-linux-arm64`
- `agent-darwin-amd64` (Intel Mac)
- `agent-darwin-arm64` (Apple Silicon)
- `agent-windows-amd64.exe`

#### Tauri 应用包
**Linux**:
- `frigate-config-tool_*.AppImage` (x86_64)
- `frigate-config-tool_*.deb` (x86_64)

**macOS**:
- `frigate-config-tool_*.dmg` (Intel)
- `frigate-config-tool_*.dmg` (Apple Silicon)
- `frigate-config-tool.app` bundles

**Windows**:
- `frigate-config-tool_*.msi` (installer)
- `frigate-config-tool_*.exe` (NSIS installer)

---

### 步骤 3: 完成 GitHub Release

1. 前往 GitHub Releases 页面
2. 找到自动创建的 v1.0.0 草稿
3. 审查发布说明
4. 验证所有构建产物已上传
5. 点击 **Publish Release**

---

## 🔍 验证发布

### 检查构建状态
```bash
# 查看 GitHub Actions 状态
open https://github.com/YOUR_ORG/frigate-config/actions
```

### 下载测试
```bash
# Linux
curl -LO https://github.com/YOUR_ORG/frigate-config/releases/download/v1.0.0/frigate-config-tool.AppImage
chmod +x frigate-config-tool.AppImage
./frigate-config-tool.AppImage

# macOS
curl -LO https://github.com/YOUR_ORG/frigate-config/releases/download/v1.0.0/frigate-config-tool.dmg
open frigate-config-tool.dmg

# Windows
# Download .msi and run
```

---

## 📦 本地构建 (可选)

如果需要本地构建:

### 构建所有平台 Agent
```bash
./scripts/build-agent.sh
```

输出: `target/release/agent-*`

### 构建当前平台 Tauri 应用
```bash
./scripts/build-release.sh
```

输出: `src-tauri/target/release/bundle/`

---

## 🐛 问题排查

### GitHub Actions 失败

1. **检查日志**:
   ```
   GitHub → Actions → Release workflow → 查看失败的 job
   ```

2. **常见问题**:
   - Rust 工具链版本
   - Node.js 依赖缺失
   - Tauri 签名密钥 (macOS/Windows)

3. **重新触发**:
   ```bash
   git tag -d v1.0.0  # 删除本地标签
   git push origin :refs/tags/v1.0.0  # 删除远程标签
   # 修复问题后重新创建标签
   git tag -a v1.0.0 -m "..."
   git push origin v1.0.0
   ```

### 本地构建失败

**Rust 错误**:
```bash
cargo clean
cargo build --release
```

**Node.js 错误**:
```bash
rm -rf node_modules src-ui/node_modules
npm install
cd src-ui && npm install
```

**Go 错误**:
```bash
cd agent
go mod tidy
go build ./cmd/agent
```

---

## 📝 发布后任务

- [ ] 更新 README badges
- [ ] 在社区宣布发布
- [ ] 创建下一版本的 milestone
- [ ] 监控 issue 反馈
- [ ] 准备补丁版本 (如需要)

---

## 🎯 下一版本规划

### v1.1.0 (可选功能)
- Cloud configuration sync
- Multi-instance management
- Camera template library
- Plugin system

### v1.0.1 (补丁)
- 根据用户反馈修复 bug
- 文档改进
- 性能优化

---

## 📞 支持

- **GitHub Issues**: Bug 报告和功能请求
- **Discussions**: 社区讨论
- **Documentation**: `docs/` 目录

---

## ✨ 发布命令总结

```bash
# 1. 推送代码和标签
git push origin 001-2-1-ui
git push origin v1.0.0

# 2. 等待 GitHub Actions 完成 (15-30 分钟)

# 3. 前往 GitHub Releases
# 4. 发布草稿 Release

# 完成! 🎉
```

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
