// Frigate config.yml Generator
// Generates Frigate configuration file (NOT docker-compose.yml)
// This file only contains camera, detector, and Frigate-specific settings

import type { CameraConfiguration, HardwareDevice } from '../types';

interface FrigateConfig {
  mqtt?: {
    enabled: boolean;
    host?: string;
    port?: number;
  };
  detectors?: Record<string, any>;
  cameras: Record<string, any>;
  record?: {
    enabled: boolean;
    retain?: {
      days: number;
      mode: string;
    };
  };
  snapshots?: {
    enabled: boolean;
    retain?: {
      default: number;
    };
  };
}

export class FrigateConfigGenerator {
  /**
   * Generate Frigate config.yml (NOT docker-compose.yml)
   * Based on official Frigate documentation format
   */
  static generateConfig(
    cameras: CameraConfiguration[],
    devices: HardwareDevice[],
    _retentionDays: number = 7
  ): string {
    // Build config following official Frigate config.yml format
    // Order: mqtt, detectors, cameras
    const config: FrigateConfig = {
      mqtt: {
        enabled: false,
      },
      detectors: this.generateDetectors(devices, cameras),
      cameras: {},
    };

    // Add cameras
    cameras.forEach((camera) => {
      if (camera.enabled) {
        config.cameras[camera.name] = this.generateCameraConfig(camera, devices);
      }
    });

    const yaml = this.toYAML(config);

    // Debug: Log first line to console
    console.log('生成的配置第一行:', yaml.split('\n')[0]);
    console.log('生成的配置总行数:', yaml.split('\n').length);
    console.log('配置开头 100 字符:', yaml.substring(0, 100));

    return yaml;
  }

  /**
   * Generate detectors configuration from cameras and hardware devices
   */
  private static generateDetectors(_devices: HardwareDevice[], cameras?: CameraConfiguration[]): Record<string, any> {
    const detectors: Record<string, any> = {};

    // Collect unique detectors from camera configurations
    const usedDetectors = new Set<string>();

    if (cameras) {
      cameras.forEach((camera) => {
        if (camera.enabled && camera.detector) {
          usedDetectors.add(camera.detector);
        }
      });
    }

    // Generate detector configs based on type
    // Following official Frigate documentation: https://docs.frigate.video/configuration/object_detectors
    // UPDATED 2025: TensorRT deprecated, use ONNX for NVIDIA GPUs
    usedDetectors.forEach((detectorType) => {
      switch (detectorType) {
        case 'onnx':
          // ONNX detector (recommended for NVIDIA GPUs in 2025)
          detectors['onnx'] = {
            type: 'onnx',
            device: 'AUTO', // AUTO will automatically use NVIDIA GPU if available
          };
          break;
        case 'edgetpu':
          detectors['edgetpu'] = {
            type: 'edgetpu',
            device: 'usb', // Can be: usb, usb:0, pci, pci:0
          };
          break;
        case 'openvino':
          detectors['openvino'] = {
            type: 'openvino',
            device: 'CPU', // Can be: CPU, GPU (AUTO has known issues)
          };
          break;
        case 'rknn':
          detectors['rknn'] = {
            type: 'rknn',
            num_cores: 0, // 0 = automatically choose
          };
          break;
        case 'hailo8l':
          // Hailo-8L detector (official name)
          detectors['hailo8l'] = {
            type: 'hailo8l',
            device: 'PCIe',
            model: {
              width: 320,
              height: 320,
              input_tensor: 'nhwc',
              input_pixel_format: 'bgr',
              model_type: 'yolov6',
            },
          };
          break;
        case 'cpu':
          detectors['cpu'] = {
            type: 'cpu',
            num_threads: 3,
          };
          break;
      }
    });

    // Add CPU detector as fallback if no detectors specified
    if (Object.keys(detectors).length === 0) {
      detectors['cpu'] = {
        type: 'cpu',
        num_threads: 3,
      };
    }

    return detectors;
  }

