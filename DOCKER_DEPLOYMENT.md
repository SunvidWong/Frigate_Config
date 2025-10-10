# 🐳 Frigate Configuration Tool - Phase 8 Docker 部署指南

**版本**: Phase 8 Complete
**更新日期**: 2025-10-10

---

## 📋 概述

Phase 8 提供两种部署方式:

1. **推荐方式**: Config Tool 在主机运行 + Frigate 在 Docker 运行
2. **实验性**: 完整 Docker 部署 (Tauri 应用在容器中有限制)

---

## 🚀 快速部署 (推荐)

### 前提条件

- ✅ Docker & Docker Compose 已安装
- ✅ 已构建 Phase 8 二进制 (`target/release/frigate-config-tool`)
- ✅ 已构建前端资源 (`src-ui/dist/`)

### 步骤 1: 运行部署脚本

```bash
# 使用一键部署脚本
./deploy-docker.sh
```

脚本会:
1. 检查 Docker 环境
2. 验证必需文件
3. 创建 .env 配置
4. 创建默认目录
5. 启动服务

### 步骤 2: 配置卷映射

```bash
# 在主机上运行 Config Tool
./target/release/frigate-config-tool
```

然后:
1. 在浏览器打开 http://localhost:1420
2. 导航到 "磁盘映射" 页面
3. 配置您的存储路径
4. 记录配置的路径

### 步骤 3: 更新 Docker 配置

编辑 `.env` 文件,使用 Config Tool 中配置的路径:

```bash
# 编辑 .env 文件
nano .env

# 修改为您配置的路径
FRIGATE_CONFIG_PATH=/your/custom/config/path
FRIGATE_RECORDINGS_PATH=/your/custom/recordings/path
FRIGATE_CLIPS_PATH=/your/custom/clips/path
FRIGATE_CACHE_PATH=/your/custom/cache/path
```

### 步骤 4: 启动 Frigate

```bash
# 启动 Frigate 和相关服务
docker-compose -f docker-compose-standalone.yml up -d
```

### 步骤 5: 验证部署

```bash
# 检查服务状态
docker-compose -f docker-compose-standalone.yml ps

# 查看日志
docker-compose -f docker-compose-standalone.yml logs -f frigate
```

访问:
- **Config Tool**: http://localhost:1420 (主机运行)
- **Frigate Web UI**: http://localhost:5000

---

## 📁 文件结构

### Docker 相关文件

```
frigate-config/
├── Dockerfile                      # 完整构建 (实验性)
├── Dockerfile.simple               # 简单部署
├── docker-compose.yml              # 基础配置
├── docker-compose-standalone.yml   # 独立 Frigate 部署 (推荐)
├── .dockerignore                   # Docker 忽略文件
├── .env.example                    # 环境变量模板
├── .env                            # 实际配置 (需创建)
└── deploy-docker.sh                # 一键部署脚本
```

### 生成的目录

```
~/frigate/                  # 默认 Frigate 数据目录
├── config/                 # 配置文件
├── recordings/             # 录像存储
├── clips/                  # 剪辑存储
└── cache/                  # 临时缓存

~/.frigate-config-tool/     # Config Tool 数据
└── database.sqlite         # 应用数据库
```

---

## 🔧 配置详解

### 环境变量 (.env)

```bash
# Frigate 卷映射路径
FRIGATE_CONFIG_PATH=~/frigate/config
FRIGATE_RECORDINGS_PATH=~/frigate/recordings
FRIGATE_CLIPS_PATH=~/frigate/clips
FRIGATE_CACHE_PATH=~/frigate/cache

# Frigate 设置
FRIGATE_RTSP_PASSWORD=your_secure_password

# 时区
TZ=Asia/Shanghai

# 日志级别
RUST_LOG=info
```

### Docker Compose 服务

#### frigate
- **作用**: Frigate 视频监控主服务
- **端口**:
  - `5000`: Web UI
  - `8554`: RTSP feeds
  - `8555`: WebRTC
- **卷映射**: 使用 Phase 8 配置的路径

#### redis
- **作用**: Frigate 依赖的缓存服务
- **端口**: 内部使用

#### mosquitto (可选)
- **作用**: MQTT broker for Home Assistant
- **端口**: `1883`, `9001`

---

## 📊 使用 Phase 8 卷映射

### 在 Config Tool 中配置

1. 启动 Config Tool
2. 导航到 "磁盘映射"
3. 点击 "使用默认路径" 或手动添加
4. 查看生成的 Docker 命令

### 应用到 Docker

生成的命令示例:
```bash
-v ~/frigate/config:/config
-v ~/frigate/recordings:/media/frigate/recordings
-v ~/frigate/clips:/media/frigate/clips
-v ~/frigate/cache:/tmp/cache
```

这些映射已在 `docker-compose-standalone.yml` 中配置！

---

## 🛠️ 常用操作

### 启动服务

```bash
# 使用部署脚本 (推荐)
./deploy-docker.sh
# 选择 1

# 或手动启动
docker-compose -f docker-compose-standalone.yml up -d
```

### 停止服务

```bash
# 使用部署脚本
./deploy-docker.sh
# 选择 3

# 或手动停止
docker-compose -f docker-compose-standalone.yml down
```

### 查看日志

```bash
# 使用部署脚本
./deploy-docker.sh
# 选择 2

# 或手动查看
docker-compose -f docker-compose-standalone.yml logs -f

# 查看特定服务
docker-compose -f docker-compose-standalone.yml logs -f frigate
```

### 重启服务

```bash
docker-compose -f docker-compose-standalone.yml restart frigate
```

### 更新配置

