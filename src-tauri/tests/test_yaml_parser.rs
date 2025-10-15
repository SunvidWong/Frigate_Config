// T054: Unit tests for YAML parser with comment preservation
// Tests for parser.rs functionality

use frigate_config_tool::config_engine::parser::ConfigParser;

#[test]
fn test_parse_simple_frigate_config() {
    let yaml_content = r#"
version: "0.13"

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://user:pass@192.168.1.100:554/stream
          roles:
            - detect
            - record
    detect:
      enabled: true
      fps: 5

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    assert!(
        result.is_ok(),
        "Failed to parse valid config: {:?}",
        result.err()
    );

    let parse_result = result.unwrap();
    let config = parse_result.config;

    // Verify cameras parsed correctly
    assert_eq!(config.cameras.len(), 1, "Should have 1 camera");
    assert!(
        config.cameras.contains_key("front_door"),
        "Should contain front_door camera"
    );

    let camera = config.cameras.get("front_door").unwrap();
    assert_eq!(camera.name, "front_door");
    assert!(camera.enabled, "Camera should be enabled by default");
    assert!(camera.detect.enabled, "Detect should be enabled");
    assert_eq!(camera.detect.fps, Some(5.0));

    // Verify detectors parsed correctly
    assert_eq!(config.detectors.len(), 1, "Should have 1 detector");
    assert!(
        config.detectors.contains_key("cpu1"),
        "Should contain cpu1 detector"
    );

    let detector = config.detectors.get("cpu1").unwrap();
    assert_eq!(detector.model, "yolov8n");
}

#[test]
fn test_parse_config_with_comments() {
    let yaml_content = r#"
# This is a header comment
version: "0.13"

# Camera configuration section
cameras:
  # Front door camera
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://user:pass@192.168.1.100:554/stream  # Main stream
          roles:
            - detect  # Enable detection
            - record  # Enable recording
    detect:
      enabled: true
      fps: 5  # Detection FPS

detectors:
  cpu1:
    model: yolov8n  # Using YOLOv8 nano model
"#;

    let parser = ConfigParser::new();
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    assert!(result.is_ok());
    let parse_result = result.unwrap();
    let config = parse_result.config;

    // Verify comments were extracted
    assert!(
        !config.metadata.comments.is_empty(),
        "Comments should be preserved"
    );

    // Check for specific comments
    let comment_texts: Vec<String> = config
        .metadata
        .comments
        .iter()
        .map(|c| c.content.clone())
        .collect();

    assert!(
        comment_texts.iter().any(|c| c.contains("header comment")),
        "Should find header comment"
    );
    assert!(
        comment_texts
            .iter()
            .any(|c| c.contains("Camera configuration")),
        "Should find camera section comment"
    );
    assert!(
        comment_texts
            .iter()
            .any(|c| c.contains("Front door camera")),
        "Should find inline camera comment"
    );

    // Verify config still parses correctly despite comments
    assert_eq!(config.cameras.len(), 1);
    assert_eq!(config.detectors.len(), 1);
}

#[test]
fn test_parse_multiple_cameras() {
    let yaml_content = r#"
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

  backyard:
    ffmpeg:
      inputs:
        - path: rtsp://192.168.1.101:554/stream
          roles: [detect]
    detect:
      enabled: true
      fps: 10

  garage:
    enabled: false
    ffmpeg:
      inputs:
        - path: rtsp://192.168.1.102:554/stream
          roles: [detect]
    detect:
      enabled: false

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    assert!(result.is_ok());
    let parse_result = result.unwrap();
    let config = parse_result.config;

    assert_eq!(config.cameras.len(), 3, "Should have 3 cameras");

    // Check front_door
    let front_door = config.cameras.get("front_door").unwrap();
    assert_eq!(front_door.name, "front_door");
    assert!(front_door.enabled);
    assert_eq!(front_door.detect.fps, Some(5.0));

    // Check backyard
    let backyard = config.cameras.get("backyard").unwrap();
    assert_eq!(backyard.name, "backyard");
    assert_eq!(backyard.detect.fps, Some(10.0));

    // Check garage (disabled)
    let garage = config.cameras.get("garage").unwrap();
    assert!(!garage.enabled, "Garage camera should be disabled");
    assert!(
        !garage.detect.enabled,
        "Garage detection should be disabled"
    );
}

