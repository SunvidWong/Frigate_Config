// Validation Panel Component
// Displays configuration validation results and warnings

import React from 'react'
import { AlertCircle, CheckCircle, AlertTriangle, RefreshCw, X } from 'lucide-react'
import { ValidationResult } from '../services/configService'

interface ValidationPanelProps {
  validationResult: ValidationResult | null
  onRefresh: () => void
}

const ValidationPanel: React.FC<ValidationPanelProps> = ({
  validationResult,
  onRefresh
}) => {
  if (!validationResult) {
    return (
      <div className="h-full bg-gray-50 border-l border-gray-200 p-4">
        <div className="flex items-center justify-center h-full text-gray-500">
          <div className="text-center">
            <RefreshCw className="h-8 w-8 mx-auto mb-2 opacity-50" />
            <p className="text-sm">等待验证...</p>
          </div>
        </div>
      </div>
    )
  }

  const hasErrors = validationResult.errors.length > 0
  const hasWarnings = validationResult.warnings.length > 0

  return (
    <div className="h-full bg-gray-50 border-l border-gray-200 flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <div className="flex items-center space-x-2">
          {validationResult.valid ? (
            <CheckCircle className="h-5 w-5 text-green-600" />
          ) : (
            <AlertCircle className="h-5 w-5 text-red-600" />
          )}
          <h3 className="font-medium text-gray-900">配置验证</h3>
        </div>

        <button
          onClick={onRefresh}
          className="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded"
          title="重新验证"
        >
          <RefreshCw className="h-4 w-4" />
        </button>
      </div>

      {/* Status Summary */}
      <div className={`p-4 border-b border-gray-200 ${
        validationResult.valid ? 'bg-green-50' : 'bg-red-50'
      }`}>
        <div className="flex items-center space-x-2">
          {validationResult.valid ? (
            <>
              <CheckCircle className="h-5 w-5 text-green-600" />
              <span className="font-medium text-green-900">配置有效</span>
            </>
          ) : (
            <>
              <AlertCircle className="h-5 w-5 text-red-600" />
              <span className="font-medium text-red-900">配置有错误</span>
            </>
          )}
        </div>

        <div className="mt-2 text-sm text-gray-600">
          <p>类型: {validationResult.config_type}</p>
          <p>版本: {validationResult.schema_version}</p>
        </div>

        {/* Statistics */}
        <div className="mt-3 flex items-center space-x-4 text-sm">
          {hasErrors && (
            <div className="flex items-center space-x-1 text-red-600">
              <X className="h-4 w-4" />
              <span>{validationResult.errors.length} 错误</span>
            </div>
          )}
          {hasWarnings && (
            <div className="flex items-center space-x-1 text-yellow-600">
              <AlertTriangle className="h-4 w-4" />
              <span>{validationResult.warnings.length} 警告</span>
            </div>
          )}
          {!hasErrors && !hasWarnings && (
            <div className="flex items-center space-x-1 text-green-600">
              <CheckCircle className="h-4 w-4" />
              <span>无问题</span>
            </div>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {/* Errors */}
        {hasErrors && (
          <div className="mb-6">
            <h4 className="font-medium text-red-900 mb-3 flex items-center space-x-2">
              <AlertCircle className="h-4 w-4" />
              <span>错误 ({validationResult.errors.length})</span>
            </h4>
            <div className="space-y-2">
              {validationResult.errors.map((error, index) => (
                <ErrorItem key={index} error={error} />
              ))}
            </div>
          </div>
        )}

        {/* Warnings */}
        {hasWarnings && (
          <div className="mb-6">
            <h4 className="font-medium text-yellow-900 mb-3 flex items-center space-x-2">
              <AlertTriangle className="h-4 w-4" />
              <span>警告 ({validationResult.warnings.length})</span>
            </h4>
            <div className="space-y-2">
              {validationResult.warnings.map((warning, index) => (
                <WarningItem key={index} warning={warning} />
              ))}
            </div>
          </div>
        )}

        {/* Success message */}
        {!hasErrors && !hasWarnings && (
          <div className="text-center py-8">
            <CheckCircle className="h-12 w-12 text-green-500 mx-auto mb-3" />
            <h3 className="text-lg font-medium text-green-900 mb-1">配置完全有效</h3>
            <p className="text-sm text-green-700">
              配置文件通过了所有验证检查，可以安全使用。
            </p>
          </div>
        )}
      </div>
    </div>
  )
}

interface ErrorItemProps {
  error: {
    field: string
    message: string
    severity: 'error' | 'warning'
  }
}

const ErrorItem: React.FC<ErrorItemProps> = ({ error }) => {
  return (
    <div className="bg-red-50 border border-red-200 rounded-lg p-3">
      <div className="flex items-start space-x-2">
        <AlertCircle className="h-4 w-4 text-red-600 mt-0.5 flex-shrink-0" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between">
            <p className="text-sm font-medium text-red-900 truncate">
              {error.field}
            </p>
            <span className="text-xs text-red-600 uppercase">
              {error.severity}
            </span>
          </div>
          <p className="text-sm text-red-700 mt-1">
            {error.message}
          </p>
        </div>
      </div>
    </div>
  )
}

interface WarningItemProps {
  warning: string
}

const WarningItem: React.FC<WarningItemProps> = ({ warning }) => {
  return (
    <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-3">
      <div className="flex items-start space-x-2">
        <AlertTriangle className="h-4 w-4 text-yellow-600 mt-0.5 flex-shrink-0" />
        <div className="flex-1">
          <p className="text-sm text-yellow-800">
            {warning}
          </p>
        </div>
      </div>
    </div>
  )
}

export default ValidationPanel