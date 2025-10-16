// TypeScript types for the application
// These match the Rust models and Go agent schemas

export type DeviceType = 'gpu' | 'tpu' | 'camera' | 'capture_card'
export type Platform = 'linux' | 'windows' | 'darwin'
export type Architecture = 'x86_64' | 'arm64' | 'arm32'

export interface HardwareDevice {
  id: string
  type: DeviceType
  name: string
  device_path: string
  capabilities: string[]
  driver?: string
  platform: Platform
  architecture: Architecture
  vendor_id?: string
  detected_at: string
  detection_source: string
  available: boolean
  in_use: boolean
  error?: string
  hwaccel?: string
  detector?: string
}

export interface DetectionResult {
  devices: HardwareDevice[]
  platform: Platform
  architecture: Architecture
  detected_at: string
}

export type HwAccelType = 'none' | 'vaapi' | 'cuda' | 'qsv' | 'videotoolbox' | 'rkmpp' | 'jetson'
export type DetectorType = 'cpu' | 'onnx' | 'edgetpu' | 'openvino' | 'rknn' | 'hailo8l'

export interface CameraConfiguration {
  id: string
  name: string
  enabled: boolean
  rtsp_url: string
  rtsp_url_display: string
  resolution: {
    width: number
    height: number
  }
  fps: number

  // Hardware acceleration
  hardware_device_id?: string
  hwaccel?: HwAccelType
  hwaccel_device?: string  // Device path or index

  // AI Detection
  detector?: DetectorType
  detector_device?: string

  // Detection settings
  detect_enabled: boolean
  detect_objects?: string[]  // Objects to detect: person, car, dog, cat, etc.
  detect_width?: number
  detect_height?: number
  detect_fps?: number

  // Recording settings
  record_enabled: boolean
  record_retain_days?: number
  record_events_retain_days?: number

  // Snapshots
  snapshots_enabled: boolean
  snapshots_timestamp?: boolean
  snapshots_bounding_box?: boolean
  snapshots_crop?: boolean
  snapshots_quality?: number

  // Advanced
  custom_ffmpeg_args?: string
  motion_mask?: string[]
  zones?: any[]

  created_at: string
  updated_at: string
  manually_edited: boolean
  validation_status: 'valid' | 'warning' | 'error'
  validation_errors: string[]
}

export interface ConfigurationSnapshot {
  id: string
  version: number
  yaml_content: string
  checksum: string
  created_at: string
  created_by: 'ui' | 'template' | 'manual_edit' | 'rollback'
  description?: string
  is_backup: boolean
  backup_reason?: string
  deployed: boolean
  deployed_at?: string
  deployment_success?: boolean
}

export interface AppError {
  type: string
  message: string
}

export type ValidationStatus = 'valid' | 'warning' | 'error'

// Hardware availability types
export interface HardwareAvailability {
  device_id: string
  device_name: string
  device_type: string
  platform: string
  available: boolean
  in_use: boolean
  last_checked: string
  usage: UsageMetrics
  performance: PerformanceMetrics
  thermal: ThermalStatus
  power: PowerStatus
  processes: ProcessInfo[]
  errors: string[]
  warnings: string[]
  capacity: CapacityInfo
}

export interface UsageMetrics {
  utilization: number
  memory_utilization: number
  memory_used: number
  memory_total: number
  cpu_utilization: number
  temperature: number
  clock_speed: number
  power_consumption: number
}

export interface PerformanceMetrics {
  temperature: number
  power_consumption: number
  clock_speed: number
  fan_speed: number
  throughput: number
  latency: number
  quality: number
  error_rate: number
}

export interface ThermalStatus {
  temperature: number
  throttling: boolean
  critical: boolean
  fan_speed: number
}

export interface PowerStatus {
  current_power: number
  power_limit: number
  voltage: number
  current: number
  efficiency: number
}

export interface ProcessInfo {
  pid: number
  name: string
  memory_usage: number
  cpu_usage: number
}

export interface CapacityInfo {
  max_throughput: number
  current_throughput: number
  max_load: number
  current_load: number
  max_temperature: number
  current_temperature: number
  max_power: number
  current_power: number
}

// Configuration generation types
export interface ConfigTemplate {
  id: string
  name: string
  description: string
  supported_devices: string[]
  requirements: string[]
}

export interface GeneratedConfig {
  config: string
  metadata: ConfigMetadata
  warnings: string[]
  recommendations: string[]
}

export interface ConfigMetadata {
  generated_at: string
  template_used: string
  devices_configured: number
  estimated_performance: string
}

export interface ConfigGenerationRequest {
  hardware_devices: HardwareDevice[]
  template_type: string
  custom_settings?: any
}

export interface ConfigValidationResponse {
  is_valid: boolean
  errors: ValidationError[]
  warnings: ValidationWarning[]
  cameras_count: number
  detectors_count: number
}

export interface ValidationError {
  message: string
  line_number?: number
  error_type: string
}

export interface ValidationWarning {
  message: string
  line_number?: number
  suggestion?: string
}
