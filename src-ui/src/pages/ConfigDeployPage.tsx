import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { HardwareDevice } from '../types'
import { DockerComposeGenerator, DockerComposeConfig } from '../services/dockerComposeGenerator'

interface PCIDevice {
  vendor_id: string
  device_id: string
  class_code: string
  description: string
  slot: string
}

interface DeploymentResult {
  success: boolean
  message: string
  container_id?: string
  warnings?: string[]
}

// Frigate 支持的所有硬件加速器
const FRIGATE_HARDWARE_OPTIONS = [
  // GPU 硬件
  { category: 'GPU', name: 'NVIDIA GPU (CUDA)', type: 'gpu', hwaccel: 'cuda', vendor: 'nvidia' },
  { category: 'GPU', name: 'Intel iGPU (QSV)', type: 'gpu', hwaccel: 'qsv', vendor: 'intel' },
  { category: 'GPU', name: 'Intel iGPU (VAAPI)', type: 'gpu', hwaccel: 'vaapi', vendor: 'intel' },
  { category: 'GPU', name: 'AMD GPU (VAAPI)', type: 'gpu', hwaccel: 'vaapi', vendor: 'amd' },
  { category: 'GPU', name: 'Raspberry Pi V4L2', type: 'gpu', hwaccel: 'v4l2m2m', vendor: 'raspberry' },
  { category: 'GPU', name: 'Jetson Nano (CUDA)', type: 'gpu', hwaccel: 'cuda', vendor: 'nvidia_jetson' },

  // TPU/NPU 硬件
  { category: 'TPU/NPU', name: 'Google Coral TPU (USB)', type: 'tpu', detector: 'edgetpu', device: 'usb' },
  { category: 'TPU/NPU', name: 'Google Coral TPU (PCIe)', type: 'tpu', detector: 'edgetpu', device: 'pci' },
  { category: 'TPU/NPU', name: 'Google Coral TPU (M.2)', type: 'tpu', detector: 'edgetpu', device: 'm2' },
  { category: 'TPU/NPU', name: 'Hailo-8 NPU', type: 'tpu', detector: 'hailo8l', device: 'hailo' },
  { category: 'TPU/NPU', name: 'Hailo-8L NPU', type: 'tpu', detector: 'hailo8l', device: 'hailo' },
  { category: 'TPU/NPU', name: 'Rockchip NPU (RKNN)', type: 'tpu', detector: 'rknn', device: 'rknn' },

  // AI 推理引擎
  { category: 'AI Engine', name: 'OpenVINO (Intel)', type: 'gpu', detector: 'openvino', vendor: 'intel' },
  { category: 'AI Engine', name: 'TensorRT (NVIDIA)', type: 'gpu', detector: 'tensorrt', vendor: 'nvidia' },
  { category: 'AI Engine', name: 'ONNX Runtime (CPU)', type: 'cpu', detector: 'onnx', vendor: 'cpu' },
  { category: 'AI Engine', name: 'ONNX Runtime (GPU)', type: 'gpu', detector: 'onnx', vendor: 'nvidia' },
]

