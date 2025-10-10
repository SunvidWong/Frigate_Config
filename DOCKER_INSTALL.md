# Frigate Configuration Tool - Docker 安装指南

本指南提供了使用 Docker 部署 Frigate Configuration Tool 的完整说明。

## 📋 部署方式对比

| 方式 | 优点 | 缺点 | 适用场景 |
|------|------|------|----------|
| **方式0: 预构建镜像** | 🚀 最快，无需构建 | 镜像大小 ~50MB | ⭐ 强烈推荐所有用户 |
| **方式1: 一键脚本** | 自动化，可自定义 | 需要 Node.js | 需要修改代码的用户 |
| **方式2: 明文配置** | 配置清晰，易理解 | 需手动构建前端 | 了解 Docker 的用户 |
| **方式3: 完整镜像** | 独立镜像，可分发 | 构建时间长 | 生产环境 |

## 目录

- [快速开始](#快速开始)
- [方式0: 使用预构建镜像 (⭐ 最推荐)](#方式0-使用预构建镜像--最推荐)
- [方式1: 一键启动脚本](#方式1-一键启动脚本-推荐)
- [方式2: 明文 Docker Compose](#方式2-明文-docker-compose)
- [方式3: 完整镜像构建](#方式3-完整镜像构建)
- [方式4: 使用完整 Tauri 版本](#方式4-使用完整-tauri-版本)
- [配置说明](#配置说明)
- [故障排除](#故障排除)

---

## 快速开始

### 前置要求

**最少要求 (方式1和2):**
- Docker 20.10+
- Docker Compose 2.0+
- Node.js 18+ (仅用于构建前端)
- 至少 1GB 可用磁盘空间

**完整构建 (方式3和4):**
- 以上所有要求
- 至少 5GB 可用磁盘空间（Rust 编译需要）

### ⚡ 最快速启动（使用预构建镜像）

```bash
# 一键安装（无需克隆仓库）
curl -fsSL https://raw.githubusercontent.com/SunvidWong/Frigate_Config/001-2-1-ui/install.sh | bash

# 访问应用
# 浏览器打开: http://localhost:8080
```

**或手动运行**:
```bash
docker run -d \
  --name frigate-config-web \
  -p 8080:80 \
  --restart unless-stopped \
  sunvidwong/frigate-config-web:latest
```

---

## 方式0: 使用预构建镜像 (⭐ 最推荐)

**最快的部署方式！** 直接从 Docker Hub 拉取已构建好的镜像，无需任何本地构建。

### 优势

- ✅ **最快速度** - 无需等待构建，直接拉取运行
- ✅ **零依赖** - 无需 Node.js、npm、Git
- ✅ **最简单** - 一条命令完成部署
- ✅ **镜像小** - 仅 ~50MB (nginx:alpine + 前端)
- ✅ **自动更新** - 拉取 latest 标签即可更新

### 方法 1: 使用一键安装脚本

```bash
# 下载并运行安装脚本
curl -fsSL https://raw.githubusercontent.com/SunvidWong/Frigate_Config/001-2-1-ui/install.sh | bash
```

脚本会自动:
1. 检查 Docker 环境
2. 拉取最新镜像
3. 启动容器
4. 显示访问地址

### 方法 2: 使用 docker run

```bash
# 拉取镜像
docker pull sunvidwong/frigate-config-web:latest

# 启动容器
docker run -d \
  --name frigate-config-web \
  -p 8080:80 \
  --restart unless-stopped \
  sunvidwong/frigate-config-web:latest

# 访问 http://localhost:8080
```

### 方法 3: 使用 docker-compose

下载 `docker-compose.prod.yml`:
```bash
curl -O https://raw.githubusercontent.com/SunvidWong/Frigate_Config/001-2-1-ui/docker-compose.prod.yml
```

内容:
```yaml
services:
  frigate-config-web:
    image: sunvidwong/frigate-config-web:latest
    container_name: frigate-config-web
    ports:
      - "8080:80"
    restart: unless-stopped
```

启动:
```bash
docker compose -f docker-compose.prod.yml up -d
```

### 访问应用

浏览器打开: `http://localhost:8080`

**远程访问**: 替换 `localhost` 为服务器 IP 地址

### 更新镜像

```bash
# 拉取最新镜像
docker pull sunvidwong/frigate-config-web:latest

# 停止并删除旧容器
docker stop frigate-config-web
docker rm frigate-config-web

# 启动新容器
docker run -d \
  --name frigate-config-web \
  -p 8080:80 \
  --restart unless-stopped \
  sunvidwong/frigate-config-web:latest

# 或使用 docker-compose
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d --force-recreate
```

### 镜像信息

- **镜像名**: `sunvidwong/frigate-config-web`
- **标签**:
  - `latest` - 最新版本
  - `v1.0.0` - 特定版本
- **镜像大小**: ~50MB
- **基础镜像**: nginx:alpine
- **架构支持**: amd64, arm64

### 停止和删除

```bash
# 停止容器
docker stop frigate-config-web

# 启动容器
docker start frigate-config-web

# 删除容器
docker rm -f frigate-config-web

# 删除镜像
docker rmi sunvidwong/frigate-config-web:latest
```

---

## 方式1: 一键启动脚本

最简单的部署方式，自动构建前端并启动容器。

### 使用方法

```bash
# 克隆仓库
git clone https://github.com/SunvidWong/Frigate_Config.git
cd Frigate_Config

# 执行一键启动脚本
./start-docker.sh
```

脚本会自动:
1. ✅ 检查 Node.js 和 Docker 环境
2. ✅ 安装前端依赖（如果需要）
3. ✅ 构建前端代码
4. ✅ 启动 Nginx 容器提供服务
5. ✅ 显示访问地址和管理命令

### 访问应用

浏览器打开: `http://localhost:8080`

### 停止服务

```bash
docker compose -f docker-compose.web.yml down
```

---

## 方式2: 明文 Docker Compose

使用简单的 `docker-compose.yml` 配置文件，直接挂载构建产物。

### 优势

- ✅ 配置文件清晰易懂，纯声明式
- ✅ 直接使用 `nginx:alpine` 官方镜像
- ✅ 通过卷挂载前端文件，无需构建镜像
- ✅ 修改配置后直接生效

### docker-compose.web.yml 内容

```yaml
version: '3.8'

services:
  # Frigate 配置工具 Web 界面
  frigate-config-web:
    image: nginx:alpine              # 使用官方 Nginx 镜像
    container_name: frigate-config-web
    ports:
      - "8080:80"                    # 端口映射
    volumes:
      # 挂载前端构建产物
      - ./src-ui/dist:/usr/share/nginx/html:ro
      # 挂载 Nginx 配置
      - ./nginx.conf:/etc/nginx/conf.d/default.conf:ro
    restart: unless-stopped
    environment:
      - NODE_ENV=production
    healthcheck:
      test: ["CMD", "wget", "--quiet", "--tries=1", "--spider", "http://localhost/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s
    labels:
      - "com.frigate.config.app=web"
      - "com.frigate.config.version=1.0"
    networks:
      - frigate-network

networks:
  frigate-network:
    name: frigate-config-network
    driver: bridge
```

### 部署步骤

```bash
# 1. 构建前端
cd src-ui
npm install
npm run build
cd ..

# 2. 启动容器（使用卷挂载，无需构建镜像）
docker compose -f docker-compose.web.yml up -d

# 3. 访问 http://localhost:8080
```

### 修改端口

编辑 `docker-compose.web.yml`:

```yaml
ports:
  - "9000:80"  # 修改为 9000 端口
```

然后重启:

```bash
docker compose -f docker-compose.web.yml restart
```

### 停止服务

```bash
docker compose -f docker-compose.web.yml down
```

---

## 方式3: 完整镜像构建

构建包含所有内容的独立 Docker 镜像，适合生产环境和镜像分发。

### 步骤 1: 构建镜像

```bash
# 使用 Dockerfile.web 构建
docker build -f Dockerfile.web -t frigate-config-web:latest .
```

⏱️ 构建时间: 约 2-5 分钟

### 步骤 2: 启动容器

使用 docker run:

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
docker stop frigate-config-web
docker rm frigate-config-web

# 删除镜像
docker rmi frigate-config-web:latest
```

---

## 方式4: 使用完整 Tauri 版本

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
