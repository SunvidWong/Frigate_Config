// Logs Page - T137-T141
// Real-time log viewing with filtering and export capabilities

import React, { useState, useEffect } from 'react'
import { useTauriCommand } from '../hooks/useTauriCommand'
import Button from '../components/Button'
import Card from '../components/Card'
import LogViewer from '../components/LogViewer'

interface DeploymentHistoryItem {
  id: string
  container_id: string
  config_path: string
  deployment_time: string
  command: string
  status: string
}

interface LogsResponse {
  container_id: string
  logs: string[]
  lines_returned: number
}

const LogsPage: React.FC = () => {
  const [selectedContainer, setSelectedContainer] = useState<string>('')
  const [logLines, setLogLines] = useState<string[]>([])
  const [maxLines, setMaxLines] = useState<number>(100)
  const [autoRefresh, setAutoRefresh] = useState<boolean>(false)
  const [filterText, setFilterText] = useState<string>('')
  const [deployments, setDeployments] = useState<DeploymentHistoryItem[]>([])

  // Tauri commands
  const {
    data: historyData,
    execute: loadHistory,
  } = useTauriCommand<{ deployments: DeploymentHistoryItem[]; total_count: number }>('list_deployment_history')

  const {
    data: logsData,
    error: logsError,
    loading: loadingLogs,
    execute: fetchLogs,
  } = useTauriCommand<LogsResponse>('get_deployment_logs')

  // Load deployment history on mount
  useEffect(() => {
    loadHistory()
  }, [])

  // Update deployments when history loads
  useEffect(() => {
    if (historyData) {
      setDeployments(historyData.deployments)
      // Auto-select the most recent running deployment
      const runningDeployment = historyData.deployments.find(d => d.status === 'Running')
      if (runningDeployment && !selectedContainer) {
        setSelectedContainer(runningDeployment.container_id)
      }
    }
  }, [historyData])

  // Update logs when data arrives
  useEffect(() => {
    if (logsData) {
      setLogLines(logsData.logs)
    }
  }, [logsData])

  // Auto-refresh logs
  useEffect(() => {
    if (autoRefresh && selectedContainer) {
      const interval = setInterval(() => {
        fetchLogs({ container_id: selectedContainer, lines: maxLines })
      }, 2000) // Refresh every 2 seconds

      return () => clearInterval(interval)
    }
  }, [autoRefresh, selectedContainer, maxLines])

  const handleContainerChange = (containerId: string) => {
    setSelectedContainer(containerId)
    setLogLines([])
    if (containerId) {
      fetchLogs({ container_id: containerId, lines: maxLines })
    }
  }

  const handleRefresh = () => {
    if (selectedContainer) {
      fetchLogs({ container_id: selectedContainer, lines: maxLines })
    }
  }

  const handleExport = () => {
    if (logLines.length === 0) {
      alert('没有可导出的日志')
      return
    }

    const blob = new Blob([logLines.join('\n')], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `frigate-logs-${selectedContainer}-${new Date().toISOString()}.txt`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  const handleClear = () => {
    setLogLines([])
  }

  // Filter logs based on search text
  const filteredLogs = filterText
    ? logLines.filter(line => line.toLowerCase().includes(filterText.toLowerCase()))
    : logLines

  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">容器日志</h1>
        <p className="text-gray-600">
          查看和监控 Frigate 容器的实时日志
        </p>
      </div>

      {/* Controls */}
      <Card className="mb-6">
        <div className="space-y-4">
          {/* Container Selection */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              选择容器
            </label>
            <select
              value={selectedContainer}
              onChange={(e) => handleContainerChange(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              disabled={deployments.length === 0}
            >
              <option value="">-- 选择一个容器 --</option>
              {deployments.map((deployment) => (
                <option key={deployment.id} value={deployment.container_id}>
                  {deployment.container_id} ({deployment.status}) - {new Date(deployment.deployment_time).toLocaleString()}
                </option>
              ))}
            </select>
            {deployments.length === 0 && (
              <p className="text-sm text-gray-500 mt-1">
                暂无部署历史。请先部署 Frigate 容器。
              </p>
            )}
          </div>

          {/* Action Buttons */}
          <div className="flex items-center justify-between flex-wrap gap-2">
            <div className="flex space-x-2">
              <Button
                onClick={handleRefresh}
                loading={loadingLogs}
                disabled={!selectedContainer}
                icon="🔄"
                size="sm"
              >
                刷新
              </Button>
              <Button
                onClick={handleExport}
                disabled={logLines.length === 0}
                icon="💾"
                variant="outline"
                size="sm"
              >
                导出
              </Button>
              <Button
                onClick={handleClear}
                disabled={logLines.length === 0}
                icon="🗑"
                variant="outline"
                size="sm"
              >
                清空
              </Button>
            </div>

            {/* Auto Refresh Toggle */}
            <label className="flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={autoRefresh}
                onChange={(e) => setAutoRefresh(e.target.checked)}
                disabled={!selectedContainer}
                className="mr-2"
              />
              <span className="text-sm text-gray-700">
                自动刷新 (每 2 秒)
              </span>
            </label>

            {/* Max Lines */}
            <div className="flex items-center space-x-2">
              <label className="text-sm text-gray-700">显示行数:</label>
              <select
                value={maxLines}
                onChange={(e) => setMaxLines(Number(e.target.value))}
                className="px-2 py-1 border border-gray-300 rounded text-sm"
              >
                <option value={50}>50</option>
                <option value={100}>100</option>
                <option value={200}>200</option>
                <option value={500}>500</option>
                <option value={1000}>1000</option>
              </select>
            </div>
          </div>

          {/* Search Filter */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              过滤日志
            </label>
            <input
              type="text"
              value={filterText}
              onChange={(e) => setFilterText(e.target.value)}
              placeholder="输入关键词过滤..."
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              disabled={logLines.length === 0}
            />
            {filterText && (
              <p className="text-sm text-gray-600 mt-1">
                显示 {filteredLogs.length} / {logLines.length} 行
              </p>
            )}
          </div>
        </div>
      </Card>

      {/* Error Display */}
      {logsError && (
        <Card className="mb-6 border-red-200 bg-red-50">
          <div className="flex items-start">
            <span className="text-2xl mr-3">⚠️</span>
            <div>
              <h3 className="font-semibold text-red-900 mb-1">获取日志失败</h3>
              <p className="text-red-700">{logsError.message}</p>
            </div>
          </div>
        </Card>
      )}

      {/* Log Viewer */}
      <LogViewer
        logs={filteredLogs}
        loading={loadingLogs}
        autoScroll={autoRefresh}
        containerId={selectedContainer}
      />

      {/* Stats */}
      {selectedContainer && logLines.length > 0 && (
        <Card className="mt-6">
          <div className="flex items-center justify-between text-sm text-gray-600">
            <div>
              <span className="font-medium">容器:</span> {selectedContainer}
            </div>
            <div>
              <span className="font-medium">总行数:</span> {logLines.length}
            </div>
            <div>
              <span className="font-medium">过滤后:</span> {filteredLogs.length}
            </div>
            <div>
              <span className="font-medium">状态:</span>{' '}
              {autoRefresh ? (
                <span className="text-green-600">实时更新</span>
              ) : (
                <span className="text-gray-600">已暂停</span>
              )}
            </div>
          </div>
        </Card>
      )}
    </div>
  )
}

export default LogsPage
