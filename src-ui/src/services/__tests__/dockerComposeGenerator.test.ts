// 测试 Docker Compose 生成器的增强功能
import { DockerComposeGenerator } from '../dockerComposeGenerator';

describe('DockerComposeGenerator 增强功能测试', () => {
  describe('Nvidia GPU 支持', () => {
    it('应当为 NVIDIA GPU 生成正确的配置', () => {
      const config = DockerComposeGenerator.generateFullConfig(
        [],
        { config: './config', storage: './storage' },
        'nvidia'
      );

      expect(config.nvidia_runtime).toBe(true);
      expect(config.nvidia_devices).toBe('all');
      expect(config.environment['NVIDIA_VISIBLE_DEVICES']).toBe('all');
      expect(config.environment['NVIDIA_DRIVER_CAPABILITIES']).toBe('compute,video,utility');
      // 验证 NVIDIA 设备映射
      expect(config.devices).toContain('/dev/nvidia0:/dev/nvidia0');
      expect(config.devices).toContain('/dev/nvidiactl:/dev/nvidiactl');
    });

    it('应当正确生成 NVIDIA 设备列表', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('nvidia');
      expect(devices).toContain('/dev/nvidia0:/dev/nvidia0');
      expect(devices).toContain('/dev/nvidiactl:/dev/nvidiactl');
      expect(devices).toContain('/dev/nvidia-modeset:/dev/nvidia-modeset');
      expect(devices).toContain('/dev/nvidia-uvm:/dev/nvidia-uvm');
    });
  });

  describe('Hailo NPU 支持', () => {
    it('应当为 Hailo NPU 生成正确的配置', () => {
      const devices = DockerComposeGenerator.getRecommendedDevices('hailo');
      expect(devices).toContain('/dev/hailo0:/dev/hailo0');

      const env = DockerComposeGenerator.getRecommendedEnvironment('hailo');
      expect(env['HAILORT_LOGGER_PATH']).toBe('/config/logs');
    });
  });

  describe('Frigate 特定环境变量', () => {
    it('应当包含所有 Frigate 核心环境变量', () => {
      const env = DockerComposeGenerator.getRecommendedEnvironment();

      expect(env['FRIGATE_RTSP_PASSWORD']).toBeDefined();
      expect(env['S6_LOGGING_SCRIPT']).toBeDefined();
      expect(env['DEFAULT_FFMPEG_VERSION']).toBe('7');
      expect(env['PYTHONDONTWRITEBYTECODE']).toBe('1');
    });
  });

  describe('安全选项和 IPC', () => {
    it('应当包含安全选项配置', () => {
      const secOpt = DockerComposeGenerator.getRecommendedSecurityOpt();
      expect(secOpt).toContain('apparmor=unconfined');

      const config = DockerComposeGenerator.generateFullConfig(
        [],
        { config: './config', storage: './storage' }
      );
      expect(config.security_opt).toContain('apparmor=unconfined');
      expect(config.ipc).toBe('private');
      expect(config.working_dir).toBe('/opt/frigate');
    });
  });

  describe('完整 YAML 生成', () => {
    it('应当生成与用户提供配置兼容的 YAML', () => {
      const config = DockerComposeGenerator.generateFullConfig(
        ['/dev/hailo0:/dev/hailo0'],
        { config: './config', storage: './storage' },
        'hailo'
      );

      const yaml = DockerComposeGenerator.generateCompose(config);

      // 验证关键配置存在
      expect(yaml).toContain('container_name: frigate');
      expect(yaml).toContain('privileged: true');
      expect(yaml).toContain('shm_size: 256mb');
      expect(yaml).toContain('ipc: private');
      expect(yaml).toContain('working_dir: /opt/frigate');
      expect(yaml).toContain('/dev/hailo0:/dev/hailo0');
      expect(yaml).toContain('HAILORT_LOGGER_PATH');
      expect(yaml).toContain('S6_LOGGING_SCRIPT');
      expect(yaml).toContain('DEFAULT_FFMPEG_VERSION');
      expect(yaml).toContain('apparmor=unconfined');
    });

    it('应当为 NVIDIA 生成运行时配置', () => {
      const config = DockerComposeGenerator.generateFullConfig(
        [],
        { config: './config', storage: './storage' },
        'nvidia'
      );

      const yaml = DockerComposeGenerator.generateCompose(config);

      expect(yaml).toContain('runtime: nvidia');
      expect(yaml).toContain('NVIDIA_VISIBLE_DEVICES');
      expect(yaml).toContain('NVIDIA_DRIVER_CAPABILITIES');
    });
  });

  describe('网络配置', () => {
    it('应当支持自定义网络配置', () => {
      const config = DockerComposeGenerator.generateFullConfig(
        [],
        { config: './config', storage: './storage' }
      );
      config.network = 'br0';

      const yaml = DockerComposeGenerator.generateCompose(config);

      expect(yaml).toContain('networks:');
      expect(yaml).toContain('br0');
      expect(yaml).toContain('external: true');
    });
  });
});

// 输出一个示例配置供调试
console.log('\n=== 示例 Hailo 配置 ===');
const hailoConfig = DockerComposeGenerator.generateFullConfig(
  ['/dev/hailo0:/dev/hailo0'],
  { config: './config', storage: './storage' },
  'hailo'
);
console.log(DockerComposeGenerator.generateCompose(hailoConfig));

console.log('\n=== 示例 NVIDIA 配置 ===');
const nvidiaConfig = DockerComposeGenerator.generateFullConfig(
  [],
  { config: './config', storage: './storage' },
  'nvidia'
);
console.log(DockerComposeGenerator.generateCompose(nvidiaConfig));