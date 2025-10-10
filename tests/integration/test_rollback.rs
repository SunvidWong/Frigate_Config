// T058: Integration test for rollback mechanism
// Test the configuration rollback functionality

use frigate_config::config_engine::backup::BackupManager;
use frigate_config::config_engine::rollback::RollbackManager;
use tempfile::TempDir;

#[cfg(test)]
mod rollback_tests {
    use super::*;

    fn setup_test_env() -> (TempDir, BackupManager, RollbackManager) {
        let temp_dir = TempDir::new().unwrap();
        let backup_manager = BackupManager::new(temp_dir.path().to_path_buf());
        let rollback_manager = RollbackManager::new(temp_dir.path().to_path_buf());
        (temp_dir, backup_manager, rollback_manager)
    }

    #[test]
    fn test_rollback_to_previous_config() {
        let (_temp_dir, backup_mgr, rollback_mgr) = setup_test_env();

        // Create initial config
        let config_v1 = "mqtt:\n  enabled: true\n  host: mqtt.local\n";
        let backup_id_v1 = backup_mgr.create_backup(config_v1, "manual").unwrap();

        // Create updated config
        let config_v2 = "mqtt:\n  enabled: true\n  host: mqtt.remote\n";
        let _backup_id_v2 = backup_mgr.create_backup(config_v2, "pre_deploy").unwrap();

        // Rollback to v1
        let result = rollback_mgr.rollback_to(&backup_id_v1);

        assert!(result.is_ok());

        let restored_config = result.unwrap();
        assert_eq!(restored_config, config_v1);
    }

    #[test]
    fn test_rollback_creates_new_backup() {
        let (_temp_dir, backup_mgr, rollback_mgr) = setup_test_env();

        let config_v1 = "mqtt:\n  enabled: true\n";
        let backup_id_v1 = backup_mgr.create_backup(config_v1, "manual").unwrap();

        let config_v2 = "mqtt:\n  enabled: false\n";
        backup_mgr.create_backup(config_v2, "manual").unwrap();

        let initial_backup_count = backup_mgr.list_backups().unwrap().len();

        // Rollback should create a new backup
        rollback_mgr.rollback_to(&backup_id_v1).unwrap();

        let final_backup_count = backup_mgr.list_backups().unwrap().len();

        assert_eq!(final_backup_count, initial_backup_count + 1);
    }

    #[test]
    fn test_rollback_preserves_history() {
        let (_temp_dir, backup_mgr, rollback_mgr) = setup_test_env();

        let config_v1 = "version: 1\n";
        let id_v1 = backup_mgr.create_backup(config_v1, "manual").unwrap();

        let config_v2 = "version: 2\n";
        let id_v2 = backup_mgr.create_backup(config_v2, "manual").unwrap();

        let config_v3 = "version: 3\n";
        let id_v3 = backup_mgr.create_backup(config_v3, "manual").unwrap();

        // Rollback to v1
        rollback_mgr.rollback_to(&id_v1).unwrap();

        // All backups should still exist
        let backups = backup_mgr.list_backups().unwrap();
        assert!(backups.iter().any(|b| b.id == id_v1));
        assert!(backups.iter().any(|b| b.id == id_v2));
        assert!(backups.iter().any(|b| b.id == id_v3));
    }

    #[test]
    fn test_rollback_to_nonexistent_backup() {
        let (_temp_dir, _backup_mgr, rollback_mgr) = setup_test_env();

        let result = rollback_mgr.rollback_to("nonexistent-id");

        assert!(result.is_err());
    }

