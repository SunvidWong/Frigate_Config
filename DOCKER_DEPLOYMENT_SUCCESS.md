# 🎉 Frigate Docker 部署成功报告

**部署时间**: 2025-10-10
**状态**: ✅ 成功部署到 Docker Desktop
**环境**: macOS + Docker Desktop

---

## ✅ 部署状态

### 运行中的容器

```
CONTAINER       STATUS              PORTS
frigate         Up (healthy)        0.0.0.0:5002->5000/tcp (Web UI)
                                    0.0.0.0:8554->8554/tcp (RTSP)
                                    0.0.0.0:8555->8555/tcp (WebRTC)
frigate-redis   Up                  6379/tcp
frigate-mqtt    Up                  0.0.0.0:1883->1883/tcp (MQTT)
                                    0.0.0.0:9001->9001/tcp (MQTT WebSocket)
```

**健康检查**: ✅ Frigate 容器健康状态良好

---

## 🔐 默认登录凭据

Frigate 已自动创建默认管理员账户:

- **用户名**: `admin`
- **密码**: `c3b7b301e06ae555bf1f8cd772adef47`

⚠️ **重要**: 首次登录后请立即修改密码！

---

## 🌐 访问地址

### Frigate Web UI (NVR 控制台)
- **URL**: http://localhost:5002
- **功能**:
  - 查看实时视频流
  - 查看录像和剪辑
  - 管理摄像头配置
  - 查看事件和对象检测

### Frigate Config Tool (配置工具)
- **URL**: http://localhost:1420
- **运行方式**: 主机上的 Tauri 应用 (不在 Docker 中)
- **启动命令**: `./target/release/frigate-config-tool`
- **功能**:
  - 🔍 摄像头发现 - 扫描内网 IP 摄像头
  - 📷 相机配置 - 可视化配置摄像头
  - 💾 磁盘映射 - 配置存储路径
  - 🚀 部署管理 - 部署和监控 Frigate

---

## 📁 卷映射路径

所有数据存储在主机路径:

```
~/frigate/
├── config/          -> /config (容器内)
│   └── config.yml   (Frigate 主配置文件)
├── recordings/      -> /media/frigate/recordings
├── clips/           -> /media/frigate/clips
└── cache/           -> /tmp/cache
```

**配置文件**: `/Users/sunvid/frigate/config/config.yml`

---

## 🎯 下一步操作

### 1. 登录 Frigate Web UI

```bash
# 打开浏览器访问:
open http://localhost:5002

# 使用默认凭据登录:
用户名: admin
密码: c3b7b301e06ae555bf1f8cd772adef47
```

### 2. 使用 Config Tool 添加摄像头

```bash
# 启动配置工具 (如果未运行)
./target/release/frigate-config-tool

# 打开浏览器访问:
open http://localhost:1420
```

**推荐工作流程**:

1. **扫描摄像头**:
   - 导航到 "摄像头发现" (🔍)
   - 点击 "快速扫描"
   - 等待扫描完成 (30-60秒)
   - 复制发现的 RTSP URL

2. **配置摄像头**:
   - 导航到 "相机配置" (📷)
   - 添加新摄像头
   - 粘贴 RTSP URL
   - 配置认证信息 (用户名/密码)
   - 设置检测区域和录像选项

3. **部署配置**:
   - 导航到 "部署" (🚀)
   - 点击 "部署配置"
   - 等待部署完成
   - 查看健康检查状态

