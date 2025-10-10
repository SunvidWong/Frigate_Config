// T082: Manual Configuration Page
// Allows users to edit YAML configuration directly with conflict detection and rollback support

import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open, save } from '@tauri-apps/api/dialog';
import { readTextFile } from '@tauri-apps/api/fs';
import YamlEditor from '../components/YamlEditor';
import ConflictDialog from '../components/ConflictDialog';
import BackupList from '../components/BackupList';
import {
  MergeResult,
  ConflictResolution,
  Snapshot
} from '../types/configuration';
import '../styles/manual-config.css';

interface ManualConfigProps {
  className?: string;
}

const ManualConfig: React.FC<ManualConfigProps> = ({ className = '' }) => {
  // State management
  const [yamlContent, setYamlContent] = useState<string>('');
  const [originalContent, setOriginalContent] = useState<string>('');
  const [currentFilePath, setCurrentFilePath] = useState<string>('');
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [validationErrors, setValidationErrors] = useState<any[]>([]);
  const [validationWarnings, setValidationWarnings] = useState<any[]>([]);

  // Conflict resolution state
  const [showConflictDialog, setShowConflictDialog] = useState<boolean>(false);
  const [mergeResult, setMergeResult] = useState<MergeResult | null>(null);
  const [pendingMerge, setPendingMerge] = useState<{
    uiConfig: string;
    manualConfig: string;
  } | null>(null);

  // Backup/snapshot state
  const [showBackupList, setShowBackupList] = useState<boolean>(false);
  const [snapshots, setSnapshots] = useState<Snapshot[]>([]);

  // Track changes
  useEffect(() => {
    const hasChanges = yamlContent !== originalContent;
    setHasUnsavedChanges(hasChanges);
  }, [yamlContent, originalContent]);

  // Warn on navigation with unsaved changes
  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (hasUnsavedChanges) {
        e.preventDefault();
        e.returnValue = '';
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => window.removeEventListener('beforeunload', handleBeforeUnload);
  }, [hasUnsavedChanges]);

  // Load configuration from file
  const handleLoadConfig = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'YAML',
          extensions: ['yml', 'yaml']
        }]
      });

      if (selected && typeof selected === 'string') {
        setIsLoading(true);
        setError(null);

        const content = await readTextFile(selected);
        setYamlContent(content);
        setOriginalContent(content);
        setCurrentFilePath(selected);

        // Validate loaded config
        await validateConfig(content, selected);

        setIsLoading(false);
      }
    } catch (err: any) {
      setError(`Failed to load configuration: ${err.message}`);
      setIsLoading(false);
    }
  }, []);

  // Save configuration to file
  const handleSaveConfig = useCallback(async () => {
    if (!currentFilePath) {
      // Prompt for save location
      const savePath = await save({
        filters: [{
          name: 'YAML',
          extensions: ['yml', 'yaml']
        }],
        defaultPath: 'frigate.yml'
      });

      if (savePath) {
        setCurrentFilePath(savePath);
      } else {
        return;
      }
    }

    try {
      setIsLoading(true);
      setError(null);

      // Validate before saving
      const validation = await validateConfig(yamlContent, currentFilePath);

      if (!validation.valid) {
        setError('Configuration has validation errors. Please fix them before saving.');
        setIsLoading(false);
        return;
      }

      // Save via Tauri command (includes automatic snapshot creation)
      await invoke('save_config', {
        configRequest: {
          file_path: currentFilePath,
          content: yamlContent,
          create_snapshot: true,
          snapshot_description: `Manual save at ${new Date().toLocaleString()}`
        }
      });

      setOriginalContent(yamlContent);
      setHasUnsavedChanges(false);
      setIsLoading(false);

      // Show success message
      console.log('Configuration saved successfully');
    } catch (err: any) {
      setError(`Failed to save configuration: ${err.message}`);
      setIsLoading(false);
    }
  }, [yamlContent, currentFilePath]);

  // Validate configuration
  const validateConfig = useCallback(async (content: string, filePath: string) => {
    try {
      const result = await invoke<any>('validate_config', {
        content,
        filePath
      });

      setValidationErrors(result.errors || []);
      setValidationWarnings(result.warnings || []);

      return result;
    } catch (err: any) {
      console.error('Validation error:', err);
      return { valid: false, errors: [{ message: err.message }], warnings: [] };
    }
  }, []);

  // Apply template (triggers merge and conflict detection)
  // TODO: Wire up to template selection UI
  // @ts-expect-error - Function reserved for future template selection UI
  const handleApplyTemplate = useCallback(async (templateContent: string) => {
    if (!yamlContent) {
      setError('Please load a configuration first');
      return;
    }

    try {
      setIsLoading(true);
      setError(null);

      // Merge configurations via Tauri command
      const result = await invoke<any>('merge_configurations', {
        mergeRequest: {
          ui_config: {
            file_path: 'template.yml',
            content: templateContent
          },
          manual_config: {
            file_path: currentFilePath || 'current.yml',
            content: yamlContent
          }
        }
      });

      setMergeResult(result);

      // Check for conflicts
      if (result.conflicts && result.conflicts.length > 0) {
        // Show conflict resolution dialog
        setPendingMerge({
          uiConfig: templateContent,
          manualConfig: yamlContent
        });
        setShowConflictDialog(true);
      } else {
        // No conflicts - apply merged config
        setYamlContent(result.merged_config.content);
      }

      setIsLoading(false);
    } catch (err: any) {
      setError(`Failed to merge configurations: ${err.message}`);
      setIsLoading(false);
    }
  }, [yamlContent, currentFilePath]);

  // Resolve conflicts
  const handleResolveConflicts = useCallback(async (resolutions: ConflictResolution[]) => {
    if (!pendingMerge) return;

    try {
      setIsLoading(true);
      setError(null);

      const result = await invoke<any>('resolve_conflicts', {
        resolveRequest: {
          ui_config: {
            file_path: 'template.yml',
            content: pendingMerge.uiConfig
          },
          manual_config: {
            file_path: currentFilePath || 'current.yml',
            content: pendingMerge.manualConfig
          },
          resolutions: resolutions.map(r => ({
            path: r.path,
            resolution_type: r.resolutionType,
            description: r.description,
            suggested_value: r.suggestedValue ? JSON.stringify(r.suggestedValue) : null,
            auto_applicable: r.autoApplicable
          }))
        }
      });

      // Apply resolved configuration
      setYamlContent(result.resolved_config.content);
      setShowConflictDialog(false);
      setPendingMerge(null);
      setMergeResult(null);
      setIsLoading(false);
    } catch (err: any) {
      setError(`Failed to resolve conflicts: ${err.message}`);
      setIsLoading(false);
    }
  }, [pendingMerge, currentFilePath]);

  // Load snapshots
  const loadSnapshots = useCallback(async () => {
    try {
      const result = await invoke<any>('list_snapshots', {});
      setSnapshots(result.snapshots || []);
    } catch (err: any) {
      console.error('Failed to load snapshots:', err);
    }
  }, []);

  // Restore from snapshot (rollback)
  const handleRestoreSnapshot = useCallback(async (snapshotId: string) => {
    try {
      setIsLoading(true);
      setError(null);

      await invoke<any>('restore_from_snapshot', {
        restoreRequest: {
          snapshot_id: snapshotId,
          target_path: currentFilePath || 'frigate.yml',
          validate: true,
          backup_before_restore: true,
          reason: `Rollback to snapshot ${snapshotId}`,
          merge_changes: false
        }
      });

      // Reload configuration after restore
      if (currentFilePath) {
        const content = await readTextFile(currentFilePath);
        setYamlContent(content);
        setOriginalContent(content);
      }

      setShowBackupList(false);
      setIsLoading(false);

      console.log('Configuration restored successfully');
    } catch (err: any) {
      setError(`Failed to restore snapshot: ${err.message}`);
      setIsLoading(false);
    }
  }, [currentFilePath]);

  // Load snapshots when backup list is opened
  useEffect(() => {
    if (showBackupList) {
      loadSnapshots();
    }
  }, [showBackupList, loadSnapshots]);

  return (
    <div className={`manual-config-page ${className}`}>
      {/* Header */}
      <div className="page-header">
        <h1>Manual Configuration</h1>
        <p className="subtitle">
          Edit your Frigate configuration directly with syntax highlighting and validation
        </p>
      </div>

      {/* Toolbar */}
      <div className="toolbar">
        <div className="toolbar-left">
          <button
            onClick={handleLoadConfig}
            disabled={isLoading}
            className="btn btn-primary"
          >
            Load Configuration
          </button>

          <button
            onClick={handleSaveConfig}
            disabled={isLoading || !yamlContent || validationErrors.length > 0}
            className="btn btn-success"
          >
            Save Configuration
          </button>

          {hasUnsavedChanges && (
            <span className="unsaved-indicator">● Unsaved changes</span>
          )}
        </div>

        <div className="toolbar-right">
          <button
            onClick={() => setShowBackupList(true)}
            disabled={isLoading}
            className="btn btn-secondary"
          >
            View Backups
          </button>
        </div>
      </div>

      {/* Current file info */}
      {currentFilePath && (
        <div className="file-info">
          <span className="label">Current file:</span>
          <span className="path">{currentFilePath}</span>
        </div>
      )}

      {/* Error display */}
      {error && (
        <div className="alert alert-error">
          <strong>Error:</strong> {error}
        </div>
      )}

      {/* Validation warnings */}
      {validationWarnings.length > 0 && (
        <div className="alert alert-warning">
          <strong>Warnings:</strong>
          <ul>
            {validationWarnings.map((warning, idx) => (
              <li key={idx}>
                {warning.line_number && `Line ${warning.line_number}: `}
                {warning.message}
                {warning.suggestion && ` (${warning.suggestion})`}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Validation errors */}
      {validationErrors.length > 0 && (
        <div className="alert alert-error">
          <strong>Validation Errors:</strong>
          <ul>
            {validationErrors.map((error, idx) => (
              <li key={idx}>
                {error.line_number && `Line ${error.line_number}: `}
                {error.message}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* YAML Editor */}
      <div className="editor-container">
        <YamlEditor
          content={yamlContent}
          onChange={setYamlContent}
          readOnly={isLoading}
          minHeight="calc(100vh - 300px)"
        />
      </div>

      {/* Conflict Resolution Dialog */}
      {showConflictDialog && mergeResult && (
        <ConflictDialog
          conflicts={mergeResult.conflicts}
          preservedEdits={mergeResult.preserved_edits}
          statistics={mergeResult.statistics}
          onResolve={handleResolveConflicts}
          onCancel={() => {
            setShowConflictDialog(false);
            setPendingMerge(null);
            setMergeResult(null);
          }}
        />
      )}

      {/* Backup List Dialog */}
      {showBackupList && (
        <BackupList
          snapshots={snapshots}
          onRestore={handleRestoreSnapshot}
          onClose={() => setShowBackupList(false)}
          isLoading={isLoading}
        />
      )}

      {/* Loading overlay */}
      {isLoading && (
        <div className="loading-overlay">
          <div className="spinner"></div>
          <p>Processing...</p>
        </div>
      )}
    </div>
  );
};

export default ManualConfig;
