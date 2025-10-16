// Deploy Page - T128-T136
// Handles deployment validation, execution, health checks, and rollback

import React, { useState, useEffect } from 'react'
import { useTauriCommand } from '../hooks/useTauriCommand'
import Button from '../components/Button'
import Card from '../components/Card'
import ValidationResults from '../components/ValidationResults'
import DeploymentProgress from '../components/DeploymentProgress'
import HealthCheckStatus from '../components/HealthCheckStatus'
import { DockerComposeGenerator } from '../services/dockerComposeGenerator'

interface DeploymentConfig {
  config_path: string
  method: string
  devices: string[]
  volumes: Array<{ host_path: string; container_path: string }>
  ports: Array<{ host_port: number; container_port: number }>
  environment: Record<string, string>
}

interface ValidationResult {
  valid: boolean
  errors: string[]
  warnings: string[]
  checks: Array<{
    name: string
    passed: boolean
    message: string
  }>
}

interface DeploymentResponse {
  success: boolean
  container_id: string | null
  command: string
  stdout: string
  stderr: string
  exit_code: number | null
  deployment_time: string
  warnings: string[]
}

interface DeviceValidationResult {
  device_path: string
  device_name: string
  device_type: string
  exists: boolean
  message: string
}

interface DeviceValidationResponse {
  total_devices: number
  valid_count: number
  invalid_count: number
  valid_devices: DeviceValidationResult[]
  invalid_devices: DeviceValidationResult[]
  warnings: string[]
}

interface HealthCheckResponse {
  container_id: string
  status: string
  checks: Array<{
    name: string
    passed: boolean
    message: string
    details: string | null
  }>
  response_time_ms: number
  timestamp: string
}

interface DeploymentHistoryItem {
  id: string
  container_id: string
  config_path: string
  deployment_time: string
  command: string
  status: string
}

