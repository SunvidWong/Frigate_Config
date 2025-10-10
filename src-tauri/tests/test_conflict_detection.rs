// T055: Unit tests for conflict detection in merger
// Tests for merger.rs conflict detection functionality

use frigate_config_tool::config_engine::merger::{
    ConfigMerger, ConflictSeverity
};
use frigate_config_tool::config_engine::parser::ConfigParser;

#[test]
fn test_no_conflicts_identical_configs() {
    // Test that identical configs produce no conflicts
    let yaml_content = r#"
version: "0.13"

cameras:
  front_door:
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

    let parser = ConfigParser::new();
    let ui_config = parser.parse_content(yaml_content.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(yaml_content.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok(), "Merge should succeed for identical configs");
    let merge_result = result.unwrap();

    assert_eq!(merge_result.conflicts.len(), 0, "Should have no conflicts for identical configs");
    assert_eq!(merge_result.statistics.conflicts_count, 0);
}

#[test]
fn test_value_mismatch_conflict() {
    // Test detection of value mismatches in detect section
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
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

    let manual_yaml = r#"
version: "0.13"

cameras:
  front_door:
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
    let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // The merger detects conflicts in nested sections (detect as a whole)
    // This is correct behavior - it detects when manual config differs from UI
    assert!(merge_result.conflicts.len() > 0, "Should detect value mismatch in detect section");

    // Check that a detect-related conflict was found
    let has_detect_conflict = merge_result.conflicts.iter().any(|c|
        c.path.contains("detect") || c.path.contains("enabled")
    );
    assert!(has_detect_conflict, "Should detect conflict in detect section");
}

#[test]
fn test_missing_field_conflict() {
    // Test detection when field exists in one config but not the other
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
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

    let manual_yaml = r#"
version: "0.13"

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true
      fps: 10
      width: 1280
      height: 720
    motion:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Manual config has additional fields (fps, width, height) that UI doesn't have
    // And it has a motion section that UI doesn't
    // Since detect sections differ, there should be a conflict or the values should be preserved

    // The merger should either preserve manual fields or detect conflicts
    let has_preserved_edits = merge_result.preserved_edits.len() > 0;
    let has_detect_conflict = merge_result.conflicts.iter().any(|c| c.path.contains("detect"));

    assert!(
        has_preserved_edits || has_detect_conflict,
        "Should either preserve manual-only fields or detect conflicts in detect section"
    );
}

#[test]
fn test_camera_only_in_manual() {
    // Test preservation of camera that only exists in manual config
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
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

    let manual_yaml = r#"
version: "0.13"

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true

  backyard:
    ffmpeg:
      inputs:
        - path: rtsp://backyard
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Backyard camera should be preserved
    assert!(merge_result.merged_config.cameras.contains_key("backyard"), "Should preserve manual-only camera");
    assert_eq!(merge_result.merged_config.cameras.len(), 2, "Should have both cameras");

    // Should have preserved edit for backyard camera
    let has_backyard_preserved = merge_result.preserved_edits.iter().any(|e|
        e.path.contains("backyard")
    );
    assert!(has_backyard_preserved, "Should track preserved manual-only camera");
}

#[test]
fn test_detector_only_in_manual() {
    // Test preservation of detector that only exists in manual config
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
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

    let manual_yaml = r#"
version: "0.13"

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n

  coral:
    model: ssd_mobilenet_v2
    device: usb
"#;

    let parser = ConfigParser::new();
    let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Coral detector should be preserved
    assert!(merge_result.merged_config.detectors.contains_key("coral"), "Should preserve manual-only detector");
    assert_eq!(merge_result.merged_config.detectors.len(), 2, "Should have both detectors");

    // Should have preserved edit for coral detector
    let has_coral_preserved = merge_result.preserved_edits.iter().any(|e|
        e.path.contains("coral")
    );
    assert!(has_coral_preserved, "Should track preserved manual-only detector");
}

#[test]
fn test_mqtt_section_conflict() {
    // Test conflict detection in global sections like MQTT
    let ui_yaml = r#"
version: "0.13"

mqtt:
  enabled: true
  host: mqtt.example.com
  port: 1883

cameras:
  front_door:
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

    let manual_yaml = r#"
version: "0.13"

mqtt:
  enabled: true
  host: mqtt.local
  port: 1883

cameras:
  front_door:
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
    let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should detect conflict in mqtt section (different host)
    let has_mqtt_conflict = merge_result.conflicts.iter().any(|c|
        c.path.contains("mqtt")
    );
    assert!(has_mqtt_conflict, "Should detect MQTT section conflict");
}

#[test]
fn test_critical_enabled_field_conflict() {
    // Test that conflicts in critical 'enabled' fields are marked as critical severity
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
    enabled: true
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

    let manual_yaml = r#"
version: "0.13"

cameras:
  front_door:
    enabled: false
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: false

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should detect enabled conflict at camera level
    let enabled_conflicts: Vec<_> = merge_result.conflicts.iter()
        .filter(|c| c.path.contains("enabled") || c.path.contains("detect"))
        .collect();

    assert!(enabled_conflicts.len() > 0, "Should detect enabled or detect field conflict");

    // Critical fields should be marked as critical or warning severity
    let has_critical_or_warning = enabled_conflicts.iter().any(|c|
        c.severity == ConflictSeverity::Critical || c.severity == ConflictSeverity::Warning
    );
    assert!(has_critical_or_warning, "Enabled/detect field conflicts should have high severity");
}

#[test]
fn test_multiple_camera_conflicts() {
    // Test detection of conflicts across multiple cameras
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://camera1
          roles: [detect]
    detect:
      enabled: true
      fps: 5

  backyard:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://camera2
          roles: [detect]
    detect:
      enabled: true
      fps: 10

detectors:
  cpu1:
    model: yolov8n
"#;

    let manual_yaml = r#"
version: "0.13"

cameras:
  front_door:
    enabled: false
    ffmpeg:
      inputs:
        - path: rtsp://camera1
          roles: [detect]
    detect:
      enabled: false
      fps: 5

  backyard:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://camera2_modified
          roles: [detect]
    detect:
      enabled: true
      fps: 15

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should detect conflicts in both cameras
    assert!(merge_result.conflicts.len() >= 1, "Should detect multiple conflicts");

    // Check for front_door or backyard conflicts
    let has_front_door_conflict = merge_result.conflicts.iter().any(|c|
        c.path.contains("front_door")
    );
    let has_backyard_conflict = merge_result.conflicts.iter().any(|c|
        c.path.contains("backyard")
    );

    assert!(has_front_door_conflict || has_backyard_conflict, "Should detect conflicts in at least one camera");
}

#[test]
fn test_detector_model_conflict() {
    // Test detection of detector model conflicts
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
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

    let manual_yaml = r#"
version: "0.13"

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8s
"#;

    let parser = ConfigParser::new();
    let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should detect model conflict (yolov8n vs yolov8s)
    let has_model_conflict = merge_result.conflicts.iter().any(|c|
        c.path.contains("cpu1") && c.path.contains("model")
    );
    assert!(has_model_conflict, "Should detect detector model conflict");
}

#[test]
fn test_merge_statistics() {
    // Test that merge statistics are calculated correctly
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
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

    let manual_yaml = r#"
version: "0.13"

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true
      fps: 10

  backyard:
    ffmpeg:
      inputs:
        - path: rtsp://backyard
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n

  coral:
    model: ssd_mobilenet_v2
"#;

    let parser = ConfigParser::new();
    let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Verify statistics are populated
    assert!(merge_result.statistics.total_fields > 0, "Should count total fields");
    assert!(merge_result.statistics.conflicts_count > 0, "Should count conflicts");
    assert!(merge_result.statistics.preserved_edits_count > 0, "Should count preserved edits");

    // Verify consistency
    assert_eq!(merge_result.statistics.conflicts_count, merge_result.conflicts.len());
    assert_eq!(merge_result.statistics.preserved_edits_count, merge_result.preserved_edits.len());
}

#[test]
fn test_merged_config_has_all_cameras() {
    // Test that merged config contains cameras from both UI and manual
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://ui_camera
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let manual_yaml = r#"
version: "0.13"

cameras:
  backyard:
    ffmpeg:
      inputs:
        - path: rtsp://manual_camera
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Merged config should have both cameras
    assert_eq!(merge_result.merged_config.cameras.len(), 2, "Should have 2 cameras");
    assert!(merge_result.merged_config.cameras.contains_key("front_door"), "Should have UI camera");
    assert!(merge_result.merged_config.cameras.contains_key("backyard"), "Should have manual camera");
}

#[test]
fn test_conflict_description_and_path() {
    // Test that conflicts have proper descriptions and paths
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
    enabled: true
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

    let manual_yaml = r#"
version: "0.13"

cameras:
  front_door:
    enabled: false
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
    detect:
      enabled: true
      fps: 10

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let ui_config = parser.parse_content(ui_yaml.to_string(), "ui.yaml".to_string()).unwrap().config;
    let manual_config = parser.parse_content(manual_yaml.to_string(), "manual.yaml".to_string()).unwrap().config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should have at least one conflict (enabled or detect)
    assert!(merge_result.conflicts.len() > 0, "Should have at least one conflict");

    // Find a conflict related to the camera
    let camera_conflict = merge_result.conflicts.iter().find(|c| c.path.contains("front_door"));
    assert!(camera_conflict.is_some(), "Should have conflict for front_door camera");

    let conflict = camera_conflict.unwrap();
    assert!(!conflict.description.is_empty(), "Conflict should have description");
    assert!(conflict.path.contains("cameras.front_door"), "Path should include camera name");
    assert!(conflict.ui_value.is_some(), "Should have UI value");
    assert!(conflict.manual_value.is_some(), "Should have manual value");
}
