# 📦 Frigate Configuration Tool - Phase 8 安装指南

**版本**: Phase 8 Complete - Disk & Volume Mapping
**平台**: macOS x86_64
**构建日期**: 2025-10-10

---

## 🚀 快速安装 (30秒)

### 步骤 1: 解压 (如果是压缩包)
```bash
# 如果下载的是 tar.gz
tar -xzf frigate-config-tool-phase8-macos.tar.gz
cd phase8-release-macos
```

### 步骤 2: 添加执行权限
```bash
chmod +x frigate-config-tool
```

### 步骤 3: 运行应用
```bash
./frigate-config-tool
```

✅ 完成！应用将在默认浏览器中打开。

---

## 📋 系统要求

### 最低要求
- **操作系统**: macOS 10.15+ (Catalina 或更新)
- **架构**: x86_64 (Intel Mac)
- **内存**: 512 MB 可用
- **磁盘空间**: 50 MB
- **浏览器**: 现代浏览器 (Safari, Chrome, Firefox, Edge)

### 推荐配置
- **操作系统**: macOS 12+ (Monterey 或更新)
- **内存**: 1 GB+ 可用
- **磁盘空间**: 100 MB+

---

## 🎯 首次运行

### 1. 启动应用
```bash
./frigate-config-tool
```

### 2. 应用将自动:
- ✅ 启动后端服务器
- ✅ 初始化数据库 (`~/.frigate-config-tool/database.sqlite`)
- ✅ 在浏览器中打开 UI (通常是 `http://localhost:某个端口`)

### 3. 如果浏览器未自动打开:
- 查看终端输出,找到类似 `http://localhost:xxxxx` 的 URL
- 手动在浏览器中打开该 URL

---

## 🧪 测试 Phase 8 功能

### 快速测试 (5分钟)

#### 1. 导航到磁盘映射页面
- 在 UI 中点击导航栏的 **"磁盘映射"** 或 **"Disk Mapping"**

#### 2. 使用默认路径
- 点击 **"使用默认路径"** 按钮
- ✅ 应该看到 4 个卷映射自动创建:
  - Config: `~/frigate/config`
  - Recordings: `~/frigate/recordings`
  - Clips: `~/frigate/clips`
  - Cache: `~/frigate/cache`

#### 3. 检查磁盘空间
- 在 "磁盘空间检查" 区域输入路径 (如 `/Users/你的用户名`)
- 点击 **"检查磁盘空间"**
- ✅ 应该看到:
  - 总容量
  - 已用空间
  - 可用空间
  - 彩色进度条 (绿色/黄色/红色)

#### 4. 扫描推荐路径
- 点击 **"扫描推荐路径"** 按钮
- ✅ 应该看到系统推荐的存储位置列表
- 每个位置显示可用容量和推荐用途

#### 5. 手动添加卷映射
- 点击 **"+ 添加映射"** 按钮
- 选择类型 (Recordings/Clips/Cache/Config)
- 输入主机路径
- ✅ 看到路径验证反馈 (✅ 或 ❌)
- 点击 **"添加映射"**

#### 6. 查看 Docker 命令
- 滚动到页面底部
- ✅ 看到 **"映射摘要"** 区域
- ✅ 看到生成的 Docker `-v` 命令

---

## 🔧 故障排除

### 问题 1: 无法运行 - "Permission denied"
**解决**:
```bash
chmod +x frigate-config-tool
```

### 问题 2: "Cannot open because developer cannot be verified" (macOS 安全提示)
**解决**:
```bash
# 方法 1: 右键点击 > 打开
右键点击 frigate-config-tool > 选择 "打开" > 点击 "打开"

# 方法 2: 命令行移除隔离属性
xattr -cr frigate-config-tool
./frigate-config-tool
```

### 问题 3: 浏览器未自动打开
**解决**:
- 查看终端输出
- 找到 URL (如 `http://localhost:1420`)
- 手动在浏览器中打开

### 问题 4: 磁盘信息显示 "Failed to get disk info"
**解决**:
```bash
# 确保 df 命令可用
which df

# 测试 df 命令
df -k /
```