    #[test]
    fn test_rollback_updates_current_config() {
        let (temp_dir, backup_mgr, rollback_mgr) = setup_test_env();

        let current_config_path = temp_dir.path().join("current.yml");

        // Write initial current config
        let config_v1 = "mqtt:\n  enabled: true\n";
        std::fs::write(&current_config_path, config_v1).unwrap();
        let backup_id = backup_mgr.create_backup(config_v1, "manual").unwrap();

        // Update current config
        let config_v2 = "mqtt:\n  enabled: false\n";
        std::fs::write(&current_config_path, config_v2).unwrap();

        // Rollback
        rollback_mgr.rollback_to(&backup_id).unwrap();

        // Verify current config restored
        let current_content = std::fs::read_to_string(&current_config_path).unwrap();
        assert_eq!(current_content, config_v1);
    }

    #[test]
    fn test_rollback_logs_action() {
        let (_temp_dir, backup_mgr, rollback_mgr) = setup_test_env();

        let config = "mqtt:\n  enabled: true\n";
        let backup_id = backup_mgr.create_backup(config, "manual").unwrap();

        rollback_mgr.rollback_to(&backup_id).unwrap();

        // Verify rollback was logged
        let logs = rollback_mgr.get_rollback_history();

        assert!(logs.is_ok());
        let log_entries = logs.unwrap();

        assert!(log_entries.len() > 0);
        assert_eq!(log_entries[0].backup_id, backup_id);
        assert_eq!(log_entries[0].action, "rollback");
    }

    #[test]
    fn test_rollback_multiple_times() {
        let (_temp_dir, backup_mgr, rollback_mgr) = setup_test_env();

        let config_v1 = "version: 1\n";
        let id_v1 = backup_mgr.create_backup(config_v1, "manual").unwrap();

        let config_v2 = "version: 2\n";
        let id_v2 = backup_mgr.create_backup(config_v2, "manual").unwrap();

        let config_v3 = "version: 3\n";
        let _id_v3 = backup_mgr.create_backup(config_v3, "manual").unwrap();

        // Rollback to v1
        let result1 = rollback_mgr.rollback_to(&id_v1);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), config_v1);

        // Rollback to v2
        let result2 = rollback_mgr.rollback_to(&id_v2);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), config_v2);
    }

    #[test]
    fn test_rollback_validates_integrity() {
        let (temp_dir, backup_mgr, rollback_mgr) = setup_test_env();

        let config = "mqtt:\n  enabled: true\n";
        let backup_id = backup_mgr.create_backup(config, "manual").unwrap();

        // Corrupt the backup
        let backup_path = temp_dir.path().join("backups").join(format!("{}.yml", backup_id));
        std::fs::write(&backup_path, "corrupted").unwrap();

        // Rollback should fail integrity check
        let result = rollback_mgr.rollback_to(&backup_id);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("integrity") ||
                result.unwrap_err().to_string().contains("corrupt"));
    }

    #[test]
    fn test_get_rollback_preview() {
        let (_temp_dir, backup_mgr, rollback_mgr) = setup_test_env();

        let config = "mqtt:\n  enabled: true\n";
        let backup_id = backup_mgr.create_backup(config, "manual").unwrap();

        // Get preview without actually rolling back
        let preview = rollback_mgr.get_rollback_preview(&backup_id);

        assert!(preview.is_ok());
        let preview_data = preview.unwrap();

        assert_eq!(preview_data.config_content, config);
        assert!(preview_data.metadata.created_at.len() > 0);
    }

    #[test]
    fn test_rollback_performance() {
        let (_temp_dir, backup_mgr, rollback_mgr) = setup_test_env();

        // Create large config
        let mut large_config = String::from("cameras:\n");
        for i in 0..100 {
            large_config.push_str(&format!("  camera_{}:\n    enabled: true\n", i));
        }

        let backup_id = backup_mgr.create_backup(&large_config, "manual").unwrap();

        // Measure rollback performance
        let start = std::time::Instant::now();
        let result = rollback_mgr.rollback_to(&backup_id);
        let duration = start.elapsed();

        assert!(result.is_ok());
        // Should complete quickly (<100ms)
        assert!(duration.as_millis() < 100);
    }
}
