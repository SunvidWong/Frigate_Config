#!/bin/bash

echo "=========================================="
echo "Frigate Config Tool - Docker 诊断脚本"
echo "=========================================="
echo ""

# 1. 检查 Docker 是否运行
echo "1. 检查 Docker 状态..."
if ! docker info &> /dev/null; then
    echo "❌ Docker 未运行或无权限访问"
    echo "请启动 Docker Desktop 或运行: sudo systemctl start docker"
    exit 1
else
    echo "✅ Docker 正在运行"
fi
echo ""

# 2. 检查容器状态
echo "2. 检查 frigate-config 相关容器..."
CONTAINERS=$(docker ps -a --filter "name=frigate-config" --format "{{.Names}}\t{{.Status}}\t{{.Ports}}")

if [ -z "$CONTAINERS" ]; then
    echo "❌ 未找到 frigate-config 容器"
    echo ""
    echo "请先启动容器:"
    echo "  docker-compose up -d"
    echo ""
    echo "或者使用 docker run:"
    echo "  docker run -d \\"
    echo "    --name frigate-config-tool \\"
    echo "    -p 15000:15000 \\"
    echo "    -e FRIGATE_HTTP_MODE=true \\"
    echo "    -e RUST_LOG=info \\"
    echo "    frigate-config-ui:latest"
    exit 1
else
    echo "找到以下容器:"
    echo "$CONTAINERS"
    echo ""
fi

# 3. 检查容器是否在运行
echo "3. 检查容器运行状态..."
RUNNING=$(docker ps --filter "name=frigate-config" --format "{{.Names}}")

if [ -z "$RUNNING" ]; then
    echo "⚠️  容器已停止"
    echo ""
    echo "查看容器日志:"
    docker logs $(docker ps -a --filter "name=frigate-config" --format "{{.Names}}" | head -1) --tail 50
    echo ""
    echo "尝试启动容器:"
    echo "  docker-compose up -d"
    echo "  或"
    echo "  docker start <container-name>"
    exit 1
else
    echo "✅ 容器正在运行: $RUNNING"
fi
echo ""

# 4. 检查容器日志
echo "4. 检查容器日志（最近 30 行）..."
echo "----------------------------------------"
docker logs $RUNNING --tail 30
echo "----------------------------------------"
echo ""

# 5. 检查是否看到 HTTP 服务器启动日志
echo "5. 检查 HTTP 服务器启动..."
HTTP_SERVER_LOG=$(docker logs $RUNNING 2>&1 | grep -i "HTTP server listening")

if [ -z "$HTTP_SERVER_LOG" ]; then
    echo "❌ 未找到 HTTP 服务器启动日志"
    echo ""
    echo "可能的原因:"
    echo "  1. 镜像还是旧版本（没有 HTTP 服务器代码）"
    echo "  2. FRIGATE_HTTP_MODE 环境变量未设置"
    echo "  3. 应用启动失败"
    echo ""
    echo "解决方案:"
    echo "  1. 重新构建镜像:"
    echo "     docker-compose down"
    echo "     docker rmi frigate-config-ui:latest"
    echo "     docker build --no-cache -t frigate-config-ui:latest ."
    echo "     docker-compose up -d"
    echo ""
    echo "  2. 检查环境变量:"
    docker exec $RUNNING env | grep FRIGATE || echo "     ⚠️  未设置 FRIGATE_HTTP_MODE"
else
    echo "✅ HTTP 服务器已启动:"
    echo "$HTTP_SERVER_LOG"
fi
echo ""

# 6. 检查端口监听
echo "6. 检查容器内端口监听..."
PORT_CHECK=$(docker exec $RUNNING sh -c "netstat -tuln 2>/dev/null || ss -tuln 2>/dev/null" | grep 15000)

if [ -z "$PORT_CHECK" ]; then
    echo "❌ 容器内未监听 15000 端口"
    echo ""
    echo "尝试检查进程:"
    docker exec $RUNNING ps aux
else
    echo "✅ 容器内正在监听:"
    echo "$PORT_CHECK"
fi
echo ""

# 7. 测试容器内 API
echo "7. 测试容器内 API 访问..."
API_TEST=$(docker exec $RUNNING sh -c "wget -q -O- http://localhost:15000/api/health 2>/dev/null || curl -s http://localhost:15000/api/health 2>/dev/null")

if [ -z "$API_TEST" ]; then
    echo "❌ 容器内无法访问 API"
else
    echo "✅ 容器内 API 响应:"
    echo "$API_TEST"
fi
echo ""

# 8. 测试主机访问
echo "8. 测试主机访问容器..."
HOST_TEST=$(curl -s http://localhost:15000/api/health 2>/dev/null)

if [ -z "$HOST_TEST" ]; then
    echo "❌ 主机无法访问容器 15000 端口"
    echo ""
    echo "检查端口映射:"
    docker port $RUNNING
    echo ""
    echo "检查防火墙:"
    echo "  macOS: 系统偏好设置 > 安全性与隐私 > 防火墙"
    echo "  Linux: sudo ufw status"
else
    echo "✅ 主机可以访问 API:"
    echo "$HOST_TEST"
fi
echo ""

# 9. 检查 docker-compose 配置
echo "9. 检查 docker-compose 配置..."
if [ -f "docker-compose.yml" ]; then
    echo "当前 docker-compose.yml 端口配置:"
    grep -A 2 "ports:" docker-compose.yml | head -5
    echo ""
    echo "环境变量配置:"
    grep -A 5 "environment:" docker-compose.yml | head -10
else
    echo "⚠️  未找到 docker-compose.yml"
fi
echo ""

# 总结
echo "=========================================="
echo "诊断总结"
echo "=========================================="
echo ""

if [ ! -z "$HTTP_SERVER_LOG" ] && [ ! -z "$API_TEST" ]; then
    echo "✅ HTTP 服务器正常运行"
    echo ""
    echo "访问地址:"
    echo "  http://localhost:15000"
    echo ""

    if [ -z "$HOST_TEST" ]; then
        echo "⚠️  主机无法访问，可能是端口映射或防火墙问题"
        echo ""
        echo "尝试以下命令:"
        echo "  1. 检查端口是否被占用:"
        echo "     lsof -i :15000"
        echo ""
        echo "  2. 重启容器:"
        echo "     docker-compose restart"
    fi
else
    echo "❌ HTTP 服务器未正确启动"
    echo ""
    echo "建议操作:"
    echo "  1. 重新构建镜像（确保使用最新代码）:"
    echo "     docker-compose down"
    echo "     docker rmi frigate-config-ui:latest"
    echo "     docker build --no-cache -t frigate-config-ui:latest ."
    echo "     docker-compose up -d"
    echo ""
    echo "  2. 查看完整日志:"
    echo "     docker logs -f $RUNNING"
fi

echo ""
echo "=========================================="
