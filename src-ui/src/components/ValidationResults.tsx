// ValidationResults Component - T129
// Displays validation results with errors and warnings

import React from 'react'
import Card from './Card'

interface ValidationCheck {
  name: string
  passed: boolean
  message: string
}

interface ValidationResult {
  valid: boolean
  errors: string[]
  warnings: string[]
  checks: ValidationCheck[]
}

interface ValidationResultsProps {
  result: ValidationResult
  error: Error | null
}

const ValidationResults: React.FC<ValidationResultsProps> = ({ result, error }) => {
  if (error) {
    return (
      <Card className="mb-6 border-red-200 bg-red-50">
        <div className="flex items-start">
          <span className="text-2xl mr-3">❌</span>
          <div>
            <h3 className="font-semibold text-red-900 mb-1">验证失败</h3>
            <p className="text-red-700">{error.message}</p>
          </div>
        </div>
      </Card>
    )
  }

  const { valid, errors, warnings, checks } = result

  return (
    <Card className={`mb-6 ${valid ? 'border-green-200 bg-green-50' : 'border-red-200 bg-red-50'}`}>
      <div className="flex items-start mb-4">
        <span className="text-3xl mr-3">{valid ? '✅' : '❌'}</span>
        <div>
          <h3 className="text-xl font-semibold mb-1">
            {valid ? '配置验证通过' : '配置验证失败'}
          </h3>
          <p className={valid ? 'text-green-700' : 'text-red-700'}>
            {valid
              ? '您的配置已通过所有验证检查，可以安全部署。'
              : '配置存在问题，请修复后再部署。'}
          </p>
        </div>
      </div>

      {/* Errors */}
      {errors.length > 0 && (
        <div className="mb-4">
          <h4 className="font-semibold text-red-900 mb-2">错误 ({errors.length})</h4>
          <div className="space-y-1">
            {errors.map((error, index) => (
              <div key={index} className="flex items-start text-sm">
                <span className="text-red-500 mr-2">•</span>
                <span className="text-red-800">{error}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Warnings */}
      {warnings.length > 0 && (
        <div className="mb-4">
          <h4 className="font-semibold text-yellow-900 mb-2">警告 ({warnings.length})</h4>
          <div className="space-y-1">
            {warnings.map((warning, index) => (
              <div key={index} className="flex items-start text-sm">
                <span className="text-yellow-500 mr-2">•</span>
                <span className="text-yellow-800">{warning}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Validation Checks */}
      <div>
        <h4 className="font-semibold text-gray-900 mb-2">验证检查项</h4>
        <div className="space-y-2">
          {checks.map((check, index) => (
            <div
              key={index}
              className="flex items-start justify-between p-2 bg-white rounded border border-gray-200"
            >
              <div className="flex items-start flex-1">
                <span className="text-lg mr-2">
                  {check.passed ? '✓' : '✗'}
                </span>
                <div>
                  <div className="font-medium text-sm">{check.name}</div>
                  <div className={`text-xs ${check.passed ? 'text-gray-600' : 'text-red-600'}`}>
                    {check.message}
                  </div>
                </div>
              </div>
              <span className={`text-xs px-2 py-1 rounded-full ${
                check.passed ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
              }`}>
                {check.passed ? '通过' : '失败'}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* Summary */}
      <div className="mt-4 pt-4 border-t border-gray-200">
        <div className="flex items-center justify-between text-sm">
          <div>
            <span className="text-gray-600">检查项: </span>
            <span className="font-medium">{checks.length}</span>
          </div>
          <div>
            <span className="text-green-600 font-medium">
              {checks.filter(c => c.passed).length} 通过
            </span>
            <span className="text-gray-400 mx-2">|</span>
            <span className="text-red-600 font-medium">
              {checks.filter(c => !c.passed).length} 失败
            </span>
          </div>
        </div>
      </div>
    </Card>
  )
}

export default ValidationResults
