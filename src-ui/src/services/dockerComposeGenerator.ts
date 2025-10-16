// Docker Compose Generator
// Generates docker-compose.yml file for Frigate deployment
// This is separate from config.yml

export interface DockerComposeConfig {
  volumes: {
    config_path: string;
    storage_path: string;
    cache_size?: number;
  };
  ports: {
    web_port: number;
    rtsp_port: number;
    webrtc_tcp_port: number;
    webrtc_udp_port: number;
  };
  devices: string[];
  environment: Record<string, string>;
  shm_size?: string;
  privileged?: boolean;
  // 新增配置选项
  nvidia_runtime?: boolean; // 使用 NVIDIA 运行时
  nvidia_devices?: string; // NVIDIA_VISIBLE_DEVICES 环境变量
  security_opt?: string[]; // 安全选项
  ipc?: string; // IPC 模式
  working_dir?: string; // 工作目录
  network?: string; // 自定义网络
  deploy_resources?: boolean; // 是否使用 deploy.resources (Compose v3)
}

export class DockerComposeGenerator {
  /**
   * Generate docker-compose.yml content
   * 增强版：支持 NVIDIA GPU、Hailo NPU、安全选项等
   */
  static generateCompose(config: DockerComposeConfig): string {
    const {
      volumes,
      ports,
      devices,
      environment,
      shm_size = '256mb',
      privileged = true,
      nvidia_runtime = false,
      nvidia_devices,
      security_opt,
      ipc,
      working_dir,
      network,
      deploy_resources = false
    } = config;

    // 合并环境变量
    const mergedEnvironment = {
      ...environment
    };

    // 如果有 NVIDIA GPU，添加相关环境变量
    if (nvidia_runtime || nvidia_devices) {
      mergedEnvironment['NVIDIA_VISIBLE_DEVICES'] = nvidia_devices || 'all';
      mergedEnvironment['NVIDIA_DRIVER_CAPABILITIES'] = 'compute,video,utility';
    }

    // 构建基础服务配置
    const serviceConfig: any = {
      container_name: 'frigate',
      privileged,
      restart: 'unless-stopped',
      stop_grace_period: '30s',
      image: 'ghcr.io/blakeblackshear/frigate:stable',
      shm_size,
      devices: devices.length > 0 ? devices : undefined,
      volumes: [
        '/etc/localtime:/etc/localtime:ro',
        `${volumes.config_path}:/config`,
        `${volumes.storage_path}:/media/frigate`,
        {
          type: 'tmpfs',
          target: '/tmp/cache',
          tmpfs: {
            size: volumes.cache_size || 1000000000
          }
        }
      ],
      ports: [
        `${ports.web_port}:8971`,
        `${ports.rtsp_port}:8554`,
        `${ports.webrtc_tcp_port}:8555/tcp`,
        `${ports.webrtc_udp_port}:8555/udp`
      ],
      environment: Object.keys(mergedEnvironment).length > 0 ? mergedEnvironment : undefined
    };

    // 添加可选配置
    if (nvidia_runtime) {
      serviceConfig.runtime = 'nvidia';
    }

    if (security_opt && security_opt.length > 0) {
      serviceConfig.security_opt = security_opt;
    }

    if (ipc) {
      serviceConfig.ipc = ipc;
    }

    if (working_dir) {
      serviceConfig.working_dir = working_dir;
    }

    // 如果使用 deploy.resources (Compose v3)
    if (deploy_resources && nvidia_runtime) {
      serviceConfig.deploy = {
        resources: {
          reservations: {
            devices: [
              {
                driver: 'nvidia',
                count: 1,
                capabilities: ['gpu']
              }
            ]
          }
        }
      };
    }

    const composeConfig: any = {
      services: {
        frigate: serviceConfig
      }
    };

    // 如果有自定义网络配置
    if (network) {
      composeConfig.services.frigate.networks = [network];
      composeConfig.networks = {
        [network]: {
          external: true
        }
      };
    }

    return this.toYAML(composeConfig);
  }

