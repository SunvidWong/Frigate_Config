// Configuration Auto-Fixer
// Automatically fixes common Frigate configuration issues

export interface FixSuggestion {
  issue: string;
  fix: string;
  description: string;
  severity: 'error' | 'warning';
}

export class ConfigFixer {
  /**
   * Analyze YAML and suggest fixes
   */
  static analyzConfig(yamlContent: string): FixSuggestion[] {
    const suggestions: FixSuggestion[] = [];

    try {
      // Parse YAML to check structure
      const lines = yamlContent.split('\n');
      let inCameras = false;
      let currentCamera = '';
      let hasDetectors = false;
      let hasObjects = false;
      let cameraHasDetector = false;

      lines.forEach((line) => {
        const trimmed = line.trim();

        // Check for detectors section
        if (trimmed.startsWith('detectors:')) {
          hasDetectors = true;
        }

        // Track camera sections
        if (trimmed.startsWith('cameras:')) {
          inCameras = true;
        } else if (inCameras && trimmed && !trimmed.startsWith('#') && !line.startsWith(' ')) {
          inCameras = false;
        }

        // Track individual cameras
        if (inCameras && line.startsWith('  ') && !line.startsWith('    ') && trimmed.endsWith(':')) {
          // Save previous camera check before starting new one
          if (currentCamera && !cameraHasDetector) {
            suggestions.push({
              issue: `摄像头 "${currentCamera}" 未指定检测器`,
              fix: '为摄像头指定检测器',
              description: '每个摄像头必须指定使用哪个检测器。建议添加 "detector: cpu"。',
              severity: 'error'
            });
          }
          if (currentCamera && !hasObjects) {
            suggestions.push({
              issue: `摄像头 "${currentCamera}" 未配置追踪对象`,
              fix: '添加默认追踪对象',
              description: '建议添加 objects/track 配置来指定要检测的物体类型(如 person)。',
              severity: 'warning'
            });
          }

          // Start tracking new camera
          currentCamera = trimmed.replace(':', '');
          cameraHasDetector = false;
          hasObjects = false;
        }

        // Check for detector assignment in camera (matches both "detector:" and "detector: cpu")
        if (currentCamera && trimmed.startsWith('detector:')) {
          cameraHasDetector = true;
        }

        // Check for objects/track configuration
        if (currentCamera && (trimmed.startsWith('objects:') || trimmed.startsWith('track:'))) {
          hasObjects = true;
        }
      });

      // Check last camera after loop ends
      if (currentCamera) {
        if (!cameraHasDetector) {
          suggestions.push({
            issue: `摄像头 "${currentCamera}" 未指定检测器`,
            fix: '为摄像头指定检测器',
            description: '每个摄像头必须指定使用哪个检测器。建议添加 "detector: cpu"。',
            severity: 'error'
          });
        }

        if (!hasObjects) {
          suggestions.push({
            issue: `摄像头 "${currentCamera}" 未配置追踪对象`,
            fix: '添加默认追踪对象',
            description: '建议添加 objects/track 配置来指定要检测的物体类型(如 person)。',
            severity: 'warning'
          });
        }
      }

      // Suggest fixes for global config
      if (!hasDetectors) {
        suggestions.push({
          issue: '缺少检测器配置',
          fix: '添加默认 CPU 检测器',
          description: '配置文件缺少 detectors 部分。建议添加 CPU 检测器作为默认选项。',
          severity: 'error'
        });
      }

      // Remove trailing empty line warnings - this is normal YAML formatting
      // Users shouldn't see warnings about trailing whitespace

    } catch (error) {
      suggestions.push({
        issue: 'YAML 解析错误',
        fix: '检查 YAML 语法',
        description: '配置文件存在语法错误,请检查缩进和格式。',
        severity: 'error'
      });
    }

    return suggestions;
  }

