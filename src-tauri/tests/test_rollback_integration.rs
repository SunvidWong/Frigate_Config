// T058: Integration tests for rollback functionality
// Tests for backup.rs restore and rollback operations

use frigate_config_tool::config_engine::backup::{SnapshotManager, SnapshotOptions, RestoreOptions};
use frigate_config_tool::config_engine::parser::ConfigParser;
use frigate_config_tool::models::configuration_snapshot::CreationSource;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_restore_snapshot_replace_mode() {
    // Test restoring a snapshot in replace mode (overwrites current config)
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();

    let yaml = r#"
version: "0.13"

cameras:
  test_cam:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let config = parser.parse_content(yaml.to_string(), "test.yaml".to_string()).unwrap().config;

    // Create snapshot
    let options = SnapshotOptions {
        description: Some("Restore test".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    let snapshot = manager.create_snapshot(&config, options).await.unwrap();

    // Create target file
    let target_path = temp_dir.path().join("restored_config.yaml");

    // Restore snapshot
    let restore_options = RestoreOptions {
        validate: true,
        backup_before_restore: false,
        reason: Some("Test restore".to_string()),
        merge_changes: false,
    };

    let result = manager.restore_snapshot(&snapshot.id, &target_path, restore_options).await;

    assert!(result.is_ok(), "Should restore snapshot successfully");
    let result = result.unwrap();

    assert!(result.success, "Restore should be successful");
    assert!(target_path.exists(), "Target file should exist");

    // Verify content
    let restored_content = tokio::fs::read_to_string(&target_path).await.unwrap();
    assert!(restored_content.contains("test_cam"));
    assert!(restored_content.contains("yolov8n"));
}

#[tokio::test]
async fn test_restore_with_validation() {
    // Test that restore validates snapshot before restoring
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();

    let yaml = r#"
version: "0.13"

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://192.168.1.100:554/stream
          roles: [detect, record]
    detect:
      enabled: true
      fps: 5

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let config = parser.parse_content(yaml.to_string(), "test.yaml".to_string()).unwrap().config;

    let options = SnapshotOptions {
        description: Some("Validation test".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    let snapshot = manager.create_snapshot(&config, options).await.unwrap();

    let target_path = temp_dir.path().join("config.yaml");

    // Restore with validation enabled
    let restore_options = RestoreOptions {
        validate: true,
        backup_before_restore: false,
        reason: None,
        merge_changes: false,
    };

    let result = manager.restore_snapshot(&snapshot.id, &target_path, restore_options).await;

    assert!(result.is_ok(), "Should restore with validation");
    assert!(target_path.exists());
}

#[tokio::test]
async fn test_restore_with_backup_before_restore() {
    // Test that restore creates a backup of current config before restoring
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();

    // Create initial config file
    let target_path = temp_dir.path().join("config.yaml");
    let initial_content = r#"
version: "0.13"

cameras:
  old_camera:
    ffmpeg:
      inputs:
        - path: rtsp://old
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;
    tokio::fs::write(&target_path, initial_content).await.unwrap();

    // Create a snapshot to restore
    let new_yaml = r#"
version: "0.13"

cameras:
  new_camera:
    ffmpeg:
      inputs:
        - path: rtsp://new
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let config = parser.parse_content(new_yaml.to_string(), "test.yaml".to_string()).unwrap().config;

    let options = SnapshotOptions {
        description: Some("New config".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    let snapshot = manager.create_snapshot(&config, options).await.unwrap();

    // Restore with backup
    let restore_options = RestoreOptions {
        validate: true,
        backup_before_restore: true,
        reason: Some("Restoring new config".to_string()),
        merge_changes: false,
    };

    let result = manager.restore_snapshot(&snapshot.id, &target_path, restore_options).await;

    assert!(result.is_ok(), "Should restore with backup");

    // Verify new content is in place
    let restored_content = tokio::fs::read_to_string(&target_path).await.unwrap();
    assert!(restored_content.contains("new_camera"));
    assert!(!restored_content.contains("old_camera"));

    // Verify backup was created
    let backups_dir = temp_dir.path().join("backups");
    assert!(backups_dir.exists(), "Backups directory should exist");

    let entries: Vec<_> = std::fs::read_dir(&backups_dir).unwrap().collect();
    assert!(entries.len() > 0, "Should have created a backup file");
}

#[tokio::test]
async fn test_restore_nonexistent_snapshot() {
    // Test that restoring a non-existent snapshot returns an error
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();

    let target_path = temp_dir.path().join("config.yaml");

    let restore_options = RestoreOptions {
        validate: true,
        backup_before_restore: false,
        reason: None,
        merge_changes: false,
    };

    let result = manager.restore_snapshot("nonexistent_snapshot", &target_path, restore_options).await;

    assert!(result.is_err(), "Should fail to restore non-existent snapshot");
}

#[tokio::test]
async fn test_rollback_sequence() {
    // Test a complete rollback sequence: create config -> modify -> rollback
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();
    let parser = ConfigParser::new();

    // Step 1: Create initial config
    let config_v1 = r#"
version: "0.13"

cameras:
  camera_v1:
    ffmpeg:
      inputs:
        - path: rtsp://v1
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let config = parser.parse_content(config_v1.to_string(), "v1.yaml".to_string()).unwrap().config;

    let options = SnapshotOptions {
        description: Some("Version 1".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    let snapshot_v1 = manager.create_snapshot(&config, options).await.unwrap();

    // Small delay for unique IDs
    tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

    // Step 2: Create modified config (version 2)
    let config_v2 = r#"
version: "0.13"

cameras:
  camera_v2:
    ffmpeg:
      inputs:
        - path: rtsp://v2
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8s
"#;

    let config = parser.parse_content(config_v2.to_string(), "v2.yaml".to_string()).unwrap().config;

    let options = SnapshotOptions {
        description: Some("Version 2 - modified".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    let snapshot_v2 = manager.create_snapshot(&config, options).await.unwrap();

    // Step 3: Create a config file with v2 content
    let target_path = temp_dir.path().join("current.yaml");
    tokio::fs::write(&target_path, config_v2).await.unwrap();

    // Step 4: Rollback to version 1
    let restore_options = RestoreOptions {
        validate: true,
        backup_before_restore: true,
        reason: Some("Rollback to v1".to_string()),
        merge_changes: false,
    };

    let result = manager.restore_snapshot(&snapshot_v1.id, &target_path, restore_options).await;

    assert!(result.is_ok(), "Rollback should succeed");

    // Step 5: Verify rolled back content
    let current_content = tokio::fs::read_to_string(&target_path).await.unwrap();
    assert!(current_content.contains("camera_v1"), "Should contain v1 camera");
    assert!(!current_content.contains("camera_v2"), "Should not contain v2 camera");
    assert!(current_content.contains("yolov8n"), "Should have original model");
}

#[tokio::test]
async fn test_multiple_rollbacks() {
    // Test multiple sequential rollbacks
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();
    let parser = ConfigParser::new();

    let target_path = temp_dir.path().join("config.yaml");

    // Create 3 versions
    let configs = vec![
        ("v1", "camera_v1"),
        ("v2", "camera_v2"),
        ("v3", "camera_v3"),
    ];

    let mut snapshots = Vec::new();

    for (version, camera_name) in &configs {
        let yaml = format!(
            r#"
version: "0.13"

cameras:
  {}:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#,
            camera_name
        );

        let config = parser.parse_content(yaml, format!("{}.yaml", version)).unwrap().config;

        let options = SnapshotOptions {
            description: Some(format!("Version {}", version)),
            creation_source: CreationSource::Ui,
            is_backup: false,
            backup_reason: None,
            tags: vec![],
        };

        let snapshot = manager.create_snapshot(&config, options).await.unwrap();
        snapshots.push(snapshot);

        // Delay for unique IDs
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;
    }

    let restore_options = RestoreOptions {
        validate: false,
        backup_before_restore: false,
        reason: None,
        merge_changes: false,
    };

    // Restore v3
    manager.restore_snapshot(&snapshots[2].id, &target_path, restore_options.clone()).await.unwrap();
    let content = tokio::fs::read_to_string(&target_path).await.unwrap();
    assert!(content.contains("camera_v3"));

    // Rollback to v2
    manager.restore_snapshot(&snapshots[1].id, &target_path, restore_options.clone()).await.unwrap();
    let content = tokio::fs::read_to_string(&target_path).await.unwrap();
    assert!(content.contains("camera_v2"));

    // Rollback to v1
    manager.restore_snapshot(&snapshots[0].id, &target_path, restore_options.clone()).await.unwrap();
    let content = tokio::fs::read_to_string(&target_path).await.unwrap();
    assert!(content.contains("camera_v1"));

    // Roll forward to v2
    manager.restore_snapshot(&snapshots[1].id, &target_path, restore_options.clone()).await.unwrap();
    let content = tokio::fs::read_to_string(&target_path).await.unwrap();
    assert!(content.contains("camera_v2"));
}

#[tokio::test]
async fn test_restore_preserves_yaml_structure() {
    // Test that restore preserves the exact YAML structure
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();

    let yaml = r#"
version: "0.13"

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://192.168.1.100:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: true
      fps: 5
    record:
      enabled: true
      retain_days: 7

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let config = parser.parse_content(yaml.to_string(), "test.yaml".to_string()).unwrap().config;

    let options = SnapshotOptions {
        description: Some("Structure test".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    let snapshot = manager.create_snapshot(&config, options).await.unwrap();

    let target_path = temp_dir.path().join("config.yaml");

    let restore_options = RestoreOptions {
        validate: true,
        backup_before_restore: false,
        reason: None,
        merge_changes: false,
    };

    manager.restore_snapshot(&snapshot.id, &target_path, restore_options).await.unwrap();

    // Verify content
    let restored_content = tokio::fs::read_to_string(&target_path).await.unwrap();
    assert!(restored_content.contains("front_door"));
    assert!(restored_content.contains("retain_days"));
    assert!(restored_content.contains("fps: 5"));
}

#[tokio::test]
async fn test_restore_result_contains_metadata() {
    // Test that restore result contains useful metadata
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();

    let yaml = r#"
version: "0.13"

cameras:
  test_cam:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let config = parser.parse_content(yaml.to_string(), "test.yaml".to_string()).unwrap().config;

    let options = SnapshotOptions {
        description: Some("Metadata test".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    let snapshot = manager.create_snapshot(&config, options).await.unwrap();

    let target_path = temp_dir.path().join("config.yaml");

    let restore_options = RestoreOptions {
        validate: true,
        backup_before_restore: false,
        reason: Some("Testing metadata".to_string()),
        merge_changes: false,
    };

    let result = manager.restore_snapshot(&snapshot.id, &target_path, restore_options).await.unwrap();

    assert!(result.success);
    assert!(!result.message.is_empty());
    assert!(result.changes_count > 0);
}

#[tokio::test]
async fn test_rollback_with_backup_snapshots() {
    // Test that backup snapshots can be used for rollback
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();

    let yaml = r#"
version: "0.13"

cameras:
  backup_test:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let config = parser.parse_content(yaml.to_string(), "test.yaml".to_string()).unwrap().config;

    // Create a backup snapshot
    let options = SnapshotOptions {
        description: Some("Backup before deployment".to_string()),
        creation_source: CreationSource::Template,
        is_backup: true,
        backup_reason: Some("Pre-deployment safety backup".to_string()),
        tags: vec![],
    };

    let backup_snapshot = manager.create_snapshot(&config, options).await.unwrap();

    assert!(backup_snapshot.is_backup);

    // Restore from backup snapshot
    let target_path = temp_dir.path().join("config.yaml");

    let restore_options = RestoreOptions {
        validate: true,
        backup_before_restore: false,
        reason: Some("Rollback from backup".to_string()),
        merge_changes: false,
    };

    let result = manager.restore_snapshot(&backup_snapshot.id, &target_path, restore_options).await;

    assert!(result.is_ok(), "Should be able to restore from backup snapshot");
    assert!(target_path.exists());

    let content = tokio::fs::read_to_string(&target_path).await.unwrap();
    assert!(content.contains("backup_test"));
}

#[tokio::test]
async fn test_restore_to_new_file() {
    // Test restoring to a file that doesn't exist yet
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();

    let yaml = r#"
version: "0.13"

cameras:
  new_file_test:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let config = parser.parse_content(yaml.to_string(), "test.yaml".to_string()).unwrap().config;

    let options = SnapshotOptions {
        description: Some("New file test".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    let snapshot = manager.create_snapshot(&config, options).await.unwrap();

    // Target file doesn't exist
    let target_path = temp_dir.path().join("new_config.yaml");
    assert!(!target_path.exists());

    let restore_options = RestoreOptions {
        validate: true,
        backup_before_restore: false,
        reason: None,
        merge_changes: false,
    };

    let result = manager.restore_snapshot(&snapshot.id, &target_path, restore_options).await;

    assert!(result.is_ok(), "Should restore to new file");
    assert!(target_path.exists(), "New file should be created");

    let content = tokio::fs::read_to_string(&target_path).await.unwrap();
    assert!(content.contains("new_file_test"));
}
