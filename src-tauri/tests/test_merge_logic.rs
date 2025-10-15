// T056: Unit tests for merge logic in merger
// Tests for merger.rs merge functionality and output

use frigate_config_tool::config_engine::merger::ConfigMerger;
use frigate_config_tool::config_engine::parser::ConfigParser;

#[test]
fn test_merge_ui_takes_precedence_by_default() {
    // Test that UI values take precedence when prefer_ui_values is true (default)
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://ui_path
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
        - path: rtsp://manual_path
          roles: [detect]
    detect:
      enabled: false

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new(); // Default: prefer_ui_values = true
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Merged config should prefer UI value for enabled
    let front_door = merge_result.merged_config.cameras.get("front_door");
    assert!(front_door.is_some());
    assert_eq!(
        front_door.unwrap().enabled,
        true,
        "Should use UI enabled value (true)"
    );
}

#[test]
fn test_merge_combines_cameras_from_both_sources() {
    // Test that cameras from both UI and manual are included
    let ui_yaml = r#"
version: "0.13"

cameras:
  ui_camera:
    ffmpeg:
      inputs:
        - path: rtsp://ui
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
  manual_camera:
    ffmpeg:
      inputs:
        - path: rtsp://manual
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should have both cameras
    assert_eq!(merge_result.merged_config.cameras.len(), 2);
    assert!(merge_result.merged_config.cameras.contains_key("ui_camera"));
    assert!(merge_result
        .merged_config
        .cameras
        .contains_key("manual_camera"));
}

#[test]
fn test_merge_combines_detectors_from_both_sources() {
    // Test that detectors from both UI and manual are included
    let ui_yaml = r#"
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

    let manual_yaml = r#"
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

  coral:
    model: ssd_mobilenet_v2
    device: usb
"#;

    let parser = ConfigParser::new();
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should have both detectors
    assert_eq!(merge_result.merged_config.detectors.len(), 2);
    assert!(merge_result.merged_config.detectors.contains_key("cpu1"));
    assert!(merge_result.merged_config.detectors.contains_key("coral"));
}

#[test]
fn test_merge_preserves_mqtt_from_manual_when_not_in_ui() {
    // Test that MQTT config is preserved from manual when UI doesn't have it
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
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should preserve MQTT from manual
    assert!(merge_result.merged_config.global_config.mqtt.is_some());
    let mqtt = merge_result
        .merged_config
        .global_config
        .mqtt
        .as_ref()
        .unwrap();
    assert_eq!(mqtt.host, "mqtt.local");
}

#[test]
fn test_merge_statistics_accurate() {
    // Test that merge statistics are accurate
    let ui_yaml = r#"
version: "0.13"

cameras:
  cam1:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://test1
          roles: [detect]
    detect:
      enabled: true

  cam2:
    ffmpeg:
      inputs:
        - path: rtsp://test2
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
  cam1:
    enabled: false
    ffmpeg:
      inputs:
        - path: rtsp://test1
          roles: [detect]
    detect:
      enabled: false

  cam3:
    ffmpeg:
      inputs:
        - path: rtsp://test3
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
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Verify statistics
    let stats = &merge_result.statistics;
    assert!(stats.total_fields > 0, "Should count total fields");
    assert_eq!(stats.conflicts_count, merge_result.conflicts.len());
    assert_eq!(
        stats.preserved_edits_count,
        merge_result.preserved_edits.len()
    );
    assert_eq!(stats.warnings_count, merge_result.warnings.len());

    // Should have cam3 and coral preserved
    assert!(
        stats.preserved_edits_count >= 2,
        "Should have at least 2 preserved edits"
    );
}

#[test]
fn test_merge_empty_manual_config() {
    // Test merging with empty manual config (only UI)
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

cameras: {}

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should have UI camera
    assert_eq!(merge_result.merged_config.cameras.len(), 1);
    assert!(merge_result
        .merged_config
        .cameras
        .contains_key("front_door"));

    // No conflicts expected
    assert_eq!(merge_result.conflicts.len(), 0);
}

#[test]
fn test_merge_empty_ui_config() {
    // Test merging with empty UI config (only manual)
    let ui_yaml = r#"
version: "0.13"

cameras: {}

detectors:
  cpu1:
    model: yolov8n
"#;

    let manual_yaml = r#"
version: "0.13"

cameras:
  manual_camera:
    ffmpeg:
      inputs:
        - path: rtsp://manual
          roles: [detect]
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should preserve manual camera
    assert_eq!(merge_result.merged_config.cameras.len(), 1);
    assert!(merge_result
        .merged_config
        .cameras
        .contains_key("manual_camera"));

    // Should have preserved edit
    assert!(merge_result.preserved_edits.len() > 0);
}

