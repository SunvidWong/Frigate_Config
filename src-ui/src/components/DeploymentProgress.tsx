// DeploymentProgress Component - T130
// Displays deployment progress through different stages

import React from 'react'
import Card from './Card'

interface DeploymentResponse {
  success: boolean
  container_id: string | null
  command: string
  stdout: string
  stderr: string
  exit_code: number | null
  deployment_time: string
}

type DeploymentStep = 'idle' | 'validating' | 'deploying' | 'health_check' | 'completed' | 'failed'

interface DeploymentProgressProps {
  step: DeploymentStep
  currentDeployment: DeploymentResponse | null
  error: Error | null
}

const DeploymentProgress: React.FC<DeploymentProgressProps> = ({
  step,
  currentDeployment,
  error,
}) => {
  if (step === 'idle') {
    return null
  }

  const steps = [
    { id: 'validating', label: '验证配置', icon: '🔍' },
    { id: 'deploying', label: '执行部署', icon: '🚀' },
    { id: 'health_check', label: '健康检查', icon: '💚' },
    { id: 'completed', label: '部署完成', icon: '✅' },
  ]

  const getStepStatus = (stepId: string): 'pending' | 'active' | 'completed' | 'error' => {
    if (step === 'failed') {
      const stepIndex = steps.findIndex(s => s.id === stepId)
      const currentIndex = steps.findIndex(s => s.id === step)
      if (stepIndex < currentIndex) return 'completed'
      if (stepIndex === currentIndex) return 'error'
      return 'pending'
    }

    const stepIndex = steps.findIndex(s => s.id === stepId)
    const currentIndex = steps.findIndex(s => s.id === step)

    if (stepIndex < currentIndex) return 'completed'
    if (stepIndex === currentIndex) return 'active'
    return 'pending'
  }

  return (
    <Card className="mb-6">
      <h3 className="text-lg font-semibold mb-4">部署进度</h3>

      {/* Progress Steps */}
      <div className="relative">
        <div className="absolute top-5 left-0 right-0 h-0.5 bg-gray-200" />
        <div className="relative flex justify-between">
          {steps.map((s) => {
            const status = getStepStatus(s.id)
            return (
              <div key={s.id} className="flex flex-col items-center z-10">
                <div
                  className={`
                    w-10 h-10 rounded-full flex items-center justify-center mb-2
                    ${status === 'completed' ? 'bg-green-500 text-white' :
                      status === 'active' ? 'bg-blue-500 text-white animate-pulse' :
                      status === 'error' ? 'bg-red-500 text-white' :
                      'bg-gray-200 text-gray-500'}
                  `}
                >
                  <span className="text-lg">{s.icon}</span>
                </div>
                <div className={`text-sm font-medium ${
                  status === 'active' ? 'text-blue-600' :
                  status === 'completed' ? 'text-green-600' :
                  status === 'error' ? 'text-red-600' :
                  'text-gray-500'
                }`}>
                  {s.label}
                </div>
              </div>
            )
          })}
        </div>
      </div>

      {/* Current Step Details */}
      <div className="mt-6 p-4 bg-gray-50 rounded-lg">
        {step === 'validating' && (
          <div className="flex items-center">
            <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-500 mr-3"></div>
            <span className="text-gray-700">正在验证配置文件...</span>
          </div>
        )}

        {step === 'deploying' && (
          <div className="space-y-2">
            <div className="flex items-center">
              <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-500 mr-3"></div>
              <span className="text-gray-700">正在启动 Frigate 容器...</span>
            </div>
            {currentDeployment?.stdout && (
              <div className="mt-2 p-2 bg-gray-900 text-green-400 rounded text-xs font-mono max-h-32 overflow-y-auto">
                {currentDeployment.stdout}
              </div>
            )}
          </div>
        )}

        {step === 'health_check' && (
          <div className="flex items-center">
            <div className="animate-pulse rounded-full h-5 w-5 bg-green-500 mr-3"></div>
            <span className="text-gray-700">正在执行健康检查...</span>
          </div>
        )}

        {step === 'completed' && (
          <div className="flex items-center">
            <span className="text-2xl mr-3">🎉</span>
            <div>
              <div className="font-semibold text-green-700">部署成功！</div>
              <div className="text-sm text-gray-600">
                容器 ID: {currentDeployment?.container_id}
              </div>
              <div className="text-sm text-gray-600">
                部署时间: {currentDeployment?.deployment_time && new Date(currentDeployment.deployment_time).toLocaleString()}
              </div>
            </div>
          </div>
        )}

        {step === 'failed' && (
          <div className="flex items-start">
            <span className="text-2xl mr-3">❌</span>
            <div className="flex-1">
              <div className="font-semibold text-red-700 mb-1">部署失败</div>
              {error && (
                <div className="text-sm text-red-600 mb-2">{error.message}</div>
              )}
              {currentDeployment?.stderr && (
                <details className="mt-2">
                  <summary className="cursor-pointer text-sm text-gray-700 hover:text-gray-900">
                    查看错误详情
                  </summary>
                  <div className="mt-2 p-2 bg-red-50 border border-red-200 rounded text-xs font-mono max-h-32 overflow-y-auto">
                    {currentDeployment.stderr}
                  </div>
                </details>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Output Logs */}
      {currentDeployment && (currentDeployment.stdout || currentDeployment.stderr) && step !== 'deploying' && (
        <details className="mt-4">
          <summary className="cursor-pointer font-medium text-gray-700 hover:text-gray-900">
            查看部署日志
          </summary>
          <div className="mt-2 space-y-2">
            {currentDeployment.stdout && (
              <div>
                <div className="text-sm text-gray-600 mb-1">标准输出:</div>
                <pre className="bg-gray-900 text-green-400 p-3 rounded text-xs font-mono max-h-48 overflow-y-auto">
                  {currentDeployment.stdout}
                </pre>
              </div>
            )}
            {currentDeployment.stderr && (
              <div>
                <div className="text-sm text-gray-600 mb-1">标准错误:</div>
                <pre className="bg-gray-900 text-red-400 p-3 rounded text-xs font-mono max-h-48 overflow-y-auto">
                  {currentDeployment.stderr}
                </pre>
              </div>
            )}
          </div>
        </details>
      )}
    </Card>
  )
}

export default DeploymentProgress
