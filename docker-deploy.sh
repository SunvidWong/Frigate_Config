#!/bin/bash

# Frigate Configuration Tool - Docker 一键部署脚本
# 支持快速部署 Web 版本和完整 Tauri 版本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}ℹ ${NC}$1"
}

print_success() {
    echo -e "${GREEN}✓ ${NC}$1"
}

print_warning() {
    echo -e "${YELLOW}⚠ ${NC}$1"
}

print_error() {
    echo -e "${RED}✗ ${NC}$1"
}

# 检查命令是否存在
check_command() {
    if ! command -v $1 &> /dev/null; then
        print_error "$1 未安装"
        return 1
    fi
    return 0
}

# 检查前置要求
check_requirements() {
    print_info "检查系统要求..."

    local all_ok=true

    if ! check_command docker; then
        print_error "请先安装 Docker: https://docs.docker.com/get-docker/"
        all_ok=false
    else
        print_success "Docker 已安装: $(docker --version)"
    fi

    if ! check_command docker-compose && ! docker compose version &> /dev/null; then
        print_error "请先安装 Docker Compose: https://docs.docker.com/compose/install/"
        all_ok=false
    else
        if docker compose version &> /dev/null; then
            print_success "Docker Compose 已安装: $(docker compose version)"
        else
            print_success "Docker Compose 已安装: $(docker-compose --version)"
        fi
    fi

    if [ "$all_ok" = false ]; then
        exit 1
    fi

    print_success "系统要求检查通过"
}

# 显示欢迎信息
show_welcome() {
    echo ""
    echo "╔══════════════════════════════════════════════════════╗"
    echo "║   Frigate Configuration Tool - Docker 部署脚本        ║"
    echo "╚══════════════════════════════════════════════════════╝"
    echo ""
}

# 选择部署模式
select_mode() {
    echo "请选择部署模式:"
    echo ""
    echo "  1) Web 版本 (推荐)"
    echo "     - 轻量级，仅前端界面"
    echo "     - 使用 Nginx 服务"
    echo "     - 快速启动，资源占用少"
    echo "     - 端口: 8080"
    echo ""
    echo "  2) 完整 Tauri 版本"
    echo "     - 包含 Rust 后端"
    echo "     - 支持所有功能"
    echo "     - 构建时间较长 (10-30分钟)"
    echo "     - 端口: 1420"
    echo ""
    echo "  3) 退出"
    echo ""

    while true; do
        read -p "请输入选项 [1-3]: " mode
        case $mode in
            1)
                deploy_web
                break
                ;;
            2)
                deploy_tauri
                break
                ;;
            3)
                print_info "退出部署"
                exit 0
                ;;
            *)
                print_error "无效选项，请输入 1-3"
                ;;
        esac
    done
}

# 部署 Web 版本
deploy_web() {
    print_info "开始部署 Web 版本..."
    echo ""

    # 检查是否已有容器在运行
    if docker ps -a | grep -q frigate-config-web; then
        print_warning "检测到已存在的容器"
        read -p "是否删除并重新部署? [y/N]: " confirm
        if [[ $confirm =~ ^[Yy]$ ]]; then
            print_info "停止并删除旧容器..."
            docker stop frigate-config-web 2>/dev/null || true
            docker rm frigate-config-web 2>/dev/null || true
            print_success "已清理旧容器"
        else
            print_info "取消部署"
            exit 0
        fi
    fi

    # 询问端口
    read -p "请输入 Web 访问端口 [默认: 8080]: " web_port
    web_port=${web_port:-8080}

    print_info "构建 Docker 镜像..."
    if docker build -f Dockerfile.web -t frigate-config-web:latest . ; then
        print_success "镜像构建成功"
    else
        print_error "镜像构建失败"
        exit 1
    fi

    print_info "启动容器..."
    if docker run -d \
        --name frigate-config-web \
        -p ${web_port}:80 \
        --restart unless-stopped \
        frigate-config-web:latest ; then
        print_success "容器启动成功"
    else
        print_error "容器启动失败"
        exit 1
    fi

    # 等待服务就绪
    print_info "等待服务启动..."
    sleep 3

    # 检查容器状态
    if docker ps | grep -q frigate-config-web; then
        print_success "服务运行正常"
        echo ""
        echo "╔══════════════════════════════════════════════════════╗"
        echo "║              部署成功! 🎉                             ║"
        echo "╚══════════════════════════════════════════════════════╝"
        echo ""
        echo "访问地址: http://localhost:${web_port}"
        echo ""
        echo "管理命令:"
        echo "  查看日志: docker logs -f frigate-config-web"
        echo "  停止服务: docker stop frigate-config-web"
        echo "  启动服务: docker start frigate-config-web"
        echo "  删除容器: docker rm -f frigate-config-web"
        echo ""
    else
        print_error "服务启动异常，请查看日志:"
        print_info "docker logs frigate-config-web"
        exit 1
    fi
}

# 部署 Tauri 版本
deploy_tauri() {
    print_info "开始部署 Tauri 版本..."
    echo ""

    print_warning "注意: 构建过程可能需要 10-30 分钟"
    read -p "是否继续? [y/N]: " confirm
    if [[ ! $confirm =~ ^[Yy]$ ]]; then
        print_info "取消部署"
        exit 0
    fi

    # 使用 docker-compose
    if docker compose version &> /dev/null; then
        COMPOSE_CMD="docker compose"
    else
        COMPOSE_CMD="docker-compose"
    fi

    print_info "使用 docker-compose 部署..."

    if $COMPOSE_CMD up -d --build; then
        print_success "部署成功"
        echo ""
        echo "╔══════════════════════════════════════════════════════╗"
        echo "║              部署成功! 🎉                             ║"
        echo "╚══════════════════════════════════════════════════════╝"
        echo ""
        echo "访问地址: http://localhost:1420"
        echo ""
        echo "管理命令:"
        echo "  查看日志: $COMPOSE_CMD logs -f"
        echo "  停止服务: $COMPOSE_CMD down"
        echo "  启动服务: $COMPOSE_CMD up -d"
        echo ""
    else
        print_error "部署失败，请查看日志"
        exit 1
    fi
}

# 显示帮助信息
show_help() {
    echo "用法: ./docker-deploy.sh [选项]"
    echo ""
    echo "选项:"
    echo "  -w, --web       部署 Web 版本"
    echo "  -t, --tauri     部署 Tauri 版本"
    echo "  -h, --help      显示帮助信息"
    echo ""
    echo "示例:"
    echo "  ./docker-deploy.sh           # 交互式选择"
    echo "  ./docker-deploy.sh --web     # 直接部署 Web 版本"
    echo "  ./docker-deploy.sh --tauri   # 直接部署 Tauri 版本"
    echo ""
}

# 主函数
main() {
    show_welcome
    check_requirements
    echo ""

    # 解析命令行参数
    if [ $# -eq 0 ]; then
        select_mode
    else
        case $1 in
            -w|--web)
                deploy_web
                ;;
            -t|--tauri)
                deploy_tauri
                ;;
            -h|--help)
                show_help
                ;;
            *)
                print_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    fi
}

# 运行主函数
main "$@"
