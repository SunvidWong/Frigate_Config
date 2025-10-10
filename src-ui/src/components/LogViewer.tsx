// LogViewer Component - T138
// Displays container logs with auto-scroll and syntax highlighting

import React, { useEffect, useRef } from 'react'
import Card from './Card'

interface LogViewerProps {
  logs: string[]
  loading: boolean
  autoScroll: boolean
  containerId: string
}

const LogViewer: React.FC<LogViewerProps> = ({
  logs,
  loading,
  autoScroll,
  containerId,
}) => {
  const logContainerRef = useRef<HTMLDivElement>(null)
  const shouldAutoScroll = useRef(true)

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (autoScroll && shouldAutoScroll.current && logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight
    }
  }, [logs, autoScroll])

  // Detect manual scrolling
  const handleScroll = () => {
    if (logContainerRef.current) {
      const { scrollTop, scrollHeight, clientHeight } = logContainerRef.current
      const isAtBottom = scrollHeight - scrollTop - clientHeight < 50
      shouldAutoScroll.current = isAtBottom
    }
  }

  // Parse log line to detect level and add styling
  const parseLogLine = (line: string, index: number) => {
    const timestamp = line.match(/^\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}/)

    let level: 'error' | 'warn' | 'info' | 'debug' | 'default' = 'default'
    if (line.match(/error|fail|exception|fatal/i)) {
      level = 'error'
    } else if (line.match(/warn|warning/i)) {
      level = 'warn'
    } else if (line.match(/info/i)) {
      level = 'info'
    } else if (line.match(/debug|trace/i)) {
      level = 'debug'
    }

    const levelColors = {
      error: 'text-red-400',
      warn: 'text-yellow-400',
      info: 'text-blue-400',
      debug: 'text-gray-500',
      default: 'text-green-400',
    }

    return (
      <div
        key={index}
        className={`font-mono text-xs leading-relaxed hover:bg-gray-800 ${levelColors[level]}`}
      >
        <span className="text-gray-600 select-none mr-2">{index + 1}</span>
        {timestamp && (
          <span className="text-gray-500 mr-2">{timestamp[0]}</span>
        )}
        <span>{line.replace(timestamp?.[0] || '', '').trim()}</span>
      </div>
    )
  }

  if (!containerId) {
    return (
      <Card className="bg-gray-50">
        <div className="text-center py-12">
          <div className="text-6xl mb-4">📋</div>
          <h3 className="text-xl font-semibold text-gray-900 mb-2">
            选择容器查看日志
          </h3>
          <p className="text-gray-600">
            从上方的下拉菜单中选择一个容器
          </p>
        </div>
      </Card>
    )
  }

  if (loading && logs.length === 0) {
    return (
      <Card className="bg-gray-50">
        <div className="text-center py-12">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-600">正在加载日志...</p>
        </div>
      </Card>
    )
  }

  if (logs.length === 0 && !loading) {
    return (
      <Card className="bg-gray-50">
        <div className="text-center py-12">
          <div className="text-6xl mb-4">📝</div>
          <h3 className="text-xl font-semibold text-gray-900 mb-2">
            暂无日志
          </h3>
          <p className="text-gray-600">
            该容器还没有生成任何日志，或者日志已被清空
          </p>
        </div>
      </Card>
    )
  }

  return (
    <Card className="bg-gray-900 p-0 overflow-hidden">
      {/* Header */}
      <div className="bg-gray-800 px-4 py-2 border-b border-gray-700 flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <span className="text-green-400 font-mono text-sm">●</span>
          <span className="text-gray-400 text-sm font-mono">
            {containerId}
          </span>
        </div>
        <div className="flex items-center space-x-2">
          {loading && (
            <div className="flex items-center text-xs text-gray-400">
              <div className="animate-spin rounded-full h-3 w-3 border-b border-blue-400 mr-2"></div>
              更新中...
            </div>
          )}
          <span className="text-xs text-gray-500">
            {logs.length} 行
          </span>
        </div>
      </div>

      {/* Log Content */}
      <div
        ref={logContainerRef}
        onScroll={handleScroll}
        className="p-4 overflow-y-auto max-h-[600px] scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800"
        style={{
          scrollBehavior: autoScroll ? 'smooth' : 'auto',
        }}
      >
        {logs.map((line, index) => parseLogLine(line, index))}
      </div>

      {/* Auto-scroll indicator */}
      {!shouldAutoScroll.current && autoScroll && (
        <div className="absolute bottom-4 right-4">
          <button
            onClick={() => {
              if (logContainerRef.current) {
                logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight
                shouldAutoScroll.current = true
              }
            }}
            className="bg-blue-600 hover:bg-blue-700 text-white px-3 py-1 rounded-full text-xs font-medium shadow-lg flex items-center space-x-1"
          >
            <span>↓</span>
            <span>滚动到底部</span>
          </button>
        </div>
      )}

      {/* Footer */}
      <div className="bg-gray-800 px-4 py-2 border-t border-gray-700">
        <div className="flex items-center justify-between text-xs text-gray-500">
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-1">
              <span className="text-red-400">●</span>
              <span>错误</span>
            </div>
            <div className="flex items-center space-x-1">
              <span className="text-yellow-400">●</span>
              <span>警告</span>
            </div>
            <div className="flex items-center space-x-1">
              <span className="text-blue-400">●</span>
              <span>信息</span>
            </div>
            <div className="flex items-center space-x-1">
              <span className="text-gray-500">●</span>
              <span>调试</span>
            </div>
          </div>
          <div>
            {autoScroll ? (
              <span className="text-green-400">● 自动滚动已启用</span>
            ) : (
              <span>自动滚动已禁用</span>
            )}
          </div>
        </div>
      </div>
    </Card>
  )
}

export default LogViewer
