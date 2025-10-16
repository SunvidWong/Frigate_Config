# Frigate 硬件加速支持文档

## 概述

本项目完整支持 Frigate NVR 的所有硬件加速设备，包括 NPU、TPU、GPU 等。每种硬件都有对应的设备映射和环境变量配置。

## 支持的硬件加速器

### 1. NPU/TPU 设备

#### Hailo NPU
- **型号**: Hailo-8, Hailo-8L
- **设备映射**:
  ```yaml
  devices:
    - /dev/hailo0:/dev/hailo0
    - /dev/hailo1:/dev/hailo1  # 支持多设备
  ```
- **环境变量**:
  ```yaml
  HAILORT_LOGGER_PATH: /config/logs
  HAILO_MONITOR: '1'
  HAILO_PROFILER: '0'
  ```

#### Google Coral EdgeTPU
- **型号**: USB Coral, M.2/PCIe Coral
- **设备映射**:
  ```yaml
  devices:
    - /dev/bus/usb:/dev/bus/usb  # USB Coral
    - /dev/apex_0:/dev/apex_0    # PCIe/M.2 Coral
  ```
- **环境变量**:
  ```yaml
  LIBEDGETPU_RELEASE: frogfish
  USB_CORAL_RESTART_ON_ERROR: '1'
  ```

#### Rockchip NPU (RKNN)
- **型号**: RK3588, RK3568
- **设备映射**:
  ```yaml
  devices:
    - /dev/dri:/dev/dri
    - /dev/npu:/dev/npu
    - /dev/rga:/dev/rga
    - /dev/mpp_service:/dev/mpp_service
    - /dev/mali0:/dev/mali0
  ```
- **环境变量**:
  ```yaml
  RKNN_RUNTIME_DIR: /usr/lib
  RKNN_SERVER_LOGLEVEL: '3'
  MALI_OVERRIDE_PLATFORM: rk3588
  ```

#### Qualcomm NPU
- **型号**: Snapdragon 系列
- **设备映射**:
  ```yaml
  devices:
    - /dev/kgsl-3d0:/dev/kgsl-3d0
    - /dev/ion:/dev/ion
  ```
- **环境变量**:
  ```yaml
  SNPE_ROOT: /opt/qualcomm/snpe
  ADSP_LIBRARY_PATH: /system/lib/rfsa/adsp
  ```

### 2. GPU 设备

#### NVIDIA GPU
- **型号**: RTX/GTX 系列, Tesla 系列
- **设备映射**:
  ```yaml
  devices:
    - /dev/nvidia0:/dev/nvidia0
    - /dev/nvidia1:/dev/nvidia1  # 多 GPU 支持
    - /dev/nvidiactl:/dev/nvidiactl
    - /dev/nvidia-modeset:/dev/nvidia-modeset
    - /dev/nvidia-uvm:/dev/nvidia-uvm
    - /dev/nvidia-uvm-tools:/dev/nvidia-uvm-tools
  ```
- **特殊配置**:
  ```yaml
  runtime: nvidia
  ```
- **环境变量**:
  ```yaml
  NVIDIA_VISIBLE_DEVICES: all
  NVIDIA_DRIVER_CAPABILITIES: compute,video,utility
  CUDA_MODULE_LOADING: LAZY
  NVIDIA_TF32_OVERRIDE: '0'
  ```

#### Intel GPU
- **型号**: Arc 系列, UHD Graphics, Iris Xe
- **设备映射**:
  ```yaml
  devices:
    - /dev/dri:/dev/dri
    - /dev/dri/renderD128:/dev/dri/renderD128
    - /dev/dri/card0:/dev/dri/card0
  ```
- **环境变量**:
  ```yaml
  LIBVA_DRIVER_NAME: iHD
  INTEL_OPENVINO_DIR: /opt/intel/openvino
  OPENVINO_CACHE_DIR: /config/openvino_cache
  INTEL_COMPUTE_RUNTIME: '1'
  NEOReadDebugKeys: '1'
  ```

#### AMD GPU
- **型号**: Radeon RX 系列
- **设备映射**:
  ```yaml
  devices:
    - /dev/dri:/dev/dri
    - /dev/kfd:/dev/kfd
    - /dev/dri/renderD128:/dev/dri/renderD128
  ```
- **环境变量**:
  ```yaml
  HSA_OVERRIDE_GFX_VERSION: 10.3.0
  ROCR_VISIBLE_DEVICES: all
  ROCM_VERSION: 5.7.0
  AMD_LOG_LEVEL: '3'
  ```

### 3. 边缘计算平台