### 问题 5: 路径验证总是失败
**解决**:
- 确保路径存在: `ls -la /path/to/check`
- 确保路径是目录,不是文件
- 确保有写权限: `touch /path/to/check/.test && rm /path/to/check/.test`

### 问题 6: 应用无法启动
**原因**: 端口被占用
**解决**:
```bash
# 查找占用端口的进程
lsof -i :1420

# 杀死进程 (如果安全)
kill -9 <PID>

# 重新运行应用
./frigate-config-tool
```

---

## 📂 文件位置

### 应用文件
```
phase8-release-macos/
├── frigate-config-tool    # 主程序
├── web/                   # 前端资源
│   ├── index.html
│   └── assets/
├── README.md              # 快速启动指南
├── INSTALL.md             # 本文件
└── docs/                  # 完整文档
    ├── PHASE_8_COMPLETION.md
    ├── DEPLOYMENT_READY.md
    └── ...
```

### 用户数据
```
~/.frigate-config-tool/
└── database.sqlite        # 应用数据库
```

### 默认卷映射路径 (如果使用默认)
```
~/frigate/
├── config/                # Frigate 配置
├── recordings/            # 录像存储
├── clips/                 # 剪辑存储
└── cache/                 # 临时缓存
```

---

## 🗑️ 卸载

### 删除应用
```bash
rm -rf phase8-release-macos/
```

### 删除用户数据 (可选)
```bash
rm -rf ~/.frigate-config-tool/
```

### 删除默认卷映射路径 (可选)
```bash
rm -rf ~/frigate/
```

---

## 📖 文档资源

### 包含的文档
- **README.md**: 5分钟快速启动
- **INSTALL.md**: 本文件 - 详细安装说明
- **docs/PHASE_8_COMPLETION.md**: 技术实现详情
- **docs/DEPLOYMENT_READY.md**: 生产部署报告
- **docs/PHASE_8_BUILD_COMPLETE.md**: 构建完成报告

### 在线资源
- GitHub 仓库: [链接]
- 问题报告: [GitHub Issues]
- 文档: [链接]

---

## 🔐 安全提示

### 下载验证
```bash
# 验证文件完整性 (如果提供了 SHA256SUMS)
shasum -a 256 -c SHA256SUMS
```

### 权限建议
- ✅ 应用会读取您指定的路径
- ✅ 应用会创建 `~/.frigate-config-tool/` 目录
- ⚠️ 不要在未验证的情况下以 root 运行
- ⚠️ 检查路径验证结果后再使用

---

## 🆘 获取帮助

### 问题诊断
1. 查看终端输出的错误消息
2. 检查 `~/.frigate-config-tool/` 中的日志文件
3. 阅读 `docs/` 中的故障排除部分

### 报告问题
如果遇到问题:
1. 记录错误消息
2. 记录重现步骤
3. 包含系统信息 (macOS 版本, 架构)
4. 在 GitHub Issues 中报告

---

## ✅ 验证安装

### 快速验证清单
- [ ] 应用成功启动
- [ ] UI 在浏览器中打开
- [ ] 可以导航到磁盘映射页面
- [ ] "使用默认路径" 功能工作
- [ ] 磁盘空间检查显示结果
- [ ] 可以添加/删除卷映射
- [ ] Docker 命令预览显示

如果所有项目都打勾,安装成功！✅

---

## 🎓 下一步

### 学习使用
1. 阅读 `README.md` 快速启动
2. 尝试所有 Phase 8 功能
3. 查看 `docs/` 中的详细文档

### 生产使用
1. 配置您的卷映射
2. 验证路径和权限
3. 复制 Docker 命令用于实际部署

### 贡献
1. 报告 bug 和建议
2. 分享您的使用经验
3. 帮助改进文档

---

**安装完成！享受 Frigate Configuration Tool 的磁盘管理功能！** 🎉

如有问题,请查阅 `docs/` 文件夹中的文档或提交 GitHub Issue。