  /**
   * Convert config object to YAML string
   */
  private static toYAML(obj: any, indent: number = 0): string {
    const spaces = '  '.repeat(indent);
    let yaml = '';

    for (const [key, value] of Object.entries(obj)) {
      // Skip null, undefined
      if (value === null || value === undefined) {
        continue;
      }

      // Skip empty objects
      if (typeof value === 'object' && !Array.isArray(value) && Object.keys(value).length === 0) {
        continue;
      }

      if (typeof value === 'object' && !Array.isArray(value)) {
        // Object - recurse
        const childYaml = this.toYAML(value, indent + 1);
        if (childYaml.trim()) {
          yaml += `${spaces}${key}:\n`;
          yaml += childYaml;
        }
      } else if (Array.isArray(value)) {
        // Skip empty arrays
        if (value.length === 0) {
          continue;
        }
        yaml += `${spaces}${key}:\n`;
        value.forEach((item) => {
          if (typeof item === 'object' && item !== null) {
            // Check if it's a volume object with type
            if (item.type === 'tmpfs') {
              yaml += `${spaces}  - type: ${item.type}\n`;
              yaml += `${spaces}    target: ${item.target}\n`;
              if (item.tmpfs) {
                yaml += `${spaces}    tmpfs:\n`;
                yaml += `${spaces}      size: ${item.tmpfs.size}\n`;
              }
            } else {
              // Other objects - inline format
              yaml += `${spaces}  - `;
              const itemEntries = Object.entries(item);
              if (itemEntries.length > 0) {
                const [firstKey, firstValue] = itemEntries[0];
                yaml += `${firstKey}: ${firstValue}\n`;
                for (let i = 1; i < itemEntries.length; i++) {
                  const [k, v] = itemEntries[i];
                  yaml += `${spaces}    ${k}: ${v}\n`;
                }
              }
            }
          } else {
            yaml += `${spaces}  - ${item}\n`;
          }
        });
      } else if (typeof value === 'string') {
        // Quote strings with special characters
        const needsQuotes = /[:#@]/.test(value) || value.includes(' ');
        yaml += `${spaces}${key}: ${needsQuotes ? `"${value}"` : value}\n`;
      } else if (typeof value === 'boolean') {
        yaml += `${spaces}${key}: ${value}\n`;
      } else {
        yaml += `${spaces}${key}: ${value}\n`;
      }
    }

    return yaml;
  }

  /**
   * Get default docker-compose.yml template
   */
  static getDefaultCompose(): string {
    return `# Docker Compose configuration for Frigate
# Generated by Frigate Configuration Tool

services:
  frigate:
    container_name: frigate
    privileged: true
    restart: unless-stopped
    stop_grace_period: 30s
    image: ghcr.io/blakeblackshear/frigate:stable
    shm_size: "256mb"
    devices:
      # Uncomment for Coral USB
      # - /dev/bus/usb:/dev/bus/usb
      # Uncomment for Intel GPU
      # - /dev/dri/renderD128:/dev/dri/renderD128
    volumes:
      - /etc/localtime:/etc/localtime:ro
      - ./config:/config
      - ./storage:/media/frigate
      - type: tmpfs
        target: /tmp/cache
        tmpfs:
          size: 1000000000
    ports:
      - "8971:8971"  # Web UI
      - "8554:8554"  # RTSP
      - "8555:8555/tcp"  # WebRTC
      - "8555:8555/udp"  # WebRTC
    environment:
      FRIGATE_RTSP_PASSWORD: "password"
`;
  }

  /**
   * 获取所有 Frigate 支持的硬件设备映射
   * 完整支持: Coral TPU, Hailo NPU, NVIDIA GPU, Intel GPU, AMD GPU, Rockchip NPU
   */
  static getRecommendedDevices(hardwareType: string): string[] {
    switch (hardwareType.toLowerCase()) {
      // Google Coral EdgeTPU (USB 和 M.2/PCIe)
      case 'coral':
      case 'edgetpu':
      case 'coral-usb':
        return [
          '/dev/bus/usb:/dev/bus/usb',  // USB Coral
          '/dev/apex_0:/dev/apex_0'      // PCIe/M.2 Coral
        ];

      // Hailo NPU 系列
      case 'hailo':
      case 'hailo8':
      case 'hailo8l':
      case 'hailo-8':
        return [
          '/dev/hailo0:/dev/hailo0',
          '/dev/hailo1:/dev/hailo1',  // 支持多个 Hailo 设备
          '/dev/hailo2:/dev/hailo2',
          '/dev/hailo3:/dev/hailo3'
        ].filter(device => {
          // 实际部署时会检查哪些设备存在
          return true;
        });

      // NVIDIA GPU (CUDA/TensorRT)
      case 'nvidia':
      case 'cuda':
      case 'tensorrt':
      case 'nvidia-gpu':
        return [
          '/dev/nvidia0:/dev/nvidia0',
          '/dev/nvidia1:/dev/nvidia1',  // 支持多 GPU
          '/dev/nvidiactl:/dev/nvidiactl',
          '/dev/nvidia-modeset:/dev/nvidia-modeset',
          '/dev/nvidia-uvm:/dev/nvidia-uvm',
          '/dev/nvidia-uvm-tools:/dev/nvidia-uvm-tools',
          '/dev/nvidia-caps:/dev/nvidia-caps'  // NVIDIA capabilities
        ];

      // Intel GPU (OpenVINO/QSV/VAAPI)
      case 'intel':
      case 'intel-gpu':
      case 'openvino':
      case 'qsv':
      case 'vaapi':
        return [
          '/dev/dri:/dev/dri',  // 映射整个 DRI 目录
          '/dev/dri/renderD128:/dev/dri/renderD128',
          '/dev/dri/renderD129:/dev/dri/renderD129',  // 多 GPU 支持
          '/dev/dri/card0:/dev/dri/card0',
          '/dev/dri/card1:/dev/dri/card1'
        ];

      // AMD GPU (ROCm/VAAPI)
      case 'amd':
      case 'amd-gpu':
      case 'rocm':
        return [
          '/dev/dri:/dev/dri',
          '/dev/kfd:/dev/kfd',  // AMD KFD (Kernel Fusion Driver)
          '/dev/dri/renderD128:/dev/dri/renderD128',
          '/dev/dri/card0:/dev/dri/card0'
        ];

      // Rockchip NPU (RKNN)
      case 'rockchip':
      case 'rknn':
      case 'rk3588':
      case 'rk3568':
        return [
          '/dev/dri:/dev/dri',
          '/dev/rga:/dev/rga',  // Rockchip RGA
          '/dev/mpp_service:/dev/mpp_service',  // Media Process Platform
          '/dev/mali0:/dev/mali0',  // Mali GPU
          '/dev/npu:/dev/npu'  // NPU 设备
        ];

      // Jetson 系列 (NVIDIA Jetson Nano/Xavier/Orin)
      case 'jetson':
      case 'jetson-nano':
      case 'jetson-xavier':
      case 'jetson-orin':
        return [
          '/dev/nvhost-ctrl:/dev/nvhost-ctrl',
          '/dev/nvhost-ctrl-gpu:/dev/nvhost-ctrl-gpu',
          '/dev/nvhost-prof-gpu:/dev/nvhost-prof-gpu',
          '/dev/nvmap:/dev/nvmap',
          '/dev/nvhost-gpu:/dev/nvhost-gpu',
          '/dev/nvhost-as-gpu:/dev/nvhost-as-gpu',
          '/dev/tegra_dc_ctrl:/dev/tegra_dc_ctrl'
        ];

      // Apple Neural Engine (实验性支持)
      case 'apple':
      case 'ane':
      case 'neural-engine':
        return [
          '/dev/ane0:/dev/ane0'
        ];

      // Qualcomm NPU (实验性支持)
      case 'qualcomm':
      case 'snapdragon':
        return [
          '/dev/kgsl-3d0:/dev/kgsl-3d0',  // Adreno GPU
          '/dev/ion:/dev/ion'
        ];

      // 通用 V4L2 设备（用于摄像头直接访问）
      case 'v4l2':
      case 'video':
        return [
          '/dev/video0:/dev/video0',
          '/dev/video1:/dev/video1',
          '/dev/video2:/dev/video2',
          '/dev/video3:/dev/video3'
        ];

      // ONNX (可以在 CPU 或 GPU 上运行)
      case 'onnx':
        // ONNX 通常不需要特殊设备，但如果有 GPU 则添加
        return [];

      // CPU 检测器（不推荐，但支持）
      case 'cpu':
      default:
        return [];
    }
  }

  /**
   * 获取 Frigate 推荐的环境变量
   * 完整支持所有硬件加速器的特定环境变量
   */
  static getRecommendedEnvironment(hardwareType?: string): Record<string, string> {
    const baseEnv: Record<string, string> = {
      // Frigate 核心环境变量
      'FRIGATE_RTSP_PASSWORD': 'password',
      'S6_LOGGING_SCRIPT': '## import json\nimport socket\nimport sys\nfields = {}\nfields["service"] = sys.argv[1]\nfields["source"] = "stderr"\nfields["level"] = "info"\nif len(sys.argv) > 2:\n    fields[\'level\'] = sys.argv[2]\nline = sys.stdin.readline()\nwhile line:\n    msg = json.dumps({"message": line.rstrip(), "host": socket.gethostname(), **fields})\n    print(msg, flush=True)\n    line = sys.stdin.readline()',
      'DEFAULT_FFMPEG_VERSION': '7',
      'PYTHONDONTWRITEBYTECODE': '1',
      'TZ': 'UTC'  // 时区设置
    };

    // 根据硬件类型添加特定环境变量
    if (hardwareType) {
      switch (hardwareType.toLowerCase()) {
        // Hailo NPU
        case 'hailo':
        case 'hailo8':
        case 'hailo8l':
        case 'hailo-8':
          baseEnv['HAILORT_LOGGER_PATH'] = '/config/logs';
          baseEnv['HAILO_MONITOR'] = '1';  // 启用 Hailo 监控
          baseEnv['HAILO_PROFILER'] = '0';  // 默认关闭性能分析
          break;

        // NVIDIA GPU (在 generateFullConfig 中额外处理)
        case 'nvidia':
        case 'cuda':
        case 'tensorrt':
        case 'nvidia-gpu':
          baseEnv['CUDA_MODULE_LOADING'] = 'LAZY';  // 优化 CUDA 加载
          baseEnv['NVIDIA_TF32_OVERRIDE'] = '0';  // TF32 精度控制
          break;

        // Google Coral EdgeTPU
        case 'coral':
        case 'edgetpu':
        case 'coral-usb':
          baseEnv['LIBEDGETPU_RELEASE'] = 'frogfish';  // EdgeTPU 运行时版本
          baseEnv['USB_CORAL_RESTART_ON_ERROR'] = '1';  // USB Coral 错误自动重启
          break;

        // Intel GPU/OpenVINO
        case 'intel':
        case 'intel-gpu':
        case 'openvino':
        case 'qsv':
        case 'vaapi':
          baseEnv['LIBVA_DRIVER_NAME'] = 'iHD';  // Intel Media Driver
          baseEnv['INTEL_OPENVINO_DIR'] = '/opt/intel/openvino';
          baseEnv['OPENVINO_CACHE_DIR'] = '/config/openvino_cache';
          baseEnv['INTEL_COMPUTE_RUNTIME'] = '1';
          break;

        // AMD GPU
        case 'amd':
        case 'amd-gpu':
        case 'rocm':
          baseEnv['HSA_OVERRIDE_GFX_VERSION'] = '10.3.0';  // ROCm 版本
          baseEnv['ROCR_VISIBLE_DEVICES'] = 'all';
          baseEnv['AMD_LOG_LEVEL'] = '3';
          break;

        // Rockchip NPU
        case 'rockchip':
        case 'rknn':
        case 'rk3588':
        case 'rk3568':
          baseEnv['RKNN_RUNTIME_DIR'] = '/usr/lib';
          baseEnv['RKNN_SERVER_LOGLEVEL'] = '3';
          baseEnv['RKNN_TENSORRT_ENABLE'] = '0';
          baseEnv['MALI_OVERRIDE_PLATFORM'] = 'rk3588';
          break;

        // NVIDIA Jetson
        case 'jetson':
        case 'jetson-nano':
        case 'jetson-xavier':
        case 'jetson-orin':
          baseEnv['JETSON_PLATFORM'] = '1';
          baseEnv['CUDA_ARCH_BIN'] = '5.3,6.2,7.2,8.7';  // Jetson CUDA 架构
          baseEnv['CUDNN_VERSION'] = '8.6.0';
          baseEnv['TENSORRT_VERSION'] = '8.5.2';
          break;

        // Qualcomm NPU
        case 'qualcomm':
        case 'snapdragon':
          baseEnv['SNPE_ROOT'] = '/opt/qualcomm/snpe';
          baseEnv['ADSP_LIBRARY_PATH'] = '/system/lib/rfsa/adsp';
          break;

        // Apple Neural Engine
        case 'apple':
        case 'ane':
        case 'neural-engine':
          baseEnv['COREML_USE_NEURAL_ENGINE'] = '1';
          baseEnv['METAL_DEVICE_WRAPPER_TYPE'] = '1';
          break;

        // ONNX Runtime
        case 'onnx':
          baseEnv['ORT_TENSORRT_ENGINE_CACHE_ENABLE'] = '1';
          baseEnv['ORT_TENSORRT_CACHE_PATH'] = '/config/onnx_cache';
          baseEnv['OMP_NUM_THREADS'] = '4';  // OpenMP 线程数
          break;

        // V4L2 直接访问
        case 'v4l2':
        case 'video':
          baseEnv['V4L2_DISABLE_CONVERSION'] = '1';
          baseEnv['V4L2_FORCE_FORMAT'] = 'MJPEG';
          break;

        // CPU 检测器
        case 'cpu':
          baseEnv['OMP_NUM_THREADS'] = '4';
          baseEnv['MKL_NUM_THREADS'] = '4';
          baseEnv['NUMEXPR_NUM_THREADS'] = '4';
          break;
      }
    }

    return baseEnv;
  }

  /**
   * 获取推荐的安全选项
   */
  static getRecommendedSecurityOpt(): string[] {
    return [
      'apparmor=unconfined'
    ];
  }

  /**
   * 生成完整的 Docker Compose 配置
   * 包含所有推荐设置
   */
  static generateFullConfig(
    hardwareDevices: string[],
    volumePaths: { config: string; storage: string },
    hardwareType?: string
  ): DockerComposeConfig {
    // 获取基础环境变量
    const baseEnvironment = this.getRecommendedEnvironment(hardwareType);

    // 如果没有传入设备列表，但指定了硬件类型，获取推荐的设备
    let finalDevices = hardwareDevices;
    if (hardwareType && hardwareDevices.length === 0) {
      finalDevices = this.getRecommendedDevices(hardwareType);
    }

    const config: DockerComposeConfig = {
      volumes: {
        config_path: volumePaths.config,
        storage_path: volumePaths.storage,
        cache_size: 1000000000
      },
      ports: {
        web_port: 8971,
        rtsp_port: 8554,
        webrtc_tcp_port: 8555,
        webrtc_udp_port: 8555
      },
      devices: finalDevices,
      environment: baseEnvironment,
      shm_size: '256mb',
      privileged: true,
      security_opt: this.getRecommendedSecurityOpt(),
      ipc: 'private',
      working_dir: '/opt/frigate'
    };

    // 处理特定硬件的额外配置
    if (hardwareType) {
      const hwType = hardwareType.toLowerCase();

      // NVIDIA GPU 特殊处理
      if (['nvidia', 'cuda', 'tensorrt', 'nvidia-gpu'].includes(hwType)) {
        config.nvidia_runtime = true;
        config.nvidia_devices = 'all';
        // 添加 NVIDIA 特定环境变量
        config.environment['NVIDIA_VISIBLE_DEVICES'] = 'all';
        config.environment['NVIDIA_DRIVER_CAPABILITIES'] = 'compute,video,utility';
      }

      // Jetson 平台特殊处理
      if (hwType.includes('jetson')) {
        config.nvidia_runtime = true;
        // Jetson 使用不同的环境变量
        config.environment['JETPACK_VERSION'] = '5.1';
      }

      // AMD GPU 特殊处理
      if (['amd', 'amd-gpu', 'rocm'].includes(hwType)) {
        // AMD 需要特殊的 group_add
        config.environment['ROCM_VERSION'] = '5.7.0';
      }

      // Intel GPU 特殊处理
      if (['intel', 'intel-gpu', 'openvino', 'qsv'].includes(hwType)) {
        // Intel 可能需要额外的权限
        config.environment['NEOReadDebugKeys'] = '1';
      }
    }

    return config;
  }
}
