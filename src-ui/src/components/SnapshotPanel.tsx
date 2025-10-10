// Snapshot Panel Component
// Displays configuration snapshots and allows restoration

import React, { useState, useEffect } from 'react'
import { History, Clock, Download, Trash2, Eye, RefreshCw, AlertCircle } from 'lucide-react'
import { configService, ConfigurationSnapshot } from '../services/configService'

interface SnapshotPanelProps {
  onRestore: (yamlContent: string) => void
  currentConfig?: string
}

const SnapshotPanel: React.FC<SnapshotPanelProps> = ({
  onRestore
}) => {
  const [snapshots, setSnapshots] = useState<ConfigurationSnapshot[]>([])
  const [isLoading, setIsLoading] = useState<boolean>(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedSnapshot, setSelectedSnapshot] = useState<ConfigurationSnapshot | null>(null)
  const [showPreview, setShowPreview] = useState<boolean>(false)

  useEffect(() => {
    loadSnapshots()
  }, [])

  const loadSnapshots = async () => {
    setIsLoading(true)
    setError(null)

    try {
      const response = await configService.listSnapshots()
      setSnapshots(response.snapshots)
    } catch (err) {
      console.error('Failed to load snapshots:', err)
      setError('加载快照失败')
    } finally {
      setIsLoading(false)
    }
  }

  const handleRestore = (snapshot: ConfigurationSnapshot) => {
    if (window.confirm(`确定要恢复到这个快照吗？\n创建时间: ${new Date(snapshot.created_at).toLocaleString()}\n${snapshot.description || ''}`)) {
      onRestore(snapshot.yaml_content)
    }
  }

  const handleDelete = async (snapshot: ConfigurationSnapshot) => {
    if (window.confirm(`确定要删除这个快照吗？\n创建时间: ${new Date(snapshot.created_at).toLocaleString()}\n${snapshot.description || ''}`)) {
      try {
        await configService.deleteSnapshot(snapshot.id)
        setSnapshots(snapshots.filter(s => s.id !== snapshot.id))
        if (selectedSnapshot?.id === snapshot.id) {
          setSelectedSnapshot(null)
          setShowPreview(false)
        }
      } catch (err) {
        console.error('Failed to delete snapshot:', err)
        setError('删除快照失败')
      }
    }
  }

  const handlePreview = (snapshot: ConfigurationSnapshot) => {
    setSelectedSnapshot(snapshot)
    setShowPreview(true)
  }

  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
  }

  const getSourceLabel = (source: string) => {
    const labels: Record<string, string> = {
      'ui': '界面',
      'template': '模板',
      'manual_edit': '手动编辑',
      'rollback': '回滚'
    }
    return labels[source] || source
  }

  const getSnapshotIcon = (snapshot: ConfigurationSnapshot) => {
    if (snapshot.deployed) {
      return snapshot.deployment_success ? '🚀' : '❌'
    }
    if (snapshot.is_backup) {
      return '💾'
    }
    return '📸'
  }

  if (isLoading) {
    return (
      <div className="h-full bg-gray-50 border-l border-gray-200 p-4">
        <div className="flex items-center justify-center h-full">
          <RefreshCw className="h-6 w-6 animate-spin text-gray-400" />
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="h-full bg-gray-50 border-l border-gray-200 p-4">
        <div className="flex items-center justify-center h-full text-red-600">
          <div className="text-center">
            <AlertCircle className="h-8 w-8 mx-auto mb-2" />
            <p className="text-sm">{error}</p>
            <button
              onClick={loadSnapshots}
              className="mt-2 px-3 py-1 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200"
            >
              重试
            </button>
          </div>
        </div>
      </div>
    )
  }

  if (showPreview && selectedSnapshot) {
    return (
      <div className="h-full bg-gray-50 border-l border-gray-200 flex flex-col">
        {/* Preview Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200">
          <div className="flex items-center space-x-2">
            <Eye className="h-5 w-5 text-gray-600" />
            <h3 className="font-medium text-gray-900">快照预览</h3>
          </div>
          <button
            onClick={() => setShowPreview(false)}
            className="text-gray-400 hover:text-gray-600"
          >
            ×
          </button>
        </div>

        {/* Preview Info */}
        <div className="p-4 bg-blue-50 border-b border-gray-200">
          <div className="text-sm text-blue-900">
            <p className="font-medium">{selectedSnapshot.description || '无描述'}</p>
            <p className="text-blue-700 mt-1">
              创建时间: {formatDate(selectedSnapshot.created_at)}
            </p>
            <p className="text-blue-700">
              版本: {selectedSnapshot.version} • 来源: {getSourceLabel(selectedSnapshot.created_by)}
            </p>
          </div>
        </div>

        {/* Preview Content */}
        <div className="flex-1 p-4 overflow-y-auto">
          <pre className="text-xs bg-gray-900 text-gray-100 p-3 rounded overflow-x-auto">
            <code>{selectedSnapshot.yaml_content}</code>
          </pre>
        </div>

        {/* Preview Actions */}
        <div className="p-4 border-t border-gray-200 flex space-x-2">
          <button
            onClick={() => handleRestore(selectedSnapshot)}
            className="flex-1 px-3 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            恢复此快照
          </button>
          <button
            onClick={() => setShowPreview(false)}
            className="px-3 py-2 border border-gray-300 text-gray-700 rounded hover:bg-gray-50"
          >
            返回列表
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="h-full bg-gray-50 border-l border-gray-200 flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <div className="flex items-center space-x-2">
          <History className="h-5 w-5 text-gray-600" />
          <h3 className="font-medium text-gray-900">配置快照</h3>
          <span className="text-sm text-gray-500">({snapshots.length})</span>
        </div>
        <button
          onClick={loadSnapshots}
          className="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded"
          title="刷新"
        >
          <RefreshCw className="h-4 w-4" />
        </button>
      </div>

      {/* Snapshot List */}
      <div className="flex-1 overflow-y-auto p-4">
        {snapshots.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            <History className="h-12 w-12 mx-auto mb-3 opacity-50" />
            <p className="text-sm">暂无配置快照</p>
            <p className="text-xs mt-1">保存配置时会自动创建快照</p>
          </div>
        ) : (
          <div className="space-y-3">
            {snapshots.map((snapshot) => (
              <div
                key={snapshot.id}
                className="bg-white border border-gray-200 rounded-lg p-3 hover:shadow-sm transition-shadow"
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center space-x-2 mb-1">
                      <span className="text-lg">{getSnapshotIcon(snapshot)}</span>
                      <h4 className="font-medium text-gray-900 truncate">
                        {snapshot.description || `快照 #${snapshot.version}`}
                      </h4>
                    </div>

                    <div className="flex items-center space-x-3 text-xs text-gray-500 mb-2">
                      <div className="flex items-center space-x-1">
                        <Clock className="h-3 w-3" />
                        <span>{formatDate(snapshot.created_at)}</span>
                      </div>
                      <div className="flex items-center space-x-1">
                        <span className="text-gray-400">•</span>
                        <span>v{snapshot.version}</span>
                      </div>
                      <div className="flex items-center space-x-1">
                        <span className="text-gray-400">•</span>
                        <span>{getSourceLabel(snapshot.created_by)}</span>
                      </div>
                    </div>

                    {snapshot.backup_reason && (
                      <div className="text-xs text-blue-600 mb-2">
                        备份原因: {snapshot.backup_reason}
                      </div>
                    )}

                    {snapshot.deployed && (
                      <div className="flex items-center space-x-1 text-xs">
                        <span className={`${
                          snapshot.deployment_success ? 'text-green-600' : 'text-red-600'
                        }`}>
                          {snapshot.deployment_success ? '✓ 部署成功' : '✗ 部署失败'}
                        </span>
                        {snapshot.deployed_at && (
                          <span className="text-gray-400">
                            • {formatDate(snapshot.deployed_at)}
                          </span>
                        )}
                      </div>
                    )}
                  </div>

                  {/* Actions */}
                  <div className="flex items-center space-x-1 ml-2">
                    <button
                      onClick={() => handlePreview(snapshot)}
                      className="p-1 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded"
                      title="预览"
                    >
                      <Eye className="h-4 w-4" />
                    </button>
                    <button
                      onClick={() => handleRestore(snapshot)}
                      className="p-1 text-gray-400 hover:text-green-600 hover:bg-green-50 rounded"
                      title="恢复"
                    >
                      <Download className="h-4 w-4" />
                    </button>
                    <button
                      onClick={() => handleDelete(snapshot)}
                      className="p-1 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded"
                      title="删除"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </div>
                </div>

                {/* Checksum */}
                <div className="mt-2 text-xs text-gray-400 font-mono">
                  SHA256: {snapshot.checksum.substring(0, 12)}...
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="p-4 border-t border-gray-200 text-xs text-gray-500">
        <p>快照按创建时间排序，最新的在前</p>
        <p>自动备份会在保存配置时创建</p>
      </div>
    </div>
  )
}

export default SnapshotPanel