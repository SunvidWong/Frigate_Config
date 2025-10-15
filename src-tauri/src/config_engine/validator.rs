// Configuration validation
// Comprehensive Frigate configuration validation with hardware compatibility checking

use crate::config_engine::parser::{CameraConfig, DetectorConfig, FrigateConfig};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<ConfigurationSuggestion>,
    pub hardware_compatibility: HardwareCompatibility,
}

/// Validation error with severity and details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub severity: ValidationSeverity,
    pub line_number: Option<usize>,
    pub suggestion: Option<String>,
}

/// Validation warning for potential issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub line_number: Option<usize>,
    pub recommendation: Option<String>,
}

/// Configuration suggestion for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationSuggestion {
    pub title: String,
    pub description: String,
    pub category: SuggestionCategory,
    pub impact: ImpactLevel,
    pub changes: Vec<ConfigChange>,
}

/// Hardware compatibility assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCompatibility {
    pub gpu_compatibility: Vec<GPUCompatibility>,
    pub detector_compatibility: Vec<DetectorCompatibility>,
    pub overall_score: f32,
    pub bottlenecks: Vec<HardwareBottleneck>,
    pub recommendations: Vec<String>,
}

/// GPU compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUCompatibility {
    pub camera_name: String,
    pub gpu_required: Option<String>,
    pub gpu_available: Vec<String>,
    pub is_compatible: bool,
    pub performance_impact: PerformanceImpact,
}

/// Detector compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorCompatibility {
    pub detector_name: String,
    pub detector_type: String,
    pub hardware_required: Vec<String>,
    pub hardware_available: Vec<String>,
    pub is_compatible: bool,
    pub performance_tier: PerformanceTier,
}

/// Hardware bottleneck detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareBottleneck {
    pub component: String,
    pub current_load: f32,
    pub predicted_load: f32,
    pub impact: String,
    pub recommendation: String,
}

/// Configuration change suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub path: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub reason: String,
}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Suggestion categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Performance,
    Compatibility,
    Reliability,
    Security,
    Usability,
}

/// Performance impact levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceImpact {
    High,
    Medium,
    Low,
    None,
}

/// Impact levels for suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
    None,
}

/// Performance tiers for hardware
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformanceTier {
    High,
    Medium,
    Low,
    Incompatible,
}

/// Configuration validator
pub struct ConfigValidator {
    hardware_detection: Option<HardwareDetection>,
    validation_rules: Vec<ValidationRule>,
}

/// Hardware detection result
#[derive(Debug, Clone)]
pub struct HardwareDetection {
    pub devices: Vec<HardwareDevice>,
    pub platform: String,
    pub architecture: String,
}

/// Hardware device information
#[derive(Debug, Clone)]
pub struct HardwareDevice {
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub capabilities: Vec<String>,
    pub available: bool,
    pub in_use: bool,
}

/// Validation rule
struct ValidationRule {
    name: String,
    validator: Box<dyn Fn(&FrigateConfig) -> Vec<ValidationError>>,
}

// Note: ValidationRule cannot implement Clone because it contains a trait object
// If needed, create a new instance instead

impl std::fmt::Debug for ValidationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationRule")
            .field("name", &self.name)
            .finish()
    }
}

impl ConfigValidator {
    /// Create a new configuration validator
    pub fn new() -> Self {
        Self {
            hardware_detection: None,
            validation_rules: Self::default_rules(),
        }
    }

    /// Create validator with hardware detection
    pub fn with_hardware_detection(hardware: HardwareDetection) -> Self {
        let mut validator = Self::new();
        validator.hardware_detection = Some(hardware);
        validator
    }

