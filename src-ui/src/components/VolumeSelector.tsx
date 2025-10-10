// T185: VolumeSelector component
// Allows users to select and configure volume mappings

import React, { useState } from 'react'
import Button from './Button'
import Card, { CardHeader } from './Card'

export interface VolumeMapping {
  host_path: string
  container_path: string
  mapping_type: 'recordings' | 'clips' | 'cache' | 'config' | 'custom'
  read_only: boolean
  description?: string | null
}

interface VolumeSelectorProps {
  mappings: VolumeMapping[]
  onAddMapping: (mapping: VolumeMapping) => void
  onRemoveMapping: (index: number) => void
  onUpdateMapping?: (index: number, mapping: VolumeMapping) => void  // Optional for future use
  defaultPaths?: {
    config: string
    recordings: string
    clips: string
    cache: string
  }
  onBrowse?: (currentPath: string) => Promise<string | null>
}

const MAPPING_TYPES = [
  { value: 'recordings', label: '录像 (Recordings)', containerPath: '/media/frigate/recordings' },
  { value: 'clips', label: '剪辑 (Clips)', containerPath: '/media/frigate/clips' },
  { value: 'cache', label: '缓存 (Cache)', containerPath: '/tmp/cache' },
  { value: 'config', label: '配置 (Config)', containerPath: '/config' },
  { value: 'custom', label: '自定义 (Custom)', containerPath: '' },
] as const

