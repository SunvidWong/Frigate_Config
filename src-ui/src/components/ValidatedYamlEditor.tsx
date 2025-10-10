// T166: Validated YAML Editor with Real-Time Validation
// Wraps YamlEditor with real-time YAML validation and error reporting

import React, { useState, useEffect, useCallback, useMemo } from 'react'
import YamlEditor from './YamlEditor'
import { invoke } from '@tauri-apps/api/tauri'

interface ValidatedYamlEditorProps {
  content: string
  onChange: (content: string) => void
  onSave?: () => void
  filePath?: string
  readOnly?: boolean
  placeholder?: string
  minHeight?: string
  showValidation?: boolean
  debounceMs?: number
}

interface ValidationError {
  message: string
  line_number?: number
  error_type?: string
}

interface ValidationWarning {
  message: string
  line_number?: number
  suggestion?: string
}

interface ValidationResult {
  valid: boolean
  errors: ValidationError[]
  warnings: ValidationWarning[]
  cameras_count: number
  detectors_count: number
}

const ValidatedYamlEditor: React.FC<ValidatedYamlEditorProps> = ({
  content,
  onChange,
  onSave,
  filePath = 'config.yaml',
  readOnly = false,
  placeholder,
  minHeight,
  showValidation = true,
  debounceMs = 500,
}) => {
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null)
  const [isValidating, setIsValidating] = useState(false)
  const [validationError, setValidationError] = useState<string | null>(null)

  // Debounced validation
  const validateContent = useCallback(async (yamlContent: string) => {
    if (!showValidation) return
    if (!yamlContent.trim()) {
      setValidationResult({
        valid: false,
        errors: [{ message: '配置不能为空', line_number: 1, error_type: 'empty' }],
        warnings: [],
        cameras_count: 0,
        detectors_count: 0,
      })
      return
    }

    setIsValidating(true)
    setValidationError(null)

    try {
      // Call Tauri backend validation command
      const result = await invoke<ValidationResult>('validate_config', {
        content: yamlContent,
        filePath: filePath,
      })

      setValidationResult(result)
    } catch (error) {
      console.error('Validation error:', error)
      setValidationError(String(error))

      // Create error result from exception
      setValidationResult({
        valid: false,
        errors: [{
          message: `验证失败: ${error}`,
          error_type: 'validation_error',
        }],
        warnings: [],
        cameras_count: 0,
        detectors_count: 0,
      })
    } finally {
      setIsValidating(false)
    }
  }, [showValidation, filePath])

  // Debounce validation
  useEffect(() => {
    const timer = setTimeout(() => {
      validateContent(content)
    }, debounceMs)

    return () => clearTimeout(timer)
  }, [content, validateContent, debounceMs])

  // Initial validation
  useEffect(() => {
    validateContent(content)
  }, []) // Only on mount

  // Handle save with validation check
  const handleSave = useCallback(() => {
    if (validationResult && validationResult.errors.length > 0) {
      // Don't save if there are errors
      return
    }

    if (onSave) {
      onSave()
    }
  }, [onSave, validationResult])

  // Memoize error and warning indicators
  const { errorCount, warningCount } = useMemo(() => {
    if (!validationResult) {
      return { errorCount: 0, warningCount: 0 }
    }

    return {
      errorCount: validationResult.errors.length,
      warningCount: validationResult.warnings.length,
    }
  }, [validationResult])

  // Get validation status badge
  const getValidationBadge = () => {
    if (isValidating) {
      return (
        <div className="flex items-center space-x-2 px-3 py-1 bg-blue-50 border border-blue-200 rounded text-sm">
          <div className="animate-spin h-4 w-4 border-2 border-blue-500 border-t-transparent rounded-full"></div>
          <span className="text-blue-700">验证中...</span>
        </div>
      )
    }

    if (!validationResult) {
      return null
    }

    if (validationResult.valid) {
      return (
        <div className="flex items-center space-x-2 px-3 py-1 bg-green-50 border border-green-200 rounded text-sm">
          <span className="text-green-600">✓</span>
          <span className="text-green-700">配置有效</span>
          {validationResult.cameras_count > 0 && (
            <span className="text-green-600 text-xs">
              • {validationResult.cameras_count} 个摄像头
            </span>
          )}
          {validationResult.detectors_count > 0 && (
            <span className="text-green-600 text-xs">
              • {validationResult.detectors_count} 个检测器
            </span>
          )}
        </div>
      )
    }

    return (
      <div className="flex items-center space-x-2 px-3 py-1 bg-red-50 border border-red-200 rounded text-sm">
        <span className="text-red-600">✗</span>
        <span className="text-red-700">
          {errorCount} 个错误
          {warningCount > 0 && `, ${warningCount} 个警告`}
        </span>
      </div>
    )
  }

  // Get inline error markers for editor (reserved for future use)
  // const getLineErrors = useCallback((lineNumber: number) => {
  //   if (!validationResult) return { errors: [], warnings: [] }

  //   const lineErrors = validationResult.errors.filter(e => e.line_number === lineNumber)
  //   const lineWarnings = validationResult.warnings.filter(w => w.line_number === lineNumber)

  //   return { errors: lineErrors, warnings: lineWarnings }
  // }, [validationResult])

  return (
    <div className="flex flex-col space-y-3 h-full">
      {/* Validation Status Header */}
      {showValidation && (
        <div className="flex items-center justify-between">
          <div>
            {getValidationBadge()}
          </div>

          {validationError && (
            <div className="text-xs text-red-600">
              验证服务错误: {validationError}
            </div>
          )}
        </div>
      )}

      {/* YAML Editor */}
      <div className="flex-1 min-h-0">
        <YamlEditor
          content={content}
          onChange={onChange}
          readOnly={readOnly}
          placeholder={placeholder}
          minHeight={minHeight}
        />
      </div>

      {/* Detailed Validation Messages */}
      {showValidation && validationResult && (errorCount > 0 || warningCount > 0) && (
        <div className="space-y-2 max-h-48 overflow-y-auto">
          {/* Errors */}
          {validationResult.errors.map((error, index) => (
            <div
              key={`error-${index}`}
              className="flex items-start space-x-2 p-3 bg-red-50 border border-red-200 rounded text-sm"
            >
              <span className="flex-shrink-0 text-red-600 font-bold">⚠️</span>
              <div className="flex-1">
                <div className="flex items-center space-x-2">
                  <span className="font-semibold text-red-700">错误</span>
                  {error.line_number && (
                    <span className="text-xs text-red-600 bg-red-100 px-2 py-0.5 rounded">
                      行 {error.line_number}
                    </span>
                  )}
                  {error.error_type && (
                    <span className="text-xs text-red-500">
                      [{error.error_type}]
                    </span>
                  )}
                </div>
                <p className="mt-1 text-red-700">{error.message}</p>
              </div>
            </div>
          ))}

          {/* Warnings */}
          {validationResult.warnings.map((warning, index) => (
            <div
              key={`warning-${index}`}
              className="flex items-start space-x-2 p-3 bg-yellow-50 border border-yellow-200 rounded text-sm"
            >
              <span className="flex-shrink-0 text-yellow-600 font-bold">⚡</span>
              <div className="flex-1">
                <div className="flex items-center space-x-2">
                  <span className="font-semibold text-yellow-700">警告</span>
                  {warning.line_number && (
                    <span className="text-xs text-yellow-600 bg-yellow-100 px-2 py-0.5 rounded">
                      行 {warning.line_number}
                    </span>
                  )}
                </div>
                <p className="mt-1 text-yellow-700">{warning.message}</p>
                {warning.suggestion && (
                  <div className="mt-2 flex items-start space-x-1 text-xs text-yellow-600">
                    <span>💡</span>
                    <span>建议: {warning.suggestion}</span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Action Buttons */}
      {!readOnly && onSave && (
        <div className="flex items-center justify-end space-x-2 pt-3 border-t border-gray-200">
          <button
            onClick={handleSave}
            disabled={!validationResult || validationResult.errors.length > 0}
            className={`
              px-4 py-2 rounded font-medium transition-colors
              ${validationResult && validationResult.errors.length === 0
                ? 'bg-blue-600 hover:bg-blue-700 text-white'
                : 'bg-gray-300 text-gray-500 cursor-not-allowed'
              }
            `}
          >
            保存配置
          </button>
        </div>
      )}

      {/* Validation Help */}
      {showValidation && !readOnly && (
        <div className="p-3 bg-gray-50 border border-gray-200 rounded text-xs text-gray-600">
          <div className="font-semibold text-gray-700 mb-2">YAML 验证规则:</div>
          <ul className="space-y-1 list-disc list-inside">
            <li>配置会在您输入时自动验证 (延迟 {debounceMs}ms)</li>
            <li>错误必须修复后才能保存配置</li>
            <li>警告不会阻止保存，但建议处理</li>
            <li>YAML 语法错误会立即显示在对应行</li>
            <li>Frigate 特定的配置验证会检查必需字段和值的有效性</li>
          </ul>
        </div>
      )}
    </div>
  )
}

export default ValidatedYamlEditor
