// YAML Editor Component
// A code editor optimized for YAML configuration editing

import React, { useRef, useEffect, useState } from 'react'
import { Search, Copy, Check } from 'lucide-react'

interface YamlEditorProps {
  content: string
  onChange: (content: string) => void
  readOnly?: boolean
  placeholder?: string
  minHeight?: string
}

const YamlEditor: React.FC<YamlEditorProps> = ({
  content,
  onChange,
  readOnly = false,
  placeholder = '# 输入您的YAML配置...',
  minHeight = '400px'
}) => {
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const [searchTerm, setSearchTerm] = useState<string>('')
  const [searchResults, setSearchResults] = useState<{ index: number; total: number }>({ index: 0, total: 0 })
  const [copied, setCopied] = useState<boolean>(false)
  const [lineNumbers, setLineNumbers] = useState<string[]>([])

  // Update line numbers when content changes
  useEffect(() => {
    const lines = content.split('\n')
    const numbers = lines.map((_, index) => (index + 1).toString())
    setLineNumbers(numbers)
  }, [content])

  // Handle content change
  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newContent = e.target.value
    onChange(newContent)
    updateSearchResults(searchTerm, newContent)
  }

  // Handle tab key for indentation
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Tab' && !readOnly) {
      e.preventDefault()
      const textarea = e.currentTarget
      const start = textarea.selectionStart
      const end = textarea.selectionEnd
      const newContent = content.substring(0, start) + '  ' + content.substring(end)
      onChange(newContent)

      // Restore cursor position
      setTimeout(() => {
        textarea.selectionStart = textarea.selectionEnd = start + 2
      }, 0)
    }

    // Handle search shortcuts
    if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
      e.preventDefault()
      document.getElementById('yaml-search')?.focus()
    }
  }

  // Search functionality
  const updateSearchResults = (term: string, text: string = content) => {
    if (!term.trim()) {
      setSearchResults({ index: 0, total: 0 })
      return
    }

    const regex = new RegExp(term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi')
    const matches = [...text.matchAll(regex)]

    setSearchResults({
      index: matches.length > 0 ? 0 : -1,
      total: matches.length
    })
  }

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    const term = e.target.value
    setSearchTerm(term)
    updateSearchResults(term)
  }

  const navigateSearch = (direction: 'next' | 'prev') => {
    if (searchResults.total === 0) return

    const textarea = textareaRef.current
    if (!textarea) return

    const regex = new RegExp(searchTerm.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi')
    const matches = [...content.matchAll(regex)]

    let newIndex = searchResults.index
    if (direction === 'next') {
      newIndex = (searchResults.index + 1) % matches.length
    } else {
      newIndex = searchResults.index - 1 < 0 ? matches.length - 1 : searchResults.index - 1
    }

    const match = matches[newIndex]
    if (match) {
      textarea.focus()
      textarea.setSelectionRange(match.index!, match.index! + searchTerm.length)
      setSearchResults({ index: newIndex, total: matches.length })
    }
  }

  // Copy to clipboard
  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(content)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (error) {
      console.error('Failed to copy:', error)
    }
  }

  // Calculate line height for synchronizing scrolling
  const lineHeight = 20 // Approximate line height in pixels

  return (
    <div className="flex flex-col h-full bg-gray-900">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center space-x-4">
          {/* Search */}
          <div className="flex items-center space-x-2">
            <Search className="h-4 w-4 text-gray-400" />
            <input
              id="yaml-search"
              type="text"
              value={searchTerm}
              onChange={handleSearch}
              placeholder="搜索..."
              className="px-2 py-1 text-sm bg-gray-700 text-gray-200 border border-gray-600 rounded focus:outline-none focus:border-blue-500 w-32"
            />
            {searchResults.total > 0 && (
              <span className="text-xs text-gray-400">
                {searchResults.index + 1}/{searchResults.total}
              </span>
            )}
            {searchResults.total > 0 && (
              <div className="flex items-center space-x-1">
                <button
                  onClick={() => navigateSearch('prev')}
                  className="px-1 py-0.5 text-xs text-gray-400 hover:text-gray-200 hover:bg-gray-700 rounded"
                >
                  ↑
                </button>
                <button
                  onClick={() => navigateSearch('next')}
                  className="px-1 py-0.5 text-xs text-gray-400 hover:text-gray-200 hover:bg-gray-700 rounded"
                >
                  ↓
                </button>
              </div>
            )}
          </div>

          {/* Copy button */}
          <button
            onClick={handleCopy}
            className="flex items-center space-x-1 px-2 py-1 text-sm text-gray-300 hover:text-white hover:bg-gray-700 rounded"
          >
            {copied ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
            <span>{copied ? '已复制' : '复制'}</span>
          </button>
        </div>

        <div className="text-xs text-gray-400">
          YAML格式 • {content.split('\n').length} 行
        </div>
      </div>

      {/* Editor */}
      <div className="flex-1 flex overflow-hidden">
        {/* Line numbers */}
        <div className="flex-shrink-0 w-12 bg-gray-800 border-r border-gray-700 text-right">
          <div
            className="text-xs text-gray-500 pr-2 select-none overflow-hidden"
            style={{ lineHeight: `${lineHeight}px`, minHeight }}
          >
            {lineNumbers.map((lineNum, index) => (
              <div key={index} className="hover:text-gray-300">
                {lineNum}
              </div>
            ))}
          </div>
        </div>

        {/* Textarea */}
        <div className="flex-1 relative">
          <textarea
            ref={textareaRef}
            value={content}
            onChange={handleChange}
            onKeyDown={handleKeyDown}
            readOnly={readOnly}
            placeholder={placeholder}
            className="absolute inset-0 w-full h-full p-0 bg-transparent text-gray-100 font-mono text-sm resize-none outline-none leading-5"
            style={{
              padding: '0 12px',
              lineHeight: `${lineHeight}px`,
              minHeight,
              tabSize: 2,
              scrollPadding: '20px'
            }}
            spellCheck={false}
          />

          {/* Highlight overlay for search results */}
          {searchTerm && searchResults.total > 0 && (
            <div className="absolute inset-0 pointer-events-none">
              {/* This would require a more sophisticated implementation for highlighting */}
            </div>
          )}
        </div>
      </div>

      {/* Status bar */}
      <div className="flex items-center justify-between px-4 py-1 bg-gray-800 border-t border-gray-700 text-xs text-gray-400">
        <div className="flex items-center space-x-4">
          <span>{content.length} 字符</span>
          <span>{content.split('\n').length} 行</span>
          {content !== content.trim() && (
            <span className="text-yellow-400">有尾随空格</span>
          )}
        </div>
        <div className="flex items-center space-x-4">
          {readOnly && (
            <span className="text-gray-500">只读模式</span>
          )}
          <span>UTF-8</span>
          <span>YAML</span>
        </div>
      </div>
    </div>
  )
}

export default YamlEditor