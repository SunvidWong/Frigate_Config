// Configuration Editor Page
// Main interface for editing Frigate configuration files

import React, { useState, useEffect } from 'react'
import { FileText, Save, Upload, Download, History, CheckCircle, AlertCircle, Eye, EyeOff } from 'lucide-react'
import { configService, ConfigurationData, SaveConfigResponse, ValidationResult } from '../services/configService'
import YamlEditor from '../components/YamlEditor'
import ValidationPanel from '../components/ValidationPanel'
import SnapshotPanel from '../components/SnapshotPanel'

const ConfigEditorPage: React.FC = () => {
  const [configData, setConfigData] = useState<ConfigurationData | null>(null)
  const [yamlContent, setYamlContent] = useState<string>('')
  const [isLoading, setIsLoading] = useState<boolean>(false)
  const [saveStatus, setSaveStatus] = useState<string>('')
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null)
  const [showValidation, setShowValidation] = useState<boolean>(true)
  const [showSnapshots, setShowSnapshots] = useState<boolean>(false)
  const [currentFilePath, setCurrentFilePath] = useState<string>('')
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState<boolean>(false)

  useEffect(() => {
    // Load default config or recent config on mount
    loadDefaultConfig()
  }, [])

  useEffect(() => {
    // Auto-validate when content changes
    if (yamlContent) {
      validateConfig()
    }
  }, [yamlContent])

  const loadDefaultConfig = async () => {
    setIsLoading(true)
    try {
      // Try to load a default config file
      const defaultPath = '/config/frigate.yml'
      const data = await configService.loadConfig(defaultPath)
      setConfigData(data)
      setYamlContent(data.raw_yaml)
      setCurrentFilePath(data.file_path)
      setSaveStatus('配置已加载')
    } catch (error) {
      console.log('No default config found, starting with empty config')
      setYamlContent('# Frigate Configuration\n# Start editing your configuration here\n\ncameras:\n  # Add your cameras here\n\ndetectors:\n  # Configure detectors here\n\n# Global configuration\nmqtt:\n  enabled: false\n  host: localhost\n  port: 1883\n')
      setSaveStatus('新配置文件')
    } finally {
      setIsLoading(false)
    }
  }

  const handleFileLoad = async () => {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.yml,.yaml'
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0]
      if (!file) return

      setIsLoading(true)
      try {
        // For now, read file content directly
        // In a real implementation, you'd upload and parse via the backend
        const content = await file.text()
        setYamlContent(content)
        setCurrentFilePath(file.name)
        setHasUnsavedChanges(false)
        setSaveStatus(`已加载: ${file.name}`)
      } catch (error) {
        console.error('Failed to load file:', error)
        setSaveStatus('加载文件失败')
      } finally {
        setIsLoading(false)
      }
    }
    input.click()
  }

  const handleSave = async () => {
    if (!yamlContent.trim()) {
      setSaveStatus('没有内容可保存')
      return
    }

    setIsLoading(true)
    setSaveStatus('保存中...')

    try {
      const saveRequest = {
        config_data: {
          cameras: configData?.cameras || {},
          detectors: configData?.detectors || {},
          global_config: configData?.global_config || {},
          raw_yaml: yamlContent,
          file_path: currentFilePath || '/config/frigate.yml',
          parsed_at: configData?.parsed_at || new Date().toISOString(),
          warnings: configData?.warnings || [],
          errors: configData?.errors || []
        },
        target_path: currentFilePath || '/config/frigate.yml',
        create_backup: true,
        validate_before_save: true
      }

      const response: SaveConfigResponse = await configService.saveConfig(saveRequest)

      if (response.success) {
        setSaveStatus(`已保存: ${response.saved_path}`)
        setHasUnsavedChanges(false)
        if (response.backup_path) {
          setSaveStatus(prev => `${prev} (备份: ${response.backup_path})`)
        }
      } else {
        setSaveStatus('保存失败')
      }
    } catch (error) {
      console.error('Save failed:', error)
      setSaveStatus('保存失败')
    } finally {
      setIsLoading(false)
    }
  }

  const handleExport = () => {
    const blob = new Blob([yamlContent], { type: 'text/yaml' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = currentFilePath.split('/').pop() || 'frigate.yml'
    a.click()
    URL.revokeObjectURL(url)
  }

  const validateConfig = async () => {
    try {
      const result = await configService.validateConfig(yamlContent)
      setValidationResult(result)
    } catch (error) {
      console.error('Validation failed:', error)
      setValidationResult({
        valid: false,
        errors: [{ field: 'general', message: '验证失败', severity: 'error' }],
        warnings: ['无法连接到验证服务'],
        config_type: 'unknown',
        schema_version: 'unknown'
      })
    }
  }

  const handleYamlChange = (content: string) => {
    setYamlContent(content)
    setHasUnsavedChanges(true)
  }

  const handleSnapshotRestore = (snapshotYaml: string) => {
    setYamlContent(snapshotYaml)
    setHasUnsavedChanges(true)
    setShowSnapshots(false)
  }

  const isConfigValid = validationResult?.valid ?? true
  const hasErrors = (validationResult?.errors?.length || 0) > 0

  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="bg-white shadow rounded-lg">
        {/* Header */}
        <div className="border-b border-gray-200 px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <FileText className="h-6 w-6 text-blue-600" />
              <div>
                <h1 className="text-2xl font-bold text-gray-900">配置编辑器</h1>
                <p className="text-sm text-gray-500">
                  {currentFilePath || '新建配置'}
                  {hasUnsavedChanges && ' (未保存)'}
                </p>
              </div>
            </div>

            {/* Status */}
            <div className="flex items-center space-x-4">
              {saveStatus && (
                <div className={`flex items-center space-x-2 text-sm ${
                  saveStatus.includes('失败') ? 'text-red-600' :
                  saveStatus.includes('保存中') ? 'text-yellow-600' : 'text-green-600'
                }`}>
                  {saveStatus.includes('失败') ? (
                    <AlertCircle className="h-4 w-4" />
                  ) : (
                    <CheckCircle className="h-4 w-4" />
                  )}
                  <span>{saveStatus}</span>
                </div>
              )}

              {isConfigValid ? (
                <div className="flex items-center space-x-1 text-green-600">
                  <CheckCircle className="h-4 w-4" />
                  <span className="text-sm">配置有效</span>
                </div>
              ) : hasErrors && (
                <div className="flex items-center space-x-1 text-red-600">
                  <AlertCircle className="h-4 w-4" />
                  <span className="text-sm">配置有误</span>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Toolbar */}
        <div className="border-b border-gray-200 px-6 py-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <button
                onClick={handleFileLoad}
                disabled={isLoading}
                className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
              >
                <Upload className="h-4 w-4 mr-2" />
                加载文件
              </button>

              <button
                onClick={handleSave}
                disabled={isLoading || !yamlContent.trim()}
                className="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
              >
                <Save className="h-4 w-4 mr-2" />
                保存
              </button>

              <button
                onClick={handleExport}
                disabled={!yamlContent.trim()}
                className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
              >
                <Download className="h-4 w-4 mr-2" />
                导出
              </button>
            </div>

            <div className="flex items-center space-x-2">
              <button
                onClick={() => setShowValidation(!showValidation)}
                className={`inline-flex items-center px-3 py-2 border text-sm leading-4 font-medium rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 ${
                  showValidation
                    ? 'border-blue-500 text-blue-700 bg-blue-50'
                    : 'border-gray-300 text-gray-700 bg-white hover:bg-gray-50'
                }`}
              >
                {showValidation ? <Eye className="h-4 w-4 mr-2" /> : <EyeOff className="h-4 w-4 mr-2" />}
                验证
              </button>

              <button
                onClick={() => setShowSnapshots(!showSnapshots)}
                className={`inline-flex items-center px-3 py-2 border text-sm leading-4 font-medium rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 ${
                  showSnapshots
                    ? 'border-blue-500 text-blue-700 bg-blue-50'
                    : 'border-gray-300 text-gray-700 bg-white hover:bg-gray-50'
                }`}
              >
                <History className="h-4 w-4 mr-2" />
                快照
              </button>
            </div>
          </div>
        </div>

        {/* Main Content */}
        <div className="flex h-screen max-h-[800px]">
          {/* Editor */}
          <div className={`flex-1 ${showValidation || showSnapshots ? 'border-r border-gray-200' : ''}`}>
            <YamlEditor
              content={yamlContent}
              onChange={handleYamlChange}
              readOnly={isLoading}
            />
          </div>

          {/* Side Panel */}
          <div className="w-96 flex flex-col">
            {showValidation && (
              <ValidationPanel
                validationResult={validationResult}
                onRefresh={validateConfig}
              />
            )}

            {showSnapshots && (
              <SnapshotPanel
                onRestore={handleSnapshotRestore}
                currentConfig={yamlContent}
              />
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export default ConfigEditorPage