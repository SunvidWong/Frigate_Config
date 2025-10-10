// Validation Panel Component
// Displays configuration validation results and warnings

import React, { useState, useEffect } from 'react'
import { AlertCircle, CheckCircle, AlertTriangle, RefreshCw, X, Wand2 } from 'lucide-react'
import { ValidationResult } from '../services/configService'
import { ConfigFixer, FixSuggestion } from '../services/configFixer'

interface ValidationPanelProps {
  validationResult: ValidationResult | null
  onRefresh: () => void
  yamlContent?: string
  onAutoFix?: (fixedContent: string, changes: string[]) => void
}

const ValidationPanel: React.FC<ValidationPanelProps> = ({
  validationResult,
  onRefresh,
  yamlContent,
  onAutoFix
}) => {
  const [fixSuggestions, setFixSuggestions] = useState<FixSuggestion[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);

  useEffect(() => {
    if (yamlContent && (validationResult?.errors.length || validationResult?.warnings.length)) {
      const suggestions = ConfigFixer.analyzConfig(yamlContent);
      setFixSuggestions(suggestions);
      setShowSuggestions(suggestions.length > 0);
    } else {
      setFixSuggestions([]);
      setShowSuggestions(false);
    }
  }, [yamlContent, validationResult]);

  const handleAutoFix = () => {
    if (!yamlContent || !onAutoFix) return;

    const { fixed, changes } = ConfigFixer.autoFix(yamlContent);
    onAutoFix(fixed, changes);
  };

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
  const canAutoFix = fixSuggestions.length > 0 && onAutoFix

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

        <div className="flex items-center space-x-2">
          {canAutoFix && (
            <button
              onClick={handleAutoFix}
              className="px-3 py-1 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded flex items-center space-x-1"
              title="自动修复常见问题"
            >
              <Wand2 className="h-4 w-4" />
              <span>自动修复</span>
            </button>
          )}
          <button
            onClick={onRefresh}
            className="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded"
            title="重新验证"
          >
            <RefreshCw className="h-4 w-4" />
          </button>
        </div>
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
        {/* Fix Suggestions */}
        {showSuggestions && fixSuggestions.length > 0 && (
          <div className="mb-6">
            <h4 className="font-medium text-blue-900 mb-3 flex items-center space-x-2">
              <Wand2 className="h-4 w-4" />
              <span>修复建议 ({fixSuggestions.length})</span>
            </h4>
            <div className="space-y-2">
              {fixSuggestions.map((suggestion, index) => (
                <SuggestionItem key={index} suggestion={suggestion} />
              ))}
            </div>
            {canAutoFix && (
              <button
                onClick={handleAutoFix}
                className="mt-3 w-full px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded flex items-center justify-center space-x-2"
              >
                <Wand2 className="h-4 w-4" />
                <span>应用所有修复</span>
              </button>
            )}
          </div>
        )}

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

interface SuggestionItemProps {
  suggestion: FixSuggestion
}

const SuggestionItem: React.FC<SuggestionItemProps> = ({ suggestion }) => {
  const bgColor = suggestion.severity === 'error' ? 'bg-red-50 border-red-200' : 'bg-blue-50 border-blue-200';
  const iconColor = suggestion.severity === 'error' ? 'text-red-600' : 'text-blue-600';
  const textColor = suggestion.severity === 'error' ? 'text-red-900' : 'text-blue-900';
  const descColor = suggestion.severity === 'error' ? 'text-red-700' : 'text-blue-700';

  return (
    <div className={`${bgColor} border rounded-lg p-3`}>
      <div className="flex items-start space-x-2">
        <Wand2 className={`h-4 w-4 ${iconColor} mt-0.5 flex-shrink-0`} />
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-1">
            <p className={`text-sm font-medium ${textColor}`}>
              {suggestion.issue}
            </p>
            <span className={`text-xs px-2 py-0.5 rounded-full ${
              suggestion.severity === 'error' ? 'bg-red-200 text-red-800' : 'bg-blue-200 text-blue-800'
            }`}>
              {suggestion.severity === 'error' ? '错误' : '警告'}
            </span>
          </div>
          <p className={`text-xs ${descColor} mb-2`}>
            {suggestion.description}
          </p>
          <div className={`text-xs font-medium ${iconColor} flex items-center space-x-1`}>
            <span>✓</span>
            <span>修复: {suggestion.fix}</span>
          </div>
        </div>
      </div>
    </div>
  )
}

export default ValidationPanel