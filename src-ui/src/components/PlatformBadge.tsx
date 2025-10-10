// Platform Badge Component
// Displays platform and architecture information with icons

import React from 'react'
import { getPlatformInfo, getPlatformIcon, getArchitectureIcon } from '../utils/platform'

interface PlatformBadgeProps {
  className?: string
  showArchitecture?: boolean
  inline?: boolean
}

const PlatformBadge: React.FC<PlatformBadgeProps> = ({
  className = '',
  showArchitecture = true,
  inline = false,
}) => {
  const platformInfo = getPlatformInfo()

  if (inline) {
    return (
      <span className={`inline-flex items-center space-x-2 ${className}`}>
        <span className="text-lg">{getPlatformIcon(platformInfo.platform)}</span>
        <span className="text-sm font-medium">{platformInfo.displayName}</span>
        {showArchitecture && (
          <>
            <span className="text-gray-400">•</span>
            <span className="text-lg">{getArchitectureIcon(platformInfo.architecture)}</span>
            <span className="text-sm text-gray-600">{platformInfo.archDisplayName}</span>
          </>
        )}
      </span>
    )
  }

  return (
    <div className={`flex flex-col space-y-1 ${className}`} data-testid="platform-info">
      <div className="flex items-center space-x-2">
        <span className="text-2xl">{getPlatformIcon(platformInfo.platform)}</span>
        <div>
          <div className="text-sm font-medium text-gray-700">平台</div>
          <div className="text-lg font-semibold">{platformInfo.displayName}</div>
        </div>
      </div>
      {showArchitecture && (
        <div className="flex items-center space-x-2" data-testid="architecture-info">
          <span className="text-2xl">{getArchitectureIcon(platformInfo.architecture)}</span>
          <div>
            <div className="text-sm font-medium text-gray-700">架构</div>
            <div className="text-lg font-semibold">{platformInfo.archDisplayName}</div>
          </div>
        </div>
      )}
      {platformInfo.isAppleSilicon && (
        <div className="mt-2 text-xs text-blue-600 bg-blue-50 px-2 py-1 rounded">
          ✨ Apple Silicon 检测到
        </div>
      )}
      {platformInfo.isARM && !platformInfo.isAppleSilicon && (
        <div className="mt-2 text-xs text-purple-600 bg-purple-50 px-2 py-1 rounded">
          🦾 ARM 架构检测到
        </div>
      )}
    </div>
  )
}

export default PlatformBadge