```bash
# 1. 修改 .env 文件
nano .env

# 2. 重新创建服务
docker-compose -f docker-compose-standalone.yml up -d --force-recreate
```

---

## 🧪 测试验证

### 验证 Frigate 运行

```bash
# 检查容器状态
docker ps | grep frigate

# 应该看到:
# - frigate
# - frigate-redis
# - frigate-mqtt (如果启用)
```

### 验证卷映射

```bash
# 检查挂载点
docker inspect frigate | grep -A 20 "Mounts"

# 应该看到您配置的路径
```

### 验证网络访问

```bash
# 测试 Frigate Web UI
curl -I http://localhost:5000

# 应该返回 200 OK
```

### 完整健康检查

```bash
# 检查所有服务健康状态
docker-compose -f docker-compose-standalone.yml ps

# 所有服务应显示 "Up" 和 "healthy"
```

---

## 🐛 故障排除

### 问题 1: Config Tool 无法连接到 Docker

**原因**: Config Tool 在主机运行,不在容器中

**解决**: Config Tool 不需要连接 Docker,它只是帮助生成配置

### 问题 2: Frigate 无法访问存储

**症状**: Frigate 日志显示权限错误

**解决**:
```bash
# 确保目录存在并有正确权限
mkdir -p ~/frigate/{config,recordings,clips,cache}
chmod -R 755 ~/frigate

# 或在 docker-compose 中添加 user 配置
```

### 问题 3: 端口冲突

**症状**: "port is already allocated"

**解决**:
```bash
# 查找占用端口的进程
lsof -i :5000
lsof -i :1420

# 修改 docker-compose.yml 中的端口映射
# 例如: "15000:5000" 改为使用 15000 端口
```

### 问题 4: 容器无法启动

**症状**: 容器反复重启

**解决**:
```bash
# 查看详细日志
docker logs frigate

# 检查配置文件
cat ~/frigate/config/config.yml

# 验证卷映射路径
ls -la ~/frigate/
```

### 问题 5: 磁盘空间不足

**症状**: Phase 8 显示低空间警告

**解决**:
```bash
# 使用 Phase 8 检查磁盘空间
./target/release/frigate-config-tool
# 导航到磁盘映射页面
# 点击 "检查磁盘空间"

# 或手动检查
df -h ~/frigate/
```

---

## 🔐 安全建议

### 1. 更改默认密码

编辑 `.env`:
```bash
FRIGATE_RTSP_PASSWORD=YourSecurePassword123!
```

### 2. 限制网络访问

在 `docker-compose-standalone.yml` 中:
```yaml
ports:
  - "127.0.0.1:5000:5000"  # 仅本地访问
```

### 3. 使用只读挂载

```yaml
volumes:
  - ~/frigate/config:/config:ro  # 只读
```

### 4. 定期备份

```bash
# 备份配置
tar -czf frigate-config-backup.tar.gz ~/frigate/config

# 备份数据库
cp ~/.frigate-config-tool/database.sqlite backup/
```

---

## 📊 性能优化

### 1. 使用 SSD 做缓存

在 Config Tool 中配置:
```
Cache: /mnt/ssd/frigate-cache
```

### 2. 使用大容量 HDD 做录像

```
Recordings: /mnt/storage/frigate-recordings
```

### 3. 调整 Docker 资源限制

在 `docker-compose-standalone.yml` 中:
```yaml
services:
  frigate:
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 4G
```

### 4. 启用硬件加速

根据您的硬件添加:
```yaml
devices:
  - /dev/dri:/dev/dri  # Intel GPU
```

---

## 📈 监控建议

### 使用 Docker 原生监控

```bash
# 实时资源使用
docker stats frigate

# 持续监控
watch -n 1 docker stats --no-stream
```

### 集成 Prometheus

添加到 `docker-compose-standalone.yml`:
```yaml
services:
  prometheus:
    image: prom/prometheus
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"
```

---

## 🎯 最佳实践

### 1. 使用 Phase 8 规划存储

在部署前:
1. 使用 Config Tool 检查所有候选路径
2. 查看智能推荐
3. 确保有足够空间
4. 记录配置

### 2. 定期维护

```bash
# 每周检查磁盘空间
df -h ~/frigate/

# 每月清理旧录像
find ~/frigate/recordings -mtime +30 -delete

# 定期更新镜像
docker-compose pull
docker-compose up -d
```

### 3. 监控日志大小

```bash
# 限制 Docker 日志大小
# 在 docker-compose.yml 中:
logging:
  options:
    max-size: "10m"
    max-file: "3"
```

---

## 📚 相关文档

- **Phase 8 完整文档**: `PHASE_8_COMPLETION.md`
- **快速启动**: `QUICKSTART_PHASE8.md`
- **部署就绪报告**: `DEPLOYMENT_READY.md`
- **Docker Compose 文档**: https://docs.docker.com/compose/
- **Frigate 文档**: https://docs.frigate.video/

---

## ✅ 部署检查清单

部署前:
- [ ] Docker 已安装并运行
- [ ] 已构建 Phase 8 二进制
- [ ] 已构建前端资源
- [ ] 已创建 .env 文件
- [ ] 已创建存储目录

部署中:
- [ ] Config Tool 在主机成功运行
- [ ] Frigate 容器成功启动
- [ ] 卷映射正确配置
- [ ] 网络访问正常

部署后:
- [ ] 可以访问 Config Tool (http://localhost:1420)
- [ ] 可以访问 Frigate (http://localhost:5000)
- [ ] 磁盘空间检查正常
- [ ] 日志无错误

---

**🎉 Docker 部署完成！享受 Phase 8 的强大功能！**

如有问题,请查看故障排除部分或提交 Issue。
