// T106-T111: Pre-deployment validation
// Validates YAML, Docker availability, device paths, ports, and volumes
// REQUIREMENT: FR-033, FR-034, FR-035, FR-036 (Deployment Module - Pre-deployment validation)

use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};

/// Validation result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

/// Validation error with category and fix suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub category: String,
    pub message: String,
    pub fix_suggestion: Option<String>,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub category: String,
    pub message: String,
}

/// Deployment configuration to validate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub yaml_path: PathBuf,
    pub device_paths: Vec<String>,
    pub ports: Vec<u16>,
    pub volume_paths: Vec<PathBuf>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, category: String, message: String, fix_suggestion: Option<String>) {
        self.valid = false;
        self.errors.push(ValidationError {
            category,
            message,
            fix_suggestion,
        });
    }

    pub fn add_warning(&mut self, category: String, message: String) {
        self.warnings.push(ValidationWarning { category, message });
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

// ========== YAML Validation ==========

/// Validate YAML syntax
pub fn validate_yaml_syntax(yaml_path: &PathBuf) -> Result<(), String> {
    // Check file exists
    if !yaml_path.exists() {
        return Err(format!("Configuration file not found: {}", yaml_path.display()));
    }

    // Read file content
    let content = fs::read_to_string(yaml_path)
        .map_err(|e| format!("Failed to read YAML file: {}", e))?;

    // Parse as YAML to check syntax
    let _: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| format!("YAML syntax error: {}", e))?;

    Ok(())
}

/// Validate YAML schema (basic Frigate structure)
pub fn validate_yaml_schema(yaml_path: &PathBuf) -> Result<(), Vec<String>> {
    let content = fs::read_to_string(yaml_path)
        .map_err(|e| vec![format!("Failed to read YAML file: {}", e)])?;

    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| vec![format!("YAML syntax error: {}", e)])?;

    let mut errors = Vec::new();

    // Check if it's a mapping (object)
    if !yaml.is_mapping() {
        errors.push("YAML root must be an object/mapping".to_string());
        return Err(errors);
    }

    let map = yaml.as_mapping().unwrap();

    // Check for cameras section (not strictly required but warn if missing)
    if !map.contains_key(&serde_yaml::Value::String("cameras".to_string())) {
        errors.push("Missing required 'cameras' section in Frigate configuration".to_string());
    }

    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(())
    }
}

