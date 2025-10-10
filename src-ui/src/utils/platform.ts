// Platform detection utility for cross-platform hardware management
// Provides platform and architecture detection with normalization

export type Platform = 'linux' | 'darwin' | 'windows' | 'unknown';
export type Architecture = 'x64' | 'arm64' | 'arm' | 'x86' | 'unknown';

export interface PlatformInfo {
  platform: Platform;
  architecture: Architecture;
  isARM: boolean;
  isAppleSilicon: boolean;
  displayName: string;
  archDisplayName: string;
}

/**
 * Detects the current platform from user agent or Tauri API
 */
export function detectPlatform(): Platform {
  // Try to use Tauri OS detection if available
  if (typeof window !== 'undefined' && (window as any).__TAURI__) {
    try {
      const osType = (window as any).__TAURI_METADATA__?.os;
      if (osType) {
        return normalizePlatform(osType);
      }
    } catch (e) {
      // Fall back to user agent detection
    }
  }

  // Fall back to user agent detection
  const userAgent = navigator.userAgent.toLowerCase();

  if (userAgent.includes('win')) {
    return 'windows';
  } else if (userAgent.includes('mac')) {
    return 'darwin';
  } else if (userAgent.includes('linux')) {
    return 'linux';
  }

  return 'unknown';
}

/**
 * Detects the current architecture
 */
export function detectArchitecture(): Architecture {
  // Try to use Tauri architecture detection if available
  if (typeof window !== 'undefined' && (window as any).__TAURI__) {
    try {
      const arch = (window as any).__TAURI_METADATA__?.arch;
      if (arch) {
        return normalizeArchitecture(arch);
      }
    } catch (e) {
      // Fall back to user agent detection
    }
  }

  // Check for ARM indicators in user agent
  const userAgent = navigator.userAgent.toLowerCase();

  if (userAgent.includes('arm64') || userAgent.includes('aarch64')) {
    return 'arm64';
  } else if (userAgent.includes('arm')) {
    return 'arm';
  }

  // Check for Apple Silicon
  if (userAgent.includes('mac') && (userAgent.includes('m1') || userAgent.includes('m2') || userAgent.includes('m3'))) {
    return 'arm64';
  }

  // Default to x64 for most desktop platforms
  if (navigator.platform) {
    const platform = navigator.platform.toLowerCase();
    if (platform.includes('64') || platform.includes('x64') || platform.includes('amd64')) {
      return 'x64';
    } else if (platform.includes('86')) {
      return 'x86';
    }
  }

  return 'x64'; // Default assumption for modern systems
}

/**
 * Normalizes platform name from various sources
 */
export function normalizePlatform(platform: string): Platform {
  const normalized = platform.toLowerCase();

  if (normalized.includes('win')) {
    return 'windows';
  } else if (normalized.includes('darwin') || normalized.includes('mac') || normalized.includes('osx')) {
    return 'darwin';
  } else if (normalized.includes('linux')) {
    return 'linux';
  }

  return 'unknown';
}

/**
 * Normalizes architecture name from various sources
 */
export function normalizeArchitecture(arch: string): Architecture {
  const normalized = arch.toLowerCase();

  if (normalized.includes('arm64') || normalized.includes('aarch64')) {
    return 'arm64';
  } else if (normalized.includes('arm')) {
    return 'arm';
  } else if (normalized.includes('x64') || normalized.includes('amd64') || normalized.includes('x86_64')) {
    return 'x64';
  } else if (normalized.includes('x86') || normalized.includes('i386') || normalized.includes('i686')) {
    return 'x86';
  }

  return 'unknown';
}

/**
 * Checks if the current system is ARM-based
 */
export function isARM(): boolean {
  const arch = detectArchitecture();
  return arch === 'arm64' || arch === 'arm';
}

/**
 * Checks if the current system is Apple Silicon (M1/M2/M3)
 */
export function isAppleSilicon(): boolean {
  const platform = detectPlatform();
  const arch = detectArchitecture();
  return platform === 'darwin' && arch === 'arm64';
}

/**
 * Gets comprehensive platform information
 */
export function getPlatformInfo(): PlatformInfo {
  const platform = detectPlatform();
  const architecture = detectArchitecture();

  return {
    platform,
    architecture,
    isARM: isARM(),
    isAppleSilicon: isAppleSilicon(),
    displayName: getPlatformDisplayName(platform),
    archDisplayName: getArchitectureDisplayName(architecture),
  };
}

/**
 * Gets human-readable platform name
 */
export function getPlatformDisplayName(platform: Platform): string {
  switch (platform) {
    case 'windows':
      return 'Windows';
    case 'darwin':
      return 'macOS';
    case 'linux':
      return 'Linux';
    default:
      return 'Unknown Platform';
  }
}

