import { useState, useEffect } from 'react'
import { FileText, Save, Upload, Download, CheckCircle, AlertCircle, Wand2 } from 'lucide-react'
import YamlEditor from '../components/YamlEditor'
import * as yaml from 'js-yaml'

interface YamlFormatError {
  line?: number
  message: string
}

const ConfigEditorPage = () => {
  const [yamlContent, setYamlContent] = useState<string>('')
  const [currentFilePath, setCurrentFilePath] = useState<string>('frigate.yml')
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState<boolean>(false)
  const [saveStatus, setSaveStatus] = useState<string>('')
  const [formatErrors, setFormatErrors] = useState<YamlFormatError[]>([])
  const [isValid, setIsValid] = useState<boolean>(true)

  useEffect(() => {
    // 从 localStorage 加载配置
    const saved = localStorage.getItem('frigate_config')
    if (saved) {
      setYamlContent(saved)
      checkYamlFormat(saved)
    } else {
      // 默认配置
      const defaultConfig = `# Frigate 配置文件
mqtt:
  enabled: false
  host: localhost

detectors:
  default:
    type: cpu

cameras:
  # 在此添加摄像头配置
`
      setYamlContent(defaultConfig)
      localStorage.setItem('frigate_config', defaultConfig)
    }
  }, [])

  // 检查 YAML 格式
  const checkYamlFormat = (content: string): boolean => {
    if (!content.trim()) {
      setFormatErrors([])
      setIsValid(true)
      return true
    }

    try {
      yaml.load(content)
      setFormatErrors([])
      setIsValid(true)
      return true
    } catch (error: any) {
      const yamlError = error as yaml.YAMLException
      setFormatErrors([{
        line: yamlError.mark?.line,
        message: yamlError.message || '格式错误'
      }])
      setIsValid(false)
      return false
    }
  }

  // 自动修复 YAML 格式
  const autoFixYaml = (content: string): string => {
    try {
      // 尝试解析并重新格式化
      const parsed = yaml.load(content)
      const fixed = yaml.dump(parsed, {
        indent: 2,
        lineWidth: 120,
        noRefs: true,
        sortKeys: false
      })
      return fixed
    } catch (error) {
      // 如果无法解析，尝试一些基础修复
      let fixed = content

      // 修复缩进问题
      fixed = fixed.split('\n').map(line => {
        // 移除行尾空格
        return line.trimEnd()
      }).join('\n')

      // 修复常见的 YAML 错误
      fixed = fixed.replace(/:\s*\n\s+/g, ':\n  ')  // 修复缩进
      fixed = fixed.replace(/:\s{2,}/g, ': ')  // 修复多余空格

      return fixed
    }
  }

  // 处理内容变化
  const handleYamlChange = (content: string) => {
    setYamlContent(content)
    setHasUnsavedChanges(true)
    checkYamlFormat(content)
    localStorage.setItem('frigate_config', content)
  }

  // 一键修复
  const handleQuickFix = () => {
    if (!yamlContent.trim()) return

    const fixed = autoFixYaml(yamlContent)
    setYamlContent(fixed)
    setHasUnsavedChanges(true)
    localStorage.setItem('frigate_config', fixed)

    const isFixed = checkYamlFormat(fixed)
    if (isFixed) {
      setSaveStatus('格式已修复！')
    } else {
      setSaveStatus('修复后仍有格式错误，请手动检查')
    }

    // 3秒后清除状态
    setTimeout(() => setSaveStatus(''), 3000)
  }

  // 加载文件
  const handleFileLoad = async () => {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.yml,.yaml'
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0]
      if (!file) return

      try {
        const content = await file.text()
        setYamlContent(content)
        setCurrentFilePath(file.name)
        setHasUnsavedChanges(false)
        checkYamlFormat(content)
        localStorage.setItem('frigate_config', content)
        setSaveStatus(`已加载: ${file.name}`)
        setTimeout(() => setSaveStatus(''), 3000)
      } catch (error) {
        console.error('Failed to load file:', error)
        setSaveStatus('加载文件失败')
        setTimeout(() => setSaveStatus(''), 3000)
      }
    }
    input.click()
  }

  // 保存文件
  const handleSave = () => {
    if (!yamlContent.trim()) {
      setSaveStatus('没有内容可保存')
      setTimeout(() => setSaveStatus(''), 3000)
      return
    }

    // 检查格式
    if (!checkYamlFormat(yamlContent)) {
      setSaveStatus('格式错误，无法保存')
      setTimeout(() => setSaveStatus(''), 3000)
      return
    }

    // 保存到 localStorage
    localStorage.setItem('frigate_config', yamlContent)
    setHasUnsavedChanges(false)
    setSaveStatus('配置已保存到本地')
    setTimeout(() => setSaveStatus(''), 3000)
  }

  // 导出文件
  const handleExport = () => {
    const blob = new Blob([yamlContent], { type: 'text/yaml' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = currentFilePath
    a.click()
    URL.revokeObjectURL(url)
    setSaveStatus('配置已导出')
    setTimeout(() => setSaveStatus(''), 3000)
  }

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      {/* 顶部标题栏 */}
      <div className="bg-white shadow-sm border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <FileText className="h-6 w-6 text-blue-600" />
            <div>
              <h1 className="text-2xl font-bold text-gray-900">配置编辑器</h1>
              <p className="text-sm text-gray-500">
                {currentFilePath}
                {hasUnsavedChanges && ' (未保存)'}
              </p>
            </div>
          </div>

          {/* 状态指示器 */}
          <div className="flex items-center space-x-4">
            {saveStatus && (
              <div className={`flex items-center space-x-2 text-sm ${
                saveStatus.includes('失败') || saveStatus.includes('错误')
                  ? 'text-red-600'
                  : 'text-green-600'
              }`}>
                {saveStatus.includes('失败') || saveStatus.includes('错误') ? (
                  <AlertCircle className="h-4 w-4" />
                ) : (
                  <CheckCircle className="h-4 w-4" />
                )}
                <span>{saveStatus}</span>
              </div>
            )}

            {isValid ? (
              <div className="flex items-center space-x-1 text-green-600">
                <CheckCircle className="h-4 w-4" />
                <span className="text-sm">格式正确</span>
              </div>
            ) : (
              <div className="flex items-center space-x-1 text-red-600">
                <AlertCircle className="h-4 w-4" />
                <span className="text-sm">格式错误</span>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 工具栏 */}
      <div className="bg-white border-b border-gray-200 px-6 py-3">
        <div className="flex items-center gap-2">
          <button
            onClick={handleFileLoad}
            className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
          >
            <Upload className="h-4 w-4 mr-2" />
            加载文件
          </button>

          <button
            onClick={handleSave}
            disabled={!yamlContent.trim()}
            className="inline-flex items-center px-3 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-50"
          >
            <Save className="h-4 w-4 mr-2" />
            保存
          </button>

          <button
            onClick={handleExport}
            disabled={!yamlContent.trim()}
            className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50"
          >
            <Download className="h-4 w-4 mr-2" />
            导出
          </button>

          <button
            onClick={handleQuickFix}
            disabled={!yamlContent.trim() || isValid}
            className="inline-flex items-center px-3 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 disabled:opacity-50"
            title="一键修复 YAML 格式错误"
          >
            <Wand2 className="h-4 w-4 mr-2" />
            一键修复
          </button>
        </div>
      </div>

      {/* 主内容区域 - 左右分栏 */}
      <div className="flex-1 flex overflow-hidden">
        {/* 左侧: 编辑器 */}
        <div className="flex-1 bg-white">
          <YamlEditor
            content={yamlContent}
            onChange={handleYamlChange}
            readOnly={false}
          />
        </div>

        {/* 右侧: 格式错误显示 */}
        {formatErrors.length > 0 && (
          <div className="w-96 border-l border-gray-200 bg-white p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">格式错误</h3>
            <div className="space-y-3">
              {formatErrors.map((error, index) => (
                <div key={index} className="p-3 bg-red-50 border border-red-200 rounded-lg">
                  {error.line !== undefined && (
                    <p className="text-sm font-medium text-red-900 mb-1">
                      第 {error.line + 1} 行
                    </p>
                  )}
                  <p className="text-sm text-red-700">{error.message}</p>
                </div>
              ))}

              <div className="pt-4">
                <button
                  onClick={handleQuickFix}
                  className="w-full px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition font-semibold"
                >
                  一键修复格式
                </button>
              </div>

              <div className="mt-4 p-3 bg-blue-50 border border-blue-200 rounded-lg">
                <p className="text-sm text-blue-800">
                  <strong>提示：</strong>点击"一键修复"按钮自动修正 YAML 格式错误
                </p>
              </div>
            </div>
          </div>
        )}

        {/* 无错误时显示帮助信息 */}
        {formatErrors.length === 0 && isValid && (
          <div className="w-96 border-l border-gray-200 bg-white p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">格式检查</h3>
            <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <CheckCircle className="h-5 w-5 text-green-600" />
                <p className="font-medium text-green-900">格式正确</p>
              </div>
              <p className="text-sm text-green-700">
                YAML 格式验证通过，配置文件格式正确
              </p>
            </div>

            <div className="mt-6">
              <h4 className="text-sm font-semibold text-gray-900 mb-2">编辑器功能</h4>
              <ul className="text-sm text-gray-600 space-y-2">
                <li>• 自动格式检查</li>
                <li>• 语法高亮显示</li>
                <li>• 一键格式修复</li>
                <li>• 本地自动保存</li>
                <li>• 导入/导出配置</li>
              </ul>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default ConfigEditorPage
