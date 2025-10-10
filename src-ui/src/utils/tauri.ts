// Tauri environment detection and fallback utilities

/**
 * Check if running in Tauri environment
 */
export const isTauriEnvironment = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_IPC__' in window;
};

/**
 * Commands supported in Docker HTTP mode
 */
const HTTP_SUPPORTED_COMMANDS = [
  'scan_for_cameras',
  'quick_scan_cameras',
  'guess_network_range_command',
  'add_hardware_device_to_config',
  'get_saved_hardware_devices',
];

/**
 * Safe invoke wrapper that checks for Tauri environment
 * Falls back to HTTP API for Docker mode
 */
export const safeInvoke = async <T>(command: string, args?: any): Promise<T> => {
  // Use Tauri IPC if available
  if (isTauriEnvironment()) {
    const { invoke } = await import('@tauri-apps/api');
    return invoke<T>(command, args);
  }

  // Check if command is supported in HTTP mode
  if (!HTTP_SUPPORTED_COMMANDS.includes(command)) {
    throw new Error(`此功能仅在桌面应用中可用: ${command}`);
  }

  // Use HTTP API for Docker/web mode
  try {
    const response = await fetch(`/api/${command}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(args || {}),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`API 调用失败 (${response.status}): ${errorText}`);
    }

    const result = await response.json();

    // Handle our ApiResponse format
    if (result.success === false) {
      throw new Error(result.error || '未知错误');
    }

    return result.data;
  } catch (error) {
    if (error instanceof Error) {
      throw error;
    }
    throw new Error(`网络请求失败: ${error}`);
  }
};

/**
 * Get environment type
 */
export const getEnvironmentType = (): 'tauri' | 'web' => {
  return isTauriEnvironment() ? 'tauri' : 'web';
};
