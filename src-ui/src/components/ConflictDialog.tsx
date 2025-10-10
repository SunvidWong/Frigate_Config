// T084: Conflict Resolution Dialog
// Three-way merge UI for resolving configuration conflicts

import React, { useState } from 'react';
import Modal from './Modal';
import Button from './Button';

interface Conflict {
  path: string;
  conflict_type: string;
  severity: string;
  description: string;
  ui_value: any;
  manual_value: any;
  suggested_resolution?: {
    resolution_type: string;
    description: string;
    auto_applicable: boolean;
  };
}

interface PreservedEdit {
  path: string;
  reason: string;
  line_number: number;
}

interface ConflictStatistics {
  total_fields: number;
  conflicts_count: number;
  preserved_edits_count: number;
  warnings_count: number;
  auto_resolutions_count: number;
}

export interface ConflictResolution {
  path: string;
  resolutionType: string;
  description: string;
  suggestedValue?: any;
  autoApplicable: boolean;
}

interface ConflictDialogProps {
  conflicts: Conflict[];
  preservedEdits: PreservedEdit[];
  statistics: ConflictStatistics;
  onResolve: (resolutions: ConflictResolution[]) => void;
  onCancel: () => void;
}

const ConflictDialog: React.FC<ConflictDialogProps> = ({
  conflicts,
  preservedEdits,
  statistics,
  onResolve,
  onCancel
}) => {
  const [resolutions, setResolutions] = useState<Map<string, ConflictResolution>>(
    new Map()
  );
  const [customValues, setCustomValues] = useState<Map<string, string>>(new Map());

  // Initialize with suggested resolutions
  React.useEffect(() => {
    const initialResolutions = new Map<string, ConflictResolution>();

    conflicts.forEach(conflict => {
      if (conflict.suggested_resolution) {
        initialResolutions.set(conflict.path, {
          path: conflict.path,
          resolutionType: conflict.suggested_resolution.resolution_type,
          description: conflict.suggested_resolution.description,
          autoApplicable: conflict.suggested_resolution.auto_applicable
        });
      }
    });

    setResolutions(initialResolutions);
  }, [conflicts]);

  const handleResolutionChange = (
    conflictPath: string,
    resolutionType: string,
    value?: any
  ) => {
    const newResolutions = new Map(resolutions);

    newResolutions.set(conflictPath, {
      path: conflictPath,
      resolutionType,
      description: getResolutionDescription(resolutionType),
      suggestedValue: value,
      autoApplicable: true
    });

    setResolutions(newResolutions);
  };

  const handleCustomValueChange = (conflictPath: string, value: string) => {
    const newCustomValues = new Map(customValues);
    newCustomValues.set(conflictPath, value);
    setCustomValues(newCustomValues);
  };

  const getResolutionDescription = (resolutionType: string): string => {
    switch (resolutionType) {
      case 'UseUiValue':
      case 'keep_existing':
        return 'Keep existing value';
      case 'UseManualValue':
      case 'use_template':
        return 'Use template value';
      case 'custom':
        return 'Use custom value';
      default:
        return 'Manual review required';
    }
  };

  const getSeverityColor = (severity: string): string => {
    switch (severity.toLowerCase()) {
      case 'critical':
        return 'text-red-600';
      case 'warning':
        return 'text-yellow-600';
      case 'info':
        return 'text-blue-600';
      default:
        return 'text-gray-600';
    }
  };

  const handleApplyResolution = () => {
    const resolutionList: ConflictResolution[] = [];

    conflicts.forEach(conflict => {
      const resolution = resolutions.get(conflict.path);

      if (resolution) {
        // If custom resolution, use custom value
        if (resolution.resolutionType === 'custom') {
          const customValue = customValues.get(conflict.path);
          if (customValue) {
            resolutionList.push({
              ...resolution,
              suggestedValue: customValue
            });
          }
        } else {
          resolutionList.push(resolution);
        }
      }
    });

    onResolve(resolutionList);
  };

  const canApplyResolution = (): boolean => {
    // Check if all conflicts have resolutions
    for (const conflict of conflicts) {
      const resolution = resolutions.get(conflict.path);
      if (!resolution) return false;

      // If custom resolution, must have custom value
      if (resolution.resolutionType === 'custom') {
        const customValue = customValues.get(conflict.path);
        if (!customValue || customValue.trim() === '') {
          return false;
        }
      }
    }

    return true;
  };

  return (
    <Modal
      isOpen={true}
      onClose={onCancel}
      title="Resolve Configuration Conflicts"
      size="xl"
      data-testid="conflict-dialog"
    >
      <div className="conflict-dialog">
        {/* Statistics Summary */}
        <div className="conflict-summary">
          <h3>Merge Summary</h3>
          <div className="stats-grid">
            <div className="stat-item">
              <span className="stat-label">Total Fields:</span>
              <span className="stat-value">{statistics.total_fields}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Conflicts:</span>
              <span className="stat-value text-red-600">{statistics.conflicts_count}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Preserved Edits:</span>
              <span className="stat-value text-green-600">{statistics.preserved_edits_count}</span>
            </div>
          </div>
        </div>

        {/* Conflicts List */}
        <div className="conflicts-list">
          <h3>Conflicts Requiring Resolution</h3>

          {conflicts.map((conflict) => (
            <div
              key={conflict.path}
              className="conflict-item"
              data-testid="conflict-item"
            >
              <div className="conflict-header">
                <span className="conflict-path">{conflict.path}</span>
                <span className={`conflict-severity ${getSeverityColor(conflict.severity)}`} data-testid="conflict-severity">
                  {conflict.severity}
                </span>
              </div>

              <p className="conflict-description">{conflict.description}</p>

              {/* Value Comparison */}
              <div className="value-comparison">
                <div className="value-column">
                  <h4>Existing Value</h4>
                  <pre className="value-display">
                    {JSON.stringify(conflict.manual_value, null, 2)}
                  </pre>
                </div>

                <div className="value-column">
                  <h4>Template Value</h4>
                  <pre className="value-display">
                    {JSON.stringify(conflict.ui_value, null, 2)}
                  </pre>
                </div>
              </div>

              {/* Resolution Options */}
              <div className="resolution-options">
                <h4>Resolution:</h4>

                <div className="radio-group">
                  <label className="radio-option">
                    <input
                      type="radio"
                      name={`resolution-${conflict.path}`}
                      checked={resolutions.get(conflict.path)?.resolutionType === 'keep_existing'}
                      onChange={() => handleResolutionChange(conflict.path, 'keep_existing', conflict.manual_value)}
                      data-testid="conflict-resolution-keep"
                    />
                    <span>Keep Existing Value</span>
                  </label>

                  <label className="radio-option">
                    <input
                      type="radio"
                      name={`resolution-${conflict.path}`}
                      checked={resolutions.get(conflict.path)?.resolutionType === 'use_template'}
                      onChange={() => handleResolutionChange(conflict.path, 'use_template', conflict.ui_value)}
                      data-testid="conflict-resolution-template"
                    />
                    <span>Use Template Value</span>
                  </label>

                  <label className="radio-option">
                    <input
                      type="radio"
                      name={`resolution-${conflict.path}`}
                      checked={resolutions.get(conflict.path)?.resolutionType === 'custom'}
                      onChange={() => handleResolutionChange(conflict.path, 'custom')}
                      data-testid="conflict-resolution-custom"
                    />
                    <span>Use Custom Value</span>
                  </label>
                </div>

                {/* Custom Value Input */}
                {resolutions.get(conflict.path)?.resolutionType === 'custom' && (
                  <div className="custom-value-input">
                    <input
                      type="text"
                      placeholder="Enter custom value"
                      value={customValues.get(conflict.path) || ''}
                      onChange={(e) => handleCustomValueChange(conflict.path, e.target.value)}
                      data-testid="custom-value-input"
                    />
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>

        {/* Preserved Edits Info */}
        {preservedEdits.length > 0 && (
          <div className="preserved-edits">
            <h3>Preserved Manual Edits</h3>
            <ul>
              {preservedEdits.map((edit, index) => (
                <li key={index}>
                  <strong>{edit.path}:</strong> {edit.reason}
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Actions */}
        <div className="dialog-actions">
          <Button variant="secondary" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            variant="primary"
            onClick={handleApplyResolution}
            disabled={!canApplyResolution()}
          >
            Apply Resolution
          </Button>
        </div>
      </div>
    </Modal>
  );
};

export default ConflictDialog;