#[test]
fn test_merge_preserves_manual_only_detector_fields() {
    // Test that manual-only fields in detectors are preserved
    let ui_yaml = r#"
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
  coral:
    model: ssd_mobilenet_v2
"#;

    let manual_yaml = r#"
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
  coral:
    model: ssd_mobilenet_v2
    device: usb
    model_path: /custom/path/model.tflite
"#;

    let parser = ConfigParser::new();
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Merged detector should have manual-only fields
    let coral = merge_result.merged_config.detectors.get("coral");
    assert!(coral.is_some());
    let coral = coral.unwrap();

    // Should have device and model_path from manual
    assert!(
        coral.device.is_some() || merge_result.preserved_edits.len() > 0,
        "Manual-only fields should be preserved"
    );
}

#[test]
fn test_merge_raw_content_is_valid_yaml() {
    // Test that merged raw_content is valid YAML
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
"#;

    let parser = ConfigParser::new();
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // raw_content should be valid YAML
    assert!(!merge_result.merged_config.raw_content.is_empty());

    // Try to parse it again to verify it's valid
    let reparse_result = parser.parse_content(
        merge_result.merged_config.raw_content.clone(),
        "merged.yaml".to_string(),
    );
    assert!(
        reparse_result.is_ok(),
        "Merged raw_content should be valid YAML"
    );
}

#[test]
fn test_merge_with_record_and_snapshots_configs() {
    // Test merging configs with record and snapshots sections
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect, record]
    detect:
      enabled: true
    record:
      enabled: true
      retain_days: 7

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
          roles: [detect, record]
    detect:
      enabled: true
    record:
      enabled: true
      retain_days: 30
    snapshots:
      enabled: true
      retain:
        default: 10

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should have merged camera with record and snapshots
    let camera = merge_result.merged_config.cameras.get("front_door");
    assert!(camera.is_some());

    let camera = camera.unwrap();
    assert!(camera.record.is_some(), "Should have record config");

    // Snapshots should be preserved from manual
    assert!(
        camera.snapshots.is_some()
            || merge_result
                .preserved_edits
                .iter()
                .any(|e| e.path.contains("snapshots")),
        "Snapshots should be preserved from manual"
    );
}

#[test]
fn test_merge_version_field() {
    // Test that version field is preserved
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
"#;

    let parser = ConfigParser::new();
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Version should be preserved
    assert!(merge_result.merged_config.raw_content.contains("version"));
}

#[test]
fn test_merge_complex_multi_camera_scenario() {
    // Test a complex real-world scenario with multiple cameras and conflicts
    let ui_yaml = r#"
version: "0.13"

cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://192.168.1.100:554/stream
          roles: [detect, record]
    detect:
      enabled: true
      fps: 5

  backyard:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://192.168.1.101:554/stream
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

cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://192.168.1.100:554/stream
          roles: [detect, record]
    detect:
      enabled: true
      fps: 5

  backyard:
    enabled: false
    ffmpeg:
      inputs:
        - path: rtsp://192.168.1.101:554/stream
          roles: [detect]
    detect:
      enabled: false

  garage:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://192.168.1.102:554/stream
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
    let ui_config = parser
        .parse_content(ui_yaml.to_string(), "ui.yaml".to_string())
        .unwrap()
        .config;
    let manual_config = parser
        .parse_content(manual_yaml.to_string(), "manual.yaml".to_string())
        .unwrap()
        .config;

    let merger = ConfigMerger::new();
    let result = merger.merge_configurations(&ui_config, &manual_config);

    assert!(result.is_ok());
    let merge_result = result.unwrap();

    // Should have 3 cameras (2 from UI + 1 manual-only)
    assert_eq!(merge_result.merged_config.cameras.len(), 3);
    assert!(merge_result
        .merged_config
        .cameras
        .contains_key("front_door"));
    assert!(merge_result.merged_config.cameras.contains_key("backyard"));
    assert!(merge_result.merged_config.cameras.contains_key("garage"));

    // Should have 2 detectors
    assert_eq!(merge_result.merged_config.detectors.len(), 2);
    assert!(merge_result.merged_config.detectors.contains_key("cpu1"));
    assert!(merge_result.merged_config.detectors.contains_key("coral"));

    // Should have MQTT from manual
    assert!(merge_result.merged_config.global_config.mqtt.is_some());

    // Should have conflicts (backyard enabled mismatch)
    assert!(merge_result.conflicts.len() > 0);

    // Should have preserved edits (garage camera, coral detector, mqtt)
    assert!(merge_result.preserved_edits.len() >= 2);
}
