// 配置验证集成测试

use frigate_config_tool::config_engine::parser::parse_frigate_config;
use frigate_config_tool::config_engine::validator::validate_frigate_config;

#[test]
fn test_validate_invalid_config() {
    // 无效配置 - 缺少必要字段
    let yaml = r#"
cameras:
  test_camera:
    enabled: true
    # 缺少 ffmpeg 输入配置

detectors:
  # 空的检测器配置

mqtt:
  enabled: false
"#;

    let parse_result = parse_frigate_config(yaml).unwrap();
    let validation_result = validate_frigate_config(&parse_result.config).unwrap();

    assert!(!validation_result.is_valid, "配置应该被标记为无效");
    assert!(!validation_result.errors.is_empty(), "应该包含错误");

    // 检查是否检测到缺少的摄像头配置
    let has_camera_error = validation_result.errors.iter().any(|e|
        e.code == "missing_ffmpeg_config"
    );
    assert!(has_camera_error, "应该检测到摄像头配置错误");
}

#[test]
fn test_validate_valid_config() {
    // 有效的最小配置
    let yaml = r#"
cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://192.168.1.100:554/stream
          roles:
            - detect
            - record

detectors:
  coral:
    type: edgetpu
    device: usb

mqtt:
  enabled: true
  host: localhost
  port: 1883
"#;

    let parse_result = parse_frigate_config(yaml).unwrap();
    let validation_result = validate_frigate_config(&parse_result.config).unwrap();

    assert!(validation_result.is_valid, "配置应该被标记为有效");
    assert!(validation_result.errors.is_empty(), "不应该包含错误");
}

#[test]
fn test_validate_config_with_warnings() {
    // 有警告但有效的配置
    let yaml = r#"
cameras:
  camera1:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://192.168.1.100:554/stream
          roles:
            - detect
    fps: 30  # FPS 过高会产生警告

detectors:
  coral:
    type: edgetpu
    device: usb

mqtt:
  enabled: false  # 禁用 MQTT 会产生警告
"#;

    let parse_result = parse_frigate_config(yaml).unwrap();
    let validation_result = validate_frigate_config(&parse_result.config).unwrap();

    assert!(validation_result.is_valid, "配置应该被标记为有效");
    assert!(!validation_result.warnings.is_empty(), "应该包含警告");

    // 检查 FPS 警告
    let has_fps_warning = validation_result.warnings.iter().any(|w|
        w.message.contains("FPS") || w.code.contains("fps")
    );
    assert!(has_fps_warning, "应该包含 FPS 警告");
}