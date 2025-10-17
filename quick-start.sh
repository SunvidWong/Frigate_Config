#!/bin/bash
# Frigate Configuration Tool - Phase 8
# Quick Start Script

set -e

echo "🚀 Frigate Configuration Tool - Phase 8"
echo "========================================"
echo ""

# 检查二进制文件
if [ ! -f "target/release/frigate-config-tool" ]; then
    echo "❌ 二进制文件不存在"
    echo "   正在构建..."
    cargo build --release
fi

# 检查前端资源
if [ ! -d "src-ui/dist" ]; then
    echo "❌ 前端资源不存在"
    echo "   正在构建..."
    cd src-ui
    npm run build
    cd ..
fi

echo "✅ 所有文件就绪"
echo ""
echo "📍 启动 Config Tool..."
echo "   访问: http://localhost:15000"
echo ""
echo "💡 提示:"
echo "   - 导航到 '磁盘映射' 页面测试 Phase 8 功能"
echo "   - 按 Ctrl+C 停止应用"
echo ""

# 启动应用
./target/release/frigate-config-tool
