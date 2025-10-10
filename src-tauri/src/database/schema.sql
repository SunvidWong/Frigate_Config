-- SQLite Database Schema for Frigate Configuration Tool
-- Purpose: Store backup history, deployment states, and metadata
-- Version: 1.0.0

-- Configuration snapshots (backups)
CREATE TABLE IF NOT EXISTS config_snapshots (
    id TEXT PRIMARY KEY,
    version INTEGER NOT NULL UNIQUE,
    yaml_content TEXT NOT NULL,
    checksum TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT NOT NULL,  -- "ui", "template", "manual_edit", "rollback"
    description TEXT,
    is_backup BOOLEAN NOT NULL DEFAULT 1,
    backup_reason TEXT,  -- "pre_deploy", "pre_merge", "manual"
    deployed BOOLEAN NOT NULL DEFAULT 0,
    deployed_at TEXT,
    deployment_success BOOLEAN
);

CREATE INDEX IF NOT EXISTS idx_snapshots_version ON config_snapshots(version DESC);
CREATE INDEX IF NOT EXISTS idx_snapshots_created_at ON config_snapshots(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_snapshots_deployed ON config_snapshots(deployed);

-- Deployment states
CREATE TABLE IF NOT EXISTS deployment_states (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    deployment_method TEXT NOT NULL,  -- "docker_run", "docker_compose"
    container_id TEXT,
    command TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    duration_ms INTEGER,
    status TEXT NOT NULL,  -- "pending", "running", "success", "failed", "rolled_back"
    exit_code INTEGER,
    health_check_status TEXT,  -- "healthy", "unhealthy", "pending"
    health_check_attempts INTEGER DEFAULT 0,
    health_check_last_attempt TEXT,
    stdout_log TEXT,
    stderr_log TEXT,
    previous_state_id TEXT,
    rolled_back BOOLEAN DEFAULT 0,
    rollback_reason TEXT,
    deployed_by TEXT NOT NULL,
    deployment_notes TEXT,
    FOREIGN KEY (snapshot_id) REFERENCES config_snapshots(id),
    FOREIGN KEY (previous_state_id) REFERENCES deployment_states(id)
);

CREATE INDEX IF NOT EXISTS idx_deployments_snapshot ON deployment_states(snapshot_id);
CREATE INDEX IF NOT EXISTS idx_deployments_started_at ON deployment_states(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_deployments_status ON deployment_states(status);

-- Volume mappings
CREATE TABLE IF NOT EXISTS volume_mappings (
    id TEXT PRIMARY KEY,
    deployment_state_id TEXT NOT NULL,
    host_path TEXT NOT NULL,
    container_path TEXT NOT NULL,
    mapping_type TEXT NOT NULL,  -- "recordings", "clips", "cache", "config", "custom"
    read_only BOOLEAN DEFAULT 0,
    disk_total_bytes INTEGER,
    disk_used_bytes INTEGER,
    disk_free_bytes INTEGER,
    disk_info_updated_at TEXT,
    path_exists BOOLEAN NOT NULL,
    path_writable BOOLEAN NOT NULL,
    validation_error TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (deployment_state_id) REFERENCES deployment_states(id)
);

CREATE INDEX IF NOT EXISTS idx_volumes_deployment ON volume_mappings(deployment_state_id);
CREATE INDEX IF NOT EXISTS idx_volumes_mapping_type ON volume_mappings(mapping_type);

-- Audit log
CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,  -- "config_change", "deployment", "rollback", "conflict_resolved", "hardware_detected"
    event_action TEXT NOT NULL,
    user_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    related_snapshot_id TEXT,
    related_deployment_id TEXT,
    related_conflict_id TEXT,
    before_state TEXT,  -- JSON blob
    after_state TEXT,   -- JSON blob
    diff TEXT,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    client_info TEXT,
    FOREIGN KEY (related_snapshot_id) REFERENCES config_snapshots(id),
    FOREIGN KEY (related_deployment_id) REFERENCES deployment_states(id)
);

CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_event_type ON audit_log(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_success ON audit_log(success);

-- Application metadata
CREATE TABLE IF NOT EXISTS app_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Insert initial metadata
INSERT OR IGNORE INTO app_metadata (key, value, updated_at) VALUES
    ('schema_version', '1', datetime('now')),
    ('app_version', '0.1.0', datetime('now')),
    ('first_run_at', datetime('now'), datetime('now'));
