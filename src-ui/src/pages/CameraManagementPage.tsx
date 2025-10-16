import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

// 摄像头品牌配置
const CAMERA_BRANDS = [
  {
    name: '海康威视 (Hikvision)',
    value: 'hikvision',
    defaultPort: 554,
    rtspTemplate: 'rtsp://admin:password@{ip}:554/Streaming/Channels/101'
  },
  {
    name: '大华 (Dahua)',
    value: 'dahua',
    defaultPort: 554,
    rtspTemplate: 'rtsp://admin:password@{ip}:554/cam/realmonitor?channel=1&subtype=0'
  },
  {
    name: 'TP-Link',
    value: 'tplink',
    defaultPort: 554,
    rtspTemplate: 'rtsp://admin:password@{ip}:554/stream1'
  },
  {
    name: '小米 (Xiaomi)',
    value: 'xiaomi',
    defaultPort: 554,
    rtspTemplate: 'rtsp://{ip}:554/live/ch00_0'
  },
  {
    name: 'ONVIF (通用)',
    value: 'onvif',
    defaultPort: 554,
    rtspTemplate: 'rtsp://admin:password@{ip}:554/stream'
  },
]

interface Camera {
  id: string
  name: string
  rtsp_url: string
  enabled: boolean
  hardware_acceleration?: string
  detector?: string
  detection_objects?: string[]
  zones?: any[]
}

interface ScanResult {
  ip: string
  brand?: string
  ports: number[]
  rtsp_url?: string
}

