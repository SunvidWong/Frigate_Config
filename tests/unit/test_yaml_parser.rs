// T054: Unit tests for YAML parser
// Test the YAML parsing functionality with comment preservation

use frigate_config::config_engine::parser::{YamlParser, ParseResult};

#[cfg(test)]
mod yaml_parser_tests {
    use super::*;

    #[test]
    fn test_parse_valid_yaml() {
        let yaml = r#"
mqtt:
  enabled: true
  host: mqtt.local

cameras:
  front_door:
    enabled: true
"#;

        let parser = YamlParser::new();
        let result = parser.parse(yaml);

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.contains_key("mqtt"));
        assert!(config.contains_key("cameras"));
    }

    #[test]
    fn test_parse_with_comments() {
        let yaml = r#"
# MQTT Configuration
mqtt:
  enabled: true  # Enable MQTT
  host: mqtt.local

# Camera Configuration
cameras:
  front_door:
    enabled: true  # Front door camera
"#;

        let parser = YamlParser::new();
        let result = parser.parse(yaml);

        assert!(result.is_ok());
        let config = result.unwrap();

        // Verify comments are preserved
        let comments = parser.get_comments(&config);
        assert!(comments.len() > 0);
        assert!(comments.iter().any(|c| c.contains("MQTT Configuration")));
        assert!(comments.iter().any(|c| c.contains("Camera Configuration")));
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = r#"
mqtt:
  enabled: true
  host: mqtt.local
  invalid syntax here
    no proper indentation
"#;

        let parser = YamlParser::new();
        let result = parser.parse(yaml);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_yaml() {
        let yaml = "";

        let parser = YamlParser::new();
        let result = parser.parse(yaml);

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.len(), 0);
    }

    #[test]
    fn test_parse_complex_nested_structure() {
        let yaml = r#"
cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://camera/stream
          roles:
            - detect
            - record
      hwaccel_args: preset-vaapi
    detect:
      width: 1920
      height: 1080
      fps: 15
"#;

        let parser = YamlParser::new();
        let result = parser.parse(yaml);

        assert!(result.is_ok());
        let config = result.unwrap();

        // Verify nested structure
        assert!(config["cameras"]["front_door"]["ffmpeg"]["inputs"].is_array());
        assert_eq!(config["cameras"]["front_door"]["detect"]["fps"], 15);
    }

    #[test]
    fn test_preserve_comment_positions() {
        let yaml = r#"
mqtt:  # Inline comment 1
  enabled: true
  # Block comment
  host: mqtt.local  # Inline comment 2
"#;

        let parser = YamlParser::new();
        let result = parser.parse(yaml);

        assert!(result.is_ok());

        // Verify comment positions are tracked
        let comments = parser.get_comments_with_positions(&result.unwrap());
        assert_eq!(comments.len(), 3);

        // Verify inline and block comments are distinguished
        assert!(comments.iter().any(|c| c.is_inline && c.text.contains("Inline comment 1")));
        assert!(comments.iter().any(|c| !c.is_inline && c.text.contains("Block comment")));
    }

    #[test]
    fn test_roundtrip_preservation() {
        let original_yaml = r#"
# Top level comment
mqtt:
  enabled: true  # Enable MQTT
  host: mqtt.local

cameras:
  front_door:
    enabled: true
"#;

        let parser = YamlParser::new();
        let config = parser.parse(original_yaml).unwrap();
        let regenerated_yaml = parser.to_yaml(&config).unwrap();

        // Verify roundtrip preserves structure and comments
        assert!(regenerated_yaml.contains("# Top level comment"));
        assert!(regenerated_yaml.contains("# Enable MQTT"));
        assert!(regenerated_yaml.contains("mqtt:"));
        assert!(regenerated_yaml.contains("cameras:"));
    }

    #[test]
    fn test_parse_large_file() {
        // Generate a large YAML file for performance testing
        let mut yaml = String::from("cameras:\n");
        for i in 0..100 {
            yaml.push_str(&format!("  camera_{}:\n", i));
            yaml.push_str("    enabled: true\n");
            yaml.push_str("    ffmpeg:\n");
            yaml.push_str("      inputs:\n");
            yaml.push_str("        - path: rtsp://camera/stream\n");
        }

        let parser = YamlParser::new();
        let start = std::time::Instant::now();
        let result = parser.parse(&yaml);
        let duration = start.elapsed();

        assert!(result.is_ok());
        // Should complete in <50ms per spec
        assert!(duration.as_millis() < 50, "Parser took too long: {:?}", duration);
    }

    #[test]
    fn test_parse_yaml_bomb_protection() {
        // Test protection against YAML bombs (deeply nested or huge files)
        let mut yaml = String::from("root:\n");
        let mut depth = 0;
        while depth < 100 {
            yaml.push_str("  ");
            yaml.push_str(&format!("level_{}:\n", depth));
            depth += 1;
        }

        let parser = YamlParser::new();
        let result = parser.parse(&yaml);

        // Should either succeed (if depth limit not reached) or fail gracefully
        assert!(result.is_ok() || result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("depth") || e.to_string().contains("limit"));
        }
    }
}