    /// Validate a Frigate configuration
    pub async fn validate(&self, config: &FrigateConfig) -> Result<ValidationResult, AppError> {
        info!("Starting configuration validation");

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Run validation rules
        for rule in &self.validation_rules {
            let rule_errors = (rule.validator)(config);
            errors.extend(rule_errors);
        }

        // Validate structure
        let structure_errors = self.validate_structure(config);
        errors.extend(structure_errors);

        // Validate camera configurations
        let camera_errors = self.validate_cameras(config);
        errors.extend(camera_errors);

        // Validate detector configurations
        let detector_errors = self.validate_detectors(config);
        errors.extend(detector_errors);

        // Validate hardware compatibility
        let hardware_compatibility = self.validate_hardware_compatibility(config);

        // Generate performance suggestions
        let performance_suggestions =
            self.generate_performance_suggestions(config, &hardware_compatibility);
        suggestions.extend(performance_suggestions);

        // Check for common misconfigurations
        let common_warnings = self.check_common_issues(config);
        warnings.extend(common_warnings);

        let is_valid = errors.iter().all(|e| {
            matches!(
                e.severity,
                ValidationSeverity::Warning | ValidationSeverity::Info
            )
        });

        Ok(ValidationResult {
            is_valid,
            errors,
            warnings,
            suggestions,
            hardware_compatibility,
        })
    }

    /// Validate configuration structure
    fn validate_structure(&self, config: &FrigateConfig) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check for required sections
        if config.cameras.is_empty() {
            errors.push(ValidationError {
                code: "NO_CAMERAS".to_string(),
                message: "Configuration must contain at least one camera".to_string(),
                field: Some("cameras".to_string()),
                severity: ValidationSeverity::Error,
                line_number: None,
                suggestion: Some("Add at least one camera configuration".to_string()),
            });
        }

        // Check for detector configurations
        if !config.cameras.is_empty() && config.detectors.is_empty() {
            errors.push(ValidationError {
                code: "NO_DETECTORS".to_string(),
                message: "Cameras are configured but no detectors are defined".to_string(),
                field: Some("detectors".to_string()),
                severity: ValidationSeverity::Warning,
                line_number: None,
                suggestion: Some(
                    "Add a detector configuration (e.g., Coral, CPU, or GPU)".to_string(),
                ),
            });
        }