const DeployPage: React.FC = () => {
  // Deployment state
  const [configPath, setConfigPath] = useState<string>('/etc/frigate/config.yml')
  const [deploymentMethod, setDeploymentMethod] = useState<string>('DockerRun')
  const [currentDeployment, setCurrentDeployment] = useState<DeploymentResponse | null>(null)
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null)
  const [healthStatus, setHealthStatus] = useState<HealthCheckResponse | null>(null)
  const [deploymentHistory, setDeploymentHistory] = useState<DeploymentHistoryItem[]>([])
  const [showCommandPreview, setShowCommandPreview] = useState(false)
  const [showComposePreview, setShowComposePreview] = useState(false)
  const [composeYaml, setComposeYaml] = useState<string>('')
  const [isDeploying, setIsDeploying] = useState(false)
  const [deploymentStep, setDeploymentStep] = useState<'idle' | 'validating' | 'deploying' | 'health_check' | 'completed' | 'failed'>('idle')
  const [deviceValidation, setDeviceValidation] = useState<DeviceValidationResponse | null>(null)
  const [showDeviceWarnings, setShowDeviceWarnings] = useState(false)
  // Phase 3 - 添加实时状态更新相关状态
  const [deploymentLogs, setDeploymentLogs] = useState<string[]>([])
  const [statusPollingInterval, setStatusPollingInterval] = useState<ReturnType<typeof setInterval> | null>(null)

  // Tauri commands
  const {
    data: validationData,
    error: validationError,
    loading: validating,
    execute: runValidation,
  } = useTauriCommand<ValidationResult>('validate_config')

  const {
    data: deploymentData,
    error: deploymentError,
    loading: deploying,
    execute: executeDeploy,
  } = useTauriCommand<DeploymentResponse>('deploy_frigate')

  const {
    data: healthData,
    error: healthError,
    loading: checkingHealth,
    execute: checkHealth,
  } = useTauriCommand<HealthCheckResponse>('check_deployment_health')

  const {
    data: historyData,
    execute: loadHistory,
  } = useTauriCommand<{ deployments: DeploymentHistoryItem[]; total_count: number }>('list_deployment_history')

  const {
    execute: rollback,
    loading: rollingBack,
  } = useTauriCommand<any>('rollback_deployment')

  const {
    data: deviceValidationData,
    loading: validatingDevices,
    execute: validateDevices,
  } = useTauriCommand<DeviceValidationResponse>('validate_hardware_devices')

  // Phase 3 - 添加获取已保存硬件设备的命令
  const {
    data: savedDevices,
    loading: loadingDevices,
    execute: loadSavedDevices,
  } = useTauriCommand<string[]>('get_saved_hardware_devices')

  // Phase 3 - 添加获取部署日志的命令
  const {
    data: logsData,
    execute: fetchLogs,
  } = useTauriCommand<{ logs: string[]; total_lines: number }>('get_deployment_logs')

  // Phase 3 - 添加获取部署状态的命令
  const {
    execute: fetchStatus,
  } = useTauriCommand<{ status: string; container_id: string | null }>('get_deployment_status_cmd')

  // Load deployment history and saved devices on mount
  useEffect(() => {
    loadHistory()
    loadSavedDevices() // Phase 3 - 加载已保存的硬件设备
  }, [])

  useEffect(() => {
    if (historyData) {
      setDeploymentHistory(historyData.deployments)
    }
  }, [historyData])

  // Handle validation result
  useEffect(() => {
    if (validationData) {
      setValidationResult(validationData)
      if (validationData.valid) {
        setDeploymentStep('idle')
      } else {
        setDeploymentStep('failed')
      }
    }
  }, [validationData])

  // Handle deployment result
  useEffect(() => {
    if (deploymentData) {
      setCurrentDeployment(deploymentData)
      if (deploymentData.success && deploymentData.container_id) {
        setDeploymentStep('health_check')
        // Phase 3 - 开始实时状态轮询
        startStatusPolling(deploymentData.container_id)
        // Automatically start health check
        checkHealth({ container_id: deploymentData.container_id })
      } else {
        setDeploymentStep('failed')
        stopStatusPolling()
      }
    }
  }, [deploymentData])

  // Handle health check result
  useEffect(() => {
    if (healthData) {
      setHealthStatus(healthData)
      const allPassed = healthData.checks.every(c => c.passed)
      setDeploymentStep(allPassed ? 'completed' : 'failed')
      setIsDeploying(false)
      // Phase 3 - 停止轮询
      if (allPassed || !allPassed) {
        stopStatusPolling()
      }
    }
  }, [healthData])

  // Handle device validation result
  useEffect(() => {
    if (deviceValidationData) {
      setDeviceValidation(deviceValidationData)
      if (deviceValidationData.warnings.length > 0) {
        setShowDeviceWarnings(true)
      }
    }
  }, [deviceValidationData])

  // Phase 3 - 处理日志数据更新
  useEffect(() => {
    if (logsData) {
      setDeploymentLogs(logsData.logs)
    }
  }, [logsData])

  // 根据已保存的设备生成 docker-compose 预览
  useEffect(() => {
    if (savedDevices !== undefined) {
      // 检测硬件类型
      let hardwareType: string | undefined;
      const hasCuda = savedDevices?.some(d => d.includes('nvidia') || d.includes('cuda'));
      const hasHailo = savedDevices?.some(d => d.includes('hailo'));
      const hasIntel = savedDevices?.some(d => d.includes('dri'));
      const hasCoral = savedDevices?.some(d => d.includes('apex') || d.includes('usb'));

      if (hasCuda) hardwareType = 'nvidia';
      else if (hasHailo) hardwareType = 'hailo';
      else if (hasIntel) hardwareType = 'intel';
      else if (hasCoral) hardwareType = 'coral';

      // 使用增强的配置生成器
      const fullConfig = DockerComposeGenerator.generateFullConfig(
        savedDevices || [],
        { config: './config', storage: './storage' },
        hardwareType
      );

      const compose = DockerComposeGenerator.generateCompose(fullConfig);
      setComposeYaml(compose);
    }
  }, [savedDevices])

  // Phase 3 - 清理轮询定时器
  useEffect(() => {
    return () => {
      if (statusPollingInterval) {
        clearInterval(statusPollingInterval)
      }
    }
  }, [statusPollingInterval])

  // Handle validation
  const handleValidate = async () => {
    setDeploymentStep('validating')
    setValidationResult(null)
    await runValidation({ config_path: configPath })
  }

  // Handle device validation
  const handleValidateDevices = async () => {
    setDeviceValidation(null)
    setShowDeviceWarnings(false)
    await validateDevices()
  }

  // Handle deployment
  const handleDeploy = async () => {
    setIsDeploying(true)
    setDeploymentStep('deploying')

    // Phase 3 - 使用已保存的硬件设备列表
    const deviceList = savedDevices || []

    // 检测硬件类型以添加对应的环境变量
    let hardwareType: string | undefined;
    const hasCuda = deviceList.some(d => d.includes('nvidia') || d.includes('cuda'));
    const hasHailo = deviceList.some(d => d.includes('hailo'));
    const hasIntel = deviceList.some(d => d.includes('dri'));
    const hasCoral = deviceList.some(d => d.includes('apex') || d.includes('usb'));

    if (hasCuda) hardwareType = 'nvidia';
    else if (hasHailo) hardwareType = 'hailo';
    else if (hasIntel) hardwareType = 'intel';
    else if (hasCoral) hardwareType = 'coral';

    // 获取推荐的环境变量
    const recommendedEnv = DockerComposeGenerator.getRecommendedEnvironment(hardwareType);

    const config: DeploymentConfig = {
      config_path: configPath,
      method: deploymentMethod,
      devices: deviceList, // Phase 3 - 使用从硬件检测页保存的设备列表
      volumes: [
        { host_path: '/etc/frigate', container_path: '/config' },
        { host_path: '/media/frigate', container_path: '/media/frigate' },
      ],
      ports: [
        { host_port: 8971, container_port: 8971 },  // 修正为 Frigate 默认端口
        { host_port: 8554, container_port: 8554 },
        { host_port: 8555, container_port: 8555 },
      ],
      environment: {
        TZ: 'UTC',
        ...recommendedEnv, // 添加所有推荐的环境变量
      },
    }

    await executeDeploy(config as any)
  }

  // Handle rollback
  const handleRollback = async () => {
    if (!currentDeployment?.container_id) {
      return
    }

    const confirmed = confirm('确定要回滚到上一个部署吗？这将停止当前容器并恢复之前的版本。')
    if (!confirmed) return

    await rollback({
      reason: '用户手动回滚',
      target_snapshot_id: null, // Rollback to previous
      preserve_data: true,
      create_backup: true,
    })

    // Reload history after rollback
    loadHistory()
    setDeploymentStep('idle')
    setCurrentDeployment(null)
    stopStatusPolling() // Phase 3 - 停止轮询
  }

  // Phase 3 - 开始实时状态轮询
  const startStatusPolling = (containerId: string) => {
    // 停止之前的轮询
    stopStatusPolling()

    // 立即获取一次日志
    fetchLogs({ container_id: containerId, tail_lines: 50 })

    // 设置轮询间隔（每2秒更新一次）
    const interval = setInterval(() => {
      // 获取最新状态
      fetchStatus()
      // 获取最新日志
      fetchLogs({ container_id: containerId, tail_lines: 50 })
    }, 2000)

    setStatusPollingInterval(interval)
  }

  // Phase 3 - 停止实时状态轮询
  const stopStatusPolling = () => {
    if (statusPollingInterval) {
      clearInterval(statusPollingInterval)
      setStatusPollingInterval(null)
    }
  }


  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">部署管理</h1>
        <p className="text-gray-600">
          验证配置、部署 Frigate 容器并监控运行状况
        </p>
      </div>

      {/* Device Validation Warnings */}
      {showDeviceWarnings && deviceValidation && deviceValidation.warnings.length > 0 && (
        <Card className="mb-6 bg-yellow-50 border border-yellow-200">
          <div className="flex items-start">
            <span className="text-yellow-500 text-2xl mr-3">⚠️</span>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-yellow-900 mb-2">设备验证警告</h3>
              <p className="text-sm text-yellow-800 mb-3">
                以下设备不存在或不可用。部署仍将继续,但这些设备可能无法被 Frigate 识别:
              </p>
              <ul className="space-y-1 mb-3">
                {deviceValidation.warnings.map((warning, idx) => (
                  <li key={idx} className="text-sm text-yellow-700">
                    {warning}
                  </li>
                ))}
              </ul>
              <div className="flex items-center justify-between mt-4 pt-3 border-t border-yellow-200">
                <div className="text-sm text-yellow-800">
                  <strong>{deviceValidation.valid_count}</strong> / {deviceValidation.total_devices} 设备可用
                </div>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setShowDeviceWarnings(false)}
                  icon="×"
                >
                  关闭
                </Button>
              </div>
            </div>
          </div>
        </Card>
      )}

      {/* Deployment Response Warnings */}
      {currentDeployment && currentDeployment.warnings && currentDeployment.warnings.length > 0 && (
        <Card className="mb-6 bg-yellow-50 border border-yellow-200">
          <div className="flex items-start">
            <span className="text-yellow-500 text-2xl mr-3">⚠️</span>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-yellow-900 mb-2">部署警告</h3>
              <p className="text-sm text-yellow-800 mb-3">
                部署已成功,但发现以下问题:
              </p>
              <ul className="space-y-1">
                {currentDeployment.warnings.map((warning, idx) => (
                  <li key={idx} className="text-sm text-yellow-700">
                    {warning}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </Card>
      )}

      {/* Configuration Section */}
      <Card className="mb-6">
        <h2 className="text-xl font-semibold mb-4">配置设置</h2>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              配置文件路径
            </label>
            <input
              type="text"
              value={configPath}
              onChange={(e) => setConfigPath(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="/etc/frigate/config.yml"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              部署方式
            </label>
            <select
              value={deploymentMethod}
              onChange={(e) => setDeploymentMethod(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="DockerRun">Docker Run</option>
              <option value="DockerCompose">Docker Compose</option>
            </select>
          </div>
        </div>

        {/* Phase 3 - 显示已配置的硬件设备 */}
        <div className="mt-4 pt-4 border-t border-gray-200">
          <div className="flex items-center justify-between mb-2">
            <div className="text-sm font-medium text-gray-700">硬件设备配置</div>
            <a
              href="/hardware"
              className="text-sm text-blue-600 hover:text-blue-700 font-medium"
            >
              管理硬件设备 →
            </a>
          </div>
          <div className="text-sm text-gray-600">
            {loadingDevices ? (
              <p>正在加载设备...</p>
            ) : savedDevices && savedDevices.length > 0 ? (
              <div>
                <p className="mb-2">已配置 <span className="font-semibold text-green-600">{savedDevices.length}</span> 个硬件设备</p>
                <div className="space-y-1 text-xs bg-gray-50 p-2 rounded max-h-32 overflow-y-auto">
                  {savedDevices.map((device, idx) => (
                    <div key={idx} className="font-mono text-gray-700">• {device}</div>
                  ))}
                </div>
              </div>
            ) : (
              <p>尚未配置硬件设备。前往 <span className="font-semibold">硬件检测</span> 页面添加 GPU/TPU 加速器。</p>
            )}
          </div>
        </div>

        {/* T189: Volume mapping integration */}
        <div className="mt-4 pt-4 border-t border-gray-200">
          <div className="flex items-center justify-between mb-2">
            <div className="text-sm font-medium text-gray-700">存储卷配置</div>
            <a
              href="/disk-mapping"
              className="text-sm text-blue-600 hover:text-blue-700 font-medium"
            >
              配置卷映射 →
            </a>
          </div>
          <div className="text-sm text-gray-600">
            <p>当前使用默认卷映射。前往 <span className="font-semibold">磁盘映射</span> 页面自定义存储位置。</p>
            <div className="mt-2 space-y-1 text-xs bg-gray-50 p-2 rounded">
              <div className="font-mono text-gray-700">• /etc/frigate → /config</div>
              <div className="font-mono text-gray-700">• /media/frigate → /media/frigate</div>
            </div>
          </div>
        </div>
      </Card>

      {/* Action Buttons */}
      <Card className="mb-6">
        <div className="flex items-center justify-between">
          <div className="flex space-x-2">
            <Button
              onClick={handleValidate}
              loading={validating}
              disabled={isDeploying}
              icon="✓"
              variant="outline"
            >
              {validating ? '验证中...' : '验证配置'}
            </Button>
            <Button
              onClick={handleValidateDevices}
              loading={validatingDevices}
              disabled={isDeploying}
              icon="🔍"
              variant="outline"
            >
              {validatingDevices ? '验证中...' : '验证设备'}
            </Button>
            <Button
              onClick={handleDeploy}
              loading={deploying || isDeploying}
              disabled={!validationResult?.valid || isDeploying}
              icon="🚀"
            >
              {deploying ? '部署中...' : '开始部署'}
            </Button>
            {currentDeployment && (
              <Button
                onClick={handleRollback}
                loading={rollingBack}
                disabled={isDeploying}
                icon="↩"
                variant="outline"
              >
                {rollingBack ? '回滚中...' : '回滚'}
              </Button>
            )}
          </div>
          <div className="flex space-x-2">
            <Button
              onClick={() => setShowCommandPreview(!showCommandPreview)}
              variant="ghost"
              icon="👁"
            >
              {showCommandPreview ? '隐藏' : '查看'}命令
            </Button>
            <Button
              onClick={() => setShowComposePreview(!showComposePreview)}
              variant="ghost"
              icon="🐳"
            >
              {showComposePreview ? '隐藏' : '查看'}docker-compose.yml
            </Button>
          </div>
        </div>
      </Card>

      {/* docker-compose.yml Preview */}
      {showComposePreview && composeYaml && (
        <Card className="mb-6 bg-gray-50">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-lg font-semibold">docker-compose.yml - Docker 部署配置</h3>
          </div>
          <pre className="bg-gray-900 text-gray-100 p-4 rounded-md overflow-x-auto text-sm font-mono max-h-96 overflow-y-auto">
            {composeYaml}
          </pre>
          <div className="mt-2 text-xs text-gray-600">
            🐳 说明: 此文件用于 Docker Compose 部署，包含容器、卷映射、端口和环境变量配置。
          </div>
        </Card>
      )}

      {/* Command Preview */}
      {showCommandPreview && currentDeployment && (
        <Card className="mb-6 bg-gray-50">
          <h3 className="text-lg font-semibold mb-2">部署命令</h3>
          <pre className="bg-gray-900 text-green-400 p-4 rounded-md overflow-x-auto text-sm font-mono">
            {currentDeployment.command}
          </pre>
        </Card>
      )}

      {/* Deployment Progress */}
      <DeploymentProgress
        step={deploymentStep}
        currentDeployment={currentDeployment}
        error={deploymentError}
      />

      {/* Phase 3 - 实时部署日志 */}
      {deploymentStep === 'deploying' && deploymentLogs.length > 0 && (
        <Card className="mb-6 bg-gray-50">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-lg font-semibold">部署日志（实时）</h3>
            <span className="text-sm text-gray-500">每2秒更新</span>
          </div>
          <div className="bg-gray-900 text-gray-100 p-4 rounded-md overflow-x-auto max-h-64 overflow-y-auto">
            <pre className="text-xs font-mono">
              {deploymentLogs.map((log, idx) => (
                <div key={idx} className="hover:bg-gray-800">
                  {log}
                </div>
              ))}
            </pre>
          </div>
        </Card>
      )}

      {/* Validation Results */}
      {validationResult && (
        <ValidationResults
          result={validationResult}
          error={validationError}
        />
      )}

      {/* Health Check Status */}
      {healthStatus && (
        <HealthCheckStatus
          status={healthStatus}
          loading={checkingHealth}
          error={healthError}
          onRecheck={() => currentDeployment?.container_id && checkHealth({ container_id: currentDeployment.container_id })}
        />
      )}

      {/* Deployment History */}
      <Card className="mt-6">
        <h2 className="text-xl font-semibold mb-4">部署历史</h2>
        {deploymentHistory.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            暂无部署历史
          </div>
        ) : (
          <div className="space-y-2">
            {deploymentHistory.slice(0, 5).map((deployment) => (
              <div
                key={deployment.id}
                className="flex items-center justify-between p-3 border border-gray-200 rounded-md hover:bg-gray-50"
              >
                <div className="flex-1">
                  <div className="font-medium text-gray-900">
                    {deployment.container_id}
                  </div>
                  <div className="text-sm text-gray-600">
                    {new Date(deployment.deployment_time).toLocaleString()}
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  <span className={`px-2 py-1 text-xs rounded-full ${
                    deployment.status === 'Running' ? 'bg-green-100 text-green-800' :
                    deployment.status === 'Failed' ? 'bg-red-100 text-red-800' :
                    'bg-gray-100 text-gray-800'
                  }`}>
                    {deployment.status}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  )
}

export default DeployPage
