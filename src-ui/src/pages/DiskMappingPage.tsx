// T183, T186, T187: DiskMapping page component
// Handles disk information display and volume mapping configuration

import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import Button from '../components/Button'
import Card, { CardHeader } from '../components/Card'
import DiskInfoCard, { DiskInfo } from '../components/DiskInfoCard'
import VolumeSelector, { VolumeMapping } from '../components/VolumeSelector'

interface DefaultVolumePaths {
  config: string
  recordings: string
  clips: string
  cache: string
}

interface RecommendedPath {
  path: string
  total_space: string
  free_space: string
  usage_percent: number
  recommended_for: string[]
}

interface VolumeValidationResponse {
  valid: boolean
  errors: string[]
  warnings: string[]
  disk_info: DiskInfo | null
}

const DiskMappingPage: React.FC = () => {
  // State
  const [selectedPath, setSelectedPath] = useState<string>('')
  const [diskInfo, setDiskInfo] = useState<DiskInfo | null>(null)
  const [diskInfoLoading, setDiskInfoLoading] = useState(false)
  const [diskInfoError, setDiskInfoError] = useState<string | null>(null)

  const [volumeMappings, setVolumeMappings] = useState<VolumeMapping[]>([])
  const [defaultPaths, setDefaultPaths] = useState<DefaultVolumePaths | null>(null)
  const [recommendedPaths, setRecommendedPaths] = useState<RecommendedPath[]>([])
  const [recommendedPathsLoading, setRecommendedPathsLoading] = useState(false)

  const [validationResult, setValidationResult] = useState<VolumeValidationResponse | null>(null)
  const [validating, setValidating] = useState(false)

  // T186: Load default paths on mount
  useEffect(() => {
    loadDefaultPaths()
    loadRecommendedPaths()
  }, [])

  const loadDefaultPaths = async () => {
    try {
      const paths = await invoke<DefaultVolumePaths>('get_default_volume_paths')
      setDefaultPaths(paths)
    } catch (error) {
      console.error('Failed to load default paths:', error)
    }
  }

  const loadRecommendedPaths = async () => {
    setRecommendedPathsLoading(true)
    try {
      const paths = await invoke<RecommendedPath[]>('get_recommended_paths')
      setRecommendedPaths(paths)
    } catch (error) {
      console.error('Failed to load recommended paths:', error)
    } finally {
      setRecommendedPathsLoading(false)
    }
  }

  // T186: Get disk info for selected path
  const handleGetDiskInfo = async (path: string) => {
    if (!path.trim()) {
      setDiskInfoError('请输入有效路径')
      return
    }

    setDiskInfoLoading(true)
    setDiskInfoError(null)
    setSelectedPath(path)

    try {
      const info = await invoke<DiskInfo>('get_disk_info_command', { path })
      setDiskInfo(info)
    } catch (error) {
      console.error('Failed to get disk info:', error)
      setDiskInfoError(String(error))
      setDiskInfo(null)
    } finally {
      setDiskInfoLoading(false)
    }
  }

  // Browse for directory
  const handleBrowseDirectory = async (currentPath: string): Promise<string | null> => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: currentPath || undefined,
      })

      if (selected && typeof selected === 'string') {
        return selected
      }
    } catch (error) {
      console.error('Failed to open directory dialog:', error)
    }
    return null
  }

  // T187: Volume mapping management
  const handleAddMapping = async (mapping: VolumeMapping) => {
    // Validate the mapping via backend
    setValidating(true)
    try {
      const created = await invoke<VolumeMapping>('create_volume_mapping', {
        hostPath: mapping.host_path,
        containerPath: mapping.container_path,
        mappingType: mapping.mapping_type,
        readOnly: mapping.read_only,
        description: mapping.description || null,
      })

      setVolumeMappings([...volumeMappings, created])
      setValidationResult(null)
    } catch (error) {
      console.error('Failed to create volume mapping:', error)
      alert(`创建卷映射失败: ${error}`)
    } finally {
      setValidating(false)
    }
  }

  const handleRemoveMapping = (index: number) => {
    const confirmed = confirm('确定要删除此卷映射吗？')
    if (confirmed) {
      setVolumeMappings(volumeMappings.filter((_, i) => i !== index))
    }
  }

  const handleUpdateMapping = (index: number, mapping: VolumeMapping) => {
    const updated = [...volumeMappings]
    updated[index] = mapping
    setVolumeMappings(updated)
  }

  const handleValidatePath = async (path: string) => {
    if (!path.trim()) {
      return
    }

    setValidating(true)
    try {
      const result = await invoke<VolumeValidationResponse>('validate_volume_path_command', { path })
      setValidationResult(result)
      if (result.disk_info) {
        setDiskInfo(result.disk_info)
      }
    } catch (error) {
      console.error('Failed to validate path:', error)
    } finally {
      setValidating(false)
    }
  }

  const handleSelectRecommendedPath = (path: string) => {
    setSelectedPath(path)
    handleGetDiskInfo(path)
  }

  const handleUseDefaultPaths = () => {
    if (!defaultPaths) return

    const defaultMappings: VolumeMapping[] = [
      {
        host_path: defaultPaths.config,
        container_path: '/config',
        mapping_type: 'config',
        read_only: false,
      },
      {
        host_path: defaultPaths.recordings,
        container_path: '/media/frigate/recordings',
        mapping_type: 'recordings',
        read_only: false,
      },
      {
        host_path: defaultPaths.clips,
        container_path: '/media/frigate/clips',
        mapping_type: 'clips',
        read_only: false,
      },
      {
        host_path: defaultPaths.cache,
        container_path: '/tmp/cache',
        mapping_type: 'cache',
        read_only: false,
      },
    ]

    // Add each mapping through the validation process
    defaultMappings.forEach(mapping => handleAddMapping(mapping))
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">磁盘与卷映射</h1>
        <p className="text-gray-600">
          配置存储位置并查看磁盘空间信息
        </p>
      </div>

      {/* Quick actions */}
      <Card className="mb-6">
        <CardHeader title="快速操作" />
        <div className="flex flex-wrap gap-2">
          <Button
            onClick={handleUseDefaultPaths}
            variant="outline"
            disabled={!defaultPaths || validating}
          >
            使用默认路径
          </Button>
          <Button
            onClick={loadRecommendedPaths}
            variant="outline"
            loading={recommendedPathsLoading}
          >
            {recommendedPathsLoading ? '扫描中...' : '扫描推荐路径'}
          </Button>
        </div>
      </Card>

      {/* Recommended paths */}
      {recommendedPaths.length > 0 && (
        <Card className="mb-6">
          <CardHeader title="推荐路径" subtitle="系统检测到以下可用存储位置" />
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4" data-testid="disk-list">
            {recommendedPaths.map((recommended, index) => (
              <div
                key={index}
                className="border border-gray-200 rounded-md p-4 hover:border-blue-400 hover:bg-blue-50 cursor-pointer transition-colors"
                onClick={() => handleSelectRecommendedPath(recommended.path)}
                data-testid="disk-card"
              >
                <div className="flex items-start justify-between mb-2">
                  <div className="flex-1">
                    <div className="font-semibold text-gray-900">{recommended.path}</div>
                    <div className="text-sm text-gray-600 mt-1">
                      总容量: {recommended.total_space} • 剩余: {recommended.free_space}
                    </div>
                  </div>
                  <div className={`text-lg font-bold ${
                    recommended.usage_percent > 80 ? 'text-orange-600' : 'text-green-600'
                  }`}>
                    {recommended.usage_percent.toFixed(0)}%
                  </div>
                </div>
                <div className="flex flex-wrap gap-1 mt-2">
                  {recommended.recommended_for.map((type) => (
                    <span
                      key={type}
                      className="px-2 py-0.5 bg-blue-100 text-blue-700 text-xs rounded-full"
                    >
                      {type}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </Card>
      )}

      {/* Path input and disk info */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        {/* Path selector */}
        <Card>
          <CardHeader title="检查磁盘空间" />
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                输入路径
              </label>
              <div className="flex space-x-2">
                <input
                  type="text"
                  value={selectedPath}
                  onChange={(e) => setSelectedPath(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && handleGetDiskInfo(selectedPath)}
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="/path/to/check"
                />
                <Button
                  variant="outline"
                  onClick={async () => {
                    const path = await handleBrowseDirectory(selectedPath)
                    if (path) {
                      setSelectedPath(path)
                      handleGetDiskInfo(path)
                    }
                  }}
                >
                  浏览
                </Button>
              </div>
            </div>
            <div className="flex space-x-2">
              <Button
                onClick={() => handleGetDiskInfo(selectedPath)}
                loading={diskInfoLoading}
                disabled={!selectedPath.trim()}
                className="flex-1"
                data-testid="configure-volume-button"
              >
                {diskInfoLoading ? '检查中...' : '检查磁盘空间'}
              </Button>
              <Button
                onClick={() => handleValidatePath(selectedPath)}
                loading={validating}
                disabled={!selectedPath.trim()}
                variant="outline"
                data-testid="validate-path-button"
              >
                验证路径
              </Button>
            </div>

            {/* Validation result */}
            {validationResult && (
              <div className={`p-3 rounded-md ${
                validationResult.valid ? 'bg-green-50 border border-green-200' : 'bg-red-50 border border-red-200'
              }`} data-testid="path-validation-result">
                <div className="font-semibold mb-2">
                  {validationResult.valid ? '✓ 路径有效' : '✗ 路径验证失败'}
                </div>
                {validationResult.errors.length > 0 && (
                  <div className="space-y-1 text-sm text-red-700" data-testid="path-validation-error">
                    {validationResult.errors.map((error, i) => (
                      <div key={i}>• {error}</div>
                    ))}
                  </div>
                )}
                {validationResult.warnings.length > 0 && (
                  <div className="space-y-1 text-sm text-yellow-700 mt-2">
                    {validationResult.warnings.map((warning, i) => (
                      <div key={i}>⚠ {warning}</div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        </Card>

        {/* T186: Disk info display */}
        <DiskInfoCard
          diskInfo={diskInfo}
          loading={diskInfoLoading}
          error={diskInfoError}
          onRefresh={() => selectedPath && handleGetDiskInfo(selectedPath)}
        />
      </div>

      {/* T187: Volume mappings configuration */}
      <VolumeSelector
        mappings={volumeMappings}
        onAddMapping={handleAddMapping}
        onRemoveMapping={handleRemoveMapping}
        onUpdateMapping={handleUpdateMapping}
        defaultPaths={defaultPaths || undefined}
        onBrowse={handleBrowseDirectory}
      />

      {/* Summary and export */}
      {volumeMappings.length > 0 && (
        <Card className="mt-6">
          <CardHeader title="映射摘要" subtitle="将用于 Docker 部署的卷映射" />
          <div className="space-y-2" data-testid="docker-run-preview">
            {volumeMappings.map((mapping, index) => (
              <div
                key={index}
                className="bg-gray-50 rounded p-3 font-mono text-xs"
              >
                <code className="text-blue-600">
                  -v {mapping.host_path}:{mapping.container_path}{mapping.read_only ? ':ro' : ''}
                </code>
              </div>
            ))}
          </div>
          <div className="mt-4 pt-4 border-t border-gray-200 text-sm text-gray-600">
            <p>这些映射将在部署时自动应用到 Frigate 容器。</p>
            <p className="mt-1">前往 <span className="font-semibold">部署页面</span> 查看完整的部署配置。</p>
          </div>
        </Card>
      )}
    </div>
  )
}

export default DiskMappingPage