#[test]
fn test_parse_config_with_detectors() {
    let yaml_content = r#"
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

  nvidia:
    model: yolov8n
    device: 0
    model_path: /config/models/yolov8n.tflite
"#;

    let parser = ConfigParser::new();
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    assert!(result.is_ok());
    let parse_result = result.unwrap();
    let config = parse_result.config;

    assert_eq!(config.detectors.len(), 3, "Should have 3 detectors");

    // Check CPU detector
    let cpu = config.detectors.get("cpu1").unwrap();
    assert_eq!(cpu.model, "yolov8n");
    assert!(cpu.device.is_none(), "CPU should have no device specified");

    // Check Coral detector
    let coral = config.detectors.get("coral").unwrap();
    assert_eq!(coral.model, "ssd_mobilenet_v2");
    assert_eq!(coral.device.as_ref().unwrap(), "usb");

    // Check NVIDIA detector
    let nvidia = config.detectors.get("nvidia").unwrap();
    assert_eq!(nvidia.model, "yolov8n");
    assert_eq!(nvidia.device.as_ref().unwrap(), "0");
    assert!(nvidia.model_path.is_some());
}

#[test]
fn test_parse_config_with_mqtt() {
    let yaml_content = r#"
version: "0.13"

mqtt:
  enabled: true
  host: mqtt.example.com
  port: 1883
  topic_prefix: frigate
  client_id: frigate_client
  user: mqtt_user
  password: mqtt_pass

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
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    assert!(result.is_ok());
    let parse_result = result.unwrap();
    let config = parse_result.config;

    // Verify MQTT configuration
    assert!(
        config.global_config.mqtt.is_some(),
        "MQTT config should be present"
    );

    let mqtt = config.global_config.mqtt.as_ref().unwrap();
    assert!(mqtt.enabled);
    assert_eq!(mqtt.host, "mqtt.example.com");
    assert_eq!(mqtt.port, 1883);
    assert_eq!(mqtt.topic_prefix, "frigate");
    assert_eq!(mqtt.client_id, "frigate_client");
    assert_eq!(mqtt.user.as_ref().unwrap(), "mqtt_user");
    assert_eq!(mqtt.password.as_ref().unwrap(), "mqtt_pass");
}

#[test]
fn test_parse_invalid_yaml() {
    let yaml_content = r#"
version: "0.13"
cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect
    # Missing closing bracket - invalid YAML
"#;

    let parser = ConfigParser::new();
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    assert!(result.is_err(), "Should fail on invalid YAML");

    let err = result.err().unwrap();
    assert!(
        err.to_string().contains("Invalid YAML"),
        "Error should mention invalid YAML"
    );
}

#[test]
fn test_parse_config_missing_required_camera_fields() {
    let yaml_content = r#"
version: "0.13"

cameras:
  front_door:
    # Missing ffmpeg inputs - required field
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let parser = parser; // Use strict mode for validation
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    // Should still parse, but camera won't be in the result due to missing inputs
    assert!(result.is_ok());
    let parse_result = result.unwrap();

    // Camera should fail to parse due to missing inputs
    assert_eq!(
        parse_result.config.cameras.len(),
        0,
        "Camera should not parse without inputs"
    );
}

#[test]
fn test_parse_config_with_record_and_snapshots() {
    let yaml_content = r#"
version: "0.13"

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect, record]
    detect:
      enabled: true
      fps: 5
    record:
      enabled: true
      retain_days: 7
      events:
        enabled: true
        pre_capture: 5
        post_capture: 10
    snapshots:
      enabled: true
      timestamp: true
      bounding_box: true
      crop: false
      height: 720
      retain:
        default: 10
        objects:
          person: 30
          car: 14

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    assert!(result.is_ok());
    let parse_result = result.unwrap();
    let config = parse_result.config;

    let camera = config.cameras.get("front_door").unwrap();

    // Verify record config
    assert!(camera.record.is_some(), "Record config should be present");
    let record = camera.record.as_ref().unwrap();
    assert!(record.enabled);
    assert_eq!(record.retain_days, Some(7));

    let events = record.events.as_ref().unwrap();
    assert!(events.enabled);
    assert_eq!(events.pre_capture, Some(5));
    assert_eq!(events.post_capture, Some(10));

    // Verify snapshots config
    assert!(
        camera.snapshots.is_some(),
        "Snapshots config should be present"
    );
    let snapshots = camera.snapshots.as_ref().unwrap();
    assert!(snapshots.enabled);
    assert!(snapshots.timestamp);
    assert!(snapshots.bounding_box);
    assert!(!snapshots.crop);
    assert_eq!(snapshots.height, Some(720));

    let retain = snapshots.retain.as_ref().unwrap();
    assert_eq!(retain.default, Some(10));

    let objects = retain.objects.as_ref().unwrap();
    assert_eq!(objects.get("person"), Some(&30));
    assert_eq!(objects.get("car"), Some(&14));
}

