// T057: Integration test for backup creation
// Test the backup system functionality

use frigate_config::config_engine::backup::{BackupManager, BackupMetadata};
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod backup_tests {
    use super::*;

    fn setup_test_env() -> (TempDir, BackupManager) {
        let temp_dir = TempDir::new().unwrap();
        let manager = BackupManager::new(temp_dir.path().to_path_buf());
        (temp_dir, manager)
    }

    #[test]
    fn test_create_backup() {
        let (_temp_dir, manager) = setup_test_env();

        let config_content = r#"
mqtt:
  enabled: true
  host: mqtt.local
"#;

        let result = manager.create_backup(config_content, "pre_deploy");

        assert!(result.is_ok());
        let backup_id = result.unwrap();
        assert!(!backup_id.is_empty());
    }

    #[test]
    fn test_backup_file_created() {
        let (temp_dir, manager) = setup_test_env();

        let config_content = r#"
mqtt:
  enabled: true
"#;

        let backup_id = manager.create_backup(config_content, "manual").unwrap();

        // Verify backup file exists
        let backup_path = temp_dir.path().join("backups").join(format!("{}.yml", backup_id));
        assert!(backup_path.exists());

        // Verify content matches
        let backed_up_content = std::fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backed_up_content, config_content);
    }

    #[test]
    fn test_backup_metadata_stored() {
        let (_temp_dir, manager) = setup_test_env();

        let config_content = "mqtt:\n  enabled: true\n";
        let backup_id = manager.create_backup(config_content, "pre_deploy").unwrap();

        // Retrieve metadata
        let metadata = manager.get_backup_metadata(&backup_id);

        assert!(metadata.is_ok());
        let meta = metadata.unwrap();

        assert_eq!(meta.id, backup_id);
        assert_eq!(meta.reason, "pre_deploy");
        assert!(meta.created_at.len() > 0);
        assert!(meta.checksum.len() > 0);
    }

    #[test]
    fn test_list_backups() {
        let (_temp_dir, manager) = setup_test_env();

        // Create multiple backups
        manager.create_backup("config1", "manual").unwrap();
        manager.create_backup("config2", "pre_deploy").unwrap();
        manager.create_backup("config3", "pre_merge").unwrap();

        let backups = manager.list_backups();

        assert!(backups.is_ok());
        let backup_list = backups.unwrap();

        assert_eq!(backup_list.len(), 3);
    }

    #[test]
    fn test_backup_retention_policy() {
        let (_temp_dir, manager) = setup_test_env();

        // Create 15 backups (retention limit is 10)
        for i in 0..15 {
            manager.create_backup(&format!("config{}", i), "manual").unwrap();
        }

        // Apply retention policy
        manager.apply_retention_policy();

        let backups = manager.list_backups().unwrap();

        // Should only keep last 10
        assert_eq!(backups.len(), 10);
    }

    #[test]
    fn test_backup_checksum_verification() {
        let (_temp_dir, manager) = setup_test_env();

        let config_content = "mqtt:\n  enabled: true\n";
        let backup_id = manager.create_backup(config_content, "manual").unwrap();

        // Verify checksum
        let is_valid = manager.verify_backup_integrity(&backup_id);

        assert!(is_valid.is_ok());
        assert!(is_valid.unwrap());
    }

    #[test]
    fn test_backup_checksum_detects_corruption() {
        let (temp_dir, manager) = setup_test_env();

        let config_content = "mqtt:\n  enabled: true\n";
        let backup_id = manager.create_backup(config_content, "manual").unwrap();

        // Corrupt the backup file
        let backup_path = temp_dir.path().join("backups").join(format!("{}.yml", backup_id));
        std::fs::write(&backup_path, "corrupted content").unwrap();

        // Verify checksum should fail
        let is_valid = manager.verify_backup_integrity(&backup_id);

        assert!(is_valid.is_ok());
        assert!(!is_valid.unwrap());
    }

    #[test]
    fn test_get_backup_content() {
        let (_temp_dir, manager) = setup_test_env();

        let original_content = r#"
cameras:
  front_door:
    enabled: true
"#;

        let backup_id = manager.create_backup(original_content, "pre_deploy").unwrap();

        // Retrieve backup content
        let content = manager.get_backup_content(&backup_id);

        assert!(content.is_ok());
        assert_eq!(content.unwrap(), original_content);
    }

    #[test]
    fn test_delete_backup() {
        let (_temp_dir, manager) = setup_test_env();

        let backup_id = manager.create_backup("test config", "manual").unwrap();

        // Delete backup
        let result = manager.delete_backup(&backup_id);

        assert!(result.is_ok());

        // Verify backup no longer exists
        let backups = manager.list_backups().unwrap();
        assert!(!backups.iter().any(|b| b.id == backup_id));
    }

    #[test]
    fn test_backup_ordering() {
        let (_temp_dir, manager) = setup_test_env();

        // Create backups with delays
        let id1 = manager.create_backup("config1", "manual").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let id2 = manager.create_backup("config2", "manual").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let id3 = manager.create_backup("config3", "manual").unwrap();

        let backups = manager.list_backups().unwrap();

        // Should be ordered by creation time (newest first)
        assert_eq!(backups[0].id, id3);
        assert_eq!(backups[1].id, id2);
        assert_eq!(backups[2].id, id1);
    }

    #[test]
    fn test_backup_with_large_config() {
        let (_temp_dir, manager) = setup_test_env();

        // Create a large config (simulate complex setup)
        let mut large_config = String::from("cameras:\n");
        for i in 0..100 {
            large_config.push_str(&format!("  camera_{}:\n    enabled: true\n", i));
        }

        let result = manager.create_backup(&large_config, "manual");

        assert!(result.is_ok());

        let backup_id = result.unwrap();
        let retrieved_content = manager.get_backup_content(&backup_id).unwrap();

        assert_eq!(retrieved_content, large_config);
    }

    #[test]
    fn test_concurrent_backup_creation() {
        use std::sync::Arc;
        use std::thread;

        let (temp_dir, manager) = setup_test_env();
        let manager = Arc::new(manager);

        let mut handles = vec![];

        for i in 0..5 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                manager_clone.create_backup(&format!("config{}", i), "manual")
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            assert!(handle.join().unwrap().is_ok());
        }

        // Verify all backups created
        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 5);
    }
}