  /**
   * Automatically fix common issues
   */
  static autoFix(yamlContent: string): { fixed: string; changes: string[] } {
    const changes: string[] = [];
    let fixed = yamlContent;
    let previousContent = yamlContent;

    // Fix 1: Remove empty array items (lines with just "-")
    fixed = fixed.replace(/^(\s+)-\s*$/gm, '');
    if (fixed !== previousContent) {
      changes.push('移除了空数组项');
      previousContent = fixed;
    }

    // Fix 2: Remove duplicate global record/snapshots if cameras have their own
    const hasCameraRecord = /cameras:[\s\S]*?record:/m.test(fixed);
    const hasCameraSnapshots = /cameras:[\s\S]*?snapshots:/m.test(fixed);
    const hasGlobalRecord = /^record:/m.test(fixed);
    const hasGlobalSnapshots = /^snapshots:/m.test(fixed);

    if (hasCameraRecord && hasGlobalRecord) {
      // Remove global record section
      fixed = fixed.replace(/^record:\n(?:  .*\n)*/gm, '');
      changes.push('移除了重复的全局录制配置');
    }

    if (hasCameraSnapshots && hasGlobalSnapshots) {
      // Remove global snapshots section
      fixed = fixed.replace(/^snapshots:\n(?:  .*\n)*/gm, '');
      changes.push('移除了重复的全局快照配置');
    }

    // Fix 3: Remove trailing empty lines
    previousContent = fixed;
    fixed = fixed.replace(/\n{3,}/g, '\n\n').trim() + '\n';
    if (fixed !== previousContent) {
      changes.push('清理了多余空行');
    }

    // Fix 4: Ensure detectors section exists
    if (!fixed.includes('detectors:')) {
      const detectorsConfig = `detectors:
  cpu:
    type: cpu
    num_threads: 3

`;
      // Insert after mqtt section or at the beginning
      if (fixed.includes('mqtt:')) {
        fixed = fixed.replace(/(mqtt:\n(?:  .*\n)*)\n?/, `$1\n${detectorsConfig}`);
      } else {
        fixed = detectorsConfig + fixed;
      }
      changes.push('添加了默认 CPU 检测器配置');
    }

    // Fix cameras without detector assignment
    const lines = fixed.split('\n');
    const fixedLines: string[] = [];
    let inCamera = false;
    let hasDetectSection = false;
    let hasDetectorAssignment = false;
    let detectSectionIndex = -1;

    lines.forEach((line) => {
      fixedLines.push(line);
      const trimmed = line.trim();

      // Detect camera entry
      if (line.startsWith('  ') && !line.startsWith('    ') && trimmed.endsWith(':') &&
          !trimmed.startsWith('#') && line.indexOf(':') > 2) {
        inCamera = true;
        hasDetectSection = false;
        hasDetectorAssignment = false;
        detectSectionIndex = -1;
      }

      // Track detect section
      if (inCamera && line.startsWith('    detect:')) {
        hasDetectSection = true;
        detectSectionIndex = fixedLines.length - 1;
      }

      // Track detector assignment
      if (inCamera && hasDetectSection && line.trim().startsWith('detector:')) {
        hasDetectorAssignment = true;
      }

      // End of camera section (detect new camera at same level)
      if (inCamera && line.startsWith('  ') && !line.startsWith('    ') &&
          trimmed.endsWith(':') && fixedLines.length > 2) {
        // Add detector if missing to previous camera
        if (hasDetectSection && !hasDetectorAssignment && detectSectionIndex >= 0) {
          fixedLines.splice(detectSectionIndex + 1, 0, '      detector: cpu');
          changes.push(`为摄像头添加了检测器配置`);
        }
        // Reset for new camera
        inCamera = true;
        hasDetectSection = false;
        hasDetectorAssignment = false;
        detectSectionIndex = -1;
      }
    });

    // Check last camera
    if (inCamera && hasDetectSection && !hasDetectorAssignment && detectSectionIndex >= 0) {
      fixedLines.splice(detectSectionIndex + 1, 0, '      detector: cpu');
      changes.push(`为最后一个摄像头添加了检测器配置`);
    }

    fixed = fixedLines.join('\n');

    // Add default objects tracking if missing
    if (fixed.includes('detect:') && !fixed.includes('objects:') && !fixed.includes('track:')) {
      // Find cameras section and add objects config
      const cameraMatch = fixed.match(/cameras:\n([\s\S]*?)(?=\n\w|$)/);
      if (cameraMatch) {
        const objectsConfig = `
objects:
  track:
    - person
`;
        fixed = fixed.replace(/detect:\n.*?(?=\n\s{2,4}\w|\n\w|$)/s, (match) => {
          return match + objectsConfig;
        });
        changes.push('添加了默认物体追踪配置 (person)');
      }
    }

    return { fixed, changes };
  }
}
