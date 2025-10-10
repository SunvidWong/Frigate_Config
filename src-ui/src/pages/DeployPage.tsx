// Deploy Page - T128-T136
// Handles deployment validation, execution, health checks, and rollback

import React, { useState, useEffect } from 'react'
import { useTauriCommand } from '../hooks/useTauriCommand'
import Button from '../components/Button'
import Card from '../components/Card'
import ValidationResults from '../components/ValidationResults'
import DeploymentProgress from '../components/DeploymentProgress'
import HealthCheckStatus from '../components/HealthCheckStatus'
import { ConfigFixer } from '../services/configFixer'
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
  const [showConfigPreview, setShowConfigPreview] = useState(false)
  const [showComposePreview, setShowComposePreview] = useState(false)
  const [configYaml, setConfigYaml] = useState<string>('')
  const [composeYaml, setComposeYaml] = useState<string>('')
  const [isDeploying, setIsDeploying] = useState(false)
  const [deploymentStep, setDeploymentStep] = useState<'idle' | 'validating' | 'deploying' | 'health_check' | 'completed' | 'failed'>('idle')

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

  // Load deployment history and configs on mount
  useEffect(() => {
    loadHistory()

    // Load config.yml from localStorage
    const savedConfig = localStorage.getItem('current_config')
    if (savedConfig) {
      setConfigYaml(savedConfig)
    }

    // Generate docker-compose.yml
    const compose = DockerComposeGenerator.generateCompose({
      volumes: {
        config_path: './config',
        storage_path: './storage',
        cache_size: 1000000000
      },
      ports: {
        web_port: 8971,
        rtsp_port: 8554,
        webrtc_tcp_port: 8555,
        webrtc_udp_port: 8555
      },
      devices: [],
      environment: {
        FRIGATE_RTSP_PASSWORD: 'password'
      },
      shm_size: '256mb',
      privileged: true
    })
    setComposeYaml(compose)
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
        // Automatically start health check
        checkHealth({ container_id: deploymentData.container_id })
      } else {
        setDeploymentStep('failed')
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
    }
  }, [healthData])

  // Handle validation
  const handleValidate = async () => {
    setDeploymentStep('validating')
    setValidationResult(null)
    await runValidation({ config_path: configPath })
  }

  // Handle deployment
  const handleDeploy = async () => {
    setIsDeploying(true)
    setDeploymentStep('deploying')

    const config: DeploymentConfig = {
      config_path: configPath,
      method: deploymentMethod,
      devices: [],
      volumes: [
        { host_path: '/etc/frigate', container_path: '/config' },
        { host_path: '/media/frigate', container_path: '/media/frigate' },
      ],
      ports: [
        { host_port: 5000, container_port: 5000 },
        { host_port: 8554, container_port: 8554 },
        { host_port: 8555, container_port: 8555 },
      ],
      environment: {
        TZ: 'UTC',
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
  }

  // Handle config quick fix
  const handleConfigQuickFix = () => {
    if (!configYaml) return
    const { fixed, changes } = ConfigFixer.autoFix(configYaml)
    setConfigYaml(fixed)
    // Save fixed content
    localStorage.setItem('current_config', fixed)
    // Show changes in alert
    if (changes.length > 0) {
      alert(`自动修复完成:\n${changes.map((c, i) => `${i + 1}. ${c}`).join('\n')}`)
    }
  }

  // Handle delete config
  const handleDeleteConfig = () => {
    const confirmed = confirm('确定要删除 config.yml 吗？此操作无法撤销。')
    if (!confirmed) return

    // Clear storage
    localStorage.removeItem('current_config')
    localStorage.removeItem('generated_config')

    // Reset state
    setConfigYaml('')
    alert('config.yml 已删除')
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
              onClick={() => setShowConfigPreview(!showConfigPreview)}
              variant="ghost"
              icon="📄"
            >
              {showConfigPreview ? '隐藏' : '查看'}config.yml
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

      {/* config.yml Preview */}
      {showConfigPreview && configYaml && (
        <Card className="mb-6 bg-gray-50">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-lg font-semibold">config.yml - Frigate 配置文件</h3>
            <div className="flex items-center space-x-2">
              <Button
                onClick={handleConfigQuickFix}
                variant="outline"
                icon="🔧"
                disabled={!configYaml}
              >
                一键修复
              </Button>
              <Button
                onClick={handleDeleteConfig}
                variant="outline"
                icon="🗑️"
                disabled={!configYaml}
              >
                删除配置
              </Button>
              <a
                href="/config-editor"
                className="text-sm text-blue-600 hover:text-blue-700 font-medium"
              >
                编辑配置 →
              </a>
            </div>
          </div>
          <pre className="bg-gray-900 text-gray-100 p-4 rounded-md overflow-x-auto text-sm font-mono max-h-96 overflow-y-auto">
            {configYaml}
          </pre>
          <div className="mt-2 text-xs text-gray-600">
            📄 说明: 此文件包含 Frigate 的摄像头、检测器和录制配置。
          </div>
        </Card>
      )}

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
