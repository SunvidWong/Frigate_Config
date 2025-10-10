# Frigate Configuration Tool - Docker 安装指南

本指南提供了使用 Docker 部署 Frigate Configuration Tool 的完整说明。

## 目录

- [快速开始](#快速开始)
- [方式1: 使用 Web 版本 (推荐)](#方式1-使用-web-版本-推荐)
- [方式2: 使用完整 Tauri 版本](#方式2-使用完整-tauri-版本)
- [配置说明](#配置说明)
- [故障排除](#故障排除)

---

## 快速开始

### 前置要求

- Docker 20.10+
- Docker Compose 2.0+
- 至少 2GB 可用磁盘空间

### 一键启动 (Web 版本)

```bash
# 克隆仓库
git clone https://github.com/SunvidWong/Frigate_Config.git
cd Frigate_Config

# 构建并启动
docker-compose -f docker-compose.web.yml up -d

# 访问应用
# 浏览器打开: http://localhost:8080
```

---

## 方式1: 使用 Web 版本 (推荐)

Web 版本使用 Nginx 提供静态文件服务，适合远程访问和服务器部署。

### 步骤 1: 构建镜像

```bash
# 使用 Dockerfile.web 构建
docker build -f Dockerfile.web -t frigate-config-web:latest .
```

### 步骤 2: 启动容器

使用 docker-compose:

```bash
docker-compose -f docker-compose.web.yml up -d
```

或直接使用 docker run:

```bash
docker run -d \
  --name frigate-config-web \
  -p 8080:80 \
  --restart unless-stopped \
  frigate-config-web:latest
```

### 步骤 3: 访问应用

浏览器打开: `http://localhost:8080`

如果在远程服务器上运行，替换 `localhost` 为服务器 IP 地址。

### 步骤 4: 停止和删除

```bash
# 停止容器
docker-compose -f docker-compose.web.yml down

# 或
docker stop frigate-config-web
docker rm frigate-config-web
```

---

## 方式2: 使用完整 Tauri 版本

完整版本包含 Rust 后端和前端，支持所有功能。

### 步骤 1: 构建镜像

```bash
# 使用主 Dockerfile 构建
docker build -t frigate-config-tool:latest .
```

⚠️ **注意**: 构建时间可能较长 (10-30 分钟)，因为需要编译 Rust 代码。

### 步骤 2: 启动容器

```bash
docker-compose up -d
```

或直接使用 docker run:

```bash
docker run -d \
  --name frigate-config-tool \
  -p 1420:1420 \
  -v frigate-config-data:/app/data \
  --restart unless-stopped \
  frigate-config-tool:latest
```

### 步骤 3: 访问应用

浏览器打开: `http://localhost:1420`

---

## 配置说明

### 端口配置

| 版本 | 默认端口 | 修改方式 |
|------|---------|---------|
| Web 版本 | 8080 | 修改 `docker-compose.web.yml` 中的 `ports` |
| Tauri 版本 | 1420 | 修改 `docker-compose.yml` 中的 `ports` |

示例 - 修改 Web 版本端口为 9000:

```yaml
# docker-compose.web.yml
services:
  frigate-config-web:
    ports:
      - "9000:80"  # 修改这里
```

### 数据持久化

Web 版本不需要持久化存储，配置保存在浏览器 localStorage 中。

Tauri 版本支持数据持久化:

```yaml
# docker-compose.yml
volumes:
  - frigate-config-data:/app/data  # 应用数据
  - frigate-config-db:/root/.frigate-config-tool  # 数据库
```

### 环境变量

```yaml
environment:
  - NODE_ENV=production          # 生产环境模式
  - RUST_LOG=info               # 日志级别 (debug/info/warn/error)
  - FRIGATE_CONFIG_DATA_DIR=/app/data  # 数据目录
```

---

## 使用指南

### 1. 发现摄像头

1. 进入"摄像头发现"页面
2. 输入网段 (如 192.168.1.0/24)
3. 点击"开始扫描"
4. 等待扫描完成

### 2. 配置摄像头

1. 进入"相机配置"页面
2. 添加发现的摄像头或手动添加
3. 配置 RTSP 地址、分辨率等参数
4. 点击"生成配置"

### 3. 编辑和验证配置

1. 进入"配置编辑器"页面
2. 查看生成的 config.yml
3. 如有错误，点击"一键修复"
4. 验证配置无误后保存

### 4. 部署配置

1. 进入"部署"页面
2. 查看 config.yml 和 docker-compose.yml
3. 下载配置文件
4. 按照说明部署 Frigate

---

## 故障排除

### 问题 1: 容器无法启动

**检查日志:**

```bash
# Web 版本
docker logs frigate-config-web

# Tauri 版本
docker logs frigate-config-tool
```

**常见原因:**
- 端口被占用 → 修改端口映射
- 内存不足 → 增加 Docker 内存限制
- 权限问题 → 使用 `sudo` 运行

### 问题 2: 无法访问 Web 界面

**检查容器状态:**

```bash
docker ps | grep frigate-config
```

**检查端口:**

```bash
# Linux/Mac
netstat -an | grep 8080

# Windows
netstat -an | findstr 8080
```

**检查健康状态:**

```bash
docker inspect --format='{{.State.Health.Status}}' frigate-config-web
```

### 问题 3: 配置丢失

**Web 版本:**
- 配置保存在浏览器 localStorage
- 清除浏览器缓存会丢失配置
- 建议导出配置到本地文件

**Tauri 版本:**
- 检查数据卷是否正确挂载
- 查看 `/app/data` 目录权限

### 问题 4: 构建失败

**常见原因:**

1. **网络问题** - npm 安装失败
   ```bash
   # 使用国内镜像
   docker build --build-arg NPM_REGISTRY=https://registry.npmmirror.com -f Dockerfile.web .
   ```

2. **磁盘空间不足**
   ```bash
   docker system prune -a  # 清理未使用的镜像
   ```

3. **Docker 版本过旧**
   ```bash
   docker --version  # 确保 20.10+
   ```

---

## 高级配置

### 自定义 Nginx 配置

如需修改 Nginx 设置，编辑 `nginx.conf`:

```nginx
server {
    listen 80;

    # 添加自定义配置
    client_max_body_size 100M;

    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

重新构建镜像以应用更改。

### 反向代理配置

使用 Nginx/Traefik 作为反向代理:

```nginx
# Nginx 反向代理示例
location /frigate-config/ {
    proxy_pass http://localhost:8080/;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
}
```

### 使用预构建镜像 (如果可用)

```bash
# 拉取预构建镜像
docker pull sunvidwong/frigate-config-web:latest

# 运行
docker run -d -p 8080:80 sunvidwong/frigate-config-web:latest
```

---

## 更新应用

### Web 版本

```bash
# 停止并删除旧容器
docker-compose -f docker-compose.web.yml down

# 拉取最新代码
git pull

# 重新构建和启动
docker-compose -f docker-compose.web.yml up -d --build
```

### Tauri 版本

```bash
docker-compose down
git pull
docker-compose up -d --build
```

---

## 卸载

### 完全清理

```bash
# 停止并删除容器
docker-compose -f docker-compose.web.yml down

# 删除镜像
docker rmi frigate-config-web:latest

# 删除数据卷 (如果有)
docker volume rm frigate-config-data frigate-config-db

# 删除网络 (如果有)
docker network rm frigate-config-network
```

---

## 常见问题

**Q: Web 版本和 Tauri 版本有什么区别？**

A:
- Web 版本: 轻量级，仅前端，使用 Nginx 服务，启动快，资源占用少
- Tauri 版本: 完整版，包含 Rust 后端，支持更多功能，但构建时间长

**Q: 配置会保存到哪里？**

A:
- Web 版本: 浏览器 localStorage (需手动导出备份)
- Tauri 版本: Docker 数据卷 (自动持久化)

**Q: 可以在生产环境使用吗？**

A: 可以，建议使用以下配置:
- 使用 HTTPS (配置反向代理)
- 限制访问 (防火墙/VPN)
- 定期备份配置
- 监控容器健康状态

**Q: 如何在远程服务器部署？**

A:
```bash
# SSH 连接服务器
ssh user@server-ip

# 安装 Docker 和 Docker Compose
curl -fsSL https://get.docker.com | sh

# 克隆并部署
git clone https://github.com/SunvidWong/Frigate_Config.git
cd Frigate_Config
docker-compose -f docker-compose.web.yml up -d

# 访问 http://server-ip:8080
```

---

## 支持

如遇到问题，请:

1. 查看日志: `docker logs frigate-config-web`
2. 检查 GitHub Issues: https://github.com/SunvidWong/Frigate_Config/issues
3. 提交新 Issue 并附上:
   - Docker 版本
   - 操作系统
   - 错误日志
   - 复现步骤

---

## 许可证

本项目使用 MIT 许可证。详见 LICENSE 文件。
