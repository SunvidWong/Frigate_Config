// CPU-Only Warning Component
// Displays warning when no hardware acceleration is detected

import React from 'react'
import Card from './Card'
import Button from './Button'
import { getPlatformInfo, getPerformanceHints } from '../utils/platform'

interface CPUOnlyWarningProps {
  onDismiss?: () => void
  showRecommendations?: boolean
  className?: string
}

const CPUOnlyWarning: React.FC<CPUOnlyWarningProps> = ({
  onDismiss,
  showRecommendations = true,
  className = '',
}) => {
  const platformInfo = getPlatformInfo()
  const performanceHints = getPerformanceHints(platformInfo.platform, platformInfo.architecture)

  return (
    <Card className={`border-l-4 border-yellow-500 bg-yellow-50 ${className}`}>
      <div className="flex items-start space-x-4">
        {/* Warning Icon */}
        <div className="flex-shrink-0">
          <div className="w-12 h-12 rounded-full bg-yellow-100 flex items-center justify-center">
            <span className="text-3xl">⚠️</span>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-yellow-900 mb-2">
            仅 CPU 模式检测
          </h3>

          <p className="text-yellow-800 mb-3">
            未检测到 GPU 或 TPU 硬件加速器。Frigate 将以 CPU-only 模式运行，这可能会导致性能受限。
          </p>

          {platformInfo.isARM && (
            <div className="mb-3 p-3 bg-yellow-100 rounded-md">
              <div className="flex items-center space-x-2 mb-1">
                <span className="text-lg">🦾</span>
                <span className="font-medium text-yellow-900">ARM 架构注意</span>
              </div>
              <p className="text-sm text-yellow-800">
                在 ARM 平台上，视频处理性能可能特别有限。强烈建议添加硬件加速。
              </p>
            </div>
          )}

          {showRecommendations && (
            <div className="space-y-2">
              <div className="font-medium text-yellow-900">推荐的改进措施：</div>
              <ul className="list-disc list-inside space-y-1 text-sm text-yellow-800">
                <li>添加支持的 GPU（NVIDIA、AMD 或 Intel）以实现硬件加速</li>
                <li>考虑使用 Google Coral TPU 进行物体检测</li>
                {platformInfo.platform === 'linux' && platformInfo.isARM && (
                  <li>在 ARM Linux 上，可以考虑 Hailo NPU 或 Mali GPU</li>
                )}
                {platformInfo.isAppleSilicon && (
                  <li>Apple Silicon 应自动支持 Metal 加速 - 请检查驱动程序</li>
                )}
                <li>降低摄像头分辨率或帧率以减少 CPU 负载</li>
                <li>限制检测区域以提高性能</li>
              </ul>
            </div>
          )}

          {performanceHints.length > 0 && (
            <div className="mt-3 p-3 bg-blue-50 border border-blue-200 rounded-md">
              <div className="font-medium text-blue-900 mb-2">💡 平台特定提示：</div>
              <ul className="list-disc list-inside space-y-1 text-sm text-blue-800">
                {performanceHints.map((hint, index) => (
                  <li key={index}>{hint}</li>
                ))}
              </ul>
            </div>
          )}

          <div className="mt-4 p-3 bg-white border border-yellow-200 rounded-md">
            <div className="flex items-center space-x-2 mb-2">
              <span className="text-lg">📊</span>
              <span className="font-medium text-gray-900">性能影响</span>
            </div>
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div>
                <div className="text-gray-600">检测性能</div>
                <div className="font-medium text-red-600">低 (~1-3 FPS)</div>
              </div>
              <div>
                <div className="text-gray-600">视频处理</div>
                <div className="font-medium text-red-600">受限</div>
              </div>
              <div>
                <div className="text-gray-600">推荐摄像头数</div>
                <div className="font-medium text-yellow-600">1-2 台</div>
              </div>
              <div>
                <div className="text-gray-600">最大分辨率</div>
                <div className="font-medium text-yellow-600">720p</div>
              </div>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="mt-4 flex items-center space-x-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => window.open('https://docs.frigate.video/configuration/hardware_acceleration', '_blank')}
              icon="📚"
            >
              查看硬件加速文档
            </Button>
            {onDismiss && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onDismiss}
              >
                我知道了
              </Button>
            )}
          </div>
        </div>
      </div>
    </Card>
  )
}

export default CPUOnlyWarning