#[test]
fn test_comment_line_numbers() {
    let yaml_content = r#"# Line 1 comment
version: "0.13"
# Line 3 comment
cameras:
  front_door:  # Line 5 inline comment
    ffmpeg:
      inputs:
        - path: rtsp://test  # Line 8 inline comment
          roles: [detect]
    detect:
      enabled: true
# Line 12 comment

detectors:
  cpu1:
    model: yolov8n  # Line 16 inline comment
"#;

    let parser = ConfigParser::new();
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    assert!(result.is_ok());
    let parse_result = result.unwrap();
    let config = parse_result.config;

    assert!(
        config.metadata.comments.len() >= 5,
        "Should have at least 5 comments"
    );

    // Check comment line numbers
    let comment_lines: Vec<usize> = config
        .metadata
        .comments
        .iter()
        .map(|c| c.line_number)
        .collect();

    assert!(comment_lines.contains(&1), "Should have comment on line 1");
    assert!(comment_lines.contains(&3), "Should have comment on line 3");
    assert!(comment_lines.contains(&5), "Should have comment on line 5");
}

#[test]
fn test_manual_edit_detection() {
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
      # TODO: Configure detection zones
      # MANUAL: This was manually added by user

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    assert!(result.is_ok());
    let parse_result = result.unwrap();
    let config = parse_result.config;

    // Should detect manual edits due to TODO and MANUAL markers
    assert!(
        config.metadata.has_manual_edits,
        "Should detect manual edits"
    );
    assert!(
        !config.metadata.manual_edit_lines.is_empty(),
        "Should have manual edit line numbers"
    );
}

#[test]
fn test_parse_config_with_hwaccel_args() {
    let yaml_content = r#"
version: "0.13"

ffmpeg:
  hwaccel_args:
    - -hwaccel
    - vaapi
    - -hwaccel_device
    - /dev/dri/renderD128

cameras:
  front_door:
    ffmpeg:
      inputs:
        - path: rtsp://test
          roles: [detect]
          hwaccel_args:
            - -hwaccel
            - cuda
    detect:
      enabled: true

detectors:
  cpu1:
    model: yolov8n
"#;

    let parser = ConfigParser::new();
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    assert!(result.is_ok());
    let parse_result = result.unwrap();
    let config = parse_result.config;

    // Check global ffmpeg config
    assert!(config.global_config.ffmpeg.is_some());
    let ffmpeg = config.global_config.ffmpeg.as_ref().unwrap();
    assert!(ffmpeg.hwaccel_args.is_some());
    let global_hwaccel = ffmpeg.hwaccel_args.as_ref().unwrap();
    assert_eq!(global_hwaccel.len(), 4);
    assert_eq!(global_hwaccel[0], "-hwaccel");
    assert_eq!(global_hwaccel[1], "vaapi");

    // Check camera-specific hwaccel args
    let camera = config.cameras.get("front_door").unwrap();
    assert!(!camera.ffmpeg_inputs.is_empty());
    let input = &camera.ffmpeg_inputs[0];
    assert!(input.hwaccel_args.is_some());
    let camera_hwaccel = input.hwaccel_args.as_ref().unwrap();
    assert_eq!(camera_hwaccel.len(), 2);
    assert_eq!(camera_hwaccel[0], "-hwaccel");
    assert_eq!(camera_hwaccel[1], "cuda");
}

#[test]
fn test_raw_content_preservation() {
    let yaml_content = r#"version: "0.13"
# Important comment
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
    let result = parser.parse_content(yaml_content.to_string(), "test.yaml".to_string());

    assert!(result.is_ok());
    let parse_result = result.unwrap();
    let config = parse_result.config;

    // Raw content should be preserved exactly
    assert_eq!(
        config.raw_content, yaml_content,
        "Raw content should be preserved exactly"
    );
    assert!(
        config.raw_content.contains("# Important comment"),
        "Comments should be in raw content"
    );
}

#[test]
fn test_default_template() {
    let config = ConfigParser::get_default_template();

    assert!(config.raw_content.contains("version"));
    assert!(config.raw_content.contains("cameras"));
    assert!(config.raw_content.contains("detectors"));
    assert!(config
        .raw_content
        .contains("# Frigate Configuration Template"));

    // Verify it parses correctly
    assert_eq!(
        config.cameras.len(),
        0,
        "Default template should have no cameras configured"
    );
    assert_eq!(
        config.detectors.len(),
        1,
        "Default template should have CPU detector"
    );
}