        errors
    }

    /// Validate camera configurations
    fn validate_cameras(&self, config: &FrigateConfig) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (camera_name, camera) in &config.cameras {
            // Validate ffmpeg inputs
            if camera.ffmpeg_inputs.is_empty() {
                errors.push(ValidationError {
                    code: "NO_FFMPEG_INPUTS".to_string(),
                    message: format!("Camera '{}' has no ffmpeg inputs configured", camera_name),
                    field: Some(format!("cameras.{}.ffmpeg.inputs", camera_name)),
                    severity: ValidationSeverity::Error,
                    line_number: None,
                    suggestion: Some("Add at least one ffmpeg input path".to_string()),
                });
            }

            // Validate input paths
            for input in &camera.ffmpeg_inputs {
                if input.path.is_empty() {
                    errors.push(ValidationError {
                        code: "EMPTY_INPUT_PATH".to_string(),
                        message: format!("Camera '{}' has empty input path", camera_name),
                        field: Some(format!("cameras.{}.ffmpeg.inputs.path", camera_name)),
                        severity: ValidationSeverity::Error,
                        line_number: None,
                        suggestion: Some("Provide a valid RTSP or file path".to_string()),
                    });
                }

                // Check for RTSP path format
                if input.path.starts_with("rtsp://")
                    && !input.path.contains("@")
                    && !input.path.contains("://")
                {
                    errors.push(ValidationError {
                        code: "INVALID_RTSP_FORMAT".to_string(),
                        message: format!("Camera '{}' RTSP path may be malformed", camera_name),
                        field: Some(format!("cameras.{}.ffmpeg.inputs.path", camera_name)),
                        severity: ValidationSeverity::Warning,
                        line_number: None,
                        suggestion: Some(
                            "Check RTSP URL format: rtsp://[username:password@]host[:port]/path"
                                .to_string(),
                        ),
                    });
                }
            }

            // Validate detection settings
            if let Some(detect) = camera.detect.fps {
                if !(1.0..=60.0).contains(&detect) {
                    errors.push(ValidationError {
                        code: "INVALID_FPS".to_string(),
                        message: format!(
                            "Camera '{}' has invalid FPS value: {}",
                            camera_name, detect
                        ),
                        field: Some(format!("cameras.{}.detect.fps", camera_name)),
                        severity: ValidationSeverity::Warning,
                        line_number: None,
                        suggestion: Some(
                            "FPS should be between 1 and 60 for optimal performance".to_string(),
                        ),
                    });
                }
            }
        }

        errors
    }

    /// Validate detector configurations
    fn validate_detectors(&self, config: &FrigateConfig) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (detector_name, detector) in &config.detectors {
            // Validate detector model
            if detector.model.is_empty() {
                errors.push(ValidationError {
                    code: "NO_DETECTOR_TYPE".to_string(),
                    message: format!("Detector '{}' has no model specified", detector_name),
                    field: Some(format!("detectors.{}.model", detector_name)),
                    severity: ValidationSeverity::Error,
                    line_number: None,
                    suggestion: Some(
                        "Specify detector model (e.g., edgetpu, cpu, gpu)".to_string(),
                    ),
                });
            }

            // Validate device path for hardware detectors
            if detector.model != "cpu" && detector.device.is_none() {
                errors.push(ValidationError {
                    code: "NO_DEVICE_PATH".to_string(),
                    message: format!(
                        "Hardware detector '{}' has no device specified",
                        detector_name
                    ),
                    field: Some(format!("detectors.{}.device", detector_name)),
                    severity: ValidationSeverity::Warning,
                    line_number: None,
                    suggestion: Some("Specify device path for hardware detectors".to_string()),
                });
            }
        }

        errors
    }

    /// Validate hardware compatibility
    fn validate_hardware_compatibility(&self, config: &FrigateConfig) -> HardwareCompatibility {
        let mut gpu_compatibility = Vec::new();
        let mut detector_compatibility = Vec::new();
        let mut bottlenecks = Vec::new();
        let mut recommendations = Vec::new();

        if let Some(ref hardware) = self.hardware_detection {
            // Check GPU compatibility for cameras
            for (camera_name, camera) in &config.cameras {
                let gpu_available: Vec<String> = hardware
                    .devices
                    .iter()
                    .filter(|d| d.r#type == "gpu" && d.available)
                    .map(|d| d.name.clone())
                    .collect();

                let gpu_required = self.get_required_gpu(camera);
                let is_compatible = gpu_available.is_empty()
                    || gpu_required.as_ref().map_or(true, |req| {
                        gpu_available.iter().any(|gpu| gpu.contains(req))
                    });

                gpu_compatibility.push(GPUCompatibility {
                    camera_name: camera_name.clone(),
                    gpu_required,
                    gpu_available: gpu_available.clone(),
                    is_compatible,
                    performance_impact: self.calculate_performance_impact(camera, &gpu_available),
                });

                // Check for GPU bottlenecks
                if !is_compatible && !gpu_available.is_empty() {
                    bottlenecks.push(HardwareBottleneck {
                        component: format!("GPU for camera {}", camera_name),
                        current_load: 0.0,
                        predicted_load: 100.0,
                        impact: "High CPU usage expected due to software decoding".to_string(),
                        recommendation: "Consider hardware acceleration or reduce resolution/FPS"
                            .to_string(),
                    });
                }
            }

            // Check detector compatibility
            for (detector_name, detector) in &config.detectors {
                let compatible_hardware: Vec<String> = hardware
                    .devices
                    .iter()
                    .filter(|d| match detector.model.as_str() {
                        "edgetpu" => {
                            d.r#type == "tpu" && d.capabilities.contains(&"edge_tpu".to_string())
                        }
                        "cpu" => d.r#type == "cpu",
                        "gpu" => d.r#type == "gpu" && d.capabilities.contains(&"cuda".to_string()),
                        _ => false,
                    })
                    .map(|d| d.name.clone())
                    .collect();

                let hardware_required = self.get_required_hardware(detector);
                let performance_tier = if compatible_hardware.is_empty() {
                    PerformanceTier::Incompatible
                } else if hardware_required
                    .iter()
                    .all(|req| compatible_hardware.iter().any(|hw| hw.contains(req)))
                {
                    PerformanceTier::High
                } else {
                    PerformanceTier::Medium
                };

                detector_compatibility.push(DetectorCompatibility {
                    detector_name: detector_name.clone(),
                    detector_type: detector.model.clone(),
                    hardware_required,
                    hardware_available: compatible_hardware.clone(),
                    is_compatible: performance_tier != PerformanceTier::Incompatible,
                    performance_tier,
                });

                // Add recommendations
                if compatible_hardware.is_empty() {
                    recommendations.push(format!(
                        "Detector '{}' ({}) has no compatible hardware available",
                        detector_name, detector.model
                    ));
                }
            }

            // Calculate overall compatibility score
            let total_checks = gpu_compatibility.len() + detector_compatibility.len();
            let compatible_checks = gpu_compatibility.iter().filter(|g| g.is_compatible).count()
                + detector_compatibility
                    .iter()
                    .filter(|d| d.is_compatible)
                    .count();
            let overall_score = if total_checks > 0 {
                (compatible_checks as f32) / (total_checks as f32) * 100.0
            } else {
                100.0
            };
        }

        HardwareCompatibility {
            gpu_compatibility,
            detector_compatibility,
            overall_score: 0.0,
            bottlenecks,
            recommendations,
        }
    }

    /// Generate performance suggestions
    fn generate_performance_suggestions(
        &self,
        _config: &FrigateConfig,
        hardware: &HardwareCompatibility,
    ) -> Vec<ConfigurationSuggestion> {
        let mut suggestions = Vec::new();

        // Suggest hardware acceleration for cameras
        for gpu_comp in &hardware.gpu_compatibility {
            if !gpu_comp.is_compatible && !gpu_comp.gpu_available.is_empty() {
                suggestions.push(ConfigurationSuggestion {
                    title: "Enable Hardware Acceleration".to_string(),
                    description: format!(
                        "Camera '{}' could benefit from hardware acceleration to reduce CPU usage",
                        gpu_comp.camera_name
                    ),
                    category: SuggestionCategory::Performance,
                    impact: ImpactLevel::High,
                    changes: vec![ConfigChange {
                        path: format!("cameras.{}.ffmpeg.hwaccel_args", gpu_comp.camera_name),
                        old_value: None,
                        new_value: serde_json::json!([
                            "-hwaccel",
                            "cuda",
                            "-hwaccel_output_format",
                            "cuda"
                        ]),
                        reason: "Enable GPU acceleration for better performance".to_string(),
                    }],
                });
            }
        }

        // Suggest detector optimization
        for detector_comp in &hardware.detector_compatibility {
            if detector_comp.performance_tier == PerformanceTier::Medium {
                suggestions.push(ConfigurationSuggestion {
                    title: "Optimize Detector Performance".to_string(),
                    description: format!(
                        "Detector '{}' could be optimized for better performance",
                        detector_comp.detector_name
                    ),
                    category: SuggestionCategory::Performance,
                    impact: ImpactLevel::Medium,
                    changes: vec![ConfigChange {
                        path: format!("detectors.{}.max_labels", detector_comp.detector_name),
                        old_value: None,
                        new_value: serde_json::json!(5),
                        reason: "Reduce max_labels for better performance".to_string(),
                    }],
                });
            }
        }

        suggestions
    }

    /// Check for common configuration issues
    fn check_common_issues(&self, config: &FrigateConfig) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        // Check for high resolution cameras without hardware acceleration
        for (camera_name, camera) in &config.cameras {
            if let Some(detect) = camera.detect.fps {
                if detect > 30.0 {
                    warnings.push(ValidationWarning {
                        code: "HIGH_FPS".to_string(),
                        message: format!(
                            "Camera '{}' has high FPS ({}), which may impact performance",
                            camera_name, detect
                        ),
                        field: Some(format!("cameras.{}.detect.fps", camera_name)),
                        line_number: None,
                        recommendation: Some(
                            "Consider reducing FPS to 15-30 for optimal performance".to_string(),
                        ),
                    });
                }
            }

            // Check for missing recording configuration
            if camera.record.is_none() {
                warnings.push(ValidationWarning {
                    code: "NO_RECORDING".to_string(),
                    message: format!("Camera '{}' has no recording configuration", camera_name),
                    field: Some(format!("cameras.{}.record", camera_name)),
                    line_number: None,
                    recommendation: Some(
                        "Consider enabling recording for security coverage".to_string(),
                    ),
                });
            }
        }

        // Check for multiple cameras with the same detector
        let mut detector_usage: HashMap<String, Vec<String>> = HashMap::new();
        for (camera_name, camera) in &config.cameras {
            // This is a simplified check - in practice, you'd need to determine which detector each camera uses
            if camera.detect.enabled {
                // Assume default detector for this example
                detector_usage
                    .entry("default".to_string())
                    .or_default()
                    .push(camera_name.clone());
            }
        }

        for (detector, cameras) in detector_usage {
            if cameras.len() > 4 {
                warnings.push(ValidationWarning {
                    code: "DETECTOR_OVERLOAD".to_string(),
                    message: format!("Detector '{}' is used by {} cameras, which may impact performance", detector, cameras.len()),
                    field: Some("detectors".to_string()),
                    line_number: None,
                    recommendation: Some("Consider adding additional detectors or reducing camera count per detector".to_string()),
                });
            }
        }

        warnings
    }

    /// Get required GPU for a camera (simplified)
    fn get_required_gpu(&self, camera: &CameraConfig) -> Option<String> {
        // This is a simplified implementation
        // In practice, you'd analyze the camera configuration to determine GPU requirements
        for input in &camera.ffmpeg_inputs {
            if input.hwaccel_args.is_some() {
                return Some("CUDA".to_string());
            }
        }
        None
    }

    /// Get required hardware for a detector
    fn get_required_hardware(&self, detector: &DetectorConfig) -> Vec<String> {
        match detector.model.as_str() {
            "edgetpu" => vec!["Coral".to_string()],
            "gpu" => vec!["CUDA".to_string()],
            "cpu" => vec![],
            _ => vec![],
        }
    }

    /// Calculate performance impact
    fn calculate_performance_impact(
        &self,
        camera: &CameraConfig,
        available_gpus: &[String],
    ) -> PerformanceImpact {
        if available_gpus.is_empty() {
            PerformanceImpact::High
        } else if camera.detect.fps.unwrap_or(0.0) > 30.0 {
            PerformanceImpact::Medium
        } else {
            PerformanceImpact::Low
        }
    }

    /// Get default validation rules
    fn default_rules() -> Vec<ValidationRule> {
        vec![
            ValidationRule {
                name: "Required Sections".to_string(),
                validator: Box::new(|config| {
                    let mut errors = Vec::new();
                    if config.cameras.is_empty() {
                        errors.push(ValidationError {
                            code: "NO_CAMERAS".to_string(),
                            message: "Configuration must contain at least one camera".to_string(),
                            field: Some("cameras".to_string()),
                            severity: ValidationSeverity::Error,
                            line_number: None,
                            suggestion: Some("Add at least one camera configuration".to_string()),
                        });
                    }
                    errors
                }),
            },
            ValidationRule {
                name: "Detector Configuration".to_string(),
                validator: Box::new(|config| {
                    let mut errors = Vec::new();
                    if !config.cameras.is_empty() && config.detectors.is_empty() {
                        errors.push(ValidationError {
                            code: "NO_DETECTORS".to_string(),
                            message: "Cameras are configured but no detectors are defined"
                                .to_string(),
                            field: Some("detectors".to_string()),
                            severity: ValidationSeverity::Warning,
                            line_number: None,
                            suggestion: Some("Add a detector configuration".to_string()),
                        });
                    }
                    errors
                }),
            },
        ]
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}