/// Validate Frigate-specific configuration
pub fn validate_frigate_config(yaml_path: &PathBuf) -> Result<(), Vec<String>> {
    let content = fs::read_to_string(yaml_path)
        .map_err(|e| vec![format!("Failed to read YAML file: {}", e)])?;

    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| vec![format!("YAML syntax error: {}", e)])?;

    let mut errors = Vec::new();

    if let Some(map) = yaml.as_mapping() {
        // Check cameras configuration
        if let Some(cameras) = map.get(&serde_yaml::Value::String("cameras".to_string())) {
            if let Some(camera_map) = cameras.as_mapping() {
                for (camera_name, camera_config) in camera_map {
                    // Check if camera has inputs
                    if let Some(config_map) = camera_config.as_mapping() {
                        if let Some(ffmpeg) = config_map.get(&serde_yaml::Value::String("ffmpeg".to_string())) {
                            if let Some(ffmpeg_map) = ffmpeg.as_mapping() {
                                if !ffmpeg_map.contains_key(&serde_yaml::Value::String("inputs".to_string())) {
                                    errors.push(format!(
                                        "Camera '{}' is missing 'ffmpeg.inputs' configuration",
                                        camera_name.as_str().unwrap_or("unknown")
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(())
    }
}

// ========== Docker Availability Checks ==========

/// Check if Docker is available and return version
pub fn check_docker_availability() -> Result<String, String> {
    let output = Command::new("docker")
        .args(&["--version"])
        .output()
        .map_err(|e| format!("Docker not found or not installed: {}", e))?;

    if !output.status.success() {
        return Err("Docker command failed".to_string());
    }

    let version_output = String::from_utf8_lossy(&output.stdout);

    // Extract version number from output like "Docker version 20.10.17, build 100c701"
    let version = version_output
        .split_whitespace()
        .nth(2) // "version" is index 1, version number is index 2
        .unwrap_or("unknown")
        .trim_end_matches(',')
        .to_string();

    Ok(version)
}

/// Check if Docker Compose is available and return version
pub fn check_docker_compose_availability() -> Result<String, String> {
    // Try docker-compose first
    let output = Command::new("docker-compose")
        .args(&["--version"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let version_output = String::from_utf8_lossy(&out.stdout);
            let version = version_output
                .split_whitespace()
                .nth(2)
                .unwrap_or("unknown")
                .trim_end_matches(',')
                .to_string();
            return Ok(version);
        }
    }

    // Try docker compose (V2 syntax)
    let output = Command::new("docker")
        .args(&["compose", "version"])
        .output()
        .map_err(|e| format!("Docker Compose not found: {}", e))?;

    if !output.status.success() {
        return Err("Docker Compose command failed".to_string());
    }

    let version_output = String::from_utf8_lossy(&output.stdout);
    let version = version_output
        .split_whitespace()
        .nth(3)
        .unwrap_or("unknown")
        .to_string();

    Ok(version)
}

/// Check if Docker can be run without sudo (user permissions)
pub fn check_docker_permissions() -> Result<(), String> {
    let output = Command::new("docker")
        .args(&["ps"])
        .output()
        .map_err(|e| format!("Failed to check Docker permissions: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("permission denied") || stderr.contains("denied") {
            return Err("Docker requires sudo. User is not in docker group or doesn't have permissions".to_string());
        }
        return Err(format!("Docker ps command failed: {}", stderr));
    }

    Ok(())
}

// ========== Device Path Validation ==========

/// Validate device paths exist and are accessible
pub fn validate_device_paths(devices: &[String]) -> Result<Vec<String>, Vec<String>> {
    if devices.is_empty() {
        return Ok(Vec::new());
    }

    let mut valid_devices = Vec::new();
    let mut invalid_devices = Vec::new();

    for device in devices {
        let path = Path::new(device);
        if path.exists() {
            valid_devices.push(device.clone());
        } else {
            invalid_devices.push(device.clone());
        }
    }

    if invalid_devices.is_empty() {
        Ok(valid_devices)
    } else {
        Err(invalid_devices)
    }
}

// ========== Port Availability Checks ==========

/// Validate ports are available and in valid range
pub fn validate_port_availability(ports: &[u16]) -> Result<Vec<u16>, Vec<u16>> {
    let mut available_ports = Vec::new();
    let mut unavailable_ports = Vec::new();

    for &port in ports {
        // Check port is not zero
        if port == 0 {
            unavailable_ports.push(port);
            continue;
        }

        // Try to bind to the port
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => {
                // Port is available
                available_ports.push(port);
            }
            Err(_) => {
                // Port is in use or not accessible
                unavailable_ports.push(port);
            }
        }
    }

    if unavailable_ports.is_empty() {
        Ok(available_ports)
    } else {
        Err(unavailable_ports)
    }
}

// ========== Volume Path Validation ==========

/// Validate volume paths exist and are writable
pub fn validate_volume_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>, Vec<(PathBuf, String)>> {
    let mut valid_paths = Vec::new();
    let mut invalid_paths = Vec::new();

    for path in paths {
        // Check if path exists
        if !path.exists() {
            invalid_paths.push((
                path.clone(),
                format!("Path does not exist: {}", path.display()),
            ));
            continue;
        }

        // Check if it's a directory
        if !path.is_dir() {
            invalid_paths.push((
                path.clone(),
                format!("Path is not a directory: {}", path.display()),
            ));
            continue;
        }

        // Check if writable by trying to create a test file
        let test_file = path.join(".frigate_write_test");
        match fs::write(&test_file, "test") {
            Ok(_) => {
                // Writable, clean up test file
                let _ = fs::remove_file(test_file);
                valid_paths.push(path.clone());
            }
            Err(_) => {
                invalid_paths.push((
                    path.clone(),
                    format!("Path is not writable: {}", path.display()),
                ));
            }
        }
    }

    if invalid_paths.is_empty() {
        Ok(valid_paths)
    } else {
        Err(invalid_paths)
    }
}

// ========== Full Deployment Validation ==========

/// Validate complete deployment configuration
pub fn validate_deployment_config(config: &DeploymentConfig) -> ValidationResult {
    let mut result = ValidationResult::new();

    // 1. Validate YAML syntax
    if let Err(e) = validate_yaml_syntax(&config.yaml_path) {
        result.add_error(
            "YAML Syntax".to_string(),
            e,
            Some("Check your YAML file for syntax errors. Use a YAML validator.".to_string()),
        );
    }

    // 2. Validate YAML schema
    if let Err(errors) = validate_yaml_schema(&config.yaml_path) {
        for error in errors {
            result.add_error(
                "YAML Schema".to_string(),
                error.clone(),
                Some("Add the required 'cameras' section to your Frigate configuration.".to_string()),
            );
        }
    }

    // 3. Validate Frigate-specific config
    if let Err(errors) = validate_frigate_config(&config.yaml_path) {
        for error in errors {
            result.add_error(
                "Frigate Config".to_string(),
                error,
                Some("Configure camera inputs with RTSP stream URLs.".to_string()),
            );
        }
    }

    // 4. Validate device paths
    if let Err(invalid_devices) = validate_device_paths(&config.device_paths) {
        for device in invalid_devices {
            result.add_error(
                "Device Path".to_string(),
                format!("Device not found: {}", device),
                Some("Check that the device exists and is accessible. For GPU: install drivers. For Coral TPU: connect device.".to_string()),
            );
        }
    }

    // 5. Warn if no devices (CPU-only mode)
    if config.device_paths.is_empty() {
        result.add_warning(
            "Hardware Acceleration".to_string(),
            "No GPU or TPU devices configured. Frigate will run in CPU-only mode, which may have lower performance.".to_string(),
        );
    }

    // 6. Validate ports
    if let Err(unavailable_ports) = validate_port_availability(&config.ports) {
        for port in unavailable_ports {
            if port == 0 {
                result.add_error(
                    "Port".to_string(),
                    "Port 0 is invalid".to_string(),
                    Some("Use a valid port number between 1 and 65535.".to_string()),
                );
            } else {
                result.add_error(
                    "Port".to_string(),
                    format!("Port {} is already in use or not accessible", port),
                    Some(format!("Stop the service using port {} or choose a different port.", port)),
                );
            }
        }
    }

    // 7. Validate volume paths
    if let Err(invalid_paths) = validate_volume_paths(&config.volume_paths) {
        for (path, error) in invalid_paths {
            result.add_error(
                "Volume Path".to_string(),
                error,
                Some(format!("Create the directory: mkdir -p {}", path.display())),
            );
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_defaults() {
        let result = ValidationResult::new();
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_add_error() {
        let mut result = ValidationResult::new();
        result.add_error(
            "Test".to_string(),
            "Test error".to_string(),
            Some("Fix it".to_string()),
        );

        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].category, "Test");
        assert_eq!(result.errors[0].message, "Test error");
    }

    #[test]
    fn test_validation_result_add_warning() {
        let mut result = ValidationResult::new();
        result.add_warning("Test".to_string(), "Test warning".to_string());

        assert!(result.valid); // Warnings don't affect validity
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_validate_device_paths_empty() {
        let devices: Vec<String> = vec![];
        let result = validate_device_paths(&devices);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_device_paths_valid() {
        let devices = vec!["/dev/null".to_string()];
        let result = validate_device_paths(&devices);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_port_zero() {
        let ports = vec![0];
        let result = validate_port_availability(&ports);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec![0]);
    }
}
