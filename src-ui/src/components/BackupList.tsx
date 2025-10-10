// T085: Backup List Component
// Display and manage configuration snapshots

import React, { useState } from 'react';
import Modal from './Modal';
import Button from './Button';

interface Snapshot {
  id: string;
  version: number;
  timestamp: string;
  created_at: string;
  trigger: string;
  description?: string;
  file_path: string;
  file_size: number;
  checksum: string;
  metadata?: {
    frigate_version?: string;
    camera_count?: number;
    detector_type?: string;
  };
}

interface BackupListProps {
  snapshots: Snapshot[];
  onRestore: (snapshotId: string) => void;
  onClose: () => void;
  isLoading: boolean;
}

const BackupList: React.FC<BackupListProps> = ({
  snapshots,
  onRestore,
  onClose,
  isLoading
}) => {
  const [selectedSnapshot, setSelectedSnapshot] = useState<string | null>(null);
  const [showConfirmDialog, setShowConfirmDialog] = useState<boolean>(false);
  const [sortBy, setSortBy] = useState<'date' | 'version'>('date');
  const [filterTrigger, setFilterTrigger] = useState<string>('all');

  // Format timestamp for display
  const formatTimestamp = (timestamp: string): string => {
    try {
      const date = new Date(timestamp);
      return date.toLocaleString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return timestamp;
    }
  };

  // Format file size
  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  // Get trigger badge color
  const getTriggerColor = (trigger: string): string => {
    switch (trigger.toLowerCase()) {
      case 'manual':
        return 'bg-blue-100 text-blue-800';
      case 'pre_deploy':
        return 'bg-green-100 text-green-800';
      case 'scheduled':
        return 'bg-purple-100 text-purple-800';
      case 'auto':
        return 'bg-gray-100 text-gray-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  // Sort snapshots
  const sortedSnapshots = [...snapshots].sort((a, b) => {
    if (sortBy === 'date') {
      return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
    } else {
      return b.version - a.version;
    }
  });

  // Filter snapshots
  const filteredSnapshots = sortedSnapshots.filter(snapshot => {
    if (filterTrigger === 'all') return true;
    return snapshot.trigger.toLowerCase() === filterTrigger.toLowerCase();
  });

  // Get unique triggers for filter
  const uniqueTriggers = ['all', ...new Set(snapshots.map(s => s.trigger.toLowerCase()))];

  // Handle restore confirmation
  const handleRestoreClick = (snapshotId: string) => {
    setSelectedSnapshot(snapshotId);
    setShowConfirmDialog(true);
  };

  const handleConfirmRestore = () => {
    if (selectedSnapshot) {
      onRestore(selectedSnapshot);
      setShowConfirmDialog(false);
      setSelectedSnapshot(null);
    }
  };

  const handleCancelRestore = () => {
    setShowConfirmDialog(false);
    setSelectedSnapshot(null);
  };

  return (
    <Modal
      isOpen={true}
      onClose={onClose}
      title="Configuration Backups"
      size="xl"
      data-testid="backup-list-dialog"
    >
      <div className="backup-list">
        {/* Toolbar */}
        <div className="backup-toolbar">
          <div className="toolbar-left">
            <label className="filter-label">
              Filter by trigger:
              <select
                value={filterTrigger}
                onChange={(e) => setFilterTrigger(e.target.value)}
                className="filter-select"
                data-testid="backup-filter-trigger"
              >
                {uniqueTriggers.map(trigger => (
                  <option key={trigger} value={trigger}>
                    {trigger.charAt(0).toUpperCase() + trigger.slice(1)}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <div className="toolbar-right">
            <label className="sort-label">
              Sort by:
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as 'date' | 'version')}
                className="sort-select"
                data-testid="backup-sort-by"
              >
                <option value="date">Date</option>
                <option value="version">Version</option>
              </select>
            </label>
          </div>
        </div>

        {/* Snapshots List */}
        <div className="snapshots-container">
          {filteredSnapshots.length === 0 ? (
            <div className="no-snapshots">
              <p>No backups found</p>
            </div>
          ) : (
            <div className="snapshots-list">
              {filteredSnapshots.map((snapshot) => (
                <div
                  key={snapshot.id}
                  className="snapshot-item"
                  data-testid="snapshot-item"
                >
                  <div className="snapshot-header">
                    <div className="snapshot-title">
                      <h4>Version {snapshot.version}</h4>
                      <span className={`trigger-badge ${getTriggerColor(snapshot.trigger)}`}>
                        {snapshot.trigger}
                      </span>
                    </div>
                    <div className="snapshot-actions">
                      <Button
                        variant="primary"
                        size="sm"
                        onClick={() => handleRestoreClick(snapshot.id)}
                        disabled={isLoading}
                        data-testid="snapshot-restore-button"
                      >
                        Restore
                      </Button>
                    </div>
                  </div>

                  <div className="snapshot-details">
                    <div className="detail-row">
                      <span className="detail-label">Created:</span>
                      <span className="detail-value">
                        {formatTimestamp(snapshot.created_at || snapshot.timestamp)}
                      </span>
                    </div>

                    {snapshot.description && (
                      <div className="detail-row">
                        <span className="detail-label">Description:</span>
                        <span className="detail-value">{snapshot.description}</span>
                      </div>
                    )}

                    <div className="detail-row">
                      <span className="detail-label">File Size:</span>
                      <span className="detail-value">{formatFileSize(snapshot.file_size)}</span>
                    </div>

                    <div className="detail-row">
                      <span className="detail-label">Checksum:</span>
                      <span className="detail-value checksum" title={snapshot.checksum}>
                        {snapshot.checksum.substring(0, 16)}...
                      </span>
                    </div>

                    {snapshot.metadata && (
                      <div className="snapshot-metadata">
                        {snapshot.metadata.frigate_version && (
                          <div className="metadata-item">
                            <span className="metadata-label">Frigate Version:</span>
                            <span className="metadata-value">
                              {snapshot.metadata.frigate_version}
                            </span>
                          </div>
                        )}
                        {snapshot.metadata.camera_count !== undefined && (
                          <div className="metadata-item">
                            <span className="metadata-label">Cameras:</span>
                            <span className="metadata-value">
                              {snapshot.metadata.camera_count}
                            </span>
                          </div>
                        )}
                        {snapshot.metadata.detector_type && (
                          <div className="metadata-item">
                            <span className="metadata-label">Detector:</span>
                            <span className="metadata-value">
                              {snapshot.metadata.detector_type}
                            </span>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Summary */}
        <div className="backup-summary">
          <p>
            Showing {filteredSnapshots.length} of {snapshots.length} backups
          </p>
        </div>

        {/* Close Button */}
        <div className="dialog-actions">
          <Button variant="secondary" onClick={onClose}>
            Close
          </Button>
        </div>
      </div>

      {/* Restore Confirmation Dialog */}
      {showConfirmDialog && (
        <Modal
          isOpen={true}
          onClose={handleCancelRestore}
          title="Confirm Restore"
          size="md"
          data-testid="restore-confirm-dialog"
        >
          <div className="confirm-dialog">
            <p className="confirm-message">
              Are you sure you want to restore this backup? Your current configuration will be
              saved as a new backup before restoring.
            </p>

            <div className="confirm-details">
              {selectedSnapshot && (
                <>
                  <p>
                    <strong>Version:</strong>{' '}
                    {snapshots.find(s => s.id === selectedSnapshot)?.version}
                  </p>
                  <p>
                    <strong>Created:</strong>{' '}
                    {formatTimestamp(
                      snapshots.find(s => s.id === selectedSnapshot)?.created_at || ''
                    )}
                  </p>
                </>
              )}
            </div>

            <div className="dialog-actions">
              <Button variant="secondary" onClick={handleCancelRestore}>
                Cancel
              </Button>
              <Button variant="primary" onClick={handleConfirmRestore} disabled={isLoading}>
                Restore Backup
              </Button>
            </div>
          </div>
        </Modal>
      )}
    </Modal>
  );
};

export default BackupList;