  /**
   * Generate single camera configuration
   */
  private static generateCameraConfig(
    camera: CameraConfiguration,
    _devices: HardwareDevice[]
  ): any {
    const cameraConfig: any = {
      enabled: camera.enabled,
      ffmpeg: {
        inputs: [
          {
            path: camera.rtsp_url,
            roles: [],
          },
        ],
      },
    };

    // Add roles based on enabled features
    const roles: string[] = [];
    if (camera.detect_enabled) roles.push('detect');
    if (camera.record_enabled) roles.push('record');
    cameraConfig.ffmpeg.inputs[0].roles = roles;

    // Add hardware decoder configuration
    if (camera.hwaccel && camera.hwaccel !== 'none') {
      cameraConfig.ffmpeg.hwaccel_args = this.getHwaccelArgsForType(camera.hwaccel);
    }

    // Add detection configuration
    if (camera.detect_enabled) {
      cameraConfig.detect = {
        enabled: true,
        width: camera.detect_width || camera.resolution.width,
        height: camera.detect_height || camera.resolution.height,
        fps: camera.detect_fps || 5,
      };

      // Always add detector assignment (cpu is the default)
      const detectorType = camera.detector || 'cpu';
      cameraConfig.detect.detector = detectorType;

      // Add detection objects filter (default to person if not specified)
      const objectsToTrack = camera.detect_objects && camera.detect_objects.length > 0
        ? camera.detect_objects
        : ['person'];

      cameraConfig.objects = {
        track: objectsToTrack,
      };
    }

    // Add snapshot configuration
    if (camera.snapshots_enabled) {
      cameraConfig.snapshots = {
        enabled: true,
        timestamp: camera.snapshots_timestamp !== false,
        bounding_box: camera.snapshots_bounding_box !== false,
        crop: camera.snapshots_crop || false,
        quality: camera.snapshots_quality || 85,
      };
    }

    // Add recording configuration
    if (camera.record_enabled) {
      cameraConfig.record = {
        enabled: true,
        retain: {
          days: camera.record_retain_days || 7,
          mode: 'motion',
        },
        events: {
          retain: {
            default: camera.record_events_retain_days || 30,
            mode: 'motion',
          },
        },
      };
    }

    return cameraConfig;
  }

  /**
   * Get hardware acceleration args for ffmpeg based on hwaccel type
   * Following official Frigate documentation: https://docs.frigate.video/configuration/ffmpeg_presets/
   */
  private static getHwaccelArgsForType(hwaccelType: string): string {
    switch (hwaccelType) {
      case 'cuda':
        // NVIDIA CUDA - Official preset supports both H.264 and H.265
        return 'preset-nvidia';
      case 'qsv':
        // Intel Quick Sync Video - H.264
        return 'preset-intel-qsv-h264';
      case 'vaapi':
        // Video Acceleration API (Intel/AMD)
        return 'preset-vaapi';
      case 'videotoolbox':
        // Apple VideoToolbox (macOS) - Not in official presets, remove
        return '';
      case 'rkmpp':
        // Rockchip Media Process Platform
        return 'preset-rkmpp';
      case 'jetson':
        // NVIDIA Jetson - H.264
        return 'preset-jetson-h264';
      default:
        return '';
    }
  }

