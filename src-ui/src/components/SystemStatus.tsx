// 系统状态指示器组件

import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

interface SystemStatusData {
  hardware_devices: number
  docker_compose_exists: boolean
  config_valid: boolean
  docker_running: boolean
}

export const SystemStatus: React.FC = () => {
  const [status, setStatus] = useState<SystemStatusData | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const checkStatus = async () => {
      try {
        const result = await invoke<SystemStatusData>('check_system_status')
        setStatus(result)
      } catch (error) {
        console.error('获取系统状态失败:', error)
      } finally {
        setLoading(false)
      }
    }

    // 初次加载
    checkStatus()

    // 每30秒检查一次
    const interval = setInterval(checkStatus, 30000)

    return () => clearInterval(interval)
  }, [])

  if (loading) {
    return (
      <div className="system-status flex items-center space-x-4 text-sm text-gray-500">
        <span>正在检查系统状态...</span>
      </div>
    )
  }

  if (!status) {
    return null
  }

  return (
    <div className="system-status flex items-center space-x-4 text-sm">
      <StatusIndicator
        label="硬件设备"
        value={status.hardware_devices}
        ok={status.hardware_devices > 0}
      />
      <StatusIndicator
        label="Docker Compose"
        ok={status.docker_compose_exists}
      />
      <StatusIndicator
        label="配置文件"
        ok={status.config_valid}
      />
      <StatusIndicator
        label="Docker"
        ok={status.docker_running}
      />
    </div>
  )
}

interface StatusIndicatorProps {
  label: string
  value?: number
  ok: boolean
}

const StatusIndicator: React.FC<StatusIndicatorProps> = ({ label, value, ok }) => {
  return (
    <div className="flex items-center space-x-1">
      <span
        className={`w-2 h-2 rounded-full ${
          ok ? 'bg-green-500' : 'bg-red-500'
        }`}
      />
      <span className={ok ? 'text-gray-700' : 'text-gray-500'}>
        {label}
        {value !== undefined && ` (${value})`}
      </span>
    </div>
  )
}

export default SystemStatus