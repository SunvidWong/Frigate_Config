// Cameras Configuration Page - T044-T053
// Complete CRUD interface for camera management with hardware assignment

import React, { useState, useEffect } from 'react'
import { useTauriCommand } from '../hooks/useTauriCommand'
import Button from '../components/Button'
import Card from '../components/Card'
import Modal from '../components/Modal'
import Input from '../components/Input'
import type { CameraConfiguration, HardwareDevice, ValidationStatus } from '../types'

// Generate UUID for new cameras
const generateId = () => `cam-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`

const CamerasPage: React.FC = () => {
  // State
  const [cameras, setCameras] = useState<CameraConfiguration[]>([])
  const [devices, setDevices] = useState<HardwareDevice[]>([])
  const [filteredCameras, setFilteredCameras] = useState<CameraConfiguration[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [filterStatus, setFilterStatus] = useState<'all' | 'enabled' | 'disabled' | 'errors'>('all')

  // Modals
  const [showAddModal, setShowAddModal] = useState(false)
  const [showEditModal, setShowEditModal] = useState(false)
  const [showDeleteModal, setShowDeleteModal] = useState(false)
  const [selectedCamera, setSelectedCamera] = useState<CameraConfiguration | null>(null)

  // Form state
  const [formData, setFormData] = useState<Partial<CameraConfiguration>>({})
  const [formErrors, setFormErrors] = useState<string[]>([])

  // API calls
  const { execute: fetchHardware } = useTauriCommand<HardwareDevice[]>('detect_hardware')

  // Load hardware devices on mount
  useEffect(() => {
    fetchHardware().then(data => {
      if (data) {
        // Filter to only GPU/TPU devices
        setDevices(data.filter(d => d.type === 'gpu' || d.type === 'tpu'))
      }
    })
  }, [])

  // Filter cameras
  useEffect(() => {
    let filtered = cameras

    // Apply status filter
    if (filterStatus !== 'all') {
      filtered = filtered.filter(cam => {
        switch (filterStatus) {
          case 'enabled':
            return cam.enabled
          case 'disabled':
            return !cam.enabled
          case 'errors':
            return cam.validation_status === 'error'
          default:
            return true
        }
      })
    }

    // Apply search
    if (searchQuery) {
      const query = searchQuery.toLowerCase()
      filtered = filtered.filter(cam =>
        cam.name.toLowerCase().includes(query) ||
        cam.rtsp_url.toLowerCase().includes(query)
      )
    }

    setFilteredCameras(filtered)
  }, [cameras, searchQuery, filterStatus])

  // Handlers
  const handleAddCamera = () => {
    setFormData({
      id: generateId(),
      name: '',
      rtsp_url: '',
      enabled: true,
      hardware_device_id: undefined,
      resolution: { width: 1920, height: 1080 },
      fps: 10,
      detect_enabled: true,
      record_enabled: true,
      snapshots_enabled: true,
      validation_status: 'valid',
      validation_errors: [],
    })
    setFormErrors([])
    setShowAddModal(true)
  }

  const handleEditCamera = (camera: CameraConfiguration) => {
    setSelectedCamera(camera)
    setFormData({ ...camera })
    setFormErrors([])
    setShowEditModal(true)
  }

  const handleDeleteCamera = (camera: CameraConfiguration) => {
    setSelectedCamera(camera)
    setShowDeleteModal(true)
  }

  const validateForm = (): boolean => {
    const errors: string[] = []

    if (!formData.name || formData.name.trim().length === 0) {
      errors.push('相机名称不能为空')
    }

    if (!formData.rtsp_url || !formData.rtsp_url.startsWith('rtsp://')) {
      errors.push('RTSP URL 必须以 rtsp:// 开头')
    }

    if (formData.fps && (formData.fps < 1 || formData.fps > 60)) {
      errors.push('FPS 必须在 1-60 之间')
    }

    if (formData.resolution) {
      if (formData.resolution.width % 2 !== 0 || formData.resolution.height % 2 !== 0) {
        errors.push('分辨率的宽度和高度必须是偶数')
      }
    }

    setFormErrors(errors)
    return errors.length === 0
  }

  const handleSaveCamera = () => {
    if (!validateForm()) return

    const now = new Date().toISOString()
    const newCamera: CameraConfiguration = {
      ...formData as CameraConfiguration,
      created_at: formData.created_at || now,
      updated_at: now,
      rtsp_url_display: maskCredentials(formData.rtsp_url || ''),
      manually_edited: false,
    }

    if (showEditModal) {
      // Update existing
      setCameras(prev => prev.map(c => c.id === newCamera.id ? newCamera : c))
    } else {
      // Add new
      setCameras(prev => [...prev, newCamera])
    }

    setShowAddModal(false)
    setShowEditModal(false)
  }

  const confirmDelete = () => {
    if (selectedCamera) {
      setCameras(prev => prev.filter(c => c.id !== selectedCamera.id))
      setShowDeleteModal(false)
      setSelectedCamera(null)
    }
  }

  const toggleEnabled = (cameraId: string) => {
    setCameras(prev => prev.map(c =>
      c.id === cameraId ? { ...c, enabled: !c.enabled } : c
    ))
  }

  const maskCredentials = (url: string): string => {
    const atIndex = url.indexOf('@')
    if (atIndex === -1) return url

    const protocolEnd = url.indexOf('://') + 3
    return url.substring(0, protocolEnd) + '***' + url.substring(atIndex)
  }

  const getValidationBadge = (status: ValidationStatus) => {
    const badges = {
      valid: { color: 'bg-green-100 text-green-800', label: '有效' },
      warning: { color: 'bg-yellow-100 text-yellow-800', label: '警告' },
      error: { color: 'bg-red-100 text-red-800', label: '错误' },
    }
    const badge = badges[status as keyof typeof badges]
    return (
      <span className={`px-2 py-1 text-xs font-medium rounded-full ${badge.color}`}>
        {badge.label}
      </span>
    )
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">相机配置</h1>
        <p className="text-gray-600">管理相机并分配硬件加速器</p>
      </div>

      {/* Actions Bar */}
      <div className="mb-6 bg-white p-4 rounded-lg shadow-sm">
        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div className="flex items-center space-x-4">
            <Button onClick={handleAddCamera} icon="➕">
              添加相机
            </Button>
            <span className="text-sm text-gray-600">
              共 <span className="font-semibold">{filteredCameras.length}</span> 台相机
            </span>
          </div>

          {/* Search */}
          <div className="flex-1 max-w-md">
            <Input
              type="text"
              placeholder="搜索相机名称或 URL..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>
      </div>

      {/* Filter Tabs */}
      <div className="mb-6 flex space-x-2">
        <FilterTab
          active={filterStatus === 'all'}
          onClick={() => setFilterStatus('all')}
          count={cameras.length}
        >
          全部
        </FilterTab>
        <FilterTab
          active={filterStatus === 'enabled'}
          onClick={() => setFilterStatus('enabled')}
          count={cameras.filter(c => c.enabled).length}
        >
          已启用
        </FilterTab>
        <FilterTab
          active={filterStatus === 'disabled'}
          onClick={() => setFilterStatus('disabled')}
          count={cameras.filter(c => !c.enabled).length}
        >
          已禁用
        </FilterTab>
        <FilterTab
          active={filterStatus === 'errors'}
          onClick={() => setFilterStatus('errors')}
          count={cameras.filter(c => c.validation_status === 'error').length}
        >
          有错误
        </FilterTab>
      </div>

      {/* Empty State */}
      {cameras.length === 0 && (
        <Card className="text-center py-12">
          <div className="text-6xl mb-4">📷</div>
          <h3 className="text-xl font-semibold text-gray-900 mb-2">还没有相机</h3>
          <p className="text-gray-600 mb-4">点击"添加相机"按钮创建第一台相机</p>
          <Button onClick={handleAddCamera}>添加相机</Button>
        </Card>
      )}

      {/* Camera List */}
      <div className="space-y-4">
        {filteredCameras.map((camera) => (
          <Card key={camera.id} className={!camera.enabled ? 'opacity-60' : ''}>
            <div className="flex items-start justify-between">
              {/* Camera Info */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center mb-2">
                  <h3 className="text-lg font-semibold text-gray-900 mr-3">
                    {camera.name}
                  </h3>
                  {getValidationBadge(camera.validation_status)}
                  {camera.hardware_device_id && (
                    <span className="ml-2 px-2 py-1 text-xs bg-purple-100 text-purple-800 rounded">
                      🎮 已分配硬件
                    </span>
                  )}
                </div>

                <div className="text-sm text-gray-600 mb-2 font-mono">
                  {camera.rtsp_url_display}
                </div>

                <div className="flex items-center space-x-4 text-sm text-gray-600">
                  <span>分辨率: {camera.resolution.width}x{camera.resolution.height}</span>
                  <span>FPS: {camera.fps}</span>
                  {camera.detect_enabled && <span className="text-blue-600">🔍 检测</span>}
                  {camera.record_enabled && <span className="text-red-600">⏺️ 录制</span>}
                  {camera.snapshots_enabled && <span className="text-green-600">📸 快照</span>}
                </div>

                {camera.validation_errors.length > 0 && (
                  <div className="mt-2 text-sm text-red-600">
                    ⚠️ {camera.validation_errors.join(', ')}
                  </div>
                )}
              </div>

              {/* Actions */}
              <div className="flex items-center space-x-2 ml-4">
                {/* Enable/Disable Toggle */}
                <button
                  onClick={() => toggleEnabled(camera.id)}
                  className={`
                    relative inline-flex h-6 w-11 items-center rounded-full transition-colors
                    ${camera.enabled ? 'bg-blue-600' : 'bg-gray-300'}
                  `}
                  title={camera.enabled ? '点击禁用' : '点击启用'}
                >
                  <span
                    className={`
                      inline-block h-4 w-4 transform rounded-full bg-white transition-transform
                      ${camera.enabled ? 'translate-x-6' : 'translate-x-1'}
                    `}
                  />
                </button>

                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => handleEditCamera(camera)}
                >
                  编辑
                </Button>
                <Button
                  variant="danger"
                  size="sm"
                  onClick={() => handleDeleteCamera(camera)}
                >
                  删除
                </Button>
              </div>
            </div>
          </Card>
        ))}
      </div>

      {/* Add/Edit Modal */}
      <Modal
        isOpen={showAddModal || showEditModal}
        onClose={() => {
          setShowAddModal(false)
          setShowEditModal(false)
        }}
        title={showEditModal ? '编辑相机' : '添加相机'}
      >
        <div className="space-y-4">
          {/* Form Errors */}
          {formErrors.length > 0 && (
            <div className="bg-red-50 border border-red-200 rounded p-3">
              <p className="text-sm font-semibold text-red-800 mb-1">请修正以下错误：</p>
              <ul className="list-disc list-inside text-sm text-red-700">
                {formErrors.map((error, i) => (
                  <li key={i}>{error}</li>
                ))}
              </ul>
            </div>
          )}

          {/* Camera Name */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              相机名称 *
            </label>
            <Input
              type="text"
              placeholder="例如: front_door, backyard"
              value={formData.name || ''}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            />
            <p className="text-xs text-gray-500 mt-1">用于 Frigate 配置中的相机标识符</p>
          </div>

          {/* RTSP URL */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              RTSP URL *
            </label>
            <Input
              type="text"
              placeholder="rtsp://username:password@192.168.1.100:554/stream1"
              value={formData.rtsp_url || ''}
              onChange={(e) => setFormData({ ...formData, rtsp_url: e.target.value })}
            />
            <p className="text-xs text-gray-500 mt-1">包含用户名和密码的完整 RTSP URL</p>
          </div>

          {/* Resolution */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">宽度</label>
              <Input
                type="number"
                value={formData.resolution?.width || 1920}
                onChange={(e) => setFormData({
                  ...formData,
                  resolution: { ...formData.resolution!, width: parseInt(e.target.value) }
                })}
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">高度</label>
              <Input
                type="number"
                value={formData.resolution?.height || 1080}
                onChange={(e) => setFormData({
                  ...formData,
                  resolution: { ...formData.resolution!, height: parseInt(e.target.value) }
                })}
              />
            </div>
          </div>

          {/* FPS */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">帧率 (FPS)</label>
            <Input
              type="number"
              min="1"
              max="60"
              value={formData.fps || 10}
              onChange={(e) => setFormData({ ...formData, fps: parseInt(e.target.value) })}
            />
            <p className="text-xs text-gray-500 mt-1">推荐: 5-15 FPS 用于检测</p>
          </div>

          {/* Hardware Assignment */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              硬件加速器（可选）
            </label>
            <select
              className="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              value={formData.hardware_device_id || ''}
              onChange={(e) => setFormData({
                ...formData,
                hardware_device_id: e.target.value || undefined
              })}
            >
              <option value="">无 (CPU)</option>
              {devices.map(device => (
                <option key={device.id} value={device.id}>
                  {device.name} ({device.type.toUpperCase()})
                </option>
              ))}
            </select>
          </div>

          {/* Feature Toggles */}
          <div className="space-y-2">
            <label className="flex items-center">
              <input
                type="checkbox"
                checked={formData.detect_enabled ?? true}
                onChange={(e) => setFormData({ ...formData, detect_enabled: e.target.checked })}
                className="rounded text-blue-600"
              />
              <span className="ml-2 text-sm">启用对象检测</span>
            </label>
            <label className="flex items-center">
              <input
                type="checkbox"
                checked={formData.record_enabled ?? true}
                onChange={(e) => setFormData({ ...formData, record_enabled: e.target.checked })}
                className="rounded text-blue-600"
              />
              <span className="ml-2 text-sm">启用视频录制</span>
            </label>
            <label className="flex items-center">
              <input
                type="checkbox"
                checked={formData.snapshots_enabled ?? true}
                onChange={(e) => setFormData({ ...formData, snapshots_enabled: e.target.checked })}
                className="rounded text-blue-600"
              />
              <span className="ml-2 text-sm">启用快照保存</span>
            </label>
          </div>

          {/* Actions */}
          <div className="flex justify-end space-x-2 pt-4">
            <Button
              variant="secondary"
              onClick={() => {
                setShowAddModal(false)
                setShowEditModal(false)
              }}
            >
              取消
            </Button>
            <Button onClick={handleSaveCamera}>
              {showEditModal ? '保存' : '添加'}
            </Button>
          </div>
        </div>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        isOpen={showDeleteModal}
        onClose={() => setShowDeleteModal(false)}
        title="确认删除"
      >
        <div className="space-y-4">
          <p className="text-gray-700">
            确定要删除相机 <strong>{selectedCamera?.name}</strong> 吗？
          </p>
          <p className="text-sm text-gray-600">
            此操作无法撤销。相机配置将从列表中永久删除。
          </p>
          <div className="flex justify-end space-x-2 pt-4">
            <Button variant="secondary" onClick={() => setShowDeleteModal(false)}>
              取消
            </Button>
            <Button variant="danger" onClick={confirmDelete}>
              确认删除
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  )
}

// Filter Tab Component
interface FilterTabProps {
  active: boolean
  onClick: () => void
  count: number
  children: React.ReactNode
}

const FilterTab: React.FC<FilterTabProps> = ({ active, onClick, count, children }) => (
  <button
    onClick={onClick}
    className={`
      px-4 py-2 rounded-lg text-sm font-medium transition-colors
      ${active
        ? 'bg-blue-600 text-white'
        : 'bg-white text-gray-700 hover:bg-gray-50 border border-gray-300'
      }
    `}
  >
    {children} ({count})
  </button>
)

export default CamerasPage
