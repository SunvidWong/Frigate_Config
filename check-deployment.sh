#!/bin/bash

echo "🔍 Frigate Docker 部署状态检查"
echo "================================"
echo ""

# 检查 Docker
echo "📦 Docker 状态:"
if docker info &> /dev/null; then
    echo "   ✅ Docker Desktop 运行中"
else
    echo "   ❌ Docker Desktop 未运行"
    exit 1
fi
echo ""

# 检查容器
echo "🐳 容器状态:"
docker ps --filter "name=frigate" --format "   {{.Names}}: {{.Status}}" | while read line; do
    if [[ $line == *"Up"* ]]; then
        echo "   ✅ $line"
    else
        echo "   ❌ $line"
    fi
done
echo ""

# 检查端口
echo "🌐 端口监听:"
if lsof -i :5002 &> /dev/null; then
    echo "   ✅ 5002 (Frigate Web UI) - 正在监听"
else
    echo "   ❌ 5002 (Frigate Web UI) - 未监听"
fi

if lsof -i :8554 &> /dev/null; then
    echo "   ✅ 8554 (RTSP) - 正在监听"
else
    echo "   ⚠️  8554 (RTSP) - 未监听 (正常,没有摄像头时)"
fi

if lsof -i :1883 &> /dev/null; then
    echo "   ✅ 1883 (MQTT) - 正在监听"
else
    echo "   ❌ 1883 (MQTT) - 未监听"
fi
echo ""

# 检查卷映射
echo "📁 存储路径:"
if [ -d "$HOME/frigate/config" ]; then
    echo "   ✅ ~/frigate/config 存在"
    if [ -f "$HOME/frigate/config/config.yml" ]; then
        echo "   ✅ config.yml 存在"
    else
        echo "   ❌ config.yml 不存在"
    fi
else
    echo "   ❌ ~/frigate/config 不存在"
fi

if [ -d "$HOME/frigate/recordings" ]; then
    echo "   ✅ ~/frigate/recordings 存在"
else
    echo "   ❌ ~/frigate/recordings 不存在"
fi

if [ -d "$HOME/frigate/clips" ]; then
    echo "   ✅ ~/frigate/clips 存在"
else
    echo "   ❌ ~/frigate/clips 不存在"
fi

if [ -d "$HOME/frigate/cache" ]; then
    echo "   ✅ ~/frigate/cache 存在"
else
    echo "   ❌ ~/frigate/cache 不存在"
fi
echo ""

# 检查 Frigate 健康状态
echo "💓 Frigate 健康检查:"
HEALTH=$(docker inspect --format='{{.State.Health.Status}}' frigate 2>/dev/null)
if [ "$HEALTH" == "healthy" ]; then
    echo "   ✅ Frigate 健康状态: $HEALTH"
elif [ "$HEALTH" == "starting" ]; then
    echo "   ⏳ Frigate 健康状态: $HEALTH (启动中)"
else
    echo "   ❌ Frigate 健康状态: ${HEALTH:-未知}"
fi
echo ""

# 访问链接
echo "🔗 访问链接:"
echo "   Frigate Web UI:  http://localhost:5002"
echo "   Config Tool:     http://localhost:1420"
echo "   MQTT:            localhost:1883"
echo ""

# 默认凭据
echo "🔐 Frigate 默认登录:"
echo "   用户名: admin"
PASS=$(docker logs frigate 2>&1 | grep "Password:" | tail -1 | awk '{print $NF}')
if [ -n "$PASS" ]; then
    echo "   密码: $PASS"
else
    echo "   密码: (请查看 Frigate 日志: docker logs frigate)"
fi
echo ""

# 快速命令
echo "⚡ 快速命令:"
echo "   查看日志:    docker logs frigate --tail 50 --follow"
echo "   重启服务:    docker restart frigate"
echo "   停止所有:    docker-compose -f docker-compose-standalone.yml down"
echo "   启动所有:    docker-compose -f docker-compose-standalone.yml up -d"
echo ""

echo "✅ 检查完成！"
