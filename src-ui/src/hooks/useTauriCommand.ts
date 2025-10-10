// Custom hook for Tauri IPC commands
// Provides a React-friendly interface for invoking Tauri commands

import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

interface UseTauriCommandOptions<T> {
  onSuccess?: (data: T) => void
  onError?: (error: Error) => void
}

interface UseTauriCommandReturn<T, P> {
  data: T | null
  error: Error | null
  loading: boolean
  execute: (params?: P) => Promise<T | null>
  reset: () => void
}

/**
 * Hook for executing Tauri commands with loading and error states
 * @param commandName - The name of the Tauri command to invoke
 * @param options - Optional callbacks for success and error
 * @returns Object containing data, error, loading state, and execute function
 */
export function useTauriCommand<T = unknown, P = Record<string, unknown>>(
  commandName: string,
  options?: UseTauriCommandOptions<T>
): UseTauriCommandReturn<T, P> {
  const [data, setData] = useState<T | null>(null)
  const [error, setError] = useState<Error | null>(null)
  const [loading, setLoading] = useState(false)

  const execute = useCallback(
    async (params?: P): Promise<T | null> => {
      setLoading(true)
      setError(null)

      try {
        const result = await invoke<T>(commandName, params || {})
        setData(result)
        options?.onSuccess?.(result)
        return result
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err))
        setError(error)
        options?.onError?.(error)
        return null
      } finally {
        setLoading(false)
      }
    },
    [commandName, options]
  )

  const reset = useCallback(() => {
    setData(null)
    setError(null)
    setLoading(false)
  }, [])

  return { data, error, loading, execute, reset }
}

/**
 * Hook that automatically executes a Tauri command on mount
 * @param commandName - The name of the Tauri command to invoke
 * @param params - Parameters to pass to the command
 * @param options - Optional callbacks for success and error
 * @returns Object containing data, error, loading state, and refetch function
 */
export function useTauriQuery<T = unknown, P = Record<string, unknown>>(
  commandName: string,
  params?: P,
  options?: UseTauriCommandOptions<T>
): UseTauriCommandReturn<T, P> & { refetch: () => Promise<T | null> } {
  const command = useTauriCommand<T, P>(commandName, options)

  // Execute on mount
  useState(() => {
    command.execute(params)
  })

  const refetch = useCallback(() => {
    return command.execute(params)
  }, [command, params])

  return { ...command, refetch }
}

export default useTauriCommand
