# Docker 环境摄像头扫描功能部署指南

## 问题诊断

如果看到 `405 (Not Allowed)` 错误，通常是因为：
1. Docker 镜像没有重新构建（还在使用旧版本）
2. 端口配置不匹配
3. HTTP 服务器没有正确启动

## 完整部署步骤

### 1. 构建新的 Docker 镜像

```bash
# 清理旧镜像
docker-compose down
docker rmi frigate-config-ui:latest

# 重新构建
docker build -t frigate-config-ui:latest .
```

### 2. 启动容器

```bash
docker-compose up -d
```

### 3. 检查日志

```bash
# 查看容器日志，确认 HTTP 服务器已启动
docker-compose logs frigate-config-tool

# 应该看到类似信息：
# INFO Starting Frigate Configuration Tool
# INFO Running in HTTP server mode (Docker)
# INFO Starting HTTP server on port 1420
# INFO HTTP server listening on http://0.0.0.0:1420
```

### 4. 测试 API

```bash
# 测试健康检查端点
curl http://localhost:1420/api/health

# 应该返回:
# {"success":true,"data":"OK","error":null}

# 测试网络范围检测
curl -X POST http://localhost:1420/api/guess_network_range_command \
  -H "Content-Type: application/json" \
  -d '{}'
```

### 5. 访问前端

打开浏览器访问: `http://localhost:1420`

前端会自动检测环境并使用 HTTP API。

## 常见问题排查

### 问题 1: 405 Method Not Allowed

**原因:** Docker 镜像未重新构建，仍在使用旧版本代码

**解决:**
```bash
docker-compose down
docker rmi frigate-config-ui:latest
docker build --no-cache -t frigate-config-ui:latest .
docker-compose up -d
```

### 问题 2: 连接到错误的端口 (8080 而不是 1420)

**原因:** 可能有反向代理或负载均衡器

**解决:** 检查前端环境变量或直接访问 1420 端口

### 问题 3: 摄像头扫描返回空列表

**原因:** 网络范围配置错误

**解决:**
- 检查容器网络模式 (`docker network ls`)
- 确保容器可以访问主机网络或摄像头所在网络
- 尝试 `host` 网络模式:
  ```yaml
  # docker-compose.yml
  services:
    frigate-config-tool:
      network_mode: "host"
  ```

### 问题 4: CORS 错误

**原因:** 浏览器跨域限制

**解决:** 已在代码中配置 CORS，允许所有来源。如果仍有问题，检查反向代理配置。

## 端口说明

- **1420**: HTTP 服务器端口（API + 静态文件）
- **8080**: 可能是你的反向代理端口

如果通过反向代理访问，确保：
1. 代理正确转发到容器的 1420 端口
2. 代理没有修改 HTTP 方法
3. 代理传递了所有必要的头部信息

## 网络配置示例

### 桥接模式（默认）
```yaml
services:
  frigate-config-tool:
    ports:
      - "1420:1420"
    networks:
      - frigate-network
```

### Host 模式（推荐用于摄像头扫描）
```yaml
services:
  frigate-config-tool:
    network_mode: "host"
    environment:
      - FRIGATE_HTTP_MODE=true
      - PORT=1420
```

Host 模式的优势：
- 直接访问主机网络
- 可以扫描主机所在网络的所有设备
- 无需端口映射

## 验证部署

1. **检查容器状态:**
   ```bash
   docker ps | grep frigate-config
   ```

2. **检查日志:**
   ```bash
   docker logs frigate-config-tool-phase8 | grep "HTTP server"
   ```

3. **测试 API:**
   ```bash
   curl http://localhost:1420/api/health
   ```

4. **测试前端:**
   打开浏览器，访问 `http://localhost:1420`，进入摄像头扫描页面

5. **测试扫描:**
   点击"快速扫描"按钮，观察网络请求和响应

## 调试技巧

1. **查看详细日志:**
   ```bash
   docker-compose logs -f frigate-config-tool
   ```

2. **进入容器调试:**
   ```bash
   docker exec -it frigate-config-tool-phase8 /bin/sh
   # 检查端口监听
   netstat -tulpn | grep 1420
   ```

3. **测试网络连通性:**
   ```bash
   # 从容器内测试
   docker exec frigate-config-tool-phase8 curl http://localhost:1420/api/health
   ```

4. **检查环境变量:**
   ```bash
   docker exec frigate-config-tool-phase8 env | grep FRIGATE
   ```

## 成功标志

部署成功后，你应该能：
- ✅ 访问 `http://localhost:1420` 看到界面
- ✅ 进入摄像头扫描页面看到蓝色提示信息（Docker/Web 模式）
- ✅ 点击"快速扫描"可以正常扫描内网摄像头
- ✅ 点击"添加硬件设备"可以选择预设硬件并添加