  /**
   * Convert config object to YAML string (clean format without empty values)
   */
  private static toYAML(obj: any, indent: number = 0): string {
    const spaces = '  '.repeat(indent);
    let yaml = '';

    for (const [key, value] of Object.entries(obj)) {
      // Skip null, undefined, and empty objects
      if (value === null || value === undefined) {
        continue;
      }

      // Skip empty objects
      if (typeof value === 'object' && !Array.isArray(value) && Object.keys(value).length === 0) {
        continue;
      }

      if (typeof value === 'object' && !Array.isArray(value)) {
        // Only add key if object has content
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
            // Inline object format for array items
            yaml += `${spaces}  - `;
            const itemEntries = Object.entries(item);
            if (itemEntries.length > 0) {
              const [firstKey, firstValue] = itemEntries[0];
              yaml += `${firstKey}: ${firstValue}\n`;
              // Add remaining properties
              for (let i = 1; i < itemEntries.length; i++) {
                const [k, v] = itemEntries[i];
                if (Array.isArray(v)) {
                  yaml += `${spaces}    ${k}:\n`;
                  v.forEach(arrItem => {
                    yaml += `${spaces}      - ${arrItem}\n`;
                  });
                } else {
                  yaml += `${spaces}    ${k}: ${v}\n`;
                }
              }
            }
          } else {
            yaml += `${spaces}  - ${item}\n`;
          }
        });
      } else if (typeof value === 'string') {
        yaml += `${spaces}${key}: ${value}\n`;
      } else {
        yaml += `${spaces}${key}: ${value}\n`;
      }
    }

    return yaml;
  }

  /**
   * Get default config.yml template following official Frigate documentation
   */
  static getTemplate(): string {
    return `# Frigate config.yml
# Official documentation: https://docs.frigate.video
# Generated by Frigate Configuration Tool

mqtt:
  enabled: False  # Set to True if using MQTT

detectors:
  cpu:
    type: cpu
    num_threads: 3

cameras:
  # Example camera configuration
  # front_door:
  #   enabled: True
  #   ffmpeg:
  #     inputs:
  #       - path: rtsp://username:password@camera_ip:554/stream
  #         roles:
  #           - detect
  #           - record
  #   detect:
  #     enabled: True
  #     width: 1280
  #     height: 720
  #   objects:
  #     track:
  #       - person
  #   record:
  #     enabled: True
  #     retain:
  #       days: 10
`;
  }

  /**
   * RTSP H.264 single camera template
   */
  static getRtspH264Template(): string {
    return `# Frigate RTSP H.264 single camera template
mqtt:
  enabled: False

detectors:
  cpu:
    type: cpu
    num_threads: 3

cameras:
  front_door:
    enabled: True
    ffmpeg:
      inputs:
        - path: rtsp://username:password@camera_ip:554/h264
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1280
      height: 720
      fps: 5
      detector: cpu
    objects:
      track:
        - person
    record:
      enabled: True
      retain:
        days: 10
`;
  }

  /**
   * Coral TPU detectors template (USB)
   */
  static getCoralTemplate(): string {
    return `# Frigate Coral TPU detectors template
mqtt:
  enabled: False

detectors:
  edgetpu:
    type: edgetpu
    device: usb

cameras:
  front_door:
    enabled: True
    ffmpeg:
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1280
      height: 720
      fps: 5
      detector: edgetpu
    objects:
      track:
        - person
`;
  }

  /**
   * Basic NVR template with recordings and snapshots
   */
  static getBasicNvrTemplate(): string {
    return `# Frigate Basic NVR template
mqtt:
  enabled: False

detectors:
  cpu:
    type: cpu
    num_threads: 3

record:
  enabled: True
  retain:
    days: 10
    mode: motion

snapshots:
  enabled: True
  retain:
    default: 5

cameras: {}
`;
  }

  /**
   * ONNX detector template (AUTO device, uses NVIDIA GPU if available)
   */
  static getOnnxTemplate(): string {
    return `# Frigate ONNX detector template (AUTO/NVIDIA)
mqtt:
  enabled: False

detectors:
  onnx:
    type: onnx
    device: AUTO

cameras:
  front_door:
    enabled: True
    ffmpeg:
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1280
      height: 720
      fps: 5
      detector: onnx
    objects:
      track:
        - person
`;
  }

  /**
   * OpenVINO detector template (Intel CPU)
   */
  static getOpenVinoTemplate(): string {
    return `# Frigate OpenVINO detector template (Intel CPU)
mqtt:
  enabled: False

detectors:
  openvino:
    type: openvino
    device: CPU

cameras:
  front_door:
    enabled: True
    ffmpeg:
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1280
      height: 720
      fps: 5
      detector: openvino
    objects:
      track:
        - person
`;
  }

  /**
   * NVIDIA CUDA ffmpeg preset + ONNX detector (AUTO)
   */
  static getCudaTemplate(): string {
    return `# Frigate NVIDIA CUDA template (ffmpeg preset + ONNX)
mqtt:
  enabled: False

detectors:
  onnx:
    type: onnx
    device: AUTO

cameras:
  front_door:
    enabled: True
    ffmpeg:
      hwaccel_args: preset-nvidia
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1920
      height: 1080
      fps: 10
      detector: onnx
    objects:
      track:
        - person
`;
  }

  /**
   * Intel QSV ffmpeg preset + OpenVINO GPU detector
   */
  static getIntelQsvTemplate(): string {
    return `# Frigate Intel QSV template (ffmpeg preset + OpenVINO GPU)
mqtt:
  enabled: False

detectors:
  openvino:
    type: openvino
    device: GPU

cameras:
  front_door:
    enabled: True
    ffmpeg:
      hwaccel_args: preset-intel-qsv-h264
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1280
      height: 720
      fps: 5
      detector: openvino
    objects:
      track:
        - person
`;
  }

  /**
   * AMD VAAPI ffmpeg preset + ONNX detector (AUTO)
   */
  static getAmdVaapiTemplate(): string {
    return `# Frigate AMD VAAPI template (ffmpeg preset + ONNX)
mqtt:
  enabled: False

detectors:
  onnx:
    type: onnx
    device: AUTO

cameras:
  front_door:
    enabled: True
    ffmpeg:
      hwaccel_args: preset-vaapi
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1280
      height: 720
      fps: 5
      detector: onnx
    objects:
      track:
        - person
`;
  }

  /**
   * Rockchip RK3588 RKNN detector template
   */
  static getRknnTemplate(): string {
    return `# Frigate Rockchip RK3588 RKNN detector template
mqtt:
  enabled: False

detectors:
  rknn:
    type: rknn
    num_cores: 0

cameras:
  front_door:
    enabled: True
    ffmpeg:
      hwaccel_args: preset-rkmpp
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1280
      height: 720
      fps: 5
      detector: rknn
    objects:
      track:
        - person
`;
  }

  /**
   * Hailo-8L NPU detector template
   */
  static getHailo8lTemplate(): string {
    return `# Frigate Hailo-8L NPU detector template
mqtt:
  enabled: False

detectors:
  hailo8l:
    type: hailo8l
    device: PCIe
    model:
      width: 320
      height: 320
      input_tensor: nhwc
      input_pixel_format: bgr
      model_type: yolov6

cameras:
  front_door:
    enabled: True
    ffmpeg:
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1280
      height: 720
      fps: 5
      detector: hailo8l
    objects:
      track:
        - person
`;
  }

  /**
   * NVIDIA Jetson ffmpeg preset + ONNX detector
   */
  static getJetsonTemplate(): string {
    return `# Frigate NVIDIA Jetson template (ffmpeg preset + ONNX)
mqtt:
  enabled: False

detectors:
  onnx:
    type: onnx
    device: AUTO

cameras:
  front_door:
    enabled: True
    ffmpeg:
      hwaccel_args: preset-jetson-h264
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1280
      height: 720
      fps: 5
      detector: onnx
    objects:
      track:
        - person
`;
  }

  /**
   * OpenVINO GPU detector template (no ffmpeg hwaccel)
   */
  static getOpenVinoGpuOnlyTemplate(): string {
    return `# Frigate OpenVINO GPU detector template (no ffmpeg hwaccel)
mqtt:
  enabled: False

detectors:
  openvino:
    type: openvino
    device: GPU

cameras:
  front_door:
    enabled: True
    ffmpeg:
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1280
      height: 720
      fps: 5
      detector: openvino
    objects:
      track:
        - person
`;
  }

  /**
   * ONNX CUDA detector template (explicit CUDA device)
   */
  static getOnnxCudaTemplate(): string {
    return `# Frigate ONNX CUDA detector template (explicit device)
mqtt:
  enabled: False

detectors:
  onnx:
    type: onnx
    device: CUDA

cameras:
  front_door:
    enabled: True
    ffmpeg:
      hwaccel_args: preset-nvidia
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1920
      height: 1080
      fps: 10
      detector: onnx
    objects:
      track:
        - person
`;
  }

  /**
   * CPU detector high-performance template
   */
  static getCpuHighPerfTemplate(): string {
    return `# Frigate CPU detector template (high-performance)
mqtt:
  enabled: False

detectors:
  cpu:
    type: cpu
    num_threads: 6

cameras:
  front_door:
    enabled: True
    ffmpeg:
      inputs:
        - path: rtsp://username:password@camera_ip:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: True
      width: 1920
      height: 1080
      fps: 10
      detector: cpu
    objects:
      track:
        - person
`;
  }
}
