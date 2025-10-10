// HealthCheckStatus Component - T131
// Displays health check results and status

import React from 'react'
import Card from './Card'
import Button from './Button'

interface HealthCheck {
  name: string
  passed: boolean
  message: string
  details: string | null
}

interface HealthCheckResponse {
  container_id: string
  status: string
  checks: HealthCheck[]
  response_time_ms: number
  timestamp: string
}

interface HealthCheckStatusProps {
  status: HealthCheckResponse
  loading: boolean
  error: Error | null
  onRecheck: () => void
}

const HealthCheckStatus: React.FC<HealthCheckStatusProps> = ({
  status,
  loading,
  error,
  onRecheck,
}) => {
  const isHealthy = status.status.toLowerCase().includes('healthy')
  const allPassed = status.checks.every(check => check.passed)

  return (
    <Card className={`mb-6 ${isHealthy && allPassed ? 'border-green-200 bg-green-50' : 'border-yellow-200 bg-yellow-50'}`}>
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-start">
          <span className="text-3xl mr-3">
            {isHealthy && allPassed ? '💚' : '⚠️'}
          </span>
          <div>
            <h3 className="text-xl font-semibold mb-1">健康检查</h3>
            <div className="flex items-center space-x-2">
              <span className={`px-3 py-1 text-sm font-medium rounded-full ${
                isHealthy && allPassed ? 'bg-green-100 text-green-800' :
                'bg-yellow-100 text-yellow-800'
              }`}>
                {status.status}
              </span>
              <span className="text-sm text-gray-600">
                响应时间: {status.response_time_ms}ms
              </span>
            </div>
          </div>
        </div>
        <Button
          onClick={onRecheck}
          loading={loading}
          variant="outline"
          size="sm"
          icon="🔄"
        >
          {loading ? '检查中...' : '重新检查'}
        </Button>
      </div>

      {error && (
        <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded">
          <div className="font-medium text-red-900 mb-1">健康检查错误</div>
          <div className="text-sm text-red-700">{error.message}</div>
        </div>
      )}

      {/* Container Info */}
      <div className="mb-4 p-3 bg-white rounded border border-gray-200">
        <div className="flex items-center justify-between">
          <div>
            <div className="text-sm text-gray-600">容器 ID</div>
            <div className="font-mono text-sm font-medium">{status.container_id}</div>
          </div>
          <div className="text-right">
            <div className="text-sm text-gray-600">检查时间</div>
            <div className="text-sm font-medium">
              {new Date(status.timestamp).toLocaleString()}
            </div>
          </div>
        </div>
      </div>

      {/* Health Checks */}
      <div>
        <h4 className="font-semibold text-gray-900 mb-2">检查项目</h4>
        <div className="space-y-2">
          {status.checks.map((check, index) => (
            <div
              key={index}
              className="p-3 bg-white rounded border border-gray-200"
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start flex-1">
                  <span className="text-xl mr-2">
                    {check.passed ? '✅' : '❌'}
                  </span>
                  <div className="flex-1">
                    <div className="font-medium text-sm">{check.name}</div>
                    <div className={`text-xs mt-1 ${check.passed ? 'text-gray-600' : 'text-red-600'}`}>
                      {check.message}
                    </div>
                    {check.details && (
                      <details className="mt-2">
                        <summary className="cursor-pointer text-xs text-blue-600 hover:text-blue-800">
                          查看详情
                        </summary>
                        <div className="mt-1 text-xs text-gray-600 font-mono bg-gray-50 p-2 rounded">
                          {check.details}
                        </div>
                      </details>
                    )}
                  </div>
                </div>
                <span className={`text-xs px-2 py-1 rounded-full ml-2 ${
                  check.passed ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                }`}>
                  {check.passed ? '通过' : '失败'}
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Summary */}
      <div className="mt-4 pt-4 border-t border-gray-200">
        <div className="flex items-center justify-between text-sm">
          <div>
            <span className="text-gray-600">检查项: </span>
            <span className="font-medium">{status.checks.length}</span>
          </div>
          <div>
            <span className="text-green-600 font-medium">
              {status.checks.filter(c => c.passed).length} 通过
            </span>
            <span className="text-gray-400 mx-2">|</span>
            <span className="text-red-600 font-medium">
              {status.checks.filter(c => !c.passed).length} 失败
            </span>
          </div>
        </div>
      </div>

      {/* Recommendations */}
      {!allPassed && (
        <div className="mt-4 p-3 bg-yellow-50 border border-yellow-200 rounded">
          <div className="font-medium text-yellow-900 mb-1">建议</div>
          <ul className="text-sm text-yellow-800 space-y-1 list-disc list-inside">
            <li>检查容器日志以获取更多错误信息</li>
            <li>验证配置文件是否正确</li>
            <li>确认所有必需的端口和设备可访问</li>
            {!isHealthy && <li>考虑回滚到之前的稳定版本</li>}
          </ul>
        </div>
      )}
    </Card>
  )
}

export default HealthCheckStatus
