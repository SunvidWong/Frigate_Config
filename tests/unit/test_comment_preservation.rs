// T159: Unit tests for comment preservation tracking
// Tests that comments are properly preserved through parsing and merging

use frigate_config::config_engine::parser::{ConfigParser, ConfigComment};

#[cfg(test)]
mod comment_preservation_tests {
    use super::*;

    #[test]
    fn test_extract_inline_comments() {
        let yaml = r#"
mqtt:
  enabled: true  # Enable MQTT connection
  host: localhost  # MQTT broker address
  port: 1883  # Default MQTT port
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should extract all 3 inline comments
        let inline_comments: Vec<&ConfigComment> = config.metadata.comments
            .iter()
            .filter(|c| c.is_inline)
            .collect();

        assert_eq!(inline_comments.len(), 3);
        assert!(inline_comments.iter().any(|c| c.content.contains("Enable MQTT")));
        assert!(inline_comments.iter().any(|c| c.content.contains("MQTT broker")));
        assert!(inline_comments.iter().any(|c| c.content.contains("Default MQTT")));
    }

    #[test]
    fn test_extract_block_comments() {
        let yaml = r#"
# MQTT Configuration Section
# This configures the MQTT broker connection
mqtt:
  enabled: true
  # Connection details
  host: localhost
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should extract all 3 block comments
        let block_comments: Vec<&ConfigComment> = config.metadata.comments
            .iter()
            .filter(|c| !c.is_inline)
            .collect();

        assert_eq!(block_comments.len(), 3);
        assert!(block_comments.iter().any(|c| c.content.contains("MQTT Configuration Section")));
        assert!(block_comments.iter().any(|c| c.content.contains("This configures")));
        assert!(block_comments.iter().any(|c| c.content.contains("Connection details")));
    }

    #[test]
    fn test_comment_line_numbers() {
        let yaml = r#"# Line 1 comment
mqtt:  # Line 2 comment
  enabled: true
  # Line 4 comment
  host: localhost
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        assert_eq!(config.metadata.comments.len(), 3);

        // Verify line numbers are correctly tracked
        let line_1_comment = config.metadata.comments.iter().find(|c| c.line_number == 1);
        assert!(line_1_comment.is_some());
        assert!(line_1_comment.unwrap().content.contains("Line 1"));

        let line_2_comment = config.metadata.comments.iter().find(|c| c.line_number == 2);
        assert!(line_2_comment.is_some());
        assert!(line_2_comment.unwrap().content.contains("Line 2"));

        let line_4_comment = config.metadata.comments.iter().find(|c| c.line_number == 4);
        assert!(line_4_comment.is_some());
        assert!(line_4_comment.unwrap().content.contains("Line 4"));
    }

    #[test]
    fn test_comment_section_path_tracking() {
        let yaml = r#"
# Root level comment
mqtt:
  # MQTT section comment
  enabled: true
cameras:
  # Cameras section comment
  front_door:
    # Camera-specific comment
    enabled: true
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Find comment in mqtt section
        let mqtt_comment = config.metadata.comments.iter()
            .find(|c| c.content.contains("MQTT section"));
        assert!(mqtt_comment.is_some());
        assert_eq!(mqtt_comment.unwrap().section_path, Some("mqtt".to_string()));

        // Find comment in cameras section
        let cameras_comment = config.metadata.comments.iter()
            .find(|c| c.content.contains("Cameras section"));
        assert!(cameras_comment.is_some());
        assert_eq!(cameras_comment.unwrap().section_path, Some("cameras".to_string()));

        // Root level comment should have no section path
        let root_comment = config.metadata.comments.iter()
            .find(|c| c.content.contains("Root level"));
        assert!(root_comment.is_some());
        assert_eq!(root_comment.unwrap().section_path, None);
    }