4. **查看实时流**:
   - 返回 Frigate Web UI (http://localhost:5002)
   - 在主页查看所有摄像头
   - 查看实时检测和事件

### 3. 修改默认密码

1. 登录 Frigate Web UI
2. 点击右上角用户图标
3. 选择 "Settings"
4. 修改密码
5. 保存并重新登录

---

## 🔧 管理命令

### 查看容器状态
```bash
docker ps --filter "name=frigate"
```

### 查看 Frigate 日志
```bash
docker logs frigate --tail 50 --follow
```

### 停止所有服务
```bash
cd /Users/sunvid/Documents/GitHub/claude/frigate-config
docker-compose -f docker-compose-standalone.yml down
```

### 启动所有服务
```bash
cd /Users/sunvid/Documents/GitHub/claude/frigate-config
docker-compose -f docker-compose-standalone.yml up -d
```

### 重启 Frigate
```bash
docker restart frigate
```

### 查看资源占用
```bash
docker stats frigate frigate-redis frigate-mqtt
```

---

## 📊 配置说明

### 当前 Frigate 配置

**检测器**: CPU (软件检测)
- 类型: CPU-based
- 线程数: 3

**对象追踪**:
- person (人)
- car (汽车)
- dog (狗)
- cat (猫)

**录像保留**:
- 运动事件: 7 天
- 检测事件: 14 天
- 快照: 14 天

**MQTT**: 已禁用 (可选)
- 如需 Home Assistant 集成,可启用 MQTT

### 示例摄像头配置

编辑 `/Users/sunvid/frigate/config/config.yml`:

```yaml
cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://username:password@192.168.1.100:554/stream1
          roles:
            - detect
            - record
    detect:
      width: 1920
      height: 1080
      fps: 5
    record:
      enabled: true
      retain:
        days: 7
        mode: motion
    snapshots:
      enabled: true
```

修改后重启 Frigate:
```bash
docker restart frigate
```

---

## 🎨 Docker Desktop 中的显示

您现在应该能在 Docker Desktop 中看到:

### Containers 标签页
- ✅ frigate (容器名)
- ✅ frigate-redis (容器名)
- ✅ frigate-mqtt (容器名)

### Networks 标签页
- ✅ frigate-network (网络名)

### Volumes 标签页
- ✅ frigate-config_mosquitto-data
- ✅ frigate-config_mosquitto-log

**点击容器可以查看**:
- 日志 (Logs)
- 检查 (Inspect)
- 终端 (Terminal)
- 统计 (Stats)

---

## 🚨 常见问题

### Q1: 端口 5000 被占用怎么办?

**A**: 已解决！我们使用了端口 5002 来避免 macOS ControlCenter 的冲突。

### Q2: 如何添加真实摄像头?

**A**:
1. 使用 Config Tool 的 "摄像头发现" 功能扫描内网
2. 复制发现的 RTSP URL
3. 在 "相机配置" 中添加摄像头
4. 或直接编辑 `~/frigate/config/config.yml`

### Q3: 如何启用硬件加速?

**A**:
1. 检测可用硬件: 在 Config Tool 的 "硬件检测" 页面
2. 编辑配置文件添加硬件加速:
```yaml
detectors:
  coral:
    type: edgetpu
    device: usb
```
3. 或使用 GPU:
```yaml
ffmpeg:
  hwaccel_args:
    - -hwaccel
    - videotoolbox  # macOS
```

### Q4: Frigate 启动失败?

**A**: 检查日志:
```bash
docker logs frigate
```

常见原因:
- 配置文件语法错误
- RTSP URL 无法访问
- 权限问题

### Q5: 如何备份配置?

**A**:
```bash
# 备份配置文件
cp ~/frigate/config/config.yml ~/frigate/config/config.yml.backup

# 备份整个配置目录
tar -czf frigate-backup-$(date +%Y%m%d).tar.gz ~/frigate/config/
```

---

## 📈 性能优化建议

### 1. 增加共享内存 (如有多个摄像头)
编辑 `docker-compose-standalone.yml`:
```yaml
shm_size: "512mb"  # 原为 256mb
```

### 2. 使用硬件加速
- macOS: VideoToolbox
- NVIDIA GPU: CUDA
- Intel GPU: VAAPI
- Google Coral: EdgeTPU

### 3. 调整检测 FPS
```yaml
detect:
  fps: 5  # 降低以减少 CPU 使用
```

### 4. 配置运动遮罩
```yaml
motion:
  mask:
    - 0,0,100,0,100,100,0,100  # 排除不需要检测的区域
```

---

## 🔮 后续增强

### 计划功能
- [ ] 集成 Config Tool 的 "部署" 功能直接重启 Frigate
- [ ] 从 Config Tool 实时查看 Frigate 日志
- [ ] 一键应用硬件加速配置
- [ ] 自动配置文件验证
- [ ] 配置模板库

### Home Assistant 集成
1. 启用 MQTT (已部署 mosquitto)
2. 配置 Frigate 集成:
```yaml
mqtt:
  enabled: true
  host: frigate-mqtt
  port: 1883
```
3. 在 Home Assistant 中添加 Frigate 集成

---

## ✅ 部署清单

- ✅ Docker Desktop 运行中
- ✅ 创建存储目录 (~/frigate/{config,recordings,clips,cache})
- ✅ 创建 .env.docker 配置文件
- ✅ 创建 Frigate 配置文件 (config.yml)
- ✅ 部署 Frigate 容器 (端口 5002)
- ✅ 部署 Redis 容器
- ✅ 部署 MQTT 容器 (端口 1883, 9001)
- ✅ 创建 frigate-network 网络
- ✅ Frigate 健康检查通过
- ✅ 默认管理员账户创建

---

## 📞 快速访问链接

- **Frigate Web UI**: http://localhost:5002
- **Config Tool**: http://localhost:1420
- **MQTT Broker**: localhost:1883
- **MQTT WebSocket**: ws://localhost:9001

---

## 🎓 学习资源

- **Frigate 官方文档**: https://docs.frigate.video
- **摄像头配置指南**: https://docs.frigate.video/configuration/cameras
- **对象检测配置**: https://docs.frigate.video/configuration/objects
- **硬件加速**: https://docs.frigate.video/configuration/hardware_acceleration

---

**部署完成！** 🎉

您现在可以:
1. 在 Docker Desktop 中看到 3 个运行中的容器
2. 访问 Frigate Web UI 查看监控界面
3. 使用 Config Tool 的摄像头发现功能添加摄像头
4. 开始使用 Frigate NVR 进行视频监控

**下一步**: 扫描并添加您的第一个摄像头！

---

**作者**: Claude Code
**日期**: 2025-10-10
**版本**: Phase 8.5 (含摄像头发现功能)
