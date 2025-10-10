# Frigate Configuration Tool - Docker 部署指南

使用预构建的 Docker 镜像快速部署。

## 快速开始

### 方式 1: 使用 docker-compose（推荐）

创建 `docker-compose.yml` 文件：

```yaml
services:
  frigate-config-web:
    image: ghcr.io/sunvidwong/frigate_config:latest
    container_name: frigate-config-web
    ports:
      - "1420:1420"
    environment:
      - FRIGATE_HTTP_MODE=true
      - RUST_LOG=info
    restart: unless-stopped
```

启动服务：

```bash
docker compose up -d
```

访问：http://localhost:1420

### 方式 2: 使用 docker run

```bash
docker run -d \
  --name frigate-config-web \
  -p 1420:1420 \
  -e FRIGATE_HTTP_MODE=true \
  -e RUST_LOG=info \
  --restart unless-stopped \
  ghcr.io/sunvidwong/frigate_config:latest
```

访问：http://localhost:1420

---

## 管理命令

### 查看日志

```bash
docker logs -f frigate-config-web
```

### 停止服务

```bash
# 使用 docker-compose
docker compose down

# 使用 docker
docker stop frigate-config-web
```

### 启动服务

```bash
# 使用 docker-compose
docker compose up -d

# 使用 docker
docker start frigate-config-web
```

### 重启服务

```bash
docker restart frigate-config-web
```

### 删除容器

```bash
docker rm -f frigate-config-web
```

---

## 更新镜像

```bash
# 拉取最新镜像
docker pull ghcr.io/sunvidwong/frigate_config:latest

# 停止并删除旧容器
docker stop frigate-config-web
docker rm frigate-config-web

# 启动新容器
docker run -d \
  --name frigate-config-web \
  -p 1420:1420 \
  -e FRIGATE_HTTP_MODE=true \
  -e RUST_LOG=info \
  --restart unless-stopped \
  ghcr.io/sunvidwong/frigate_config:latest
```

或使用 docker-compose：

```bash
docker compose pull
docker compose up -d --force-recreate
```

---

## 配置说明

### 修改端口

修改 `docker-compose.yml` 中的端口映射：

```yaml
ports:
  - 9000:80  # 改为 9000 端口
```

或在 docker run 中修改：

```bash
docker run -d -p 9000:80 ghcr.io/sunvidwong/frigate_config:latest
```

### 远程访问

如果在服务器上部署，访问地址为：

```
http://服务器IP:1420
```

---

## 故障排除

### 查看容器状态

```bash
docker ps | grep frigate-config-web
```

### 查看日志

```bash
docker logs frigate-config-web
```

### 端口被占用

修改端口映射：

```yaml
ports:
  - 8081:80  # 使用其他端口
```

### 容器无法启动

1. 检查 Docker 是否运行：`docker ps`
2. 查看日志：`docker logs frigate-config-web`
3. 删除并重新创建：`docker rm -f frigate-config-web && docker compose up -d`

---

## 镜像信息

- **镜像名称**: ghcr.io/sunvidwong/frigate_config
- **镜像大小**: ~50MB
- **基础镜像**: nginx:alpine
- **架构支持**: amd64, arm64
- **自动更新**: 每次代码推送自动构建

---

## 卸载

```bash
# 停止并删除容器
docker stop frigate-config-web
docker rm frigate-config-web

# 删除镜像
docker rmi ghcr.io/sunvidwong/frigate_config:latest

# 删除配置文件
rm docker-compose.yml
```
