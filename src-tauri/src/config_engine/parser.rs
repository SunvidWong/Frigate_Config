// YAML parser with comment preservation and Frigate-specific handling
// T053: Configuration file loader implementation

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use std::path::Path;
use tracing::{error, info, warn};

/// Parsed Frigate configuration with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrigateConfig {
    /// Original YAML content (preserves comments and formatting)
    pub raw_content: String,
    /// Parsed YAML structure
    pub yaml: Value,
    /// File metadata
    pub metadata: ConfigMetadata,
    /// Parsed sections for easier access
    pub cameras: HashMap<String, CameraConfig>,
    pub detectors: HashMap<String, DetectorConfig>,
    pub global_config: GlobalConfig,
}

/// Configuration file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub file_path: String,
    pub file_size: u64,
    pub last_modified: String,
    pub format_version: Option<String>,
    pub has_manual_edits: bool,
    /// Line numbers of manually edited sections
    pub manual_edit_lines: Vec<usize>,
    /// Comments found in the file
    pub comments: Vec<ConfigComment>,
}

/// Configuration comment with line position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigComment {
    pub line_number: usize,
    pub content: String,
    pub is_inline: bool,
    pub section_path: Option<String>,
}

/// Camera configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    pub name: String,
    pub enabled: bool,
    pub ffmpeg_inputs: Vec<FfmpegInput>,
    pub detect: DetectConfig,
    pub record: Option<RecordConfig>,
    pub snapshots: Option<SnapshotsConfig>,
    pub objects: Option<ObjectsConfig>,
}

/// FFMPEG input configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfmpegInput {
    pub path: String,
    pub roles: Vec<String>,
    pub input_args: Option<Vec<String>>,
    pub hwaccel_args: Option<Vec<String>>,
}

/// Detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectConfig {
    pub enabled: bool,
    pub max_labels: Option<u32>,
    pub fps: Option<f32>,
}

/// Recording configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordConfig {
    pub enabled: bool,
    pub retain_days: Option<u32>,
    pub events: Option<EventRecordConfig>,
}

/// Event recording configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecordConfig {
    pub enabled: bool,
    pub pre_capture: Option<u32>,
    pub post_capture: Option<u32>,
}

/// Snapshots configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotsConfig {
    pub enabled: bool,
    pub timestamp: bool,
    pub bounding_box: bool,
    pub crop: bool,
    pub height: Option<u32>,
    pub retain: Option<SnapshotsRetainConfig>,
}

/// Snapshot retention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotsRetainConfig {
    pub default: Option<u32>,
    pub objects: Option<HashMap<String, u32>>,
}

/// Objects configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectsConfig {
    pub filters: Option<Vec<ObjectFilter>>,
}

/// Object filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectFilter {
    pub object: String,
    pub area: Option<Vec<Vec<f32>>>,
}

/// Detector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorConfig {
    pub name: String,
    pub model: String,
    pub model_path: Option<String>,
    pub labelmap_path: Option<String>,
    pub input_tensor: Option<String>,
    pub input_pixel_format: Option<String>,
    pub device: Option<String>,
}

/// Global Frigate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub mqtt: Option<MqttConfig>,
    pub database: Option<DatabaseConfig>,
    pub go2rtc: Option<Go2rtcConfig>,
    pub ffmpeg: Option<FfmpegGlobalConfig>,
    pub logger: Option<LoggerConfig>,
}

/// MQTT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub topic_prefix: String,
    pub client_id: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

/// go2rtc configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Go2rtcConfig {
    pub streams: HashMap<String, Go2rtcStream>,
}

/// go2rtc stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Go2rtcStream {
    pub rtsp: String,
    pub input: Option<Vec<String>>,
}

/// Global FFMPEG configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfmpegGlobalConfig {
    pub hwaccel_args: Option<Vec<String>>,
}

/// Logger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    pub default: String,
    pub logs: HashMap<String, String>,
}

/// Configuration parser result
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub config: FrigateConfig,
    pub warnings: Vec<ParseWarning>,
    pub errors: Vec<ParseError>,
}

