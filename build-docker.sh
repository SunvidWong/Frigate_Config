#!/bin/bash
set -e

echo "🔨 准备 Docker 构建..."

# 1. 确保后端二进制存在
if [ ! -f "target/release/frigate-config-tool" ]; then
    echo "❌ 后端二进制不存在,请先运行: cargo build --release"
    exit 1
fi

# 2. 确保前端dist存在
if [ ! -d "src-ui/dist" ]; then
    echo "❌ 前端构建不存在,请先运行: cd src-ui && npm run build"
    exit 1
fi

# 3. 构建 Docker 镜像
echo "📦 构建 Docker 镜像..."
docker build -f Dockerfile.local -t frigate-config-tool:latest .

echo "✅ Docker 镜像构建完成!"
echo "📋 使用以下命令运行:"
echo "   docker run -d -p 15000:15000 --name frigate-config frigate-config-tool:latest"
