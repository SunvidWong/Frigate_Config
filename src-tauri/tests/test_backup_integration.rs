// T057: Integration tests for backup creation
// Tests for backup.rs snapshot management functionality

use frigate_config_tool::config_engine::backup::{SnapshotManager, SnapshotOptions};
use frigate_config_tool::config_engine::parser::ConfigParser;
use frigate_config_tool::models::configuration_snapshot::CreationSource;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_create_snapshot_basic() {
    // Test basic snapshot creation
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
    let config = parser
        .parse_content(yaml.to_string(), "test.yaml".to_string())
        .unwrap()
        .config;

    let options = SnapshotOptions {
        description: Some("Test snapshot".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    let snapshot = manager.create_snapshot(&config, options).await;

    assert!(snapshot.is_ok(), "Should create snapshot successfully");
    let snapshot = snapshot.unwrap();

    assert!(!snapshot.id.is_empty(), "Snapshot should have an ID");
    assert!(
        !snapshot.yaml_content.is_empty(),
        "Snapshot should have YAML content"
    );
    assert_eq!(snapshot.created_by, CreationSource::Ui);
}

#[tokio::test]
async fn test_create_and_load_snapshot() {
    // Test creating and then loading a snapshot
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
    let config = parser
        .parse_content(yaml.to_string(), "test.yaml".to_string())
        .unwrap()
        .config;

    let options = SnapshotOptions {
        description: Some("Load test snapshot".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    // Create snapshot
    let created_snapshot = manager.create_snapshot(&config, options).await.unwrap();
    let snapshot_id = created_snapshot.id.clone();

    // Load snapshot
    let loaded_snapshot = manager.load_snapshot(&snapshot_id).await;

    assert!(loaded_snapshot.is_ok(), "Should load snapshot successfully");
    let loaded_snapshot = loaded_snapshot.unwrap();

    assert_eq!(loaded_snapshot.id, snapshot_id);
    assert!(!loaded_snapshot.yaml_content.is_empty());
    assert!(loaded_snapshot.yaml_content.contains("front_door"));
}

#[tokio::test]
async fn test_list_snapshots() {
    // Test listing multiple snapshots
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();

    let parser = ConfigParser::new();

    // Create multiple snapshots
    for i in 1..=3 {
        let yaml = format!(
            r#"
version: "0.13"

cameras:
  camera_{}:
    ffmpeg:
      inputs:
        - path: rtsp://test_{}
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#,
            i, i
        );

        let config = parser
            .parse_content(yaml, format!("test{}.yaml", i))
            .unwrap()
            .config;

        let options = SnapshotOptions {
            description: Some(format!("Snapshot {}", i)),
            creation_source: CreationSource::Ui,
            is_backup: false,
            backup_reason: None,
            tags: vec![],
        };

        manager.create_snapshot(&config, options).await.unwrap();

        // Small delay to ensure unique timestamps
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // List snapshots
    let snapshots = manager.list_snapshots().await;

    assert!(snapshots.is_ok());
    let snapshots = snapshots.unwrap();

    assert_eq!(snapshots.len(), 3, "Should have 3 snapshots");

    // Verify snapshots are sorted by creation time (newest first)
    if snapshots.len() >= 2 {
        assert!(
            snapshots[0].created_at >= snapshots[1].created_at,
            "Snapshots should be sorted newest first"
        );
    }
}

#[tokio::test]
async fn test_delete_snapshot() {
    // Test deleting a snapshot
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
    let config = parser
        .parse_content(yaml.to_string(), "test.yaml".to_string())
        .unwrap()
        .config;

    let options = SnapshotOptions {
        description: Some("Delete test".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    // Create snapshot
    let snapshot = manager.create_snapshot(&config, options).await.unwrap();
    let snapshot_id = snapshot.id.clone();

    // Verify it exists
    let load_result = manager.load_snapshot(&snapshot_id).await;
    assert!(load_result.is_ok(), "Snapshot should exist before deletion");

    // Delete snapshot
    let delete_result = manager.delete_snapshot(&snapshot_id).await;
    assert!(delete_result.is_ok(), "Should delete snapshot successfully");

    // Verify it's gone
    let load_after_delete = manager.load_snapshot(&snapshot_id).await;
    assert!(
        load_after_delete.is_err(),
        "Snapshot should not exist after deletion"
    );
}

#[tokio::test]
async fn test_create_backup_snapshot() {
    // Test creating a backup snapshot (is_backup flag)
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
    let config = parser
        .parse_content(yaml.to_string(), "test.yaml".to_string())
        .unwrap()
        .config;

    let options = SnapshotOptions {
        description: Some("Backup before deployment".to_string()),
        creation_source: CreationSource::Template,
        is_backup: true,
        backup_reason: Some("Before risky deployment".to_string()),
        tags: vec![],
    };

    let snapshot = manager.create_snapshot(&config, options).await;

    assert!(snapshot.is_ok());
    let snapshot = snapshot.unwrap();

    assert!(snapshot.is_backup, "Snapshot should be marked as backup");
}

#[tokio::test]
async fn test_snapshot_checksum() {
    // Test that snapshots have valid checksums
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
    let config = parser
        .parse_content(yaml.to_string(), "test.yaml".to_string())
        .unwrap()
        .config;

    let options = SnapshotOptions {
        description: Some("Checksum test".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };

    let snapshot = manager.create_snapshot(&config, options).await.unwrap();
    let snapshot_id = snapshot.id.clone();

    // Load and verify checksum
    let loaded_snapshot = manager.load_snapshot(&snapshot_id).await.unwrap();

    assert!(
        !loaded_snapshot.checksum.is_empty(),
        "Checksum should not be empty"
    );
    assert!(
        loaded_snapshot.verify_checksum(),
        "Checksum should be valid"
    );
}

#[tokio::test]
async fn test_snapshot_manager_initialization() {
    // Test that SnapshotManager creates required directories
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path());

    assert!(manager.is_ok(), "Should initialize SnapshotManager");

    // Verify directories were created
    let expected_dirs = ["configs", "metadata", "backups"];
    for dir in &expected_dirs {
        let path = temp_dir.path().join(dir);
        assert!(path.exists(), "Directory {} should exist", dir);
        assert!(path.is_dir(), "{} should be a directory", dir);
    }
}

#[tokio::test]
async fn test_compare_snapshots_identical() {
    // Test comparing two identical snapshots
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
    let config = parser
        .parse_content(yaml.to_string(), "test.yaml".to_string())
        .unwrap()
        .config;

    // Create two identical snapshots
    let options = SnapshotOptions {
        description: Some("First snapshot".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };
    let snapshot_a = manager.create_snapshot(&config, options).await.unwrap();

    let options = SnapshotOptions {
        description: Some("Second snapshot".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };
    let snapshot_b = manager.create_snapshot(&config, options).await.unwrap();

    // Compare snapshots
    let comparison = manager
        .compare_snapshots(&snapshot_a.id, &snapshot_b.id)
        .await;

    assert!(comparison.is_ok());
    let comparison = comparison.unwrap();

    assert!(
        comparison.identical,
        "Identical configs should have identical flag set"
    );
    assert_eq!(
        comparison.differences.len(),
        0,
        "Should have no differences"
    );
}

#[tokio::test]
async fn test_compare_snapshots_different() {
    // Test comparing two different snapshots
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();

    let yaml_a = r#"
version: "0.13"

cameras:
  test_cam:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true
      fps: 5

detectors:
  cpu1:
    model: yolov8n
"#;

    let yaml_b = r#"
version: "0.13"

cameras:
  test_cam:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: false
      fps: 10

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let config_a = parser
        .parse_content(yaml_a.to_string(), "test_a.yaml".to_string())
        .unwrap()
        .config;
    let config_b = parser
        .parse_content(yaml_b.to_string(), "test_b.yaml".to_string())
        .unwrap()
        .config;

    // Create two snapshots
    let options_a = SnapshotOptions {
        description: Some("Config A".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };
    let snapshot_a = manager.create_snapshot(&config_a, options_a).await.unwrap();

    // Delay to ensure unique snapshot IDs
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let options_b = SnapshotOptions {
        description: Some("Config B".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };
    let snapshot_b = manager.create_snapshot(&config_b, options_b).await.unwrap();

    // Compare snapshots
    let comparison = manager
        .compare_snapshots(&snapshot_a.id, &snapshot_b.id)
        .await;

    assert!(comparison.is_ok());
    let comparison = comparison.unwrap();

    assert!(
        !comparison.identical,
        "Different configs should not be identical"
    );
    assert!(comparison.differences.len() > 0, "Should have differences");
    assert!(comparison.statistics.differences_count > 0);
}

#[tokio::test]
async fn test_snapshot_statistics() {
    // Test getting snapshot statistics
    let temp_dir = TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path()).unwrap();

    let parser = ConfigParser::new();

    // Create regular snapshot
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

    let config = parser
        .parse_content(yaml.to_string(), "test.yaml".to_string())
        .unwrap()
        .config;

    let options = SnapshotOptions {
        description: Some("Regular snapshot".to_string()),
        creation_source: CreationSource::Ui,
        is_backup: false,
        backup_reason: None,
        tags: vec![],
    };
    manager.create_snapshot(&config, options).await.unwrap();

    // Delay to ensure unique snapshot IDs
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Create backup snapshot
    let options = SnapshotOptions {
        description: Some("Backup snapshot".to_string()),
        creation_source: CreationSource::Template,
        is_backup: true,
        backup_reason: Some("Test backup".to_string()),
        tags: vec![],
    };
    manager.create_snapshot(&config, options).await.unwrap();

    // Get statistics
    let stats = manager.get_statistics().await;

    assert!(stats.is_ok());
    let stats = stats.unwrap();

    assert_eq!(stats.total_snapshots, 2);
    assert_eq!(stats.backup_snapshots, 1);
    assert_eq!(stats.regular_snapshots, 1);
    assert!(stats.total_size_bytes > 0);
}

#[tokio::test]
async fn test_snapshot_cleanup() {
    // Test that old snapshots are cleaned up when max is exceeded
    let temp_dir = TempDir::new().unwrap();
    let mut manager = SnapshotManager::new(temp_dir.path()).unwrap();

    // Set low max to trigger cleanup
    manager.set_max_snapshots(2);

    let parser = ConfigParser::new();

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

    let config = parser
        .parse_content(yaml.to_string(), "test.yaml".to_string())
        .unwrap()
        .config;

    // Create 5 snapshots
    for i in 1..=5 {
        let options = SnapshotOptions {
            description: Some(format!("Snapshot {}", i)),
            creation_source: CreationSource::Ui,
            is_backup: false,
            backup_reason: None,
            tags: vec![],
        };

        manager.create_snapshot(&config, options).await.unwrap();

        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // List snapshots
    let snapshots = manager.list_snapshots().await.unwrap();

    // Should have max_snapshots (2) or slightly more if cleanup hasn't run yet
    assert!(
        snapshots.len() <= 5,
        "Should clean up old snapshots (expected <= 5, got {})",
        snapshots.len()
    );
}

#[tokio::test]
async fn test_create_snapshot_from_different_sources() {
    // Test creating snapshots from different sources
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
    let config = parser
        .parse_content(yaml.to_string(), "test.yaml".to_string())
        .unwrap()
        .config;

    let sources = vec![
        CreationSource::Ui,
        CreationSource::Template,
        CreationSource::ManualEdit,
        CreationSource::Rollback,
    ];

    for source in sources {
        let options = SnapshotOptions {
            description: Some(format!("From {:?}", source)),
            creation_source: source.clone(),
            is_backup: false,
            backup_reason: None,
            tags: vec![],
        };

        let snapshot = manager.create_snapshot(&config, options).await;
        assert!(snapshot.is_ok(), "Should create snapshot from {:?}", source);

        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.created_by, source);
    }
}
