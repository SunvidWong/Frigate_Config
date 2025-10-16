// 完整测试所有 Frigate 支持的硬件加速器
import { DockerComposeGenerator } from '../dockerComposeGenerator';

describe('DockerComposeGenerator - 完整硬件支持测试', () => {

  // 测试所有 NPU/TPU 设备
  describe('NPU/TPU 硬件加速器', () => {

    test('Hailo NPU 配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('hailo');
      expect(devices).toContain('/dev/hailo0:/dev/hailo0');

      const env = DockerComposeGenerator.getRecommendedEnvironment('hailo');
      expect(env['HAILORT_LOGGER_PATH']).toBe('/config/logs');
      expect(env['HAILO_MONITOR']).toBe('1');
    });

    test('Google Coral EdgeTPU 配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('coral');
      expect(devices).toContain('/dev/bus/usb:/dev/bus/usb');
      expect(devices).toContain('/dev/apex_0:/dev/apex_0');

      const env = DockerComposeGenerator.getRecommendedEnvironment('coral');
      expect(env['LIBEDGETPU_RELEASE']).toBe('frogfish');
      expect(env['USB_CORAL_RESTART_ON_ERROR']).toBe('1');
    });

    test('Rockchip NPU (RKNN) 配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('rknn');
      expect(devices).toContain('/dev/dri:/dev/dri');
      expect(devices).toContain('/dev/npu:/dev/npu');
      expect(devices).toContain('/dev/rga:/dev/rga');

      const env = DockerComposeGenerator.getRecommendedEnvironment('rknn');
      expect(env['RKNN_RUNTIME_DIR']).toBe('/usr/lib');
      expect(env['RKNN_SERVER_LOGLEVEL']).toBe('3');
    });

    test('Qualcomm NPU 配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('qualcomm');
      expect(devices).toContain('/dev/kgsl-3d0:/dev/kgsl-3d0');

      const env = DockerComposeGenerator.getRecommendedEnvironment('qualcomm');
      expect(env['SNPE_ROOT']).toBe('/opt/qualcomm/snpe');
    });
  });

  // 测试所有 GPU 设备
  describe('GPU 硬件加速器', () => {

    test('NVIDIA GPU 配置', () => {
      const config = DockerComposeGenerator.generateFullConfig(
        [],
        { config: './config', storage: './storage' },
        'nvidia'
      );

      expect(config.nvidia_runtime).toBe(true);
      expect(config.devices).toContain('/dev/nvidia0:/dev/nvidia0');
      expect(config.devices).toContain('/dev/nvidiactl:/dev/nvidiactl');
      expect(config.environment['NVIDIA_VISIBLE_DEVICES']).toBe('all');
      expect(config.environment['CUDA_MODULE_LOADING']).toBe('LAZY');
    });

    test('Intel GPU (OpenVINO) 配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('intel');
      expect(devices).toContain('/dev/dri:/dev/dri');
      expect(devices).toContain('/dev/dri/renderD128:/dev/dri/renderD128');

      const env = DockerComposeGenerator.getRecommendedEnvironment('intel');
      expect(env['LIBVA_DRIVER_NAME']).toBe('iHD');
      expect(env['INTEL_OPENVINO_DIR']).toBe('/opt/intel/openvino');
      expect(env['OPENVINO_CACHE_DIR']).toBe('/config/openvino_cache');
    });

    test('AMD GPU (ROCm) 配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('amd');
      expect(devices).toContain('/dev/dri:/dev/dri');
      expect(devices).toContain('/dev/kfd:/dev/kfd');

      const env = DockerComposeGenerator.getRecommendedEnvironment('amd');
      expect(env['HSA_OVERRIDE_GFX_VERSION']).toBe('10.3.0');
      expect(env['ROCR_VISIBLE_DEVICES']).toBe('all');
    });
  });

  // 测试边缘设备
  describe('边缘计算平台', () => {

    test('NVIDIA Jetson 配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('jetson');
      expect(devices).toContain('/dev/nvhost-ctrl:/dev/nvhost-ctrl');
      expect(devices).toContain('/dev/nvhost-gpu:/dev/nvhost-gpu');

      const config = DockerComposeGenerator.generateFullConfig(
        [],
        { config: './config', storage: './storage' },
        'jetson'
      );
      expect(config.nvidia_runtime).toBe(true);
      expect(config.environment['JETPACK_VERSION']).toBe('5.1');

      const env = DockerComposeGenerator.getRecommendedEnvironment('jetson');
      expect(env['JETSON_PLATFORM']).toBe('1');
      expect(env['CUDA_ARCH_BIN']).toBe('5.3,6.2,7.2,8.7');
    });

    test('Rockchip RK3588 配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('rk3588');
      expect(devices).toContain('/dev/mali0:/dev/mali0');
      expect(devices).toContain('/dev/mpp_service:/dev/mpp_service');

      const env = DockerComposeGenerator.getRecommendedEnvironment('rk3588');
      expect(env['MALI_OVERRIDE_PLATFORM']).toBe('rk3588');
    });
  });

  // 测试其他检测器
  describe('其他检测器', () => {

    test('ONNX Runtime 配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('onnx');
      expect(devices).toEqual([]);  // ONNX 不需要特殊设备

      const env = DockerComposeGenerator.getRecommendedEnvironment('onnx');
      expect(env['ORT_TENSORRT_ENGINE_CACHE_ENABLE']).toBe('1');
      expect(env['ORT_TENSORRT_CACHE_PATH']).toBe('/config/onnx_cache');
      expect(env['OMP_NUM_THREADS']).toBe('4');
    });

    test('CPU 检测器配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('cpu');
      expect(devices).toEqual([]);

      const env = DockerComposeGenerator.getRecommendedEnvironment('cpu');
      expect(env['OMP_NUM_THREADS']).toBe('4');
      expect(env['MKL_NUM_THREADS']).toBe('4');
      expect(env['NUMEXPR_NUM_THREADS']).toBe('4');
    });

    test('V4L2 直接访问配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('v4l2');
      expect(devices).toContain('/dev/video0:/dev/video0');

      const env = DockerComposeGenerator.getRecommendedEnvironment('v4l2');
      expect(env['V4L2_DISABLE_CONVERSION']).toBe('1');
      expect(env['V4L2_FORCE_FORMAT']).toBe('MJPEG');
    });
  });

  // 测试完整配置生成
  describe('完整 Docker Compose 生成', () => {

    test('生成 Hailo + Coral 混合配置', () => {
      const devices = [
        '/dev/hailo0:/dev/hailo0',
        '/dev/bus/usb:/dev/bus/usb'
      ];

      const config = DockerComposeGenerator.generateFullConfig(
        devices,
        { config: './config', storage: './storage' },
        'hailo'  // 主检测器类型
      );

      const yaml = DockerComposeGenerator.generateCompose(config);

      // 验证设备映射
      expect(yaml).toContain('/dev/hailo0:/dev/hailo0');
      expect(yaml).toContain('/dev/bus/usb:/dev/bus/usb');

      // 验证环境变量
      expect(yaml).toContain('HAILORT_LOGGER_PATH');
      expect(yaml).toContain('S6_LOGGING_SCRIPT');

      // 验证安全配置
      expect(yaml).toContain('apparmor=unconfined');
      expect(yaml).toContain('ipc: private');
      expect(yaml).toContain('working_dir: /opt/frigate');
    });

    test('生成多 GPU 配置', () => {
      const devices = [
        '/dev/nvidia0:/dev/nvidia0',
        '/dev/nvidia1:/dev/nvidia1',
        '/dev/dri/renderD128:/dev/dri/renderD128'
      ];

      const config = DockerComposeGenerator.generateFullConfig(
        devices,
        { config: './config', storage: './storage' },
        'nvidia'
      );

      const yaml = DockerComposeGenerator.generateCompose(config);

      expect(yaml).toContain('runtime: nvidia');
      expect(yaml).toContain('NVIDIA_VISIBLE_DEVICES: all');
      expect(yaml).toContain('/dev/nvidia0:/dev/nvidia0');
      expect(yaml).toContain('/dev/nvidia1:/dev/nvidia1');
    });
  });

  // 测试硬件检测和自动配置
  describe('硬件自动检测', () => {

    test('检测硬件类型（大小写不敏感）', () => {
      const types = ['HAILO', 'Hailo', 'hailo', 'HaiLo8'];

      types.forEach(type => {
        const devices = DockerComposeGenerator.getRecommendedDevices(type);
        expect(devices).toContain('/dev/hailo0:/dev/hailo0');
      });
    });

    test('多硬件设备支持', () => {
      // Hailo 支持多设备
      const hailoDevices = DockerComposeGenerator.getRecommendedDevices('hailo');
      expect(hailoDevices.length).toBeGreaterThan(1);
      expect(hailoDevices).toContain('/dev/hailo1:/dev/hailo1');

      // NVIDIA 支持多 GPU
      const nvidiaDevices = DockerComposeGenerator.getRecommendedDevices('nvidia');
      expect(nvidiaDevices).toContain('/dev/nvidia1:/dev/nvidia1');
    });
  });
});

// 生成示例配置供参考
console.log('\n=== 所有硬件类型示例配置 ===\n');

const hardwareTypes = [
  'hailo', 'coral', 'nvidia', 'intel', 'amd',
  'rockchip', 'jetson', 'qualcomm', 'onnx', 'cpu'
];

hardwareTypes.forEach(hwType => {
  console.log(`\n--- ${hwType.toUpperCase()} 配置 ---`);
  const devices = DockerComposeGenerator.getRecommendedDevices(hwType);
  const env = DockerComposeGenerator.getRecommendedEnvironment(hwType);

  console.log('设备映射:', devices.slice(0, 3));  // 只显示前3个
  console.log('环境变量:', Object.keys(env).filter(k => !k.includes('S6_LOGGING')).slice(0, 5));
});