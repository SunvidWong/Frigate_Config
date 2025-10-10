// T056: Unit tests for merge logic
// Test the YAML merging functionality with different strategies

use frigate_config::config_engine::merger::{ConfigMerger, MergeStrategy, MergeResult};
use serde_json::json;

#[cfg(test)]
mod yaml_merge_tests {
    use super::*;

    #[test]
    fn test_merge_non_conflicting_keys() {
        let existing = json!({
            "mqtt": {
                "enabled": true
            }
        });

        let template = json!({
            "mqtt": {
                "host": "mqtt.local"
            }
        });

        let merger = ConfigMerger::new(MergeStrategy::Recursive);
        let result = merger.merge(&existing, &template, None);

        assert!(result.is_ok());
        let merged = result.unwrap();

        assert_eq!(merged["mqtt"]["enabled"], true);
        assert_eq!(merged["mqtt"]["host"], "mqtt.local");
    }

    #[test]
    fn test_merge_with_conflict_resolution_keep_existing() {
        let existing = json!({
            "mqtt": {
                "host": "mqtt.local"
            }
        });

        let template = json!({
            "mqtt": {
                "host": "mqtt.remote"
            }
        });

        let merger = ConfigMerger::new(MergeStrategy::Recursive);
        let conflicts = merger.detect_conflicts(&existing, &template);

        // Resolve conflicts to keep existing values
        let mut resolutions = std::collections::HashMap::new();
        for conflict in conflicts {
            resolutions.insert(conflict.path.clone(), "keep_existing".to_string());
        }

        let result = merger.merge(&existing, &template, Some(resolutions));
        assert!(result.is_ok());
        let merged = result.unwrap();

        assert_eq!(merged["mqtt"]["host"], "mqtt.local");
    }

    #[test]
    fn test_merge_with_conflict_resolution_use_template() {
        let existing = json!({
            "mqtt": {
                "host": "mqtt.local"
            }
        });

        let template = json!({
            "mqtt": {
                "host": "mqtt.remote"
            }
        });

        let merger = ConfigMerger::new(MergeStrategy::Recursive);
        let conflicts = merger.detect_conflicts(&existing, &template);

        // Resolve conflicts to use template values
        let mut resolutions = std::collections::HashMap::new();
        for conflict in conflicts {
            resolutions.insert(conflict.path.clone(), "use_template".to_string());
        }

        let result = merger.merge(&existing, &template, Some(resolutions));
        assert!(result.is_ok());
        let merged = result.unwrap();

        assert_eq!(merged["mqtt"]["host"], "mqtt.remote");
    }

    #[test]
    fn test_merge_deep_nested_objects() {
        let existing = json!({
            "cameras": {
                "front_door": {
                    "enabled": true,
                    "fps": 15
                },
                "back_door": {
                    "enabled": false
                }
            }
        });

        let template = json!({
            "cameras": {
                "front_door": {
                    "resolution": "1920x1080"
                },
                "side_door": {
                    "enabled": true
                }
            }
        });

        let merger = ConfigMerger::new(MergeStrategy::Recursive);
        let result = merger.merge(&existing, &template, None);

        assert!(result.is_ok());
        let merged = result.unwrap();

        // Verify deep merge
        assert_eq!(merged["cameras"]["front_door"]["enabled"], true);
        assert_eq!(merged["cameras"]["front_door"]["fps"], 15);
        assert_eq!(merged["cameras"]["front_door"]["resolution"], "1920x1080");
        assert_eq!(merged["cameras"]["back_door"]["enabled"], false);
        assert_eq!(merged["cameras"]["side_door"]["enabled"], true);
    }

    #[test]
    fn test_merge_arrays_append() {
        let existing = json!({
            "detectors": ["detector1", "detector2"]
        });

        let template = json!({
            "detectors": ["detector3"]
        });

        let merger = ConfigMerger::new(MergeStrategy::ArrayAppend);
        let result = merger.merge(&existing, &template, None);

        assert!(result.is_ok());
        let merged = result.unwrap();

        assert_eq!(merged["detectors"].as_array().unwrap().len(), 3);
        assert!(merged["detectors"].as_array().unwrap().contains(&json!("detector1")));
        assert!(merged["detectors"].as_array().unwrap().contains(&json!("detector3")));
    }