/**
 * Gets human-readable architecture name
 */
export function getArchitectureDisplayName(architecture: Architecture): string {
  switch (architecture) {
    case 'x64':
      return 'x86_64 (64-bit)';
    case 'arm64':
      return 'ARM64 (64-bit)';
    case 'arm':
      return 'ARM (32-bit)';
    case 'x86':
      return 'x86 (32-bit)';
    default:
      return 'Unknown Architecture';
  }
}

/**
 * Gets platform-specific icon/emoji
 */
export function getPlatformIcon(platform: Platform): string {
  switch (platform) {
    case 'windows':
      return '🪟';
    case 'darwin':
      return '🍎';
    case 'linux':
      return '🐧';
    default:
      return '💻';
  }
}

/**
 * Gets architecture-specific icon/emoji
 */
export function getArchitectureIcon(architecture: Architecture): string {
  switch (architecture) {
    case 'arm64':
    case 'arm':
      return '🦾';
    case 'x64':
    case 'x86':
      return '⚡';
    default:
      return '🔧';
  }
}

/**
 * Formats device path based on platform conventions
 */
export function formatDevicePath(path: string, platform?: Platform): string {
  const currentPlatform = platform || detectPlatform();

  // Don't modify if path is already formatted or empty
  if (!path || path.startsWith('http')) {
    return path;
  }

  switch (currentPlatform) {
    case 'windows':
      // Windows paths use backslashes
      return path.replace(/\//g, '\\');
    case 'darwin':
    case 'linux':
      // Unix-like systems use forward slashes
      return path.replace(/\\/g, '/');
    default:
      return path;
  }
}

/**
 * Gets platform-specific device path examples
 */
export function getDevicePathExample(deviceType: string, platform?: Platform): string {
  const currentPlatform = platform || detectPlatform();

  switch (currentPlatform) {
    case 'linux':
      switch (deviceType) {
        case 'gpu':
          return '/dev/dri/renderD128';
        case 'camera':
          return '/dev/video0';
        case 'tpu':
          return '/dev/apex_0';
        default:
          return '/dev/device';
      }
    case 'darwin':
      switch (deviceType) {
        case 'gpu':
          return '/dev/gpu';
        case 'camera':
          return '/dev/video0';
        default:
          return '/dev/device';
      }
    case 'windows':
      switch (deviceType) {
        case 'gpu':
          return '\\\\.\\GPU0';
        case 'camera':
          return '\\\\.\\VIDEO0';
        default:
          return '\\\\.\\DEVICE';
      }
    default:
      return '/dev/device';
  }
}

/**
 * Checks if a device path is valid for the current platform
 */
export function isValidDevicePath(path: string, platform?: Platform): boolean {
  const currentPlatform = platform || detectPlatform();

  if (!path || path.trim() === '') {
    return false;
  }

  switch (currentPlatform) {
    case 'linux':
    case 'darwin':
      // Unix-like paths should start with /
      return path.startsWith('/dev/') || path.startsWith('/sys/');
    case 'windows':
      // Windows device paths often use \\.\\ notation
      return path.startsWith('\\\\.\\') || path.includes(':');
    default:
      return true; // Unknown platform - be permissive
  }
}

/**
 * Gets platform-specific performance hints
 */
export function getPerformanceHints(platform: Platform, architecture: Architecture): string[] {
  const hints: string[] = [];

  if (architecture === 'arm64') {
    if (platform === 'darwin') {
      hints.push('Apple Silicon detected - VideoToolbox acceleration available');
      hints.push('Metal GPU acceleration available');
      hints.push('Neural Engine available for AI tasks');
    } else if (platform === 'linux') {
      hints.push('ARM64 detected - ensure hardware acceleration drivers are installed');
      hints.push('Consider using Mali or Adreno GPU acceleration if available');
    }
  } else if (architecture === 'x64') {
    if (platform === 'linux') {
      hints.push('x86_64 detected - VAAPI/QSV acceleration may be available');
      hints.push('Check for NVIDIA/AMD GPU drivers');
    } else if (platform === 'windows') {
      hints.push('Windows x64 - DirectX acceleration available');
      hints.push('Check for NVIDIA NVENC or AMD VCE');
    } else if (platform === 'darwin') {
      hints.push('Intel Mac - VideoToolbox and Metal available');
    }
  }

  return hints;
}

export default {
  detectPlatform,
  detectArchitecture,
  normalizePlatform,
  normalizeArchitecture,
  isARM,
  isAppleSilicon,
  getPlatformInfo,
  getPlatformDisplayName,
  getArchitectureDisplayName,
  getPlatformIcon,
  getArchitectureIcon,
  formatDevicePath,
  getDevicePathExample,
  isValidDevicePath,
  getPerformanceHints,
};
