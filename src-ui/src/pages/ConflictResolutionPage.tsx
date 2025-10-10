// Conflict Resolution Page
// Interface for resolving configuration merge conflicts

import React, { useState, useEffect } from 'react'
import { AlertTriangle, CheckCircle, ArrowLeft, ArrowRight, Save, GitCompare } from 'lucide-react'
import { configService, MergeResultResponse, ConflictResolutionRequest } from '../services/configService'

const ConflictResolutionPage: React.FC = () => {
  const [mergeResult, setMergeResult] = useState<MergeResultResponse | null>(null)
  const [isLoading, setIsLoading] = useState<boolean>(false)
  const [error, setError] = useState<string | null>(null)
  const [resolutions, setResolutions] = useState<Map<string, any>>(new Map())
  const [currentConflictIndex, setCurrentConflictIndex] = useState<number>(0)
  const [showPreview, setShowPreview] = useState<boolean>(false)
  const [previewContent, setPreviewContent] = useState<string>('')

  useEffect(() => {
    // Load sample data for demonstration
    loadSampleConflict()
  }, [])

  const loadSampleConflict = async () => {
    setIsLoading(true)
    setError(null)

    try {
      // Simulate merge result with sample conflicts
      const mockMergeResult: MergeResultResponse = {
        merged_config: '',
        conflicts: [
          {
            path: 'cameras.front_door.detect.max_frames',
            base_value: 50,
            incoming_value: 100,
            conflict_type: 'ValueMismatch',
            severity: 'low',
            suggestion: 'Choose based on your performance requirements',
            description: 'Maximum frames to analyze for detection'
          },
          {
            path: 'cameras.back_yard.detect.enabled',
            base_value: false,
            incoming_value: true,
            conflict_type: 'ValueMismatch',
            severity: 'medium',
            suggestion: 'Enable detection if you want motion alerts for this camera',
            description: 'Whether to enable motion detection for this camera'
          },
          {
            path: 'cameras.back_yard.ffmpeg.inputs[0].roles',
            base_value: ['detect'],
            incoming_value: ['detect', 'record'],
            conflict_type: 'ArrayDifference',
            severity: 'medium',
            suggestion: 'Add record role if you want to save footage',
            description: 'Roles assigned to the video input stream'
          }
        ],
        warnings: [
          'Detected multiple cameras, ensure your system can handle the processing load',
          'Review detector configurations for optimal performance'
        ],
        auto_resolved: 0,
        manual_resolution_required: 3
      }

      setMergeResult(mockMergeResult)
    } catch (err) {
      console.error('Failed to load conflict data:', err)
      setError('加载冲突数据失败')
    } finally {
      setIsLoading(false)
    }
  }

  const handleResolveConflict = (path: string, resolution: 'use_base' | 'use_incoming' | 'use_custom', customValue?: any) => {
    const newResolutions = new Map(resolutions)
    newResolutions.set(path, { resolution, custom_value: customValue })
    setResolutions(newResolutions)
  }

  const handleNextConflict = () => {
    if (currentConflictIndex < (mergeResult?.conflicts.length || 0) - 1) {
      setCurrentConflictIndex(currentConflictIndex + 1)
    }
  }

  const handlePreviousConflict = () => {
    if (currentConflictIndex > 0) {
      setCurrentConflictIndex(currentConflictIndex - 1)
    }
  }

  const handleResolveAll = async () => {
    if (!mergeResult) return

    const unresolvedConflicts = mergeResult.conflicts.filter(conflict => !resolutions.has(conflict.path))
    if (unresolvedConflicts.length > 0) {
      setError(`还有 ${unresolvedConflicts.length} 个冲突未解决`)
      return
    }

    setIsLoading(true)
    try {
      const resolutionRequest: ConflictResolutionRequest = {
        merge_result_id: 'sample-merge-id',
        conflicts: Array.from(resolutions.entries()).map(([path, resolution]) => ({
          path,
          resolution: resolution.resolution,
          custom_value: resolution.custom_value
        }))
      }

      const response = await configService.resolveConflicts(resolutionRequest)
      if (response.success) {
        setPreviewContent(response.resolved_config)
        setShowPreview(true)
      }
    } catch (err) {
      console.error('Failed to resolve conflicts:', err)
      setError('解决冲突失败')
    } finally {
      setIsLoading(false)
    }
  }

  const formatValue = (value: any): string => {
    if (value === null || value === undefined) {
      return 'null'
    }
    if (typeof value === 'object') {
      return JSON.stringify(value, null, 2)
    }
    return String(value)
  }

  const getConflictIcon = (severity: string) => {
    switch (severity) {
      case 'critical':
        return '🔴'
      case 'high':
        return '🟠'
      case 'medium':
        return '🟡'
      case 'low':
        return '🟢'
      default:
        return '⚪'
    }
  }

  const getConflictColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'text-red-600 bg-red-50 border-red-200'
      case 'high':
        return 'text-orange-600 bg-orange-50 border-orange-200'
      case 'medium':
        return 'text-yellow-600 bg-yellow-50 border-yellow-200'
      case 'low':
        return 'text-green-600 bg-green-50 border-green-200'
      default:
        return 'text-gray-600 bg-gray-50 border-gray-200'
    }
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">处理中...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-center text-red-600">
          <AlertTriangle className="h-12 w-12 mx-auto mb-4" />
          <p className="text-lg font-medium">发生错误</p>
          <p className="text-sm mt-2">{error}</p>
          <button
            onClick={loadSampleConflict}
            className="mt-4 px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
          >
            重试
          </button>
        </div>
      </div>
    )
  }

  if (!mergeResult) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-center">
          <GitCompare className="h-12 w-12 text-gray-400 mx-auto mb-4" />
          <p className="text-gray-600">加载冲突数据...</p>
        </div>
      </div>
    )
  }

  if (showPreview) {
    return (
      <div className="p-6 max-w-7xl mx-auto">
        <div className="bg-white shadow rounded-lg">
          <div className="border-b border-gray-200 px-6 py-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <CheckCircle className="h-6 w-6 text-green-600" />
                <h1 className="text-2xl font-bold text-gray-900">冲突已解决</h1>
              </div>
              <button
                onClick={() => setShowPreview(false)}
                className="px-4 py-2 border border-gray-300 text-gray-700 rounded hover:bg-gray-50"
              >
                返回编辑
              </button>
            </div>
          </div>

          <div className="p-6">
            <div className="bg-green-50 border border-green-200 rounded-lg p-4 mb-6">
              <h3 className="text-green-900 font-medium mb-2">合并摘要</h3>
              <div className="text-sm text-green-800 space-y-1">
                <p>✓ 所有冲突已解决</p>
                <p>✓ 配置已准备就绪</p>
                <p>✓ 可以保存配置文件</p>
              </div>
            </div>

            <div className="mb-4">
              <h3 className="text-lg font-medium text-gray-900 mb-3">合并后的配置</h3>
              <div className="bg-gray-900 rounded-lg p-4 overflow-x-auto">
                <pre className="text-sm text-gray-100">
                  <code>{previewContent || '合并后的配置将显示在这里...'}</code>
                </pre>
              </div>
            </div>

            <div className="flex space-x-3">
              <button
                onClick={() => {
                  // Save the resolved configuration
                  console.log('Saving resolved configuration...')
                }}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
              >
                保存配置
              </button>
              <button
                onClick={() => setShowPreview(false)}
                className="px-4 py-2 border border-gray-300 text-gray-700 rounded hover:bg-gray-50"
              >
                继续编辑
              </button>
            </div>
          </div>
        </div>
      </div>
    )
  }

  const currentConflict = mergeResult.conflicts[currentConflictIndex]
  const totalConflicts = mergeResult.conflicts.length
  const resolvedCount = resolutions.size
  const progressPercentage = (resolvedCount / totalConflicts) * 100

  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* Header */}
      <div className="bg-white shadow rounded-lg mb-6">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <GitCompare className="h-6 w-6 text-blue-600" />
              <div>
                <h1 className="text-2xl font-bold text-gray-900">配置冲突解决</h1>
                <p className="text-sm text-gray-500">
                  解决 {totalConflicts} 个配置合并冲突
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Progress Bar */}
        <div className="px-6 py-4">
          <div className="mb-2 flex items-center justify-between text-sm">
            <span className="text-gray-600">进度</span>
            <span className="text-gray-900 font-medium">
              {resolvedCount} / {totalConflicts} 已解决
            </span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-blue-600 h-2 rounded-full transition-all duration-300"
              style={{ width: `${progressPercentage}%` }}
            ></div>
          </div>
        </div>
      </div>

      {/* Warnings */}
      {mergeResult.warnings.length > 0 && (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
          <h3 className="text-yellow-900 font-medium mb-2 flex items-center space-x-2">
            <AlertTriangle className="h-4 w-4" />
            <span>警告</span>
          </h3>
          <ul className="text-sm text-yellow-800 space-y-1">
            {mergeResult.warnings.map((warning, index) => (
              <li key={index} className="flex items-start space-x-2">
                <span>•</span>
                <span>{warning}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Conflict Navigation */}
      <div className="bg-white shadow rounded-lg mb-6">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <button
                onClick={handlePreviousConflict}
                disabled={currentConflictIndex === 0}
                className="p-2 border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ArrowLeft className="h-4 w-4" />
              </button>
              <span className="text-sm text-gray-600">
                冲突 {currentConflictIndex + 1} / {totalConflicts}
              </span>
              <button
                onClick={handleNextConflict}
                disabled={currentConflictIndex === totalConflicts - 1}
                className="p-2 border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ArrowRight className="h-4 w-4" />
              </button>
            </div>

            <div className="flex items-center space-x-2">
              <span className={`px-3 py-1 rounded-full text-sm font-medium ${getConflictColor(currentConflict.severity)}`}>
                {getConflictIcon(currentConflict.severity)} {currentConflict.severity.toUpperCase()}
              </span>
              {resolutions.has(currentConflict.path) && (
                <span className="px-3 py-1 bg-green-100 text-green-800 rounded-full text-sm font-medium">
                  ✓ 已解决
                </span>
              )}
            </div>
          </div>
        </div>

        {/* Conflict Details */}
        <div className="p-6">
          <div className="mb-6">
            <h3 className="text-lg font-medium text-gray-900 mb-2">{currentConflict.path}</h3>
            <p className="text-gray-600 mb-4">{currentConflict.description}</p>
            {currentConflict.suggestion && (
              <div className="bg-blue-50 border border-blue-200 rounded p-3">
                <p className="text-sm text-blue-800">
                  <strong>建议:</strong> {currentConflict.suggestion}
                </p>
              </div>
            )}
          </div>

          {/* Value Comparison */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
            {/* Base Value */}
            <div className="border border-gray-200 rounded-lg p-4">
              <h4 className="font-medium text-gray-900 mb-3">基础配置值</h4>
              <div className="bg-gray-100 rounded p-3">
                <pre className="text-sm text-gray-800 whitespace-pre-wrap">
                  {formatValue(currentConflict.base_value)}
                </pre>
              </div>
              <button
                onClick={() => handleResolveConflict(currentConflict.path, 'use_base')}
                className={`mt-3 w-full px-3 py-2 rounded text-sm font-medium ${
                  resolutions.get(currentConflict.path)?.resolution === 'use_base'
                    ? 'bg-blue-600 text-white'
                    : 'border border-gray-300 text-gray-700 hover:bg-gray-50'
                }`}
              >
                使用此值
              </button>
            </div>

            {/* Incoming Value */}
            <div className="border border-gray-200 rounded-lg p-4">
              <h4 className="font-medium text-gray-900 mb-3">传入配置值</h4>
              <div className="bg-gray-100 rounded p-3">
                <pre className="text-sm text-gray-800 whitespace-pre-wrap">
                  {formatValue(currentConflict.incoming_value)}
                </pre>
              </div>
              <button
                onClick={() => handleResolveConflict(currentConflict.path, 'use_incoming')}
                className={`mt-3 w-full px-3 py-2 rounded text-sm font-medium ${
                  resolutions.get(currentConflict.path)?.resolution === 'use_incoming'
                    ? 'bg-blue-600 text-white'
                    : 'border border-gray-300 text-gray-700 hover:bg-gray-50'
                }`}
              >
                使用此值
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex justify-between">
        <button
          onClick={() => {
            // Cancel resolution
            console.log('Canceling conflict resolution...')
          }}
          className="px-4 py-2 border border-gray-300 text-gray-700 rounded hover:bg-gray-50"
        >
          取消
        </button>

        <div className="flex space-x-3">
          {resolvedCount < totalConflicts && (
            <button
              onClick={() => {
                // Auto-resolve remaining conflicts
                mergeResult.conflicts.forEach(conflict => {
                  if (!resolutions.has(conflict.path)) {
                    // Use incoming value as default
                    handleResolveConflict(conflict.path, 'use_incoming')
                  }
                })
              }}
              className="px-4 py-2 border border-gray-300 text-gray-700 rounded hover:bg-gray-50"
            >
              自动解决剩余
            </button>
          )}

          <button
            onClick={handleResolveAll}
            disabled={resolvedCount === 0}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center space-x-2"
          >
            <Save className="h-4 w-4" />
            <span>解决所有冲突</span>
          </button>
        </div>
      </div>
    </div>
  )
}

export default ConflictResolutionPage