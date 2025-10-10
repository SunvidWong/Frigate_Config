// Tauri environment detection and fallback utilities

/**
 * Check if running in Tauri environment
 */
export const isTauriEnvironment = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_IPC__' in window;
};

/**
 * Safe invoke wrapper that checks for Tauri environment
 */
export const safeInvoke = async <T>(command: string, args?: any): Promise<T> => {
  if (!isTauriEnvironment()) {
    throw new Error('摄像头扫描功能仅在桌面应用中可用。Docker 部署版本不支持此功能。');
  }

  const { invoke } = await import('@tauri-apps/api');
  return invoke<T>(command, args);
};

/**
 * Get environment type
 */
export const getEnvironmentType = (): 'tauri' | 'web' => {
  return isTauriEnvironment() ? 'tauri' : 'web';
};
