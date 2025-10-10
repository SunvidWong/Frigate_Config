#!/bin/bash

# Frigate Configuration Tool - 简单 Docker 启动脚本
# 构建前端并使用 Nginx 容器提供服务

set -e

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_step() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

echo ""
echo "╔═══════════════════════════════════════════════════╗"
echo "║  Frigate Configuration Tool - Docker 部署          ║"
echo "╚═══════════════════════════════════════════════════╝"
echo ""

# 检查 Node.js
if ! command -v node &> /dev/null; then
    print_error "未检测到 Node.js，请先安装 Node.js 18+"
    echo "  下载地址: https://nodejs.org/"
    exit 1
fi

# 检查 Docker
if ! command -v docker &> /dev/null; then
    print_error "未检测到 Docker，请先安装 Docker"
    echo "  下载地址: https://docs.docker.com/get-docker/"
    exit 1
fi

# 检查 Docker Compose
if ! docker compose version &> /dev/null && ! command -v docker-compose &> /dev/null; then
    print_error "未检测到 Docker Compose"
    exit 1
fi

print_success "环境检查通过"
echo ""

# 步骤 1: 构建前端
print_step "步骤 1/3: 构建前端..."

if [ ! -d "src-ui/node_modules" ]; then
    print_step "安装前端依赖..."
    cd src-ui
    npm install
    cd ..
    print_success "依赖安装完成"
fi

print_step "构建前端代码..."
cd src-ui
npm run build

if [ ! -d "dist" ]; then
    print_error "前端构建失败，未找到 dist 目录"
    exit 1
fi

cd ..
print_success "前端构建完成"
echo ""

# 步骤 2: 停止旧容器（如果存在）
print_step "步骤 2/3: 检查现有容器..."

if docker ps -a | grep -q frigate-config-web; then
    print_warning "发现已存在的容器，正在停止..."
    docker stop frigate-config-web 2>/dev/null || true
    docker rm frigate-config-web 2>/dev/null || true
    print_success "已清理旧容器"
fi

echo ""

# 步骤 3: 启动 Docker 容器
print_step "步骤 3/3: 启动 Docker 容器..."

if docker compose version &> /dev/null; then
    docker compose -f docker-compose.web.yml up -d
else
    docker-compose -f docker-compose.web.yml up -d
fi

print_success "容器启动成功"
echo ""

# 等待服务就绪
print_step "等待服务启动..."
sleep 3

# 检查容器状态
if docker ps | grep -q frigate-config-web; then
    echo ""
    echo "╔═══════════════════════════════════════════════════╗"
    echo "║           🎉 部署成功！                            ║"
    echo "╚═══════════════════════════════════════════════════╝"
    echo ""
    echo "  访问地址: ${GREEN}http://localhost:8080${NC}"
    echo ""
    echo "  管理命令:"
    echo "    查看日志: docker logs -f frigate-config-web"
    echo "    停止服务: docker compose -f docker-compose.web.yml down"
    echo "    重启服务: docker restart frigate-config-web"
    echo ""
else
    print_error "容器启动失败，请查看日志:"
    echo "  docker logs frigate-config-web"
    exit 1
fi
