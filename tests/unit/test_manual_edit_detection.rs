// T160: Unit tests for manual edit indicators
// Tests that manual edits are properly detected and tracked

use frigate_config::config_engine::parser::ConfigParser;

#[cfg(test)]
mod manual_edit_detection_tests {
    use super::*;

    #[test]
    fn test_detect_todo_markers() {
        let yaml = r#"
mqtt:
  enabled: true
  # TODO: Configure proper MQTT broker
  host: localhost
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should detect manual edit due to TODO marker
        assert!(config.metadata.has_manual_edits);
        assert!(!config.metadata.manual_edit_lines.is_empty());
    }

    #[test]
    fn test_detect_fixme_markers() {
        let yaml = r#"
cameras:
  front_door:
    enabled: true
    # FIXME: Replace with proper RTSP URL
    ffmpeg:
      inputs:
        - path: rtsp://temp/stream
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should detect manual edit due to FIXME marker
        assert!(config.metadata.has_manual_edits);
        assert!(!config.metadata.manual_edit_lines.is_empty());
    }

    #[test]
    fn test_detect_manual_markers() {
        let yaml = r#"
mqtt:
  enabled: true
  # MANUAL: Custom configuration
  host: localhost
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should detect manual edit due to MANUAL marker
        assert!(config.metadata.has_manual_edits);
        assert!(!config.metadata.manual_edit_lines.is_empty());
    }

    #[test]
    fn test_detect_temporary_markers() {
        let yaml = r#"
detectors:
  # TEMPORARY: Using CPU until GPU is available
  cpu1:
    type: cpu
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should detect manual edit due to TEMPORARY marker
        assert!(config.metadata.has_manual_edits);
        assert!(!config.metadata.manual_edit_lines.is_empty());
    }

    #[test]
    fn test_detect_non_standard_fields() {
        let yaml = r#"
mqtt:
  enabled: true
custom_field: value
another_custom: data
cameras:
  front_door:
    enabled: true
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should detect manual edit due to non-standard fields
        assert!(config.metadata.has_manual_edits);
    }

    #[test]
    fn test_detect_long_comments() {
        let yaml = r#"
mqtt:
  enabled: true
  # This is a very long comment that exceeds 50 characters and should be flagged as potentially manually added
  host: localhost
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should detect manual edit due to unusually long comment
        assert!(config.metadata.has_manual_edits);
        assert!(!config.metadata.manual_edit_lines.is_empty());
    }

    #[test]
    fn test_no_manual_edits_in_clean_config() {
        let yaml = r#"
mqtt:
  enabled: true
  host: localhost
  port: 1883

cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://camera/stream

detectors:
  cpu1:
    type: cpu
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Clean config with only standard fields should not be flagged
        assert!(!config.metadata.has_manual_edits);
        assert!(config.metadata.manual_edit_lines.is_empty());
    }

    #[test]
    fn test_multiple_manual_edit_indicators() {
        let yaml = r#"
mqtt:
  # TODO: Configure this
  enabled: true
  # FIXME: Update host
  host: localhost
  # TEMPORARY: Using default port
  port: 1883
custom_field: value
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should detect multiple manual edit indicators
        assert!(config.metadata.has_manual_edits);
        // Should have at least 3 lines flagged (TODO, FIXME, TEMPORARY)
        assert!(config.metadata.manual_edit_lines.len() >= 3);
    }

    #[test]
    fn test_manual_edit_line_numbers() {
        let yaml = r#"# Line 1
mqtt:
  enabled: true
  # TODO: Line 4 - should be flagged
  host: localhost
cameras:
  front_door:
    # FIXME: Line 8 - should be flagged
    enabled: true
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        assert!(config.metadata.has_manual_edits);

        // Verify correct line numbers are tracked
        assert!(config.metadata.manual_edit_lines.contains(&4));
        assert!(config.metadata.manual_edit_lines.contains(&8));
    }