export default function CameraManagementPage() {
  // 左侧：发现摄像头
  const [networkRange, setNetworkRange] = useState('192.168.1.0/24')
  const [selectedBrand, setSelectedBrand] = useState('hikvision')
  const [isScanning, setIsScanning] = useState(false)
  const [scanResults, setScanResults] = useState<ScanResult[]>([])
  const [scanProgress, setScanProgress] = useState(0)

  // 右侧：摄像头列表
  const [cameras, setCameras] = useState<Camera[]>([])
  const [selectedCamera, setSelectedCamera] = useState<Camera | null>(null)
  const [isEditing, setIsEditing] = useState(false)

  // 编辑表单
  const [editForm, setEditForm] = useState<Partial<Camera>>({
    name: '',
    rtsp_url: '',
    enabled: true,
    hardware_acceleration: 'auto',
    detector: 'default',
    detection_objects: ['person', 'car']
  })

  useEffect(() => {
    loadCameras()
    forceGuessNetworkRange()
  }, [])

  // 强制使用本地IP段
  const forceGuessNetworkRange = async () => {
    try {
      const range = await invoke<string>('guess_network_range_command')
      setNetworkRange(range)
    } catch (error) {
      console.error('Failed to guess network range:', error)
      // 如果失败，尝试使用备用方法
      setNetworkRange('192.168.1.0/24')
    }
  }

  // 加载已配置的摄像头
  const loadCameras = () => {
    // 从 localStorage 加载
    const saved = localStorage.getItem('frigate_cameras')
    if (saved) {
      setCameras(JSON.parse(saved))
    }
  }

  // 保存摄像头配置
  const saveCameras = (newCameras: Camera[]) => {
    setCameras(newCameras)
    localStorage.setItem('frigate_cameras', JSON.stringify(newCameras))
  }


  // 扫描摄像头
  const scanForCameras = async () => {
    setIsScanning(true)
    setScanResults([])
    setScanProgress(0)

    try {
      const results = await invoke<ScanResult[]>('scan_for_cameras', {
        networkRange,
        ports: [80, 554, 8000, 8554],
        timeout: 2000
      })

      // 为每个结果生成 RTSP URL
      const resultsWithRTSP = results.map(result => {
        const brand = CAMERA_BRANDS.find(b => b.value === selectedBrand)
        if (brand) {
          return {
            ...result,
            brand: selectedBrand,
            rtsp_url: brand.rtspTemplate.replace('{ip}', result.ip)
          }
        }
        return result
      })

      setScanResults(resultsWithRTSP)
    } catch (error) {
      console.error('Scan failed:', error)
      alert('扫描失败: ' + String(error))
    } finally {
      setIsScanning(false)
    }
  }

  // 从扫描结果添加摄像头
  const addFromScanResult = (result: ScanResult) => {
    const newCamera: Camera = {
      id: `camera-${Date.now()}`,
      name: `摄像头 ${result.ip}`,
      rtsp_url: result.rtsp_url || '',
      enabled: true,
      hardware_acceleration: 'auto',
      detector: 'default',
      detection_objects: ['person', 'car']
    }

    saveCameras([...cameras, newCamera])
  }

  // 新建摄像头
  const createNewCamera = () => {
    setEditForm({
      name: '新摄像头',
      rtsp_url: 'rtsp://admin:password@192.168.1.100:554/stream',
      enabled: true,
      hardware_acceleration: 'auto',
      detector: 'default',
      detection_objects: ['person', 'car']
    })
    setSelectedCamera(null)
    setIsEditing(true)
  }

  // 编辑摄像头
  const editCamera = (camera: Camera) => {
    setSelectedCamera(camera)
    setEditForm({ ...camera })
    setIsEditing(true)
  }

  // 保存编辑
  const saveEdit = () => {
    if (selectedCamera) {
      // 更新现有摄像头
      const updated = cameras.map(c =>
        c.id === selectedCamera.id ? { ...c, ...editForm } as Camera : c
      )
      saveCameras(updated)
    } else {
      // 创建新摄像头
      const newCamera: Camera = {
        id: `camera-${Date.now()}`,
        ...editForm as Camera
      }
      saveCameras([...cameras, newCamera])
    }
    setIsEditing(false)
    setSelectedCamera(null)
  }

  // 删除摄像头
  const deleteCamera = (id: string) => {
    if (confirm('确定要删除这个摄像头吗？')) {
      saveCameras(cameras.filter(c => c.id !== id))
      if (selectedCamera?.id === id) {
        setSelectedCamera(null)
        setIsEditing(false)
      }
    }
  }

  // 切换启用状态
  const toggleEnabled = (id: string) => {
    const updated = cameras.map(c =>
      c.id === id ? { ...c, enabled: !c.enabled } : c
    )
    saveCameras(updated)
  }

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      {/* 顶部标题栏 */}
      <div className="bg-white shadow-sm border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900">摄像头管理</h1>
          <button
            onClick={createNewCamera}
            className="px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition"
          >
            新建摄像头
          </button>
        </div>
      </div>

      {/* 主内容区域 - 左右分栏 */}
      <div className="flex-1 flex overflow-hidden">
        {/* 左侧: 摄像头发现 */}
        <div className="w-1/2 border-r border-gray-200 flex flex-col bg-white">
          <div className="px-6 py-4 border-b border-gray-200 bg-gray-50">
            <h2 className="text-lg font-semibold text-gray-900">摄像头发现</h2>
            <p className="text-sm text-gray-500 mt-1">扫描网络中的IP摄像头</p>
          </div>

          <div className="p-6 border-b border-gray-200">
            {/* 网络范围 */}
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                网络范围 (自动检测)
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={networkRange}
                  readOnly
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-lg bg-gray-50 text-gray-700 cursor-not-allowed"
                  placeholder="自动检测中..."
                />
                <button
                  onClick={forceGuessNetworkRange}
                  className="px-3 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition"
                >
                  刷新
                </button>
              </div>
              <p className="text-xs text-gray-500 mt-1">
                自动检测本地网络IP段，点击"刷新"重新检测
              </p>
            </div>

            {/* 摄像头品牌 */}
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                摄像头品牌
              </label>
              <select
                value={selectedBrand}
                onChange={(e) => setSelectedBrand(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              >
                {CAMERA_BRANDS.map(brand => (
                  <option key={brand.value} value={brand.value}>
                    {brand.name}
                  </option>
                ))}
              </select>
            </div>

            {/* 扫描按钮 */}
            <button
              onClick={scanForCameras}
              disabled={isScanning}
              className="w-full px-4 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition disabled:opacity-50 font-semibold"
            >
              {isScanning ? '扫描中...' : '开始扫描'}
            </button>
          </div>

          {/* 扫描结果 */}
          <div className="flex-1 overflow-auto p-6">
            <h3 className="text-sm font-semibold text-gray-900 mb-3">
              扫描结果 ({scanResults.length})
            </h3>
            {scanResults.length === 0 ? (
              <p className="text-sm text-gray-500">暂无结果，请开始扫描</p>
            ) : (
              <div className="space-y-2">
                {scanResults.map((result, index) => (
                  <div
                    key={index}
                    className="p-4 border border-gray-200 rounded-lg hover:border-blue-300 bg-white"
                  >
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="font-medium text-gray-900">{result.ip}</p>
                        <p className="text-xs text-gray-500 mt-1">
                          开放端口: {result.ports.join(', ')}
                        </p>
                        {result.rtsp_url && (
                          <p className="text-xs text-gray-600 mt-1 font-mono">
                            {result.rtsp_url}
                          </p>
                        )}
                      </div>
                      <button
                        onClick={() => addFromScanResult(result)}
                        className="px-3 py-1 text-sm bg-green-500 text-white rounded-lg hover:bg-green-600 transition"
                      >
                        添加
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* 右侧: 摄像头列表与编辑 */}
        <div className="w-1/2 flex flex-col bg-white">
          <div className="px-6 py-4 border-b border-gray-200 bg-gray-50">
            <h2 className="text-lg font-semibold text-gray-900">
              已配置的摄像头 ({cameras.length})
            </h2>
          </div>

          {!isEditing ? (
            /* 摄像头列表视图 */
            <div className="flex-1 overflow-auto p-6">
              {cameras.length === 0 ? (
                <div className="text-center py-12">
                  <p className="text-gray-500 mb-4">暂无摄像头配置</p>
                  <button
                    onClick={createNewCamera}
                    className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition"
                  >
                    新建第一个摄像头
                  </button>
                </div>
              ) : (
                <div className="space-y-3">
                  {cameras.map(camera => (
                    <div
                      key={camera.id}
                      className={`p-4 border rounded-lg ${
                        camera.enabled
                          ? 'border-green-300 bg-green-50'
                          : 'border-gray-300 bg-gray-50'
                      }`}
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-2">
                            <h3 className="font-semibold text-gray-900">
                              {camera.name}
                            </h3>
                            <span
                              className={`px-2 py-0.5 text-xs rounded-full ${
                                camera.enabled
                                  ? 'bg-green-200 text-green-800'
                                  : 'bg-gray-200 text-gray-600'
                              }`}
                            >
                              {camera.enabled ? '已启用' : '已禁用'}
                            </span>
                          </div>
                          <p className="text-xs text-gray-600 mt-1 font-mono">
                            {camera.rtsp_url}
                          </p>
                          <div className="mt-2 flex flex-wrap gap-2">
                            {camera.hardware_acceleration && (
                              <span className="px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded">
                                硬件加速: {camera.hardware_acceleration}
                              </span>
                            )}
                            {camera.detector && (
                              <span className="px-2 py-1 text-xs bg-purple-100 text-purple-800 rounded">
                                检测器: {camera.detector}
                              </span>
                            )}
                          </div>
                        </div>
                        <div className="flex gap-2 ml-4">
                          <button
                            onClick={() => toggleEnabled(camera.id)}
                            className="px-3 py-1 text-sm text-blue-600 hover:bg-blue-50 rounded"
                          >
                            {camera.enabled ? '禁用' : '启用'}
                          </button>
                          <button
                            onClick={() => editCamera(camera)}
                            className="px-3 py-1 text-sm text-gray-600 hover:bg-gray-100 rounded"
                          >
                            编辑
                          </button>
                          <button
                            onClick={() => deleteCamera(camera.id)}
                            className="px-3 py-1 text-sm text-red-600 hover:bg-red-50 rounded"
                          >
                            删除
                          </button>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ) : (
            /* 编辑视图 */
            <div className="flex-1 overflow-auto p-6">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">
                {selectedCamera ? '编辑摄像头' : '新建摄像头'}
              </h3>

              <div className="space-y-4">
                {/* 名称 */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    摄像头名称
                  </label>
                  <input
                    type="text"
                    value={editForm.name || ''}
                    onChange={(e) => setEditForm({ ...editForm, name: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                    placeholder="例如: 前门摄像头"
                  />
                </div>

                {/* RTSP URL */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    RTSP 流地址
                  </label>
                  <input
                    type="text"
                    value={editForm.rtsp_url || ''}
                    onChange={(e) => setEditForm({ ...editForm, rtsp_url: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 font-mono text-sm"
                    placeholder="rtsp://admin:password@192.168.1.100:554/stream"
                  />
                </div>

                {/* 硬件加速 */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    硬件加速
                  </label>
                  <select
                    value={editForm.hardware_acceleration || 'auto'}
                    onChange={(e) =>
                      setEditForm({ ...editForm, hardware_acceleration: e.target.value })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                  >
                    <option value="auto">自动</option>
                    <option value="cuda">NVIDIA CUDA</option>
                    <option value="qsv">Intel QSV</option>
                    <option value="vaapi">VAAPI</option>
                    <option value="v4l2m2m">V4L2 M2M</option>
                    <option value="none">无</option>
                  </select>
                </div>

                {/* 检测器 */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    AI 检测器
                  </label>
                  <select
                    value={editForm.detector || 'default'}
                    onChange={(e) => setEditForm({ ...editForm, detector: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                  >
                    <option value="default">默认 (CPU)</option>
                    <option value="edgetpu">Google Coral EdgeTPU</option>
                    <option value="openvino">Intel OpenVINO</option>
                    <option value="tensorrt">NVIDIA TensorRT</option>
                    <option value="onnx">ONNX Runtime</option>
                    <option value="rknn">Rockchip RKNN</option>
                    <option value="hailo8l">Hailo-8L</option>
                  </select>
                </div>

                {/* 检测对象 */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    检测对象
                  </label>
                  <div className="grid grid-cols-3 gap-2">
                    {['person', 'car', 'dog', 'cat', 'bicycle', 'motorcycle'].map(obj => (
                      <label key={obj} className="flex items-center gap-2">
                        <input
                          type="checkbox"
                          checked={editForm.detection_objects?.includes(obj)}
                          onChange={(e) => {
                            const current = editForm.detection_objects || []
                            if (e.target.checked) {
                              setEditForm({
                                ...editForm,
                                detection_objects: [...current, obj]
                              })
                            } else {
                              setEditForm({
                                ...editForm,
                                detection_objects: current.filter(o => o !== obj)
                              })
                            }
                          }}
                          className="rounded"
                        />
                        <span className="text-sm text-gray-700">{obj}</span>
                      </label>
                    ))}
                  </div>
                </div>

                {/* 启用状态 */}
                <div>
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={editForm.enabled !== false}
                      onChange={(e) => setEditForm({ ...editForm, enabled: e.target.checked })}
                      className="rounded"
                    />
                    <span className="text-sm font-medium text-gray-700">启用此摄像头</span>
                  </label>
                </div>

                {/* 操作按钮 */}
                <div className="flex gap-3 pt-4">
                  <button
                    onClick={saveEdit}
                    className="flex-1 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition"
                  >
                    保存
                  </button>
                  <button
                    onClick={() => {
                      setIsEditing(false)
                      setSelectedCamera(null)
                    }}
                    className="flex-1 px-4 py-2 bg-gray-300 text-gray-700 rounded-lg hover:bg-gray-400 transition"
                  >
                    取消
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