export default function ConfigDeployPage() {
  // Docker Compose 状态
  const [dockerComposeContent, setDockerComposeContent] = useState<string>('')
  const [dockerComposePath, setDockerComposePath] = useState<string>('')

  // 硬件状态
  const [availableDevices, setAvailableDevices] = useState<HardwareDevice[]>([])
  const [selectedDevices, setSelectedDevices] = useState<HardwareDevice[]>([])
  const [pciDevices, setPciDevices] = useState<PCIDevice[]>([])
  const [isScanning, setIsScanning] = useState(false)
  const [showManualAddModal, setShowManualAddModal] = useState(false)
  const [manualDeviceType, setManualDeviceType] = useState<string>('')

  // 配置状态
  const [volumePaths, setVolumePaths] = useState({
    config: './config',
    storage: './storage'
  })
  const [ports, setPorts] = useState({
    web_port: 8971,
    rtsp_port: 8554,
    webrtc_tcp_port: 8555,
    webrtc_udp_port: 8555
  })

  // 部署状态
  const [isDeploying, setIsDeploying] = useState(false)
  const [deploymentStatus, setDeploymentStatus] = useState<string>('')
  const [deploymentLogs, setDeploymentLogs] = useState<string[]>([])

  // 初始化加载
  useEffect(() => {
    loadDockerComposePath()
    loadSavedDevices()
    generateInitialCompose()
  }, [])

  // 加载 docker-compose 路径
  const loadDockerComposePath = async () => {
    try {
      const path = await invoke<string>('get_docker_compose_path')
      setDockerComposePath(path)
    } catch (error) {
      console.error('Failed to get docker-compose path:', error)
    }
  }

  // 加载已保存的硬件设备
  const loadSavedDevices = async () => {
    try {
      const devices = await invoke<HardwareDevice[]>('get_saved_hardware_devices')
      setSelectedDevices(devices)
      setAvailableDevices(devices)
    } catch (error) {
      console.error('Failed to load saved devices:', error)
    }
  }

  // 生成初始 docker-compose
  const generateInitialCompose = () => {
    const defaultCompose = DockerComposeGenerator.getDefaultCompose()
    setDockerComposeContent(defaultCompose)
  }

  // 扫描 PCI 设备
  const scanPCIDevices = async () => {
    setIsScanning(true)
    try {
      const devices = await invoke<PCIDevice[]>('scan_pci_devices')
      setPciDevices(devices)

      // 自动转换为 HardwareDevice
      const hardwareDevices: HardwareDevice[] = devices.map((pci, index) => ({
        id: `pci-${pci.slot}`,
        type: detectDeviceType(pci),
        name: pci.description,
        device_path: `/dev/dri/renderD${128 + index}`, // 默认路径
        capabilities: [],
        driver: undefined,
        platform: 'linux' as const,
        architecture: 'x86_64' as const,
        vendor_id: pci.vendor_id,
        detected_at: new Date().toISOString(),
        detection_source: 'pci_scan',
        available: true,
        in_use: false
      }))

      setAvailableDevices(hardwareDevices)
    } catch (error) {
      console.error('Failed to scan PCI devices:', error)
      alert('扫描硬件失败: ' + String(error))
    } finally {
      setIsScanning(false)
    }
  }

  // 检测设备类型
  const detectDeviceType = (pci: PCIDevice): 'gpu' | 'tpu' | 'camera' | 'capture_card' => {
    const desc = pci.description.toLowerCase()
    if (desc.includes('vga') || desc.includes('3d') || desc.includes('display')) {
      return 'gpu'
    } else if (desc.includes('coral') || desc.includes('edge tpu')) {
      return 'tpu'
    } else if (desc.includes('capture') || desc.includes('video input')) {
      return 'capture_card'
    }
    return 'gpu' // 默认
  }

  // 添加硬件设备
  const addDevice = async (device: HardwareDevice) => {
    try {
      await invoke('add_hardware_device_to_config', { device })
      setSelectedDevices([...selectedDevices, device])
      updateDockerCompose([...selectedDevices, device])
    } catch (error) {
      console.error('Failed to add device:', error)
      alert('添加设备失败: ' + String(error))
    }
  }

  // 手动添加硬件设备
  const addManualDevice = (hardwareOption: typeof FRIGATE_HARDWARE_OPTIONS[0]) => {
    const newDevice: HardwareDevice = {
      id: `manual-${Date.now()}`,
      type: hardwareOption.type as any,
      name: hardwareOption.name,
      device_path: hardwareOption.device === 'usb'
        ? '/dev/bus/usb'
        : hardwareOption.device === 'pci' || hardwareOption.device === 'm2'
        ? '/dev/apex_0'
        : hardwareOption.device === 'hailo'
        ? '/dev/hailo0'
        : hardwareOption.vendor === 'nvidia'
        ? '/dev/nvidia0'
        : '/dev/dri/renderD128',
      capabilities: [],
      driver: hardwareOption.vendor,
      platform: 'linux' as const,
      architecture: 'x86_64' as const,
      vendor_id: hardwareOption.vendor,
      detected_at: new Date().toISOString(),
      detection_source: 'manual',
      available: true,
      in_use: false,
      hwaccel: hardwareOption.hwaccel,
      detector: hardwareOption.detector
    }

    setSelectedDevices([...selectedDevices, newDevice])
    updateDockerCompose([...selectedDevices, newDevice])
    setShowManualAddModal(false)
  }

  // 移除硬件设备
  const removeDevice = (deviceId: string) => {
    const newDevices = selectedDevices.filter(d => d.id !== deviceId)
    setSelectedDevices(newDevices)
    updateDockerCompose(newDevices)
  }

  // 更新 docker-compose 内容
  const updateDockerCompose = (devices: HardwareDevice[]) => {
    // 收集所有设备路径
    const devicePaths = devices.map(d => d.device_path)

    // 检测主要硬件类型
    const hasNvidia = devices.some(d => d.vendor_id?.toLowerCase().includes('10de'))
    const hasIntel = devices.some(d => d.vendor_id?.toLowerCase().includes('8086'))
    const hasAMD = devices.some(d => d.vendor_id?.toLowerCase().includes('1002'))
    const hasCoral = devices.some(d => d.type === 'tpu')

    let hardwareType: string | undefined
    if (hasNvidia) hardwareType = 'nvidia'
    else if (hasIntel) hardwareType = 'intel'
    else if (hasAMD) hardwareType = 'amd'
    else if (hasCoral) hardwareType = 'coral'

    // 生成配置
    const config = DockerComposeGenerator.generateFullConfig(
      devicePaths,
      volumePaths,
      hardwareType
    )

    // 更新端口
    config.ports = ports

    // 生成 YAML
    const yaml = DockerComposeGenerator.generateCompose(config)
    setDockerComposeContent(yaml)
  }

  // 设置 docker-compose 路径
  const setComposePath = async () => {
    try {
      const newPath = prompt('请输入 docker-compose.yml 文件路径:', dockerComposePath)
      if (newPath) {
        await invoke('set_docker_compose_path', { path: newPath })
        setDockerComposePath(newPath)
      }
    } catch (error) {
      console.error('Failed to set docker-compose path:', error)
      alert('设置路径失败: ' + String(error))
    }
  }

  // 保存 docker-compose
  const saveDockerCompose = async () => {
    try {
      // 这里可以调用后端保存文件
      // 暂时使用浏览器下载
      const blob = new Blob([dockerComposeContent], { type: 'text/yaml' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = 'docker-compose.yml'
      a.click()
      URL.revokeObjectURL(url)
    } catch (error) {
      console.error('Failed to save docker-compose:', error)
      alert('保存失败: ' + String(error))
    }
  }

  // 一键部署
  const deployFrigate = async () => {
    if (!dockerComposeContent.trim()) {
      alert('请先配置 docker-compose.yml')
      return
    }

    if (selectedDevices.length === 0) {
      const confirm = window.confirm('未添加任何硬件设备，是否继续部署？')
      if (!confirm) return
    }

    setIsDeploying(true)
    setDeploymentStatus('部署中...')
    setDeploymentLogs([])

    try {
      // 先保存 docker-compose 内容到文件
      await invoke('save_docker_compose_content', {
        path: dockerComposePath,
        content: dockerComposeContent
      })

      setDeploymentLogs(prev => [...prev, '✓ 已保存 docker-compose.yml'])

      // 执行部署
      const result = await invoke<DeploymentResult>('deploy_frigate', {
        dockerComposePath,
        useCompose: true
      })

      if (result.success) {
        setDeploymentStatus('部署成功！')
        setDeploymentLogs(prev => [
          ...prev,
          '✓ Frigate 容器已启动',
          `✓ 容器 ID: ${result.container_id}`,
          result.message
        ])

        if (result.warnings && result.warnings.length > 0) {
          setDeploymentLogs(prev => [
            ...prev,
            '⚠️ 警告:',
            ...result.warnings
          ])
        }

        // 等待几秒后进行健康检查
        setTimeout(() => checkHealth(), 5000)
      } else {
        setDeploymentStatus('部署失败')
        setDeploymentLogs(prev => [...prev, `✗ ${result.message}`])
      }
    } catch (error) {
      setDeploymentStatus('部署失败')
      setDeploymentLogs(prev => [...prev, `✗ 错误: ${String(error)}`])
    } finally {
      setIsDeploying(false)
    }
  }

  // 健康检查
  const checkHealth = async () => {
    try {
      const health = await invoke<any>('check_deployment_health')
      setDeploymentLogs(prev => [
        ...prev,
        '\n健康检查结果:',
        `状态: ${health.status}`,
        `响应时间: ${health.response_time_ms}ms`
      ])
    } catch (error) {
      setDeploymentLogs(prev => [
        ...prev,
        `⚠️ 健康检查失败: ${String(error)}`
      ])
    }
  }

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      {/* 顶部标题栏 */}
      <div className="bg-white shadow-sm border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900">Frigate 配置与部署</h1>
          <div className="flex items-center gap-4">
            <button
              onClick={saveDockerCompose}
              className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition"
            >
              保存配置
            </button>
            <button
              onClick={deployFrigate}
              disabled={isDeploying}
              className="px-6 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition disabled:opacity-50 disabled:cursor-not-allowed font-semibold"
            >
              {isDeploying ? '部署中...' : '一键部署'}
            </button>
          </div>
        </div>
      </div>

      {/* 主内容区域 - 左右分栏 */}
      <div className="flex-1 flex overflow-hidden">
        {/* 左侧: docker-compose.yml 编辑器 */}
        <div className="w-1/2 border-r border-gray-200 flex flex-col bg-white">
          <div className="px-6 py-4 border-b border-gray-200 bg-gray-50">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold text-gray-900">docker-compose.yml</h2>
              <button
                onClick={setComposePath}
                className="text-sm text-blue-600 hover:text-blue-700"
              >
                设置路径
              </button>
            </div>
            <p className="text-sm text-gray-500 mt-1">{dockerComposePath}</p>
          </div>

          <div className="flex-1 overflow-auto p-4">
            <textarea
              value={dockerComposeContent}
              onChange={(e) => setDockerComposeContent(e.target.value)}
              className="w-full h-full font-mono text-sm bg-gray-900 text-gray-100 p-4 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
              spellCheck={false}
            />
          </div>

          {/* 配置选项 */}
          <div className="border-t border-gray-200 p-4 bg-gray-50">
            <h3 className="text-sm font-semibold text-gray-900 mb-3">基础配置</h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-xs text-gray-600 mb-1">配置目录</label>
                <input
                  type="text"
                  value={volumePaths.config}
                  onChange={(e) => {
                    setVolumePaths({ ...volumePaths, config: e.target.value })
                    updateDockerCompose(selectedDevices)
                  }}
                  className="w-full px-3 py-2 text-sm border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
              <div>
                <label className="block text-xs text-gray-600 mb-1">存储目录</label>
                <input
                  type="text"
                  value={volumePaths.storage}
                  onChange={(e) => {
                    setVolumePaths({ ...volumePaths, storage: e.target.value })
                    updateDockerCompose(selectedDevices)
                  }}
                  className="w-full px-3 py-2 text-sm border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
              <div>
                <label className="block text-xs text-gray-600 mb-1">Web 端口</label>
                <input
                  type="number"
                  value={ports.web_port}
                  onChange={(e) => {
                    setPorts({ ...ports, web_port: parseInt(e.target.value) })
                    updateDockerCompose(selectedDevices)
                  }}
                  className="w-full px-3 py-2 text-sm border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
              <div>
                <label className="block text-xs text-gray-600 mb-1">RTSP 端口</label>
                <input
                  type="number"
                  value={ports.rtsp_port}
                  onChange={(e) => {
                    setPorts({ ...ports, rtsp_port: parseInt(e.target.value) })
                    updateDockerCompose(selectedDevices)
                  }}
                  className="w-full px-3 py-2 text-sm border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
            </div>
          </div>
        </div>

        {/* 右侧: 硬件扫描与添加 */}
        <div className="w-1/2 flex flex-col bg-white">
          <div className="px-6 py-4 border-b border-gray-200 bg-gray-50">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold text-gray-900">硬件设备</h2>
              <div className="flex gap-2">
                <button
                  onClick={() => setShowManualAddModal(true)}
                  className="px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition"
                >
                  手动添加
                </button>
                <button
                  onClick={scanPCIDevices}
                  disabled={isScanning}
                  className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition disabled:opacity-50"
                >
                  {isScanning ? '扫描中...' : '自动扫描'}
                </button>
              </div>
            </div>
          </div>

          <div className="flex-1 overflow-auto">
            {/* 已选设备 */}
            <div className="p-4 border-b border-gray-200">
              <h3 className="text-sm font-semibold text-gray-900 mb-3">已添加的设备 ({selectedDevices.length})</h3>
              {selectedDevices.length === 0 ? (
                <p className="text-sm text-gray-500">暂无设备，请扫描并添加硬件设备</p>
              ) : (
                <div className="space-y-2">
                  {selectedDevices.map((device) => (
                    <div
                      key={device.id}
                      className="flex items-center justify-between p-3 bg-green-50 border border-green-200 rounded-lg"
                    >
                      <div>
                        <p className="text-sm font-medium text-gray-900">{device.name}</p>
                        <p className="text-xs text-gray-500">{device.device_path}</p>
                        <p className="text-xs text-gray-500">类型: {device.type}</p>
                      </div>
                      <button
                        onClick={() => removeDevice(device.id)}
                        className="px-3 py-1 text-sm text-red-600 hover:bg-red-50 rounded-lg transition"
                      >
                        移除
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* 可用设备 */}
            <div className="p-4">
              <h3 className="text-sm font-semibold text-gray-900 mb-3">可用设备 ({availableDevices.length})</h3>
              {availableDevices.length === 0 ? (
                <p className="text-sm text-gray-500">点击"扫描硬件"查找可用设备</p>
              ) : (
                <div className="space-y-2">
                  {availableDevices.map((device) => {
                    const isSelected = selectedDevices.some(d => d.id === device.id)
                    return (
                      <div
                        key={device.id}
                        className={`p-3 border rounded-lg ${
                          isSelected
                            ? 'bg-gray-100 border-gray-300'
                            : 'bg-white border-gray-200 hover:border-blue-300'
                        }`}
                      >
                        <div className="flex items-center justify-between">
                          <div>
                            <p className="text-sm font-medium text-gray-900">{device.name}</p>
                            <p className="text-xs text-gray-500">{device.device_path}</p>
                            <p className="text-xs text-gray-500">
                              类型: {device.type} | 供应商: {device.vendor_id || '未知'}
                            </p>
                          </div>
                          {!isSelected && (
                            <button
                              onClick={() => addDevice(device)}
                              className="px-3 py-1 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition"
                            >
                              添加
                            </button>
                          )}
                          {isSelected && (
                            <span className="px-3 py-1 text-xs text-gray-500 bg-gray-200 rounded-lg">
                              已添加
                            </span>
                          )}
                        </div>
                      </div>
                    )
                  })}
                </div>
              )}
            </div>
          </div>

          {/* 部署日志 */}
          {deploymentLogs.length > 0 && (
            <div className="border-t border-gray-200 p-4 bg-gray-50 max-h-64 overflow-auto">
              <h3 className="text-sm font-semibold text-gray-900 mb-2">部署日志</h3>
              <div className="space-y-1 font-mono text-xs">
                {deploymentLogs.map((log, index) => (
                  <div key={index} className={`
                    ${log.startsWith('✓') ? 'text-green-600' : ''}
                    ${log.startsWith('✗') ? 'text-red-600' : ''}
                    ${log.startsWith('⚠️') ? 'text-yellow-600' : ''}
                    ${!log.startsWith('✓') && !log.startsWith('✗') && !log.startsWith('⚠️') ? 'text-gray-700' : ''}
                  `}>
                    {log}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* 手动添加硬件模态框 */}
      {showManualAddModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full max-h-[80vh] overflow-hidden">
            <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
              <h2 className="text-xl font-semibold text-gray-900">手动添加硬件加速器</h2>
              <button
                onClick={() => setShowManualAddModal(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <span className="text-2xl">&times;</span>
              </button>
            </div>

            <div className="p-6 overflow-auto max-h-[60vh]">
              {/* 按类别分组显示硬件选项 */}
              {['GPU', 'TPU/NPU', 'AI Engine'].map(category => {
                const items = FRIGATE_HARDWARE_OPTIONS.filter(opt => opt.category === category)
                return (
                  <div key={category} className="mb-6">
                    <h3 className="text-lg font-semibold text-gray-900 mb-3 border-b pb-2">
                      {category}
                    </h3>
                    <div className="grid grid-cols-2 gap-3">
                      {items.map((option, index) => (
                        <button
                          key={index}
                          onClick={() => addManualDevice(option)}
                          className="text-left p-4 border border-gray-200 rounded-lg hover:border-blue-500 hover:bg-blue-50 transition"
                        >
                          <div className="font-medium text-gray-900">{option.name}</div>
                          <div className="text-xs text-gray-500 mt-1">
                            {option.hwaccel && `硬件加速: ${option.hwaccel}`}
                            {option.detector && ` | 检测器: ${option.detector}`}
                          </div>
                        </button>
                      ))}
                    </div>
                  </div>
                )
              })}
            </div>

            <div className="px-6 py-4 border-t border-gray-200 bg-gray-50">
              <button
                onClick={() => setShowManualAddModal(false)}
                className="px-4 py-2 bg-gray-500 text-white rounded-lg hover:bg-gray-600 transition"
              >
                取消
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
