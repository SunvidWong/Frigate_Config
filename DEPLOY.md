# Frigate Config Tool - Docker 部署指南

## 更新说明

本次更新主要修复了:
1. ✅ 恢复了"手动配置"和"磁盘映射"页面
2. ✅ 移除了不正确的"桌面模式限制"警告
3. ✅ 硬件扫描功能现在支持 Docker/HTTP 模式
4. ✅ 一键部署功能提供清晰的 Docker 部署指引

## 部署步骤

### 1. 上传镜像到服务器 (10.10.0.129)

```bash
# 在本地机器上(当前目录)
scp frigate-config-tool-latest.tar.gz user@10.10.0.129:/tmp/
```

### 2. 在服务器上加载并运行新镜像

SSH 登录到 10.10.0.129:

```bash
ssh user@10.10.0.129
```

然后执行以下命令:

```bash
# 停止并删除旧容器(如果存在)
docker stop frigate-config 2>/dev/null || true
docker rm frigate-config 2>/dev/null || true

# 加载新镜像
cd /tmp
docker load < frigate-config-tool-latest.tar.gz

# 运行新容器
docker run -d \
  --name frigate-config \
  --restart unless-stopped \
  -p 15000:15000 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  frigate-config-tool:latest

# 查看日志确认启动成功
docker logs frigate-config --tail 50 --follow
```

### 3. 验证部署

访问: `http://10.10.0.129:15000`

应该能看到:
- ✅ 完整的导航栏(包括"手动配置"和"磁盘映射")
- ✅ 硬件扫描功能可用
- ✅ 一键部署功能提供清晰指引
- ✅ 摄像头扫描功能正常

### 4. 清理

```bash
# 删除临时文件
rm /tmp/frigate-config-tool-latest.tar.gz
```

## 快速命令参考

```bash
# 查看容器状态
docker ps | grep frigate-config

# 查看日志
docker logs frigate-config

# 重启容器
docker restart frigate-config

# 进入容器
docker exec -it frigate-config sh
```

## 注意事项

1. **Docker Socket 挂载**: `-v /var/run/docker.sock:/var/run/docker.sock` 允许容器内部操作宿主机的 Docker(用于部署 Frigate)
2. **端口映射**: 15000端口用于 Web UI
3. **重启策略**: `--restart unless-stopped` 确保容器在系统重启后自动启动

## 故障排查

### 如果端口冲突

```bash
# 查看占用15000端口的进程
lsof -i :15000

# 或使用其他端口
docker run -d --name frigate-config -p 16000:15000 frigate-config-tool:latest
```

### 如果容器启动失败

```bash
# 查看详细错误
docker logs frigate-config

# 检查容器状态
docker inspect frigate-config
```

## 生成时间

- 构建时间: 2025-10-19 03:32
- Git 分支: 001-2-1-ui
- Git Commit: b53df9a