#### NVIDIA Jetson
- **型号**: Jetson Nano, Xavier, Orin
- **设备映射**:
  ```yaml
  devices:
    - /dev/nvhost-ctrl:/dev/nvhost-ctrl
    - /dev/nvhost-gpu:/dev/nvhost-gpu
    - /dev/nvmap:/dev/nvmap
    - /dev/tegra_dc_ctrl:/dev/tegra_dc_ctrl
  ```
- **特殊配置**:
  ```yaml
  runtime: nvidia
  ```
- **环境变量**:
  ```yaml
  JETSON_PLATFORM: '1'
  JETPACK_VERSION: '5.1'
  CUDA_ARCH_BIN: 5.3,6.2,7.2,8.7
  CUDNN_VERSION: 8.6.0
  TENSORRT_VERSION: 8.5.2
  ```

### 4. 其他检测器

#### ONNX Runtime
- **设备映射**: 无需特殊设备
- **环境变量**:
  ```yaml
  ORT_TENSORRT_ENGINE_CACHE_ENABLE: '1'
  ORT_TENSORRT_CACHE_PATH: /config/onnx_cache
  OMP_NUM_THREADS: '4'
  ```

#### CPU 检测器
- **设备映射**: 无需特殊设备
- **环境变量**:
  ```yaml
  OMP_NUM_THREADS: '4'
  MKL_NUM_THREADS: '4'
  NUMEXPR_NUM_THREADS: '4'
  ```

## 通用配置

所有配置都包含以下基础环境变量：

```yaml
environment:
  FRIGATE_RTSP_PASSWORD: password
  DEFAULT_FFMPEG_VERSION: '7'
  PYTHONDONTWRITEBYTECODE: '1'
  TZ: UTC
  S6_LOGGING_SCRIPT: |
    ## import json
    import socket
    import sys
    # ... (日志脚本)
```

安全选项：
```yaml
security_opt:
  - apparmor=unconfined
ipc: private
working_dir: /opt/frigate
privileged: true
shm_size: 256mb
```

## 使用示例

### 单硬件配置

```typescript
// Hailo NPU 配置
const config = DockerComposeGenerator.generateFullConfig(
  [],  // 自动添加推荐设备
  { config: './config', storage: './storage' },
  'hailo'
);
```

### 多硬件混合配置

```typescript
// Hailo + Coral 混合使用
const devices = [
  '/dev/hailo0:/dev/hailo0',
  '/dev/bus/usb:/dev/bus/usb'
];

const config = DockerComposeGenerator.generateFullConfig(
  devices,
  { config: './config', storage: './storage' },
  'hailo'  // 主检测器
);
```

### 自动检测硬件类型

部署页面会自动检测硬件类型：

```typescript
const hasCuda = devices.some(d => d.includes('nvidia'));
const hasHailo = devices.some(d => d.includes('hailo'));
const hasIntel = devices.some(d => d.includes('dri'));
const hasCoral = devices.some(d => d.includes('apex') || d.includes('usb'));
```

## 注意事项

1. **设备映射**: 某些设备可能不存在，部署时会自动跳过不存在的设备
2. **运行时**: NVIDIA 和 Jetson 需要 `nvidia` 运行时
3. **权限**: 大部分硬件加速需要 `privileged: true`
4. **多设备**: 支持同类型多个设备（如多 GPU、多 Hailo）
5. **环境变量优先级**: 特定硬件的环境变量会覆盖基础配置

## 性能建议

1. **Hailo**: 最适合边缘计算，低功耗高性能
2. **Coral**: USB 版本易用，PCIe 版本性能更好
3. **NVIDIA**: 适合高性能需求，支持 TensorRT 加速
4. **Intel**: OpenVINO 在 CPU 模式下也有不错性能
5. **Rockchip**: RK3588 内置 NPU 性价比高

## 故障排除

### Hailo 检测不到
- 检查 `/dev/hailo0` 设备是否存在
- 确认驱动已安装：`ls /dev/hailo*`
- 检查 PCIe 设备：`lspci | grep 1e60`

### NVIDIA GPU 不工作
- 确认 nvidia-docker 运行时已安装
- 检查驱动版本 >= 545
- 验证 CUDA 支持：`nvidia-smi`

### Coral 性能问题
- USB Coral 可能受 USB 带宽限制
- 使用 PCIe/M.2 版本获得更好性能
- 检查 EdgeTPU 运行时版本

## 更新历史

- 2025-10-16: 完整支持所有 Frigate 硬件加速器
- 支持 Hailo, Coral, NVIDIA, Intel, AMD, Rockchip, Jetson 等
- 添加完整的设备映射和环境变量配置
- 支持多设备和混合配置