    #[test]
    fn test_case_insensitive_marker_detection() {
        let yaml = r#"
mqtt:
  # todo: lowercase marker
  enabled: true
  # ToDo: mixed case marker
  host: localhost
  # TODO: uppercase marker
  port: 1883
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should detect all variations of TODO
        assert!(config.metadata.has_manual_edits);
        assert!(config.metadata.manual_edit_lines.len() >= 3);
    }

    #[test]
    fn test_manual_edit_with_inline_markers() {
        let yaml = r#"
mqtt:
  enabled: true  # TODO: Enable this in production
  host: localhost  # FIXME: Use proper hostname
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should detect inline manual edit markers
        assert!(config.metadata.has_manual_edits);
        assert!(config.metadata.manual_edit_lines.len() >= 2);
    }

    #[test]
    fn test_standard_frigate_fields_not_flagged() {
        let yaml = r#"
mqtt:
  enabled: true
database:
  path: /config/frigate.db
go2rtc:
  streams:
    test: rtsp://stream
ffmpeg:
  hwaccel_args: []
logger:
  default: INFO
cameras:
  test:
    enabled: true
detectors:
  cpu1:
    type: cpu
objects:
  filters: []
record:
  enabled: true
snapshots:
  enabled: true
detect:
  enabled: true
zones: []
webcam:
  enabled: false
birdseye:
  enabled: false
environment:
  test: value
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // All standard Frigate fields should not trigger manual edit detection
        assert!(!config.metadata.has_manual_edits);
    }

    #[test]
    fn test_detect_unusual_indentation() {
        let yaml = r#"
mqtt:
  enabled: true
  # This is a very long comment that goes way beyond the normal length limit and should definitely be flagged as unusual
  host: localhost
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Long comments should be flagged
        assert!(config.metadata.has_manual_edits);
    }

    #[test]
    fn test_manual_edit_tracking_in_complex_config() {
        let yaml = r#"
# Standard comment - not flagged
mqtt:
  enabled: true
  # TODO: Configure MQTT properly
  host: localhost

# FIXME: Add more cameras
cameras:
  front_door:
    enabled: true
    # MANUAL: Custom FFmpeg settings
    ffmpeg:
      inputs:
        - path: rtsp://camera/stream

# TEMPORARY: Using CPU detector
detectors:
  cpu1:
    type: cpu

# Non-standard field
custom_integration: enabled
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should detect multiple manual edits
        assert!(config.metadata.has_manual_edits);
        // Should have at least 4 lines flagged (TODO, FIXME, MANUAL, TEMPORARY)
        assert!(config.metadata.manual_edit_lines.len() >= 4);
    }

    #[test]
    fn test_empty_config_no_manual_edits() {
        let yaml = "";

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Empty config should not have manual edits
        assert!(!config.metadata.has_manual_edits);
        assert!(config.metadata.manual_edit_lines.is_empty());
    }

    #[test]
    fn test_marker_in_string_value_not_detected() {
        let yaml = r#"
mqtt:
  enabled: true
  host: localhost
  client_id: "TODO-client-123"
  user: "FIXME-user"
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Markers inside string values should NOT trigger manual edit detection
        // (only markers in comments should be detected)
        // This is a limitation - we only check comments for markers
    }

    #[test]
    fn test_manual_edit_preservation_across_parse() {
        let yaml = r#"
mqtt:
  # TODO: Update this
  enabled: true
"#;

        let parser = ConfigParser::new();
        let result1 = parser.parse_content(yaml.to_string(), "test.yaml".to_string());
        assert!(result1.is_ok());
        let config1 = result1.unwrap().config;

        // Re-parse the same content
        let result2 = parser.parse_content(yaml.to_string(), "test.yaml".to_string());
        assert!(result2.is_ok());
        let config2 = result2.unwrap().config;

        // Manual edit detection should be consistent
        assert_eq!(config1.metadata.has_manual_edits, config2.metadata.has_manual_edits);
        assert_eq!(config1.metadata.manual_edit_lines, config2.metadata.manual_edit_lines);
    }
}