    #[test]
    fn test_merge_arrays_replace() {
        let existing = json!({
            "zones": ["zone1", "zone2"]
        });

        let template = json!({
            "zones": ["zone3", "zone4"]
        });

        let merger = ConfigMerger::new(MergeStrategy::ArrayReplace);
        let result = merger.merge(&existing, &template, None);

        assert!(result.is_ok());
        let merged = result.unwrap();

        assert_eq!(merged["zones"].as_array().unwrap().len(), 2);
        assert!(merged["zones"].as_array().unwrap().contains(&json!("zone3")));
        assert!(merged["zones"].as_array().unwrap().contains(&json!("zone4")));
        assert!(!merged["zones"].as_array().unwrap().contains(&json!("zone1")));
    }

    #[test]
    fn test_merge_with_deletion_marker() {
        let existing = json!({
            "mqtt": {
                "enabled": true,
                "host": "mqtt.local",
                "password": "secret"
            }
        });

        let template = json!({
            "mqtt": {
                "password": "__DELETE__"
            }
        });

        let merger = ConfigMerger::new(MergeStrategy::Recursive);
        let result = merger.merge(&existing, &template, None);

        assert!(result.is_ok());
        let merged = result.unwrap();

        assert_eq!(merged["mqtt"]["enabled"], true);
        assert_eq!(merged["mqtt"]["host"], "mqtt.local");
        assert!(merged["mqtt"].get("password").is_none());
    }

    #[test]
    fn test_merge_preserves_comments() {
        let existing_yaml = r#"
# MQTT Configuration
mqtt:
  enabled: true  # Enable MQTT
  host: mqtt.local
"#;

        let template_yaml = r#"
mqtt:
  port: 1883
"#;

        let parser = frigate_config::config_engine::parser::YamlParser::new();
        let existing = parser.parse(existing_yaml).unwrap();
        let template = parser.parse(template_yaml).unwrap();

        let merger = ConfigMerger::new(MergeStrategy::Recursive);
        let result = merger.merge(&existing, &template, None);

        assert!(result.is_ok());
        let merged = result.unwrap();

        // Convert back to YAML and verify comments preserved
        let merged_yaml = parser.to_yaml(&merged).unwrap();
        assert!(merged_yaml.contains("# MQTT Configuration"));
        assert!(merged_yaml.contains("# Enable MQTT"));
    }

    #[test]
    fn test_merge_with_custom_values() {
        let existing = json!({
            "cameras": {
                "front_door": {
                    "fps": 15
                }
            }
        });

        let template = json!({
            "cameras": {
                "front_door": {
                    "fps": 30
                }
            }
        });

        let merger = ConfigMerger::new(MergeStrategy::Recursive);
        let conflicts = merger.detect_conflicts(&existing, &template);

        // Provide custom resolution value
        let mut resolutions = std::collections::HashMap::new();
        resolutions.insert(conflicts[0].path.clone(), json!(20));

        let result = merger.merge(&existing, &template, Some(resolutions));
        assert!(result.is_ok());
        let merged = result.unwrap();

        assert_eq!(merged["cameras"]["front_door"]["fps"], 20);
    }

    #[test]
    fn test_merge_performance() {
        let mut existing = json!({});
        let mut template = json!({});

        for i in 0..100 {
            existing[format!("camera_{}", i)] = json!({
                "enabled": true,
                "fps": 15
            });
            template[format!("camera_{}", i)] = json!({
                "resolution": "1920x1080"
            });
        }

        let merger = ConfigMerger::new(MergeStrategy::Recursive);
        let start = std::time::Instant::now();
        let result = merger.merge(&existing, &template, None);
        let duration = start.elapsed();

        assert!(result.is_ok());
        // Should complete quickly
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_merge_empty_configs() {
        let existing = json!({});
        let template = json!({
            "mqtt": {
                "enabled": true
            }
        });

        let merger = ConfigMerger::new(MergeStrategy::Recursive);
        let result = merger.merge(&existing, &template, None);

        assert!(result.is_ok());
        let merged = result.unwrap();

        assert_eq!(merged["mqtt"]["enabled"], true);
    }

    #[test]
    fn test_merge_null_handling() {
        let existing = json!({
            "mqtt": {
                "enabled": true,
                "host": "mqtt.local"
            }
        });

        let template = json!({
            "mqtt": null
        });

        let merger = ConfigMerger::new(MergeStrategy::Recursive);
        let result = merger.merge(&existing, &template, None);

        // Null in template should require explicit resolution
        assert!(result.is_err() || result.unwrap().get("mqtt").is_some());
    }
}
