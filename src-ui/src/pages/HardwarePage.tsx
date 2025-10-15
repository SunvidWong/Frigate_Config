// Hardware Detection Page - T037-T043
// Displays detected hardware devices and allows user to refresh detection

import React, { useState, useEffect } from 'react'
import { useTauriCommand } from '../hooks/useTauriCommand'
import { safeInvoke, isTauriEnvironment } from '../utils/tauri'
import Button from '../components/Button'
import Card from '../components/Card'
import Modal from '../components/Modal'
import type { HardwareDevice, DeviceType, HardwareAvailability } from '../types'

// PCI Device types
interface PciDeviceInfo {
  slot: string
  class: string
  vendor: string
  device: string
  subsystem: string | null
  revision: string | null
  device_path: string
  is_gpu: boolean
  is_nvidia: boolean
  is_amd: boolean
  is_intel: boolean
  recommended_device_path: string | null
}

// Preset hardware configurations for common devices
interface PresetHardware {
  id: string
  name: string
  devicePath: string
  type: DeviceType
  description: string
}

const PRESET_HARDWARE: PresetHardware[] = [
  // NVIDIA GPUs - NOTE: NVIDIA uses runtime, not device mapping (for reference only)
  // { id: 'nvidia-gpu', name: 'NVIDIA GPU', devicePath: 'runtime:nvidia', type: 'gpu', description: 'NVIDIA GPU 使用 runtime 配置,无需设备映射' },

  // Intel GPUs
  { id: 'intel-qsv', name: 'Intel Quick Sync Video', devicePath: '/dev/dri/renderD128', type: 'gpu', description: 'Intel 核显硬件加速' },
  { id: 'intel-vaapi', name: 'Intel VA-API', devicePath: '/dev/dri/card0', type: 'gpu', description: 'Intel 视频加速 API' },

  // AMD GPUs
  { id: 'amd-gpu', name: 'AMD GPU', devicePath: '/dev/dri/renderD128', type: 'gpu', description: 'AMD 显卡硬件加速' },
  { id: 'amd-card0', name: 'AMD Card0', devicePath: '/dev/dri/card0', type: 'gpu', description: 'AMD 显卡设备0' },

  // Google Coral TPUs
  { id: 'coral-pci', name: 'Google Coral PCIe/M.2', devicePath: '/dev/apex_0', type: 'tpu', description: 'Google Coral PCIe/M.2 加速器' },
  { id: 'coral-usb', name: 'Google Coral USB', devicePath: '/dev/bus/usb', type: 'tpu', description: 'Google Coral USB 加速器' },

  // Hailo AI Accelerators (Updated 2025)
  { id: 'hailo8l', name: 'Hailo-8L AI Accelerator', devicePath: '/dev/hailo0', type: 'tpu', description: 'Hailo-8L AI 加速器(官方推荐型号)' },

  // Rockchip NPUs
  { id: 'rockchip-npu', name: 'Rockchip NPU', devicePath: '/dev/rknpu', type: 'tpu', description: 'Rockchip NPU (RK3588/RK3576)' },

  // Common Video Devices
  { id: 'video0', name: '视频设备 0', devicePath: '/dev/video0', type: 'camera', description: 'USB 摄像头或采集卡' },
  { id: 'video1', name: '视频设备 1', devicePath: '/dev/video1', type: 'camera', description: 'USB 摄像头或采集卡' },
]

// Phase 4 - 扩展的添加设备响应结构
interface AddHardwareDeviceResponse {
  success: boolean
  message: string
  device_path: string
  device_type: string
  docker_compose_updated: boolean
  docker_compose_path: string | null
  frigate_service_name: string | null
  warnings: string[]
}

// Phase 2 - Docker Compose 路径响应
interface DockerComposePathResponse {
  path: string | null
  is_custom: boolean
  message: string
}

