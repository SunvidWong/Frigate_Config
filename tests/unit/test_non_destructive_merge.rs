// T161: Unit tests for non-destructive merge logic
// Tests that manual edits are preserved during UI configuration merges

use frigate_config::config_engine::parser::ConfigParser;
use frigate_config::config_engine::merger::{ConfigMerger, ConflictSeverity};

#[cfg(test)]
mod non_destructive_merge_tests {
    use super::*;

    #[test]
    fn test_preserve_manual_only_fields() {
        let ui_yaml = r#"
mqtt:
  enabled: true
  host: localhost
"#;

        let manual_yaml = r#"
mqtt:
  enabled: true
  host: localhost
  custom_field: manual_value
  another_custom: data
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // Manual-only fields should be preserved
        assert!(merge_result.statistics.preserved_edits_count > 0);

        // Verify custom fields are in merged config
        let merged_yaml = &merge_result.merged_config.yaml;
        assert!(merged_yaml.get("mqtt").and_then(|m| m.get("custom_field")).is_some());
        assert!(merged_yaml.get("mqtt").and_then(|m| m.get("another_custom")).is_some());
    }

    #[test]
    fn test_preserve_manual_comments() {
        let ui_yaml = r#"
mqtt:
  enabled: true
  host: localhost
"#;

        let manual_yaml = r#"
# MANUAL: Custom MQTT configuration
mqtt:
  enabled: true  # TODO: Configure properly
  host: localhost
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // Comments from manual config should be preserved
        let merged_config = &merge_result.merged_config;
        assert!(!merged_config.metadata.comments.is_empty());

        // Verify manual edit markers are preserved
        assert!(merged_config.metadata.has_manual_edits);
    }

    #[test]
    fn test_ui_value_wins_on_conflict_by_default() {
        let ui_yaml = r#"
mqtt:
  enabled: true
  host: ui.example.com
  port: 1883
"#;

        let manual_yaml = r#"
mqtt:
  enabled: true
  host: manual.example.com
  port: 1883
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // By default, UI values should win
        let merged_mqtt = merge_result.merged_config.yaml.get("mqtt").unwrap();
        assert_eq!(merged_mqtt.get("host").unwrap().as_str().unwrap(), "ui.example.com");

        // Conflict should be detected
        assert!(merge_result.statistics.conflicts_count > 0);
    }

    #[test]
    fn test_preserve_manual_only_cameras() {
        let ui_yaml = r#"
cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://ui/stream
"#;

        let manual_yaml = r#"
cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://ui/stream
  back_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://manual/stream
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // Manual-only camera (back_door) should be preserved
        let merged_cameras = &merge_result.merged_config.cameras;
        assert!(merged_cameras.contains_key("front_door"));
        assert!(merged_cameras.contains_key("back_door"));

        // Should track preserved edit
        assert!(merge_result.statistics.preserved_edits_count > 0);
    }

    #[test]
    fn test_preserve_manual_only_detectors() {
        let ui_yaml = r#"
detectors:
  cpu1:
    type: cpu
"#;

        let manual_yaml = r#"
detectors:
  cpu1:
    type: cpu
  coral1:
    type: edgetpu
    device: usb
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // Manual-only detector (coral1) should be preserved
        let merged_detectors = &merge_result.merged_config.detectors;
        assert!(merged_detectors.contains_key("cpu1"));
        assert!(merged_detectors.contains_key("coral1"));

        // Should track preserved edit
        assert!(merge_result.statistics.preserved_edits_count > 0);
    }

    #[test]
    fn test_merge_without_conflicts() {
        let ui_yaml = r#"
mqtt:
  enabled: true
  host: localhost
cameras:
  front_door:
    enabled: true
"#;

        let manual_yaml = r#"
mqtt:
  enabled: true
  host: localhost
  port: 1883
cameras:
  front_door:
    enabled: true
  back_door:
    enabled: true
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // Should merge successfully without conflicts
        assert_eq!(merge_result.statistics.conflicts_count, 0);

        // All fields should be present
        let merged_mqtt = merge_result.merged_config.yaml.get("mqtt").unwrap();
        assert!(merged_mqtt.get("enabled").is_some());
        assert!(merged_mqtt.get("host").is_some());
        assert!(merged_mqtt.get("port").is_some());

        let merged_cameras = &merge_result.merged_config.cameras;
        assert_eq!(merged_cameras.len(), 2);
    }

    #[test]
    fn test_detect_value_mismatch_conflict() {
        let ui_yaml = r#"
mqtt:
  enabled: true
  host: ui.example.com
"#;

        let manual_yaml = r#"
mqtt:
  enabled: false
  host: manual.example.com
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // Should detect conflicts for both enabled and host
        assert!(merge_result.statistics.conflicts_count >= 2);

        // Conflicts should have details
        for conflict in &merge_result.conflicts {
            assert!(conflict.ui_value.is_some());
            assert!(conflict.manual_value.is_some());
            assert!(!conflict.description.is_empty());
        }
    }

