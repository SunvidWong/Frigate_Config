#!/bin/bash
# Frigate Configuration Tool - Phase 8
# Docker Deployment Script

set -e

echo "🚀 Frigate Configuration Tool - Phase 8 Docker 部署"
echo "=================================================="
echo ""

# 检查 Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker 未安装。请先安装 Docker。"
    exit 1
fi

if ! docker info &> /dev/null; then
    echo "❌ Docker daemon 未运行。请启动 Docker。"
    exit 1
fi

echo "✅ Docker 已就绪"
echo ""

# 检查必需文件
echo "📋 检查必需文件..."

if [ ! -f "target/release/frigate-config-tool" ]; then
    echo "❌ 未找到二进制文件: target/release/frigate-config-tool"
    echo "   请先运行: cargo build --release"
    exit 1
fi

if [ ! -d "src-ui/dist" ]; then
    echo "❌ 未找到前端资源: src-ui/dist"
    echo "   请先运行: cd src-ui && npm run build"
    exit 1
fi

echo "✅ 所有必需文件存在"
echo ""

# 创建 .env 文件
if [ ! -f ".env" ]; then
    echo "📝 创建 .env 文件..."
    cp .env.example .env
    echo "✅ 已创建 .env 文件,请根据需要修改路径"
    echo ""
fi

# 创建目录
echo "📁 创建默认卷映射目录..."
mkdir -p ~/frigate/config
mkdir -p ~/frigate/recordings
mkdir -p ~/frigate/clips
mkdir -p ~/frigate/cache
echo "✅ 目录创建完成"
echo ""

# 选择部署方式
echo "请选择部署方式:"
echo "1) 仅启动 Frigate (推荐 - Config Tool 在主机运行)"
echo "2) 查看 Docker 日志"
echo "3) 停止所有服务"
echo "4) 清理所有数据"
echo ""
read -p "请选择 [1-4]: " choice

case $choice in
    1)
        echo ""
        echo "🚀 启动 Frigate 服务..."
        docker-compose -f docker-compose-standalone.yml up -d
        echo ""
        echo "✅ Frigate 已启动！"
        echo ""
        echo "📍 访问地址:"
        echo "   - Frigate Web UI: http://localhost:5000"
        echo ""
        echo "💡 启动 Config Tool (在主机上运行):"
        echo "   ./target/release/frigate-config-tool"
        echo "   然后访问: http://localhost:15000"
        echo ""
        ;;
    2)
        echo ""
        echo "📜 查看 Docker 日志..."
        docker-compose -f docker-compose-standalone.yml logs -f
        ;;
    3)
        echo ""
        echo "⏹️  停止所有服务..."
        docker-compose -f docker-compose-standalone.yml down
        echo "✅ 所有服务已停止"
        ;;
    4)
        echo ""
        echo "⚠️  警告: 这将删除所有 Docker 卷数据！"
        read -p "确认删除? [y/N]: " confirm
        if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
            docker-compose -f docker-compose-standalone.yml down -v
            echo "✅ 所有数据已清理"
        else
            echo "❌ 取消操作"
        fi
        ;;
    *)
        echo "❌ 无效选择"
        exit 1
        ;;
esac

echo ""
echo "=================================================="
echo "🎉 操作完成！"