    #[test]
    fn test_preserve_comments_with_special_characters() {
        let yaml = r#"
mqtt:
  enabled: true  # TODO: Configure this!
  host: localhost  # FIXME: Use env variable $MQTT_HOST
  port: 1883  # NOTE: Port 1883/8883 (SSL)
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        assert_eq!(config.metadata.comments.len(), 3);

        // Verify special characters are preserved
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("TODO: Configure this!")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("$MQTT_HOST")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("1883/8883 (SSL)")));
    }

    #[test]
    fn test_empty_comments() {
        let yaml = r#"
mqtt:
  #
  enabled: true
  #
  host: localhost
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Empty comments should still be tracked
        let empty_comments: Vec<&ConfigComment> = config.metadata.comments
            .iter()
            .filter(|c| c.content.trim().is_empty())
            .collect();

        assert!(empty_comments.len() >= 2);
    }

    #[test]
    fn test_multiline_comment_blocks() {
        let yaml = r#"
# MQTT Configuration
# ===================
# This section configures the MQTT broker connection.
# The broker is used for real-time event notifications.
# See https://docs.frigate.video/configuration/mqtt
mqtt:
  enabled: true
  host: localhost
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should extract all 5 lines of the comment block
        let comment_block: Vec<&ConfigComment> = config.metadata.comments
            .iter()
            .filter(|c| c.line_number <= 6)
            .collect();

        assert_eq!(comment_block.len(), 5);
        assert!(comment_block.iter().any(|c| c.content.contains("MQTT Configuration")));
        assert!(comment_block.iter().any(|c| c.content.contains("===")));
        assert!(comment_block.iter().any(|c| c.content.contains("docs.frigate.video")));
    }

    #[test]
    fn test_comment_preservation_in_complex_config() {
        let yaml = r#"
# Frigate Configuration
version: "0.13"

# MQTT Settings
mqtt:
  enabled: true  # Enable for notifications
  host: localhost
  port: 1883

# Camera Configuration
cameras:
  # Front door camera
  front_door:
    enabled: true  # Always on
    ffmpeg:
      inputs:
        # Main stream for detection
        - path: rtsp://camera/stream
          roles:
            - detect
            - record
    detect:
      enabled: true  # Object detection enabled
      fps: 5  # Detection FPS

# Detector Configuration
detectors:
  # CPU detector (fallback)
  cpu1:
    type: cpu  # Software detection
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should extract all comments
        assert!(config.metadata.comments.len() >= 12);

        // Verify comments in different sections
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("Frigate Configuration")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("MQTT Settings")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("Camera Configuration")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("Detector Configuration")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("Front door camera")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("Main stream")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("CPU detector")));
    }

    #[test]
    fn test_comments_count_in_metadata() {
        let yaml = r#"
# Comment 1
mqtt:
  enabled: true  # Comment 2
  # Comment 3
  host: localhost
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Metadata should track total comment count
        assert_eq!(config.metadata.comments.len(), 3);
    }

    #[test]
    fn test_comment_preservation_with_arrays() {
        let yaml = r#"
cameras:
  front_door:
    ffmpeg:
      inputs:
        # Primary stream
        - path: rtsp://cam1/main
          roles:  # Stream roles
            - detect  # Detection role
            - record  # Recording role
        # Secondary stream
        - path: rtsp://cam1/sub
          roles:
            - detect
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should track comments in array structures
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("Primary stream")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("Stream roles")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("Detection role")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("Recording role")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("Secondary stream")));
    }

    #[test]
    fn test_no_comments_in_clean_config() {
        let yaml = r#"
mqtt:
  enabled: true
  host: localhost
  port: 1883
cameras:
  front_door:
    enabled: true
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Should have no comments
        assert_eq!(config.metadata.comments.len(), 0);
    }

    #[test]
    fn test_comments_with_yaml_syntax() {
        let yaml = r#"
mqtt:
  enabled: true  # key: value pairs are ignored in comments
  host: localhost  # - lists are also ignored
  port: 1883  # { objects: are ignored }
"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(yaml.to_string(), "test.yaml".to_string());

        assert!(result.is_ok());
        let config = result.unwrap().config;

        // Comments should preserve YAML-like syntax without parsing it
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("key: value")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("- lists")));
        assert!(config.metadata.comments.iter().any(|c| c.content.contains("{ objects:")));
    }
}