/// Parse warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseWarning {
    pub message: String,
    pub line_number: Option<usize>,
    pub suggestion: Option<String>,
}

/// Parse error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    pub message: String,
    pub line_number: Option<usize>,
    pub error_type: ParseErrorType,
}

/// Types of parse errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParseErrorType {
    Syntax,
    Validation,
    MissingRequired,
    InvalidValue,
}

/// Main configuration parser
pub struct ConfigParser {
    preserve_comments: bool,
    strict_mode: bool,
}

impl ConfigParser {
    /// Create a new configuration parser
    pub fn new() -> Self {
        Self {
            preserve_comments: true,
            strict_mode: false,
        }
    }

    /// Parse a Frigate configuration file
    pub async fn parse_file<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<ParseResult, AppError> {
        let path = file_path.as_ref();
        info!("Parsing Frigate config file: {}", path.display());

        // Read file content
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| AppError::Config(format!("Failed to read config file: {}", e)))?;

        self.parse_content(content, path.to_string_lossy().to_string())
    }

    /// Parse configuration from string content
    pub fn parse_content(&self, content: String, file_path: String) -> Result<ParseResult, AppError> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Parse YAML
        let yaml_result = serde_yaml::from_str::<Value>(&content);
        let yaml = match yaml_result {
            Ok(yaml) => yaml,
            Err(e) => {
                error!("YAML parsing failed: {}", e);
                return Err(AppError::Config(format!("Invalid YAML: {}", e)));
            }
        };

        // Extract comments
        let comments = self.extract_comments(&content);

        // Detect manual edits
        let (has_manual_edits, manual_edit_lines) = self.detect_manual_edits(&content, &yaml);

        // Parse sections
        let cameras = self.parse_cameras(&yaml, &mut warnings);
        let detectors = self.parse_detectors(&yaml, &mut warnings);
        let global_config = self.parse_global_config(&yaml, &mut warnings);

        // Validate configuration
        if self.strict_mode {
            self.validate_config(&yaml, &mut warnings, &mut errors);
        }

        // Create metadata
        let metadata = ConfigMetadata {
            file_path: file_path.clone(),
            file_size: content.len() as u64,
            last_modified: chrono::Utc::now().to_rfc3339(),
            format_version: self.extract_version(&yaml),
            has_manual_edits,
            manual_edit_lines,
            comments,
        };

        let config = FrigateConfig {
            raw_content: content,
            yaml,
            metadata,
            cameras: cameras.clone(),
            detectors: detectors.clone(),
            global_config,
        };

        info!("Successfully parsed config with {} cameras and {} detectors",
              cameras.len(), detectors.len());

        Ok(ParseResult {
            config,
            warnings,
            errors,
        })
    }

    /// Extract comments from YAML content
    fn extract_comments(&self, content: &str) -> Vec<ConfigComment> {
        let mut comments = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_section = String::new();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track section path
            if trimmed.ends_with(':') && !trimmed.starts_with('#') {
                current_section = trimmed.trim_end_matches(':').to_string();
            }

            // Extract comments
            if let Some(comment_start) = line.find('#') {
                let before_comment = &line[..comment_start].trim();
                let comment_content = &line[comment_start + 1..].trim();

                let is_inline = !before_comment.is_empty();

                comments.push(ConfigComment {
                    line_number: line_num + 1,
                    content: comment_content.to_string(),
                    is_inline,
                    section_path: if current_section.is_empty() {
                        None
                    } else {
                        Some(current_section.clone())
                    },
                });
            }
        }

        comments
    }

    /// Detect manual edits by comparing with expected structure
    fn detect_manual_edits(&self, content: &str, yaml: &Value) -> (bool, Vec<usize>) {
        let mut manual_edit_lines = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Look for common manual edit indicators
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Manual edit indicators
            if trimmed.contains("TODO")
                || trimmed.contains("FIXME")
                || trimmed.contains("MANUAL")
                || trimmed.contains("TEMPORARY") {
                manual_edit_lines.push(line_num + 1);
            }

            // Check for unusual indentation (common manual edit sign)
            if trimmed.starts_with('#') && trimmed.len() > 50 {
                manual_edit_lines.push(line_num + 1);
            }
        }

        // Check for fields not in standard Frigate schema
        if let Some(mapping) = yaml.as_mapping() {
            for (key, _) in mapping {
                if let Some(key_str) = key.as_str() {
                    if !self.is_standard_frigate_field(key_str) {
                        manual_edit_lines.push(1); // Flag as manually edited
                    }
                }
            }
        }

        (!manual_edit_lines.is_empty(), manual_edit_lines)
    }

    /// Check if a field is part of standard Frigate configuration
    fn is_standard_frigate_field(&self, field: &str) -> bool {
        matches!(field,
            "cameras" | "detectors" | "mqtt" | "database" | "go2rtc" |
            "ffmpeg" | "logger" | "objects" | "record" | "snapshots" |
            "detect" | "zones" | "webcam" | "birdseye" | "environment"
        )
    }

    /// Extract version from configuration
    fn extract_version(&self, yaml: &Value) -> Option<String> {
        // Look for version in various places
        if let Some(version) = yaml.get("version").and_then(|v| v.as_str()) {
            return Some(version.to_string());
        }

        if let Some(ffmpeg) = yaml.get("ffmpeg") {
            if let Some(version) = ffmpeg.get("version").and_then(|v| v.as_str()) {
                return Some(version.to_string());
            }
        }

        None
    }

    /// Parse camera configurations
    fn parse_cameras(&self, yaml: &Value, warnings: &mut Vec<ParseWarning>) -> HashMap<String, CameraConfig> {
        let mut cameras = HashMap::new();

        if let Some(cameras_yaml) = yaml.get("cameras").and_then(|c| c.as_mapping()) {
            for (camera_key, camera_yaml) in cameras_yaml {
                if let (Some(camera_name), Some(camera_mapping)) =
                    (camera_key.as_str(), camera_yaml.as_mapping()) {

                    match self.parse_single_camera(camera_name, camera_mapping, warnings) {
                        Ok(camera) => {
                            cameras.insert(camera_name.to_string(), camera);
                        }
                        Err(e) => {
                            warn!("Failed to parse camera '{}': {}", camera_name, e);
                        }
                    }
                }
            }
        }

        cameras
    }

    /// Parse a single camera configuration
    fn parse_single_camera(
        &self,
        name: &str,
        yaml: &Mapping,
        warnings: &mut Vec<ParseWarning>
    ) -> Result<CameraConfig, String> {
        let enabled = yaml.get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let ffmpeg_inputs = self.parse_ffmpeg_inputs(yaml, warnings)?;

        let detect = self.parse_detect_config(yaml, warnings)?;

        let record = yaml.get("record")
            .and_then(|r| self.parse_record_config(r, warnings).ok());

        let snapshots = yaml.get("snapshots")
            .and_then(|s| self.parse_snapshots_config(s, warnings).ok());

        let objects = yaml.get("objects")
            .and_then(|o| self.parse_objects_config(o, warnings).ok());

        Ok(CameraConfig {
            name: name.to_string(),
            enabled,
            ffmpeg_inputs,
            detect,
            record,
            snapshots,
            objects,
        })
    }

    /// Parse FFMPEG input configurations
    fn parse_ffmpeg_inputs(
        &self,
        yaml: &Mapping,
        warnings: &mut Vec<ParseWarning>
    ) -> Result<Vec<FfmpegInput>, String> {
        let mut inputs = Vec::new();

        // First try to get ffmpeg.inputs (standard Frigate format)
        if let Some(ffmpeg_section) = yaml.get("ffmpeg").and_then(|f| f.as_mapping()) {
            if let Some(inputs_yaml) = ffmpeg_section.get("inputs").and_then(|i| i.as_sequence()) {
                for item in inputs_yaml {
                    if let Some(input_obj) = item.as_mapping() {
                        if let Ok(input) = self.parse_single_ffmpeg_input(input_obj, warnings) {
                            inputs.push(input);
                        }
                    }
                }
            }
        }

        // Fallback: try direct keys for backwards compatibility
        if inputs.is_empty() {
            let input_keys = ["input", "inputs", "ffmpeg_inputs"];

            for key in &input_keys {
                if let Some(input_yaml) = yaml.get(Value::String(key.to_string())) {
                    match input_yaml {
                        Value::String(path) => {
                            inputs.push(FfmpegInput {
                                path: path.clone(),
                                roles: vec!["detect".to_string(), "record".to_string()],
                                input_args: None,
                                hwaccel_args: None,
                            });
                        }
                        Value::Sequence(seq) => {
                            for item in seq {
                                if let Some(input_obj) = item.as_mapping() {
                                    if let Ok(input) = self.parse_single_ffmpeg_input(input_obj, warnings) {
                                        inputs.push(input);
                                    }
                                }
                            }
                        }
                        Value::Mapping(mapping) => {
                            if let Ok(input) = self.parse_single_ffmpeg_input(mapping, warnings) {
                                inputs.push(input);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if inputs.is_empty() {
            return Err("No valid ffmpeg inputs found".to_string());
        }

        Ok(inputs)
    }

    /// Parse a single FFMPEG input
    fn parse_single_ffmpeg_input(
        &self,
        yaml: &Mapping,
        _warnings: &mut Vec<ParseWarning>
    ) -> Result<FfmpegInput, String> {
        let path = yaml.get("path")
            .or_else(|| yaml.get("input"))
            .and_then(|p| p.as_str())
            .ok_or("Missing path for ffmpeg input")?
            .to_string();

        let roles = yaml.get("roles")
            .and_then(|r| r.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_else(|| vec!["detect".to_string(), "record".to_string()]);

        let input_args = yaml.get("input_args")
            .and_then(|args| args.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        let hwaccel_args = yaml.get("hwaccel_args")
            .and_then(|args| args.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        Ok(FfmpegInput {
            path,
            roles,
            input_args,
            hwaccel_args,
        })
    }

    /// Parse detection configuration
    fn parse_detect_config(
        &self,
        yaml: &Mapping,
        _warnings: &mut Vec<ParseWarning>
    ) -> Result<DetectConfig, String> {
        let detect_yaml = yaml.get("detect")
            .and_then(|d| d.as_mapping())
            .ok_or("Missing detect configuration")?;

        let enabled = detect_yaml.get("enabled")
            .and_then(|e| e.as_bool())
            .unwrap_or(true);

        let max_labels = detect_yaml.get("max_labels")
            .and_then(|m| m.as_u64())
            .map(|m| m as u32);

        let fps = detect_yaml.get("fps")
            .and_then(|f| f.as_f64())
            .map(|f| f as f32);

        Ok(DetectConfig {
            enabled,
            max_labels,
            fps,
        })
    }

    /// Parse recording configuration
    fn parse_record_config(
        &self,
        yaml: &Value,
        _warnings: &mut Vec<ParseWarning>
    ) -> Result<RecordConfig, String> {
        let record_yaml = yaml.as_mapping()
            .ok_or("Record config must be a mapping")?;

        let enabled = record_yaml.get("enabled")
            .and_then(|e| e.as_bool())
            .unwrap_or(true);

        let retain_days = record_yaml.get("retain_days")
            .and_then(|r| r.as_u64())
            .map(|r| r as u32);

        let events = record_yaml.get("events")
            .and_then(|e| self.parse_event_record_config(e));

        Ok(RecordConfig {
            enabled,
            retain_days,
            events,
        })
    }

    /// Parse event recording configuration
    fn parse_event_record_config(
        &self,
        yaml: &Value
    ) -> Option<EventRecordConfig> {
        let events_yaml = yaml.as_mapping()?;

        let enabled = events_yaml.get("enabled")
            .and_then(|e| e.as_bool())
            .unwrap_or(true);

        let pre_capture = events_yaml.get("pre_capture")
            .and_then(|p| p.as_u64())
            .map(|p| p as u32);

        let post_capture = events_yaml.get("post_capture")
            .and_then(|p| p.as_u64())
            .map(|p| p as u32);

        Some(EventRecordConfig {
            enabled,
            pre_capture,
            post_capture,
        })
    }

    /// Parse snapshots configuration
    fn parse_snapshots_config(
        &self,
        yaml: &Value,
        _warnings: &mut Vec<ParseWarning>
    ) -> Result<SnapshotsConfig, String> {
        let snapshots_yaml = yaml.as_mapping()
            .ok_or("Snapshots config must be a mapping")?;

        let enabled = snapshots_yaml.get("enabled")
            .and_then(|e| e.as_bool())
            .unwrap_or(true);

        let timestamp = snapshots_yaml.get("timestamp")
            .and_then(|t| t.as_bool())
            .unwrap_or(true);

        let bounding_box = snapshots_yaml.get("bounding_box")
            .and_then(|b| b.as_bool())
            .unwrap_or(true);

        let crop = snapshots_yaml.get("crop")
            .and_then(|c| c.as_bool())
            .unwrap_or(false);

        let height = snapshots_yaml.get("height")
            .and_then(|h| h.as_u64())
            .map(|h| h as u32);

        let retain = snapshots_yaml.get("retain")
            .and_then(|r| self.parse_snapshots_retain_config(r));

        Ok(SnapshotsConfig {
            enabled,
            timestamp,
            bounding_box,
            crop,
            height,
            retain,
        })
    }

    /// Parse snapshot retention configuration
    fn parse_snapshots_retain_config(&self, yaml: &Value) -> Option<SnapshotsRetainConfig> {
        let retain_yaml = yaml.as_mapping()?;

        let default = retain_yaml.get("default")
            .and_then(|d| d.as_u64())
            .map(|d| d as u32);

        let objects = retain_yaml.get("objects")
            .and_then(|o| o.as_mapping())
            .map(|obj_map| {
                obj_map.iter()
                    .filter_map(|(k, v)| {
                        k.as_str().and_then(|key| {
                            v.as_u64().map(|val| (key.to_string(), val as u32))
                        })
                    })
                    .collect()
            });

        Some(SnapshotsRetainConfig {
            default,
            objects,
        })
    }

    /// Parse objects configuration
    fn parse_objects_config(
        &self,
        yaml: &Value,
        _warnings: &mut Vec<ParseWarning>
    ) -> Result<ObjectsConfig, String> {
        let objects_yaml = yaml.as_mapping()
            .ok_or("Objects config must be a mapping")?;

        let filters = objects_yaml.get("filters")
            .and_then(|f| f.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|item| self.parse_object_filter(item))
                    .collect()
            });

        Ok(ObjectsConfig { filters })
    }

    /// Parse object filter
    fn parse_object_filter(&self, yaml: &Value) -> Option<ObjectFilter> {
        let filter_yaml = yaml.as_mapping()?;

        let object = filter_yaml.get("object")
            .and_then(|o| o.as_str())?
            .to_string();

        let area = filter_yaml.get("area")
            .and_then(|a| a.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|coord| {
                        coord.as_sequence().map(|s| {
                            s.iter()
                                .filter_map(|c| c.as_f64())
                                .map(|c| c as f32)
                                .collect()
                        })
                    })
                    .collect()
            });

        Some(ObjectFilter { object, area })
    }

    /// Parse detector configurations
    fn parse_detectors(&self, yaml: &Value, warnings: &mut Vec<ParseWarning>) -> HashMap<String, DetectorConfig> {
        let mut detectors = HashMap::new();

        if let Some(detectors_yaml) = yaml.get("detectors").and_then(|d| d.as_mapping()) {
            for (detector_key, detector_yaml) in detectors_yaml {
                if let (Some(detector_name), Some(detector_mapping)) =
                    (detector_key.as_str(), detector_yaml.as_mapping()) {

                    match self.parse_single_detector(detector_name, detector_mapping, warnings) {
                        Ok(detector) => {
                            detectors.insert(detector_name.to_string(), detector);
                        }
                        Err(e) => {
                            warn!("Failed to parse detector '{}': {}", detector_name, e);
                        }
                    }
                }
            }
        }

        detectors
    }

    /// Parse a single detector configuration
    fn parse_single_detector(
        &self,
        name: &str,
        yaml: &Mapping,
        _warnings: &mut Vec<ParseWarning>
    ) -> Result<DetectorConfig, String> {
        // Try 'model' first, then fall back to 'type' for compatibility
        let model = yaml.get("model")
            .or_else(|| yaml.get("type"))
            .and_then(|m| m.as_str())
            .ok_or("Missing model/type for detector")?
            .to_string();

        let model_path = yaml.get("model_path")
            .and_then(|p| p.as_str())
            .map(|p| p.to_string());

        let labelmap_path = yaml.get("labelmap_path")
            .and_then(|p| p.as_str())
            .map(|p| p.to_string());

        let input_tensor = yaml.get("input_tensor")
            .and_then(|t| t.as_str())
            .map(|t| t.to_string());

        let input_pixel_format = yaml.get("input_pixel_format")
            .and_then(|f| f.as_str())
            .map(|f| f.to_string());

        // Device can be string or number, convert to string
        let device = yaml.get("device").and_then(|d| {
            if let Some(s) = d.as_str() {
                Some(s.to_string())
            } else if let Some(n) = d.as_u64() {
                Some(n.to_string())
            } else if let Some(n) = d.as_i64() {
                Some(n.to_string())
            } else {
                None
            }
        });

        Ok(DetectorConfig {
            name: name.to_string(),
            model,
            model_path,
            labelmap_path,
            input_tensor,
            input_pixel_format,
            device,
        })
    }

    /// Parse global configuration
    fn parse_global_config(&self, yaml: &Value, warnings: &mut Vec<ParseWarning>) -> GlobalConfig {
        let mqtt = yaml.get("mqtt")
            .and_then(|m| self.parse_mqtt_config(m, warnings));

        let database = yaml.get("database")
            .and_then(|d| self.parse_database_config(d, warnings));

        let go2rtc = yaml.get("go2rtc")
            .and_then(|g| self.parse_go2rtc_config(g, warnings));

        let ffmpeg = yaml.get("ffmpeg")
            .and_then(|f| self.parse_ffmpeg_global_config(f, warnings));

        let logger = yaml.get("logger")
            .and_then(|l| self.parse_logger_config(l, warnings));

        GlobalConfig {
            mqtt,
            database,
            go2rtc,
            ffmpeg,
            logger,
        }
    }

    /// Parse MQTT configuration
    fn parse_mqtt_config(&self, yaml: &Value, _warnings: &mut Vec<ParseWarning>) -> Option<MqttConfig> {
        let mqtt_yaml = yaml.as_mapping()?;

        let enabled = mqtt_yaml.get("enabled")
            .and_then(|e| e.as_bool())
            .unwrap_or(false);

        if !enabled {
            return None;
        }

        let host = mqtt_yaml.get("host")
            .and_then(|h| h.as_str())
            .unwrap_or("localhost")
            .to_string();

        let port = mqtt_yaml.get("port")
            .and_then(|p| p.as_u64())
            .map(|p| p as u16)
            .unwrap_or(1883);

        let topic_prefix = mqtt_yaml.get("topic_prefix")
            .and_then(|t| t.as_str())
            .unwrap_or("frigate")
            .to_string();

        let client_id = mqtt_yaml.get("client_id")
            .and_then(|c| c.as_str())
            .unwrap_or("frigate")
            .to_string();

        let user = mqtt_yaml.get("user")
            .and_then(|u| u.as_str())
            .map(|u| u.to_string());

        let password = mqtt_yaml.get("password")
            .and_then(|p| p.as_str())
            .map(|p| p.to_string());

        Some(MqttConfig {
            enabled,
            host,
            port,
            topic_prefix,
            client_id,
            user,
            password,
        })
    }

    /// Parse database configuration
    fn parse_database_config(&self, yaml: &Value, _warnings: &mut Vec<ParseWarning>) -> Option<DatabaseConfig> {
        let path = yaml.as_mapping()?
            .get("path")
            .and_then(|p| p.as_str())
            .map(|p| p.to_string())?;

        Some(DatabaseConfig { path })
    }

    /// Parse go2rtc configuration
    fn parse_go2rtc_config(&self, yaml: &Value, _warnings: &mut Vec<ParseWarning>) -> Option<Go2rtcConfig> {
        let go2rtc_yaml = yaml.as_mapping()?;

        let streams = go2rtc_yaml.get("streams")
            .and_then(|s| s.as_mapping())
            .map(|streams_map| {
                streams_map.iter()
                    .filter_map(|(k, v)| {
                        k.as_str().and_then(|key| {
                            self.parse_go2rtc_stream(v).map(|stream| (key.to_string(), stream))
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Some(Go2rtcConfig { streams })
    }

    /// Parse go2rtc stream
    fn parse_go2rtc_stream(&self, yaml: &Value) -> Option<Go2rtcStream> {
        let stream_yaml = yaml.as_mapping()?;

        let rtsp = stream_yaml.get("rtsp")
            .and_then(|r| r.as_str())
            .map(|r| r.to_string())?;

        let input = stream_yaml.get("input")
            .and_then(|i| i.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        Some(Go2rtcStream { rtsp, input })
    }

    /// Parse global FFMPEG configuration
    fn parse_ffmpeg_global_config(&self, yaml: &Value, _warnings: &mut Vec<ParseWarning>) -> Option<FfmpegGlobalConfig> {
        let ffmpeg_yaml = yaml.as_mapping()?;

        let hwaccel_args = ffmpeg_yaml.get("hwaccel_args")
            .and_then(|args| args.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });

        Some(FfmpegGlobalConfig { hwaccel_args })
    }

    /// Parse logger configuration
    fn parse_logger_config(&self, yaml: &Value, _warnings: &mut Vec<ParseWarning>) -> Option<LoggerConfig> {
        let logger_yaml = yaml.as_mapping()?;

        let default = logger_yaml.get("default")
            .and_then(|d| d.as_str())
            .unwrap_or("INFO")
            .to_string();

        let logs = logger_yaml.get("logs")
            .and_then(|l| l.as_mapping())
            .map(|logs_map| {
                logs_map.iter()
                    .filter_map(|(k, v)| {
                        k.as_str().and_then(|key| {
                            v.as_str().map(|val| (key.to_string(), val.to_string()))
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Some(LoggerConfig { default, logs })
    }

    /// Validate configuration and add warnings/errors
    fn validate_config(
        &self,
        yaml: &Value,
        warnings: &mut Vec<ParseWarning>,
        errors: &mut Vec<ParseError>,
    ) {
        // Check for required sections
        if yaml.get("cameras").is_none() {
            errors.push(ParseError {
                message: "Missing required 'cameras' section".to_string(),
                line_number: None,
                error_type: ParseErrorType::MissingRequired,
            });
        }

        // Check for deprecated fields
        if let Some(cameras) = yaml.get("cameras").and_then(|c| c.as_mapping()) {
            for (camera_key, camera_yaml) in cameras {
                if let (Some(camera_name), Some(camera_mapping)) =
                    (camera_key.as_str(), camera_yaml.as_mapping()) {

                    // Check for deprecated fields
                    if camera_mapping.contains_key(&Value::String("width".to_string())) {
                        warnings.push(ParseWarning {
                            message: format!("Camera '{}' uses deprecated 'width' field, use ffmpeg inputs instead", camera_name),
                            line_number: None,
                            suggestion: Some("Move width/height to ffmpeg inputs".to_string()),
                        });
                    }

                    if camera_mapping.contains_key(&Value::String("height".to_string())) {
                        warnings.push(ParseWarning {
                            message: format!("Camera '{}' uses deprecated 'height' field, use ffmpeg inputs instead", camera_name),
                            line_number: None,
                            suggestion: Some("Move width/height to ffmpeg inputs".to_string()),
                        });
                    }
                }
            }
        }

        // Validate camera configurations
        self.validate_cameras(yaml, warnings, errors);

        // Validate detector configurations
        self.validate_detectors(yaml, warnings, errors);
    }

    /// Validate camera configurations
    fn validate_cameras(
        &self,
        yaml: &Value,
        warnings: &mut Vec<ParseWarning>,
        errors: &mut Vec<ParseError>,
    ) {
        if let Some(cameras) = yaml.get("cameras").and_then(|c| c.as_mapping()) {
            for (camera_key, camera_yaml) in cameras {
                if let (Some(camera_name), Some(camera_mapping)) =
                    (camera_key.as_str(), camera_yaml.as_mapping()) {

                    // Check for ffmpeg inputs
                    let has_inputs = camera_mapping.contains_key(&Value::String("input".to_string())) ||
                                  camera_mapping.contains_key(&Value::String("inputs".to_string())) ||
                                  camera_mapping.contains_key(&Value::String("ffmpeg_inputs".to_string()));

                    if !has_inputs {
                        errors.push(ParseError {
                            message: format!("Camera '{}' has no input configuration", camera_name),
                            line_number: None,
                            error_type: ParseErrorType::MissingRequired,
                        });
                    }

                    // Check for valid roles
                    if let Some(detect_config) = camera_mapping.get("detect") {
                        if let Some(detect_enabled) = detect_config.get("enabled") {
                            if let Some(false) = detect_enabled.as_bool() {
                                warnings.push(ParseWarning {
                                    message: format!("Camera '{}' has detection disabled", camera_name),
                                    line_number: None,
                                    suggestion: Some("Consider enabling detection or remove detect section".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    /// Validate detector configurations
    fn validate_detectors(
        &self,
        yaml: &Value,
        warnings: &mut Vec<ParseWarning>,
        errors: &mut Vec<ParseError>,
    ) {
        if let Some(detectors) = yaml.get("detectors").and_then(|d| d.as_mapping()) {
            for (detector_key, detector_yaml) in detectors {
                if let (Some(detector_name), Some(detector_mapping)) =
                    (detector_key.as_str(), detector_yaml.as_mapping()) {

                    // Check for model path
                    if !detector_mapping.contains_key(&Value::String("model".to_string())) {
                        errors.push(ParseError {
                            message: format!("Detector '{}' has no model specified", detector_name),
                            line_number: None,
                            error_type: ParseErrorType::MissingRequired,
                        });
                    }

                    // Check for device assignment
                    if detector_mapping.contains_key(&Value::String("device".to_string())) {
                        if let Some(device) = detector_mapping.get("device").and_then(|d| d.as_str()) {
                            if device.is_empty() {
                                warnings.push(ParseWarning {
                                    message: format!("Detector '{}' has empty device specification", detector_name),
                                    line_number: None,
                                    suggestion: Some("Specify device (e.g., 'cpu', 'gpu:0')".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get default Frigate configuration template
    pub fn get_default_template() -> FrigateConfig {
        let default_yaml = r#"
# Frigate Configuration Template
# Generated by Frigate Config Tool

version: "0.13"

# Global MQTT settings
mqtt:
  enabled: false
  host: localhost
  port: 1883
  topic_prefix: frigate
  client_id: frigate

# Database configuration
database:
  path: /config/frigate.db

# Global FFMPEG settings
ffmpeg:
  hwaccel_args: []

# Logging configuration
logger:
  default: INFO

# Camera configurations
cameras:
  # Add your cameras here
  # front_door:
  #   ffmpeg:
  #     inputs:
  #       - path: rtsp://user:pass@192.168.1.100:554/stream
  #         roles:
  #           - detect
  #           - record
  #   detect:
  #     enabled: true
  #     fps: 5
  #   record:
  #     enabled: true
  #     retain_days: 7
  #     events:
  #       enabled: true
  #       pre_capture: 5
  #       post_capture: 10

# Detector configurations
detectors:
  cpu1:
    type: cpu

"#;

        let parser = ConfigParser::new();
        let result = parser.parse_content(default_yaml.to_string(), "template.yaml".to_string());
        result.unwrap().config
    }
}

impl Default for ConfigParser {
    fn default() -> Self {
        Self::new()
    }
}
