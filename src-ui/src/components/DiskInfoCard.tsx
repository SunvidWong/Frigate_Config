// T184: DiskInfoCard component
// Displays disk information with low space warnings

import React from 'react'
import Card, { CardHeader } from './Card'

export interface DiskInfo {
  path: string
  total_bytes: number
  used_bytes: number
  free_bytes: number
  available_bytes: number
  total_formatted: string
  used_formatted: string
  free_formatted: string
  available_formatted: string
  mount_point: string
  filesystem: string
  usage_percent: number
  is_low_space: boolean
}

interface DiskInfoCardProps {
  diskInfo: DiskInfo | null
  loading?: boolean
  error?: string | null
  onRefresh?: () => void
}

const DiskInfoCard: React.FC<DiskInfoCardProps> = ({
  diskInfo,
  loading = false,
  error = null,
  onRefresh
}) => {
  const getUsageColor = (percent: number, isLowSpace: boolean) => {
    if (isLowSpace) return 'text-red-600'
    if (percent > 80) return 'text-orange-600'
    if (percent > 60) return 'text-yellow-600'
    return 'text-green-600'
  }

  const getProgressBarColor = (percent: number, isLowSpace: boolean) => {
    if (isLowSpace) return 'bg-red-500'
    if (percent > 80) return 'bg-orange-500'
    if (percent > 60) return 'bg-yellow-500'
    return 'bg-green-500'
  }

  if (loading) {
    return (
      <Card>
        <CardHeader title="磁盘信息" />
        <div className="flex items-center justify-center py-8">
          <div className="animate-spin h-8 w-8 border-4 border-blue-500 border-t-transparent rounded-full"></div>
          <span className="ml-3 text-gray-600">加载中...</span>
        </div>
      </Card>
    )
  }

  if (error) {
    return (
      <Card>
        <CardHeader title="磁盘信息" />
        <div className="bg-red-50 border border-red-200 rounded-md p-4">
          <div className="flex items-start space-x-2">
            <span className="text-red-600 text-xl">⚠️</span>
            <div>
              <p className="font-semibold text-red-800">无法获取磁盘信息</p>
              <p className="text-sm text-red-600 mt-1">{error}</p>
            </div>
          </div>
        </div>
      </Card>
    )
  }

  if (!diskInfo) {
    return (
      <Card>
        <CardHeader title="磁盘信息" />
        <div className="text-center py-8 text-gray-500">
          选择路径以查看磁盘信息
        </div>
      </Card>
    )
  }

  return (
    <Card data-testid="disk-card">
      <CardHeader
        title="磁盘信息"
        subtitle={diskInfo.mount_point}
        action={
          onRefresh && (
            <button
              onClick={onRefresh}
              className="text-blue-600 hover:text-blue-700 text-sm font-medium"
            >
              刷新
            </button>
          )
        }
      />

      {/* T188: Low disk space warning */}
      {diskInfo.is_low_space && (
        <div className="mb-4 bg-red-50 border border-red-200 rounded-md p-4" data-testid="low-space-warning">
          <div className="flex items-start space-x-2">
            <span className="text-red-600 text-xl flex-shrink-0">⚠️</span>
            <div className="flex-1">
              <p className="font-semibold text-red-800">磁盘空间不足</p>
              <p className="text-sm text-red-600 mt-1">
                剩余空间少于 10GB。录像文件需要大量存储空间，建议选择空间更大的磁盘或清理当前磁盘。
              </p>
              <div className="mt-2 text-xs text-red-600 space-y-1">
                <div>• 建议录像盘至少 100GB 可用空间</div>
                <div>• 建议剪辑盘至少 20GB 可用空间</div>
                <div>• 建议缓存盘至少 10GB 可用空间</div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Disk usage visualization */}
      <div className="space-y-4">
        <div>
          <div className="flex justify-between items-baseline mb-2">
            <span className="text-sm font-medium text-gray-700">磁盘使用率</span>
            <span className={`text-2xl font-bold ${getUsageColor(diskInfo.usage_percent, diskInfo.is_low_space)}`} data-testid="disk-usage-percent">
              {diskInfo.usage_percent.toFixed(1)}%
            </span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-3 overflow-hidden" data-testid="disk-usage-bar">
            <div
              className={`h-full rounded-full transition-all duration-500 ${getProgressBarColor(diskInfo.usage_percent, diskInfo.is_low_space)}`}
              style={{ width: `${Math.min(diskInfo.usage_percent, 100)}%` }}
            />
          </div>
        </div>

        {/* Detailed information */}
        <div className="grid grid-cols-2 gap-4 pt-4 border-t border-gray-200">
          <div>
            <div className="text-xs text-gray-500 mb-1">总容量</div>
            <div className="text-lg font-semibold text-gray-900" data-testid="disk-total-space">{diskInfo.total_formatted}</div>
          </div>
          <div>
            <div className="text-xs text-gray-500 mb-1">已使用</div>
            <div className="text-lg font-semibold text-gray-900">{diskInfo.used_formatted}</div>
          </div>
          <div>
            <div className="text-xs text-gray-500 mb-1">剩余空间</div>
            <div className={`text-lg font-semibold ${diskInfo.is_low_space ? 'text-red-600' : 'text-green-600'}`} data-testid="disk-free-space">
              {diskInfo.free_formatted}
            </div>
          </div>
          <div>
            <div className="text-xs text-gray-500 mb-1">可用空间</div>
            <div className={`text-lg font-semibold ${diskInfo.is_low_space ? 'text-red-600' : 'text-green-600'}`}>
              {diskInfo.available_formatted}
            </div>
          </div>
        </div>

        {/* Filesystem info */}
        <div className="pt-4 border-t border-gray-200">
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-gray-500">文件系统:</span>
              <span className="ml-2 font-medium text-gray-900">{diskInfo.filesystem}</span>
            </div>
            <div>
              <span className="text-gray-500">挂载点:</span>
              <span className="ml-2 font-medium text-gray-900" data-testid="disk-mount-point">{diskInfo.mount_point}</span>
            </div>
          </div>
        </div>

        {/* Path info */}
        <div className="pt-2 text-xs text-gray-500 break-all">
          路径: {diskInfo.path}
        </div>
      </div>
    </Card>
  )
}

export default DiskInfoCard