const HardwarePage: React.FC = () => {
  const [devices, setDevices] = useState<HardwareDevice[]>([])
  const [filteredDevices, setFilteredDevices] = useState<HardwareDevice[]>([])
  const [selectedFilter, setSelectedFilter] = useState<DeviceType | 'all'>('all')
  const [selectedDevice, setSelectedDevice] = useState<HardwareDevice | null>(null)
  const [showDetailsModal, setShowDetailsModal] = useState(false)
  const [availabilityData, setAvailabilityData] = useState<HardwareAvailability[]>([])
  const [showAvailabilityModal, setShowAvailabilityModal] = useState(false)
  const [selectedAvailability, setSelectedAvailability] = useState<HardwareAvailability | null>(null)
  const [addingDevice, setAddingDevice] = useState<string | null>(null)
  const [addSuccess, setAddSuccess] = useState<string | null>(null)
  const [isInTauriEnv, setIsInTauriEnv] = useState(false)
  const [showPresetSelector, setShowPresetSelector] = useState(false)
  const [selectedPreset, setSelectedPreset] = useState<string>('')
  const [presetFilter, setPresetFilter] = useState<DeviceType | 'all'>('all')
  const [scanningPci, setScanningPci] = useState(false)
  const [pciDevices, setPciDevices] = useState<PciDeviceInfo[]>([])
  const [showPciDevices, setShowPciDevices] = useState(false)

  // Phase 2 - Docker Compose 路径管理状态
  const [dockerComposePath, setDockerComposePath] = useState<string | null>(null)
  const [isCustomPath, setIsCustomPath] = useState(false)
  const [showPathConfig, setShowPathConfig] = useState(false)
  const [customPathInput, setCustomPathInput] = useState('')

  // Phase 4 - 详细的添加结果信息
  const [lastAddResult, setLastAddResult] = useState<AddHardwareDeviceResponse | null>(null)

  const {
    data: detectionData,
    error: detectionError,
    loading: detecting,
    execute: runDetection,
  } = useTauriCommand<HardwareDevice[]>('detect_hardware')

  const {
    data: availabilityResponse,
    loading: checkingAvailability,
    execute: checkAvailability,
  } = useTauriCommand<HardwareAvailability[]>('get_hardware_availability')

  // Check environment on mount
  useEffect(() => {
    setIsInTauriEnv(isTauriEnvironment())
  }, [])

  // Phase 2 - 加载 docker-compose.yml 路径配置
  useEffect(() => {
    const loadDockerComposePath = async () => {
      try {
        const result = await safeInvoke<DockerComposePathResponse>('get_docker_compose_path')
        setDockerComposePath(result.path || '自动检测')
        setIsCustomPath(result.is_custom)
      } catch (err) {
        console.error('加载 docker-compose 路径失败:', err)
      }
    }

    if (isInTauriEnv) {
      loadDockerComposePath()
    }
  }, [isInTauriEnv])

  // Initial detection on mount (only in Tauri environment)
  useEffect(() => {
    if (isInTauriEnv) {
      runDetection()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isInTauriEnv])

  // Update devices when detection completes
  useEffect(() => {
    if (detectionData) {
      setDevices(detectionData)
      setFilteredDevices(detectionData)
    }
  }, [detectionData])

  // Update availability data when check completes
  useEffect(() => {
    if (availabilityResponse) {
      setAvailabilityData(availabilityResponse)
    }
  }, [availabilityResponse])

  // Filter devices when filter changes
  useEffect(() => {
    if (selectedFilter === 'all') {
      setFilteredDevices(devices)
    } else {
      setFilteredDevices(devices.filter(d => d.type === selectedFilter))
    }
  }, [selectedFilter, devices])

  const handleDetect = () => {
    runDetection({ force_refresh: true })
  }

  const handleDeviceClick = (device: HardwareDevice) => {
    setSelectedDevice(device)
    setShowDetailsModal(true)
  }

  const handleCheckAvailability = () => {
    checkAvailability()
  }

  const handleAvailabilityClick = (device: HardwareDevice) => {
    const availability = availabilityData.find(a => a.device_id === device.id)
    if (availability) {
      setSelectedAvailability(availability)
      setShowAvailabilityModal(true)
    } else {
      // If no availability data exists for this device, run availability check
      checkAvailability()
    }
  }

  // Phase 4 - 处理扩展的响应结构
  const handleAddToConfig = async (device: HardwareDevice) => {
    setAddingDevice(device.id)
    setAddSuccess(null)
    setLastAddResult(null)

    try {
      const result = await safeInvoke<AddHardwareDeviceResponse>('add_hardware_device_to_config', {
        devicePath: device.device_path,
        deviceType: device.type,
        deviceName: device.name
      })

      setLastAddResult(result)
      setAddSuccess(`已添加 ${device.name} 到配置`)

      // Clear success message after 5 seconds
      setTimeout(() => {
        setAddSuccess(null)
        setLastAddResult(null)
      }, 5000)
    } catch (err) {
      alert(`添加失败: ${err}`)
    } finally {
      setAddingDevice(null)
    }
  }

  // Phase 4 - 处理扩展的响应结构
  const handlePresetAdd = async () => {
    if (!selectedPreset) {
      alert('请选择一个硬件设备')
      return
    }

    const preset = PRESET_HARDWARE.find(p => p.id === selectedPreset)
    if (!preset) return

    setAddingDevice(preset.id)
    setAddSuccess(null)
    setLastAddResult(null)

    try {
      const result = await safeInvoke<AddHardwareDeviceResponse>('add_hardware_device_to_config', {
        devicePath: preset.devicePath,
        deviceType: preset.type,
        deviceName: preset.name
      })

      setLastAddResult(result)
      setAddSuccess(`已添加 ${preset.name} 到配置和 docker-compose.yml`)

      // Clear selection
      setSelectedPreset('')
      setShowPresetSelector(false)

      // Clear success message after 5 seconds
      setTimeout(() => {
        setAddSuccess(null)
        setLastAddResult(null)
      }, 5000)
    } catch (err) {
      alert(`添加失败: ${err}`)
    } finally {
      setAddingDevice(null)
    }
  }

  const getFilteredPresets = () => {
    if (presetFilter === 'all') {
      return PRESET_HARDWARE
    }
    return PRESET_HARDWARE.filter(p => p.type === presetFilter)
  }

  const handleScanPciDevices = async () => {
    setScanningPci(true)
    try {
      let result: { devices: PciDeviceInfo[], total_count: number }

      if (isInTauriEnv) {
        // Tauri desktop mode
        result = await safeInvoke<{ devices: PciDeviceInfo[], total_count: number }>('scan_pci_devices')
      } else {
        // HTTP mode (Docker/Browser)
        const response = await fetch('/api/scan_pci_devices', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
        })

        // Parse JSON response regardless of status code
        const apiResponse = await response.json()

        // Check if the API returned an error
        if (!apiResponse.success) {
          throw new Error(apiResponse.error || `HTTP ${response.status}: ${response.statusText}`)
        }

        result = apiResponse.data
      }

      setPciDevices(result.devices.filter(d => d.is_gpu)) // Only show GPUs
      setShowPciDevices(true)
    } catch (err) {
      alert(`PCI 设备扫描失败: ${err}`)
    } finally {
      setScanningPci(false)
    }
  }

  // Phase 4 - 处理扩展的响应结构
  const handleAddPciDevice = async (pciDevice: PciDeviceInfo) => {
    if (!pciDevice.recommended_device_path) {
      alert('无法确定此设备的推荐路径')
      return
    }

    setAddingDevice(pciDevice.slot)
    setAddSuccess(null)
    setLastAddResult(null)

    try {
      let deviceName = pciDevice.vendor
      let devicePath = pciDevice.recommended_device_path

      if (pciDevice.is_nvidia) {
        deviceName = 'NVIDIA GPU'
        // For NVIDIA, use a special marker that backend will recognize
        // Backend will add deploy.resources.reservations instead of device mapping
        devicePath = 'nvidia-gpu-runtime'
      } else if (pciDevice.is_amd) {
        deviceName = 'AMD GPU'
        devicePath = pciDevice.recommended_device_path.split(', ')[0]
      } else if (pciDevice.is_intel) {
        deviceName = 'Intel GPU'
        devicePath = pciDevice.recommended_device_path.split(', ')[0]
      } else {
        devicePath = pciDevice.recommended_device_path.split(', ')[0]
      }

      const result = await safeInvoke<AddHardwareDeviceResponse>('add_hardware_device_to_config', {
        devicePath,
        deviceType: 'gpu',
        deviceName
      })

      setLastAddResult(result)

      if (pciDevice.is_nvidia) {
        setAddSuccess(`已添加 ${deviceName} 到配置 (使用 deploy.resources.reservations)`)
      } else {
        setAddSuccess(`已添加 ${deviceName} 到配置`)
      }
      setTimeout(() => {
        setAddSuccess(null)
        setLastAddResult(null)
      }, 5000)
    } catch (err) {
      alert(`添加失败: ${err}`)
    } finally {
      setAddingDevice(null)
    }
  }

  // Phase 2 - 设置自定义路径
  const handleSetCustomPath = async () => {
    if (!customPathInput.trim()) {
      alert('请输入有效的路径')
      return
    }

    try {
      const result = await safeInvoke<{ success: boolean; message: string; path: string }>('set_docker_compose_path', {
        path: customPathInput
      })

      if (result.success) {
        setDockerComposePath(result.path)
        setIsCustomPath(true)
        setShowPathConfig(false)
        setCustomPathInput('')
        alert(`✓ ${result.message}`)
      }
    } catch (err) {
      alert(`设置路径失败: ${err}`)
    }
  }

  // Phase 2 - 重置为自动检测
  const handleResetPath = async () => {
    try {
      // 通过设置空路径来重置为自动检测
      const result = await safeInvoke<{ success: boolean; message: string; path: string }>('set_docker_compose_path', {
        path: ''
      })

      if (result.success) {
        setDockerComposePath('自动检测')
        setIsCustomPath(false)
        setShowPathConfig(false)
        alert('✓ 已重置为自动检测模式')
      }
    } catch (err) {
      alert(`重置失败: ${err}`)
    }
  }

  const getDeviceIcon = (type: DeviceType): string => {
    const icons: Record<DeviceType, string> = {
      gpu: '🎮',
      tpu: '🧠',
      camera: '📷',
      capture_card: '📹',
    }
    return icons[type] || '❓'
  }

  const getDeviceTypeLabel = (type: DeviceType): string => {
    const labels: Record<DeviceType, string> = {
      gpu: 'GPU',
      tpu: 'TPU',
      camera: '相机',
      capture_card: '采集卡',
    }
    return labels[type] || type
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">硬件检测</h1>
        <p className="text-gray-600">
          检测可用的硬件加速器和视频设备
        </p>
      </div>

      {/* Phase 2 - Docker Compose 路径配置 */}
      {dockerComposePath && (
        <Card className="mb-6 bg-gradient-to-r from-purple-50 to-pink-50 border border-purple-200">
          <div className="flex items-start">
            <span className="text-purple-500 text-2xl mr-3">📂</span>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-purple-900 mb-2">Docker Compose 配置文件</h3>
              <div className="space-y-2">
                <div className="flex items-center">
                  <span className="text-sm text-purple-800 mr-2">当前路径:</span>
                  <code className="text-sm bg-purple-100 px-2 py-1 rounded font-mono text-purple-900">
                    {dockerComposePath}
                  </code>
                  {isCustomPath && (
                    <span className="ml-2 px-2 py-0.5 bg-purple-200 text-purple-900 text-xs rounded">自定义</span>
                  )}
                </div>

                <div className="flex space-x-2 mt-3">
                  <Button
                    size="sm"
                    onClick={() => setShowPathConfig(!showPathConfig)}
                    icon={showPathConfig ? '−' : '⚙️'}
                    variant="outline"
                  >
                    {showPathConfig ? '关闭设置' : '配置路径'}
                  </Button>
                  {isCustomPath && (
                    <Button
                      size="sm"
                      onClick={handleResetPath}
                      icon="↺"
                      variant="outline"
                    >
                      重置为自动检测
                    </Button>
                  )}
                </div>

                {showPathConfig && (
                  <div className="mt-4 p-4 bg-white rounded-lg border border-purple-200">
                    <h4 className="font-semibold text-gray-900 mb-3">设置自定义路径</h4>
                    <p className="text-sm text-gray-600 mb-3">
                      输入 docker-compose.yml 文件的完整路径。例如: /opt/frigate/docker-compose.yml
                    </p>
                    <div className="flex space-x-2">
                      <input
                        type="text"
                        value={customPathInput}
                        onChange={(e) => setCustomPathInput(e.target.value)}
                        placeholder="/path/to/docker-compose.yml"
                        className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-purple-500"
                      />
                      <Button
                        size="sm"
                        onClick={handleSetCustomPath}
                        disabled={!customPathInput.trim()}
                        icon="✓"
                      >
                        应用
                      </Button>
                      <Button
                        size="sm"
                        onClick={() => {
                          setShowPathConfig(false)
                          setCustomPathInput('')
                        }}
                        variant="ghost"
                      >
                        取消
                      </Button>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </Card>
      )}

      {/* Phase 4 - 增强的成功消息 */}
      {addSuccess && lastAddResult && (
        <div className="mb-6 bg-green-50 border border-green-200 rounded-lg p-4">
          <div className="flex items-start">
            <span className="text-green-500 text-2xl mr-3">✓</span>
            <div className="flex-1">
              <p className="text-green-800 font-medium mb-2">{addSuccess}</p>

              {/* 详细信息 */}
              <div className="space-y-1 text-sm">
                {lastAddResult.docker_compose_updated && (
                  <div className="flex items-center text-green-700">
                    <span className="mr-2">📝</span>
                    <span>docker-compose.yml 已更新</span>
                  </div>
                )}

                {lastAddResult.docker_compose_path && (
                  <div className="flex items-center text-green-700">
                    <span className="mr-2">📂</span>
                    <span>文件路径: <code className="bg-green-100 px-1 rounded font-mono text-xs">{lastAddResult.docker_compose_path}</code></span>
                  </div>
                )}

                {lastAddResult.frigate_service_name && (
                  <div className="flex items-center text-green-700">
                    <span className="mr-2">🎯</span>
                    <span>服务名称: <code className="bg-green-100 px-1 rounded font-mono text-xs">{lastAddResult.frigate_service_name}</code></span>
                  </div>
                )}

                {lastAddResult.warnings && lastAddResult.warnings.length > 0 && (
                  <div className="mt-2 p-2 bg-yellow-50 border border-yellow-200 rounded">
                    <p className="text-yellow-800 font-medium text-xs mb-1">⚠️ 警告:</p>
                    {lastAddResult.warnings.map((warning, idx) => (
                      <p key={idx} className="text-yellow-700 text-xs">• {warning}</p>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Preset Hardware Selector for Docker/Browser Mode */}
      {!isInTauriEnv && (
        <Card className="mb-6 bg-gradient-to-r from-blue-50 to-indigo-50 border border-blue-200">
          <div className="flex items-start">
            <span className="text-blue-500 text-2xl mr-3">🔧</span>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-blue-900 mb-2">选择硬件加速器</h3>
              <p className="text-sm text-blue-800 mb-3">
                请选择您的硬件设备,系统将自动添加相关参数到 docker-compose.yml 文件中。
              </p>

              <div className="flex space-x-2 mb-3">
                <Button
                  size="sm"
                  onClick={() => setShowPresetSelector(!showPresetSelector)}
                  icon={showPresetSelector ? '−' : '+'}
                  variant="outline"
                >
                  {showPresetSelector ? '关闭选择器' : '添加硬件设备'}
                </Button>

                <Button
                  size="sm"
                  onClick={handleScanPciDevices}
                  icon="🔍"
                  variant="outline"
                  loading={scanningPci}
                >
                  {scanningPci ? '扫描中...' : '扫描 PCI 设备'}
                </Button>
              </div>

              {/* PCI Device Scanner Results */}
              {showPciDevices && pciDevices.length > 0 && (
                <div className="mt-4 p-5 bg-white rounded-lg border border-green-200 shadow-sm">
                  <div className="flex justify-between items-center mb-4">
                    <h4 className="font-semibold text-gray-900">检测到的 GPU 设备 ({pciDevices.length})</h4>
                    <button
                      onClick={() => setShowPciDevices(false)}
                      className="text-gray-500 hover:text-gray-700"
                    >
                      ✕
                    </button>
                  </div>

                  <div className="space-y-3">
                    {pciDevices.map((pci) => (
                      <div
                        key={pci.slot}
                        className="p-4 border border-gray-200 rounded-lg hover:border-green-400 transition-colors"
                      >
                        <div className="flex justify-between items-start">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-2">
                              <span className="text-xl">
                                {pci.is_nvidia ? '🟢' : pci.is_amd ? '🔴' : '🔵'}
                              </span>
                              <h5 className="font-semibold text-gray-900">{pci.vendor}</h5>
                              {pci.is_nvidia && <span className="px-2 py-0.5 bg-green-100 text-green-800 text-xs rounded">NVIDIA</span>}
                              {pci.is_amd && <span className="px-2 py-0.5 bg-red-100 text-red-800 text-xs rounded">AMD</span>}
                              {pci.is_intel && <span className="px-2 py-0.5 bg-blue-100 text-blue-800 text-xs rounded">Intel</span>}
                            </div>
                            <p className="text-sm text-gray-600 mb-1">{pci.device}</p>
                            <p className="text-xs text-gray-500 font-mono">PCI 插槽: {pci.slot}</p>
                            {pci.recommended_device_path && (
                              <div className="mt-2 p-2 bg-gray-50 rounded">
                                <p className="text-xs text-gray-600 mb-1">推荐设备路径:</p>
                                <code className="text-xs text-blue-600">{pci.recommended_device_path}</code>
                              </div>
                            )}
                          </div>
                          <Button
                            size="sm"
                            onClick={() => handleAddPciDevice(pci)}
                            loading={addingDevice === pci.slot}
                            icon="+"
                            className="ml-3"
                          >
                            {addingDevice === pci.slot ? '添加中...' : '添加'}
                          </Button>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {showPresetSelector && (
                <div className="mt-4 p-5 bg-white rounded-lg border border-blue-200 shadow-sm">
                  <h4 className="font-semibold text-gray-900 mb-4">选择硬件类型</h4>

                  {/* Type Filter */}
                  <div className="flex flex-wrap gap-2 mb-4">
                    <button
                      onClick={() => setPresetFilter('all')}
                      className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${
                        presetFilter === 'all' ? 'bg-blue-600 text-white' : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                      }`}
                    >
                      全部
                    </button>
                    <button
                      onClick={() => setPresetFilter('gpu')}
                      className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${
                        presetFilter === 'gpu' ? 'bg-blue-600 text-white' : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                      }`}
                    >
                      🎮 GPU
                    </button>
                    <button
                      onClick={() => setPresetFilter('tpu')}
                      className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${
                        presetFilter === 'tpu' ? 'bg-blue-600 text-white' : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                      }`}
                    >
                      🧠 TPU/NPU
                    </button>
                    <button
                      onClick={() => setPresetFilter('camera')}
                      className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${
                        presetFilter === 'camera' ? 'bg-blue-600 text-white' : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                      }`}
                    >
                      📷 摄像头
                    </button>
                  </div>

                  {/* Preset List */}
                  <div className="space-y-2 mb-4 max-h-80 overflow-y-auto">
                    {getFilteredPresets().map((preset) => (
                      <label
                        key={preset.id}
                        className={`flex items-start p-3 border rounded-lg cursor-pointer transition-all ${
                          selectedPreset === preset.id
                            ? 'border-blue-500 bg-blue-50 shadow-sm'
                            : 'border-gray-200 hover:border-blue-300 hover:bg-gray-50'
                        }`}
                      >
                        <input
                          type="radio"
                          name="preset"
                          value={preset.id}
                          checked={selectedPreset === preset.id}
                          onChange={(e) => setSelectedPreset(e.target.value)}
                          className="mt-1 mr-3"
                        />
                        <div className="flex-1">
                          <div className="font-semibold text-gray-900">{preset.name}</div>
                          <div className="text-xs text-gray-500 font-mono mt-1">{preset.devicePath}</div>
                          <div className="text-sm text-gray-600 mt-1">{preset.description}</div>
                        </div>
                        <span className="text-xl ml-2">{getDeviceIcon(preset.type)}</span>
                      </label>
                    ))}
                  </div>

                  {/* Action Buttons */}
                  <div className="flex space-x-2 pt-3 border-t border-gray-200">
                    <Button
                      onClick={handlePresetAdd}
                      loading={addingDevice !== null}
                      disabled={!selectedPreset}
                      icon="+"
                      className="flex-1"
                    >
                      {addingDevice ? '添加中...' : '添加到配置'}
                    </Button>
                    <Button
                      onClick={() => {
                        setShowPresetSelector(false)
                        setSelectedPreset('')
                      }}
                      variant="ghost"
                    >
                      取消
                    </Button>
                  </div>
                </div>
              )}
            </div>
          </div>
        </Card>
      )}

      {/* Success Message - 简化版本 (如果没有详细结果) */}
      {addSuccess && !lastAddResult && (
        <div className="mb-6 bg-green-50 border border-green-200 rounded-lg p-4">
          <div className="flex items-center">
            <span className="text-green-500 text-2xl mr-3">✓</span>
            <p className="text-green-800 font-medium">{addSuccess}</p>
          </div>
        </div>
      )}

      {/* Actions Bar */}
      <div className="mb-6 flex items-center justify-between bg-white p-4 rounded-lg shadow-sm">
        <div className="flex space-x-2">
          <Button
            onClick={handleDetect}
            loading={detecting}
            disabled={!isInTauriEnv}
            icon="🔄"
          >
            {detecting ? '检测中...' : '重新检测'}
          </Button>
          <Button
            onClick={handleCheckAvailability}
            loading={checkingAvailability}
            disabled={!isInTauriEnv}
            icon="📊"
            variant="outline"
          >
            {checkingAvailability ? '检查中...' : '检查可用性'}
          </Button>
        </div>

        {/* Device Count */}
        <div className="text-sm text-gray-600">
          找到 <span className="font-semibold text-gray-900">{filteredDevices.length}</span> 个设备
        </div>
      </div>

      {/* Filter Buttons */}
      <div className="mb-6 flex space-x-2">
        <FilterButton
          active={selectedFilter === 'all'}
          onClick={() => setSelectedFilter('all')}
          count={devices.length}
        >
          全部
        </FilterButton>
        <FilterButton
          active={selectedFilter === 'gpu'}
          onClick={() => setSelectedFilter('gpu')}
          count={devices.filter(d => d.type === 'gpu').length}
          icon="🎮"
        >
          GPU
        </FilterButton>
        <FilterButton
          active={selectedFilter === 'tpu'}
          onClick={() => setSelectedFilter('tpu')}
          count={devices.filter(d => d.type === 'tpu').length}
          icon="🧠"
        >
          TPU
        </FilterButton>
        <FilterButton
          active={selectedFilter === 'camera'}
          onClick={() => setSelectedFilter('camera')}
          count={devices.filter(d => d.type === 'camera').length}
          icon="📷"
        >
          相机
        </FilterButton>
        <FilterButton
          active={selectedFilter === 'capture_card'}
          onClick={() => setSelectedFilter('capture_card')}
          count={devices.filter(d => d.type === 'capture_card').length}
          icon="📹"
        >
          采集卡
        </FilterButton>
      </div>

      {/* Error Display */}
      {detectionError && (
        <Card className="mb-6 border-red-200 bg-red-50">
          <div className="flex items-start">
            <span className="text-2xl mr-3">⚠️</span>
            <div>
              <h3 className="font-semibold text-red-900 mb-1">检测失败</h3>
              <p className="text-red-700">{detectionError.message}</p>
              <Button
                variant="ghost"
                onClick={handleDetect}
                className="mt-2"
              >
                重试
              </Button>
            </div>
          </div>
        </Card>
      )}

      {/* Device Grid */}
      {filteredDevices.length === 0 && !detecting && !detectionError && (
        <Card className="text-center py-12">
          <div className="text-6xl mb-4">🔍</div>
          <h3 className="text-xl font-semibold text-gray-900 mb-2">
            未检测到设备
          </h3>
          <p className="text-gray-600 mb-4">
            点击"重新检测"按钮扫描硬件设备
          </p>
        </Card>
      )}

      {/* CPU-Only Warning Banner */}
      {devices.some(d => d.id === 'cpu-only-warning') && (
        <Card className="mb-6 border-yellow-200 bg-yellow-50">
          <div className="flex items-start">
            <span className="text-3xl mr-3">⚠️</span>
            <div className="flex-1">
              <h3 className="font-semibold text-yellow-900 mb-2">CPU-Only 模式检测</h3>
              <p className="text-yellow-800 mb-2">
                未检测到 GPU 或 TPU 硬件加速器。系统将使用 CPU 进行视频处理,性能可能受限。
              </p>
              <div className="bg-yellow-100 rounded p-3 text-sm">
                <p className="font-medium text-yellow-900 mb-1">💡 建议:</p>
                <ul className="list-disc list-inside text-yellow-800 space-y-1">
                  <li>考虑添加 GPU (NVIDIA/AMD/Intel) 或 TPU (Google Coral/Hailo) 以提升性能</li>
                  <li>视频编码/解码可能会导致 CPU 使用率过高</li>
                  <li>实时视频分析可能受到帧率限制</li>
                </ul>
              </div>
            </div>
          </div>
        </Card>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {filteredDevices.map((device) => {
          // Check if this is a CPU-only warning device
          const isCPUWarning = device.id === 'cpu-only-warning';
          const cardClassName = isCPUWarning
            ? "cursor-pointer hover:shadow-lg transition-shadow border-2 border-yellow-300 bg-yellow-50"
            : "cursor-pointer hover:shadow-lg transition-shadow";

          return (
          <Card
            key={device.id}
            className={cardClassName}
            onClick={() => handleDeviceClick(device)}
          >
            {/* Device Header */}
            <div className="flex items-start mb-3">
              <span className="text-3xl mr-3">
                {isCPUWarning ? '⚠️' : getDeviceIcon(device.type)}
              </span>
              <div className="flex-1 min-w-0">
                <h3 className={`font-semibold truncate mb-1 ${isCPUWarning ? 'text-yellow-900' : 'text-gray-900'}`}>
                  {device.name}
                </h3>
                <span className={`inline-block px-2 py-1 text-xs font-medium rounded-full ${
                  isCPUWarning ? 'bg-yellow-200 text-yellow-900' : 'bg-blue-100 text-blue-800'
                }`}>
                  {getDeviceTypeLabel(device.type)}
                </span>
              </div>
              <div className="ml-2">
                {device.available ? (
                  <span className="text-green-500 text-xl" title="可用">✓</span>
                ) : (
                  <span className="text-red-500 text-xl" title="不可用">✗</span>
                )}
              </div>
            </div>

            {/* Device Path */}
            <div className="text-sm text-gray-600 mb-2 font-mono text-xs">
              {device.device_path}
            </div>

            {/* Capabilities */}
            <div className="flex flex-wrap gap-1">
              {device.capabilities.slice(0, 3).map((cap) => (
                <span
                  key={cap}
                  className="px-2 py-1 text-xs bg-gray-100 text-gray-700 rounded"
                >
                  {cap}
                </span>
              ))}
              {device.capabilities.length > 3 && (
                <span className="px-2 py-1 text-xs bg-gray-100 text-gray-700 rounded">
                  +{device.capabilities.length - 3}
                </span>
              )}
            </div>

            {/* In Use Badge */}
            {device.in_use && (
              <div className="mt-2 text-xs text-orange-600">
                🔸 使用中
              </div>
            )}

            {/* Action Buttons */}
            <div className="mt-3 flex flex-col space-y-2">
              <div className="flex space-x-2">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={(e) => {
                    e.stopPropagation()
                    handleDeviceClick(device)
                  }}
                >
                  详情
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={(e) => {
                    e.stopPropagation()
                    handleAvailabilityClick(device)
                  }}
                >
                  可用性
                </Button>
              </div>
              {!isCPUWarning && device.available && (
                <Button
                  size="sm"
                  onClick={(e) => {
                    e.stopPropagation()
                    handleAddToConfig(device)
                  }}
                  loading={addingDevice === device.id}
                  icon="+"
                  className="w-full"
                >
                  {addingDevice === device.id ? '添加中...' : '添加到配置'}
                </Button>
              )}
            </div>
          </Card>
          );
        })}
      </div>

      {/* Device Details Modal */}
      <Modal
        isOpen={showDetailsModal}
        onClose={() => setShowDetailsModal(false)}
        title="设备详情"
      >
        {selectedDevice && (
          <div className="space-y-4">
            <div className="flex items-center">
              <span className="text-4xl mr-4">{getDeviceIcon(selectedDevice.type)}</span>
              <div>
                <h3 className="text-xl font-semibold">{selectedDevice.name}</h3>
                <span className="text-sm text-gray-600">{selectedDevice.id}</span>
              </div>
            </div>

            <div className="border-t pt-4 space-y-3">
              <DetailRow label="类型" value={getDeviceTypeLabel(selectedDevice.type)} />
              <DetailRow label="设备路径" value={selectedDevice.device_path} mono />
              <DetailRow label="平台" value={selectedDevice.platform} />
              <DetailRow label="架构" value={selectedDevice.architecture} />
              <DetailRow label="检测来源" value={selectedDevice.detection_source} />
              {selectedDevice.driver && (
                <DetailRow label="驱动" value={selectedDevice.driver} />
              )}
              {selectedDevice.vendor_id && (
                <DetailRow label="厂商 ID" value={selectedDevice.vendor_id} mono />
              )}
              <DetailRow
                label="状态"
                value={selectedDevice.available ? '可用' : '不可用'}
                valueColor={selectedDevice.available ? 'text-green-600' : 'text-red-600'}
              />
              <DetailRow
                label="使用状态"
                value={selectedDevice.in_use ? '使用中' : '空闲'}
                valueColor={selectedDevice.in_use ? 'text-orange-600' : 'text-gray-600'}
              />
            </div>

            <div className="border-t pt-4">
              <h4 className="font-semibold mb-2">能力列表</h4>
              <div className="flex flex-wrap gap-2">
                {selectedDevice.capabilities.map((cap) => (
                  <span
                    key={cap}
                    className="px-3 py-1 text-sm bg-blue-100 text-blue-800 rounded-full"
                  >
                    {cap}
                  </span>
                ))}
              </div>
            </div>

            {selectedDevice.error && (
              <div className="border-t pt-4">
                <h4 className="font-semibold text-red-600 mb-2">错误信息</h4>
                <p className="text-sm text-red-700">{selectedDevice.error}</p>
              </div>
            )}
          </div>
        )}
      </Modal>

      {/* Hardware Availability Modal */}
      <Modal
        isOpen={showAvailabilityModal}
        onClose={() => setShowAvailabilityModal(false)}
        title="硬件可用性状态"
        size="lg"
      >
        {selectedAvailability && (
          <div className="space-y-4">
            <div className="flex items-center">
              <span className="text-4xl mr-4">
                {selectedAvailability.available ? '✅' : '❌'}
              </span>
              <div>
                <h3 className="text-xl font-semibold">{selectedAvailability.device_name}</h3>
                <span className="text-sm text-gray-600">{selectedAvailability.device_type.toUpperCase()}</span>
              </div>
            </div>

            <div className="border-t pt-4 space-y-4">
              {/* Status Overview */}
              <div className="grid grid-cols-2 gap-4">
                <Card className="p-4">
                  <h4 className="font-semibold text-sm text-gray-600 mb-2">基本状态</h4>
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-sm">可用性:</span>
                      <span className={`text-sm font-medium ${selectedAvailability.available ? 'text-green-600' : 'text-red-600'}`}>
                        {selectedAvailability.available ? '可用' : '不可用'}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm">使用状态:</span>
                      <span className={`text-sm font-medium ${selectedAvailability.in_use ? 'text-orange-600' : 'text-gray-600'}`}>
                        {selectedAvailability.in_use ? '使用中' : '空闲'}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm">最后检查:</span>
                      <span className="text-sm font-medium">
                        {new Date(selectedAvailability.last_checked).toLocaleString()}
                      </span>
                    </div>
                  </div>
                </Card>

                {/* Performance Metrics */}
                <Card className="p-4">
                  <h4 className="font-semibold text-sm text-gray-600 mb-2">性能指标</h4>
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-sm">使用率:</span>
                      <span className="text-sm font-medium">
                        {selectedAvailability.usage.utilization.toFixed(1)}%
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm">温度:</span>
                      <span className={`text-sm font-medium ${selectedAvailability.thermal.temperature > 80 ? 'text-red-600' : 'text-gray-600'}`}>
                        {selectedAvailability.thermal.temperature.toFixed(1)}°C
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm">功耗:</span>
                      <span className="text-sm font-medium">
                        {selectedAvailability.power.current_power.toFixed(1)}W
                      </span>
                    </div>
                  </div>
                </Card>
              </div>

              {/* Memory Usage */}
              <Card className="p-4">
                <h4 className="font-semibold text-sm text-gray-600 mb-2">内存使用</h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm">已使用:</span>
                    <span className="text-sm font-medium">
                      {(selectedAvailability.usage.memory_used / 1024 / 1024).toFixed(1)}MB
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">总计:</span>
                    <span className="text-sm font-medium">
                      {(selectedAvailability.usage.memory_total / 1024 / 1024).toFixed(1)}MB
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">利用率:</span>
                    <span className="text-sm font-medium">
                      {selectedAvailability.usage.memory_utilization.toFixed(1)}%
                    </span>
                  </div>
                </div>
              </Card>

              {/* Processes */}
              {selectedAvailability.processes.length > 0 && (
                <Card className="p-4">
                  <h4 className="font-semibold text-sm text-gray-600 mb-2">正在运行的进程</h4>
                  <div className="space-y-2">
                    {selectedAvailability.processes.map((process) => (
                      <div key={process.pid} className="flex justify-between text-sm">
                        <span>{process.name}</span>
                        <span className="text-gray-600">PID: {process.pid}</span>
                      </div>
                    ))}
                  </div>
                </Card>
              )}

              {/* Errors and Warnings */}
              {(selectedAvailability.errors.length > 0 || selectedAvailability.warnings.length > 0) && (
                <Card className="p-4">
                  <h4 className="font-semibold text-sm text-gray-600 mb-2">问题和警告</h4>
                  {selectedAvailability.errors.length > 0 && (
                    <div className="mb-2">
                      <h5 className="text-sm font-medium text-red-600 mb-1">错误:</h5>
                      {selectedAvailability.errors.map((error, index) => (
                        <div key={index} className="text-sm text-red-700">
                          • {error}
                        </div>
                      ))}
                    </div>
                  )}
                  {selectedAvailability.warnings.length > 0 && (
                    <div>
                      <h5 className="text-sm font-medium text-yellow-600 mb-1">警告:</h5>
                      {selectedAvailability.warnings.map((warning, index) => (
                        <div key={index} className="text-sm text-yellow-700">
                          • {warning}
                        </div>
                      ))}
                    </div>
                  )}
                </Card>
              )}
            </div>
          </div>
        )}
      </Modal>
    </div>
  )
}

// Filter Button Component
interface FilterButtonProps {
  active: boolean
  onClick: () => void
  count: number
  icon?: string
  children: React.ReactNode
}

const FilterButton: React.FC<FilterButtonProps> = ({
  active,
  onClick,
  count,
  icon,
  children,
}) => (
  <button
    onClick={onClick}
    className={`
      px-4 py-2 rounded-lg text-sm font-medium transition-colors
      ${active
        ? 'bg-blue-600 text-white shadow-sm'
        : 'bg-white text-gray-700 hover:bg-gray-50 border border-gray-300'
      }
    `}
  >
    {icon && <span className="mr-1">{icon}</span>}
    {children}
    <span className="ml-2 opacity-75">({count})</span>
  </button>
)

// Detail Row Component
interface DetailRowProps {
  label: string
  value: string
  mono?: boolean
  valueColor?: string
}

const DetailRow: React.FC<DetailRowProps> = ({ label, value, mono, valueColor }) => (
  <div className="flex justify-between">
    <span className="text-sm text-gray-600">{label}:</span>
    <span className={`text-sm font-medium ${valueColor || 'text-gray-900'} ${mono ? 'font-mono' : ''}`}>
      {value}
    </span>
  </div>
)

export default HardwarePage
