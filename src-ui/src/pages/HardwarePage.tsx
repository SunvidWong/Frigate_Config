// Hardware Detection Page - T037-T043
// Displays detected hardware devices and allows user to refresh detection

import React, { useState, useEffect } from 'react'
import { useTauriCommand } from '../hooks/useTauriCommand'
import Button from '../components/Button'
import Card from '../components/Card'
import Modal from '../components/Modal'
import type { HardwareDevice, DeviceType, HardwareAvailability } from '../types'

const HardwarePage: React.FC = () => {
  const [devices, setDevices] = useState<HardwareDevice[]>([])
  const [filteredDevices, setFilteredDevices] = useState<HardwareDevice[]>([])
  const [selectedFilter, setSelectedFilter] = useState<DeviceType | 'all'>('all')
  const [selectedDevice, setSelectedDevice] = useState<HardwareDevice | null>(null)
  const [showDetailsModal, setShowDetailsModal] = useState(false)
  const [availabilityData, setAvailabilityData] = useState<HardwareAvailability[]>([])
  const [showAvailabilityModal, setShowAvailabilityModal] = useState(false)
  const [selectedAvailability, setSelectedAvailability] = useState<HardwareAvailability | null>(null)

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

  // Initial detection on mount
  useEffect(() => {
    runDetection()
  }, [])

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

  const getDeviceIcon = (type: DeviceType | 'cpu'): string => {
    const icons: Record<string, string> = {
      gpu: '🎮',
      tpu: '🧠',
      camera: '📷',
      capture_card: '📹',
      cpu: '⚠️',
    }
    return icons[type] || '❓'
  }

  const getDeviceTypeLabel = (type: DeviceType | 'cpu'): string => {
    const labels: Record<string, string> = {
      gpu: 'GPU',
      tpu: 'TPU',
      camera: '相机',
      capture_card: '采集卡',
      cpu: 'CPU-Only',
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

      {/* Actions Bar */}
      <div className="mb-6 flex items-center justify-between bg-white p-4 rounded-lg shadow-sm">
        <div className="flex space-x-2">
          <Button
            onClick={handleDetect}
            loading={detecting}
            icon="🔄"
          >
            {detecting ? '检测中...' : '重新检测'}
          </Button>
          <Button
            onClick={handleCheckAvailability}
            loading={checkingAvailability}
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
          const isCPUWarning = device.id === 'cpu-only-warning' || device.type === 'cpu';
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
            <div className="mt-3 flex space-x-2">
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
