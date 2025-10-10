# Docker Hub 镜像推送指南

本文档说明如何将构建好的镜像推送到 Docker Hub，供用户远程拉取部署。

## 前置要求

1. Docker Hub 账号（免费注册：https://hub.docker.com/signup）
2. 本地已安装 Docker
3. 已构建好镜像

## 步骤 1: 登录 Docker Hub

```bash
docker login
```

输入你的 Docker Hub 用户名和密码。

## 步骤 2: 构建镜像

```bash
# 构建镜像并打上标签
# 格式: docker build -t 用户名/镜像名:标签 .

docker build -f Dockerfile.web \
  -t sunvidwong/frigate-config-web:latest \
  -t sunvidwong/frigate-config-web:v1.0.0 \
  .
```

说明:
- `sunvidwong` - 你的 Docker Hub 用户名
- `frigate-config-web` - 镜像名称
- `latest` - 最新版本标签
- `v1.0.0` - 特定版本标签

## 步骤 3: 推送镜像到 Docker Hub

```bash
# 推送 latest 版本
docker push sunvidwong/frigate-config-web:latest

# 推送特定版本
docker push sunvidwong/frigate-config-web:v1.0.0
```

## 步骤 4: 验证推送成功

访问 Docker Hub 查看镜像:
```
https://hub.docker.com/r/sunvidwong/frigate-config-web
```

## 用户使用方式

推送成功后，用户可以直接拉取镜像部署：

### 方式 1: 使用 docker run

```bash
docker pull sunvidwong/frigate-config-web:latest

docker run -d \
  --name frigate-config-web \
  -p 8080:80 \
  --restart unless-stopped \
  sunvidwong/frigate-config-web:latest
```

### 方式 2: 使用 docker-compose

创建 `docker-compose.prod.yml`:

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

### 方式 3: 一键脚本

```bash
curl -fsSL https://raw.githubusercontent.com/SunvidWong/Frigate_Config/001-2-1-ui/install.sh | bash
```

## 自动化推送（GitHub Actions）

可以配置 GitHub Actions 自动推送镜像：

```yaml
# .github/workflows/docker-publish.yml
name: Publish Docker Image

on:
  push:
    branches: [ main ]
    tags: [ 'v*.*.*' ]

jobs:
  push:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Login to Docker Hub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          file: ./Dockerfile.web
          push: true
          tags: |
            sunvidwong/frigate-config-web:latest
            sunvidwong/frigate-config-web:${{ github.ref_name }}
```

## 镜像信息

### 镜像大小
- 约 50MB (nginx:alpine + 前端代码)

### 包含内容
- Nginx 1.29+
- Frigate Configuration Tool 前端界面
- 生产环境优化配置

### 端口
- 80 (容器内)
- 可映射到主机任意端口

## 更新镜像

当代码更新后，重新构建并推送：

```bash
# 1. 更新版本号
NEW_VERSION="v1.0.1"

# 2. 构建新版本
docker build -f Dockerfile.web \
  -t sunvidwong/frigate-config-web:latest \
  -t sunvidwong/frigate-config-web:$NEW_VERSION \
  .

# 3. 推送新版本
docker push sunvidwong/frigate-config-web:latest
docker push sunvidwong/frigate-config-web:$NEW_VERSION
```

## 用户更新镜像

用户拉取最新版本：

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
docker compose -f docker-compose.prod.yml up -d
```

## 故障排除

### 问题: 推送权限被拒绝

```
push access denied, repository does not exist or may require authorization
```

**解决方法**:
1. 确认已登录: `docker login`
2. 检查镜像名格式: `用户名/镜像名:标签`
3. 确认用户名正确

### 问题: 网络超时

**解决方法**:
1. 检查网络连接
2. 使用代理: `docker --config ~/.docker push ...`
3. 重试推送

### 问题: 镜像过大

**解决方法**:
1. 使用 `.dockerignore` 排除不必要文件
2. 多阶段构建
3. 使用 Alpine 基础镜像

## 镜像安全

### 扫描漏洞

```bash
docker scan sunvidwong/frigate-config-web:latest
```

### 签名镜像

```bash
docker trust sign sunvidwong/frigate-config-web:v1.0.0
```

---

## 参考资料

- Docker Hub 文档: https://docs.docker.com/docker-hub/
- Docker 镜像构建最佳实践: https://docs.docker.com/develop/develop-images/dockerfile_best-practices/
- GitHub Actions Docker: https://docs.docker.com/build/ci/github-actions/