const VolumeSelector: React.FC<VolumeSelectorProps> = ({
  mappings,
  onAddMapping,
  onRemoveMapping,
  onUpdateMapping: _onUpdateMapping,  // Reserved for future use
  defaultPaths,
  onBrowse,
}) => {
  const [newMapping, setNewMapping] = useState<Partial<VolumeMapping>>({
    mapping_type: 'recordings',
    host_path: '',
    container_path: '/media/frigate/recordings',
    read_only: false,
  })
  const [showAddForm, setShowAddForm] = useState(false)

  const handleTypeChange = (type: VolumeMapping['mapping_type']) => {
    const typeConfig = MAPPING_TYPES.find(t => t.value === type)
    const defaultPath = defaultPaths?.[type as keyof typeof defaultPaths] || ''

    setNewMapping({
      ...newMapping,
      mapping_type: type,
      container_path: typeConfig?.containerPath || '',
      host_path: defaultPath,
    })
  }

  const handleAddMapping = () => {
    if (!newMapping.host_path || !newMapping.container_path || !newMapping.mapping_type) {
      alert('请填写所有必需字段')
      return
    }

    onAddMapping(newMapping as VolumeMapping)
    setNewMapping({
      mapping_type: 'recordings',
      host_path: '',
      container_path: '/media/frigate/recordings',
      read_only: false,
    })
    setShowAddForm(false)
  }

  const handleBrowse = async (currentPath: string, callback: (path: string) => void) => {
    if (onBrowse) {
      const selected = await onBrowse(currentPath)
      if (selected) {
        callback(selected)
      }
    }
  }

  const getMappingTypeLabel = (type: string) => {
    return MAPPING_TYPES.find(t => t.value === type)?.label || type
  }

  const getMappingIcon = (type: string) => {
    switch (type) {
      case 'recordings': return '🎥'
      case 'clips': return '✂️'
      case 'cache': return '💾'
      case 'config': return '⚙️'
      case 'custom': return '📁'
      default: return '📦'
    }
  }

  return (
    <Card>
      <CardHeader
        title="卷映射配置"
        subtitle={`已配置 ${mappings.length} 个卷映射`}
        action={
          !showAddForm && (
            <Button onClick={() => setShowAddForm(true)} variant="outline" size="sm">
              + 添加映射
            </Button>
          )
        }
      />

      {/* Existing mappings */}
      {mappings.length === 0 && !showAddForm && (
        <div className="text-center py-8 text-gray-500">
          <p className="mb-2">暂无卷映射</p>
          <p className="text-sm">点击"添加映射"开始配置存储路径</p>
        </div>
      )}

      <div className="space-y-3" data-testid="configured-volumes-list">
        {mappings.map((mapping, index) => (
          <div
            key={index}
            className="border border-gray-200 rounded-md p-4 hover:border-gray-300 transition-colors"
            data-testid="volume-item"
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center space-x-2 mb-2">
                  <span className="text-2xl">{getMappingIcon(mapping.mapping_type)}</span>
                  <div>
                    <div className="font-semibold text-gray-900">
                      {getMappingTypeLabel(mapping.mapping_type)}
                    </div>
                    {mapping.description && (
                      <div className="text-xs text-gray-500">{mapping.description}</div>
                    )}
                  </div>
                  {mapping.read_only && (
                    <span className="px-2 py-0.5 bg-blue-100 text-blue-700 text-xs rounded-full">
                      只读
                    </span>
                  )}
                </div>
                <div className="space-y-1 text-sm">
                  <div className="flex items-center space-x-2">
                    <span className="text-gray-500 w-16">主机:</span>
                    <code className="flex-1 px-2 py-1 bg-gray-50 rounded text-xs font-mono">
                      {mapping.host_path}
                    </code>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className="text-gray-500 w-16">容器:</span>
                    <code className="flex-1 px-2 py-1 bg-gray-50 rounded text-xs font-mono">
                      {mapping.container_path}
                    </code>
                  </div>
                </div>
              </div>
              <button
                onClick={() => onRemoveMapping(index)}
                className="ml-4 text-red-600 hover:text-red-700 text-sm"
                data-testid="remove-volume-button"
              >
                删除
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Add mapping form */}
      {showAddForm && (
        <div className="mt-4 border-t border-gray-200 pt-4" data-testid="volume-config-dialog">
          <h3 className="font-semibold text-gray-900 mb-4">添加新的卷映射</h3>
          <div className="space-y-4">
            {/* Mapping type */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                映射类型 *
              </label>
              <select
                value={newMapping.mapping_type}
                onChange={(e) => handleTypeChange(e.target.value as VolumeMapping['mapping_type'])}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                {MAPPING_TYPES.map((type) => (
                  <option key={type.value} value={type.value}>
                    {type.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Host path */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                主机路径 *
              </label>
              <div className="flex space-x-2">
                <input
                  type="text"
                  value={newMapping.host_path || ''}
                  onChange={(e) => setNewMapping({ ...newMapping, host_path: e.target.value })}
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="/path/on/host"
                  data-testid="host-path-input"
                />
                {onBrowse && (
                  <Button
                    variant="outline"
                    onClick={() => handleBrowse(newMapping.host_path || '', (path) =>
                      setNewMapping({ ...newMapping, host_path: path })
                    )}
                  >
                    浏览
                  </Button>
                )}
              </div>
              <p className="mt-1 text-xs text-gray-500">
                存储文件的主机目录路径
              </p>
            </div>

            {/* Container path */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                容器路径 *
              </label>
              <input
                type="text"
                value={newMapping.container_path || ''}
                onChange={(e) => setNewMapping({ ...newMapping, container_path: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="/path/in/container"
                data-testid="container-path-input"
              />
              <p className="mt-1 text-xs text-gray-500">
                容器内的挂载路径
              </p>
            </div>

            {/* Description */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                描述 (可选)
              </label>
              <input
                type="text"
                value={newMapping.description || ''}
                onChange={(e) => setNewMapping({ ...newMapping, description: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="此映射的说明"
              />
            </div>

            {/* Read-only toggle */}
            <div className="flex items-center">
              <input
                type="checkbox"
                id="read-only"
                checked={newMapping.read_only || false}
                onChange={(e) => setNewMapping({ ...newMapping, read_only: e.target.checked })}
                className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
              <label htmlFor="read-only" className="ml-2 text-sm text-gray-700">
                只读模式
              </label>
            </div>

            {/* Action buttons */}
            <div className="flex justify-end space-x-2 pt-4">
              <Button
                variant="outline"
                onClick={() => {
                  setShowAddForm(false)
                  setNewMapping({
                    mapping_type: 'recordings',
                    host_path: '',
                    container_path: '/media/frigate/recordings',
                    read_only: false,
                  })
                }}
              >
                取消
              </Button>
              <Button onClick={handleAddMapping} data-testid="save-volume-button">
                添加映射
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Helper info */}
      {!showAddForm && mappings.length === 0 && (
        <div className="mt-4 p-4 bg-blue-50 border border-blue-200 rounded-md text-sm text-blue-800">
          <div className="font-semibold mb-2">💡 提示</div>
          <ul className="space-y-1 text-xs">
            <li>• 录像需要大量存储空间，建议使用至少 100GB 的磁盘</li>
            <li>• 剪辑通常需要 20GB 以上空间</li>
            <li>• 缓存和配置文件占用空间较小</li>
            <li>• 可以将不同类型的数据存储在不同的磁盘上</li>
          </ul>
        </div>
      )}
    </Card>
  )
}

export default VolumeSelector
