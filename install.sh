#!/bin/bash

# Frigate Configuration Tool - 一键安装脚本
# 使用预构建的 Docker 镜像，无需本地构建

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
echo "╔═══════════════════════════════════════════════════════╗"
echo "║  Frigate Configuration Tool - 一键安装                 ║"
echo "║  使用预构建镜像，无需本地构建                          ║"
echo "╚═══════════════════════════════════════════════════════╝"
echo ""

# 检查 Docker
print_step "检查 Docker..."
if ! command -v docker &> /dev/null; then
    print_error "未检测到 Docker，请先安装 Docker"
    echo "  下载地址: https://docs.docker.com/get-docker/"
    exit 1
fi
print_success "Docker 已安装: $(docker --version | head -1)"

# 检查容器名是否已存在
if docker ps -a | grep -q frigate-config-web; then
    print_warning "检测到已存在的容器"
    read -p "是否删除并重新安装? [y/N]: " confirm
    if [[ $confirm =~ ^[Yy]$ ]]; then
        print_step "停止并删除旧容器..."
        docker stop frigate-config-web 2>/dev/null || true
        docker rm frigate-config-web 2>/dev/null || true
        print_success "已清理旧容器"
    else
        print_info "取消安装"
        exit 0
    fi
fi

# 询问端口
read -p "请输入 Web 访问端口 [默认: 8080]: " web_port
web_port=${web_port:-8080}

# 拉取镜像
print_step "拉取 Docker 镜像（sunvidwong/frigate-config-web:latest）..."
if docker pull sunvidwong/frigate-config-web:latest; then
    print_success "镜像拉取成功"
else
    print_error "镜像拉取失败"
    print_warning "请检查网络连接或稍后重试"
    exit 1
fi

# 启动容器
print_step "启动容器..."
if docker run -d \
    --name frigate-config-web \
    -p ${web_port}:80 \
    --restart unless-stopped \
    sunvidwong/frigate-config-web:latest; then
    print_success "容器启动成功"
else
    print_error "容器启动失败"
    exit 1
fi

# 等待服务就绪
print_step "等待服务启动..."
sleep 3

# 检查容器状态
if docker ps | grep -q frigate-config-web; then
    echo ""
    echo "╔═══════════════════════════════════════════════════════╗"
    echo "║           🎉 安装成功！                                ║"
    echo "╚═══════════════════════════════════════════════════════╝"
    echo ""
    echo "  访问地址: ${GREEN}http://localhost:${web_port}${NC}"
    echo ""
    echo "  管理命令:"
    echo "    查看日志: docker logs -f frigate-config-web"
    echo "    停止服务: docker stop frigate-config-web"
    echo "    启动服务: docker start frigate-config-web"
    echo "    删除容器: docker rm -f frigate-config-web"
    echo ""
    echo "  更新镜像:"
    echo "    docker pull sunvidwong/frigate-config-web:latest"
    echo "    docker stop frigate-config-web"
    echo "    docker rm frigate-config-web"
    echo "    然后重新运行本脚本"
    echo ""
else
    print_error "服务启动异常，请查看日志:"
    echo "  docker logs frigate-config-web"
    exit 1
fi