    #[test]
    fn test_conflict_severity_levels() {
        let ui_yaml = r#"
cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://ui/stream
    detect:
      fps: 5
"#;

        let manual_yaml = r#"
cameras:
  front_door:
    enabled: false
    ffmpeg:
      inputs:
        - path: rtsp://manual/stream
    detect:
      fps: 10
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // "enabled" conflicts should be Critical severity
        let enabled_conflicts: Vec<_> = merge_result.conflicts.iter()
            .filter(|c| c.path.contains("enabled"))
            .collect();
        assert!(!enabled_conflicts.is_empty());

        for conflict in enabled_conflicts {
            assert_eq!(conflict.severity, ConflictSeverity::Critical);
        }
    }

    #[test]
    fn test_preserve_manual_camera_fields() {
        let ui_yaml = r#"
cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://stream
"#;

        let manual_yaml = r#"
cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://stream
    custom_field: manual_value
    advanced_settings:
      custom: true
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // Manual-only camera fields should be preserved
        assert!(merge_result.statistics.preserved_edits_count > 0);

        let preserved_paths: Vec<&str> = merge_result.preserved_edits.iter()
            .map(|e| e.path.as_str())
            .collect();

        assert!(preserved_paths.iter().any(|p| p.contains("custom_field")));
    }

    #[test]
    fn test_merge_statistics_accuracy() {
        let ui_yaml = r#"
mqtt:
  enabled: true
  host: ui.example.com
cameras:
  cam1:
    enabled: true
"#;

        let manual_yaml = r#"
mqtt:
  enabled: false
  host: manual.example.com
  port: 1883
cameras:
  cam1:
    enabled: true
  cam2:
    enabled: true
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // Verify statistics are accurate
        let stats = &merge_result.statistics;

        assert!(stats.total_fields > 0);
        assert!(stats.conflicts_count > 0);
        assert!(stats.preserved_edits_count > 0);
        assert_eq!(stats.warnings_count, merge_result.warnings.len());
    }

    #[test]
    fn test_empty_manual_config_merge() {
        let ui_yaml = r#"
mqtt:
  enabled: true
cameras:
  front_door:
    enabled: true
"#;

        let manual_yaml = "";

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // Should use UI config as-is
        assert_eq!(merge_result.statistics.conflicts_count, 0);
        assert_eq!(merge_result.statistics.preserved_edits_count, 0);

        // Merged config should match UI config
        assert!(merge_result.merged_config.yaml.get("mqtt").is_some());
        assert!(merge_result.merged_config.yaml.get("cameras").is_some());
    }

    #[test]
    fn test_empty_ui_config_merge() {
        let ui_yaml = "";

        let manual_yaml = r#"
mqtt:
  enabled: true
cameras:
  front_door:
    enabled: true
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // Should preserve all manual config
        assert_eq!(merge_result.statistics.conflicts_count, 0);
        assert!(merge_result.statistics.preserved_edits_count > 0);

        // Merged config should match manual config
        assert!(merge_result.merged_config.yaml.get("mqtt").is_some());
        assert!(merge_result.merged_config.yaml.get("cameras").is_some());
    }

    #[test]
    fn test_preserve_nested_manual_edits() {
        let ui_yaml = r#"
cameras:
  front_door:
    enabled: true
    detect:
      fps: 5
"#;

        let manual_yaml = r#"
cameras:
  front_door:
    enabled: true
    detect:
      fps: 5
      custom_threshold: 0.8
      advanced:
        setting1: value1
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // Nested manual edits should be preserved
        assert!(merge_result.statistics.preserved_edits_count > 0);
    }

    #[test]
    fn test_conflict_descriptions_are_informative() {
        let ui_yaml = r#"
mqtt:
  enabled: true
  host: ui.example.com
"#;

        let manual_yaml = r#"
mqtt:
  enabled: false
  host: manual.example.com
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // All conflicts should have informative descriptions
        for conflict in &merge_result.conflicts {
            assert!(!conflict.description.is_empty());
            assert!(conflict.description.len() > 10);
            assert!(conflict.path.len() > 0);
        }
    }

    #[test]
    fn test_preserved_edit_reasons() {
        let ui_yaml = r#"
mqtt:
  enabled: true
"#;

        let manual_yaml = r#"
mqtt:
  enabled: true
  custom_field: manual_value
cameras:
  manual_camera:
    enabled: true
"#;

        let parser = ConfigParser::new();
        let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
        let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&ui_config, &manual_config).unwrap();

        // All preserved edits should have reasons
        for edit in &merge_result.preserved_edits {
            assert!(!edit.reason.is_empty());
            assert!(!edit.path.is_empty());
        }
    }

    #[test]
    fn test_merge_identical_configs() {
        let yaml = r#"
mqtt:
  enabled: true
  host: localhost
cameras:
  front_door:
    enabled: true
"#;

        let parser = ConfigParser::new();
        let config1 = parser.parse_content(yaml.to_string(), "config1.yaml".to_string()).unwrap().config;
        let config2 = parser.parse_content(yaml.to_string(), "config2.yaml".to_string()).unwrap().config;

        let merger = ConfigMerger::new();
        let merge_result = merger.merge_configurations(&config1, &config2).unwrap();

        // Identical configs should have no conflicts
        assert_eq!(merge_result.statistics.conflicts_count, 0);
        assert_eq!(merge_result.statistics.preserved_edits_count, 0);
        assert_eq!(merge_result.statistics.warnings_count, 0);
    }
}
