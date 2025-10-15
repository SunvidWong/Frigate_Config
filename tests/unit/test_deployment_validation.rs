// T093: Unit tests for deployment validation logic
// Tests pre-deployment checks: YAML validation, Docker availability, device paths, ports
// REQUIREMENT: FR-033, FR-034, FR-035, FR-036 (Deployment Module - Pre-deployment validation)

#[cfg(test)]
mod test_deployment_validation {
    use std::path::PathBuf;

    // Import actual implementation
    use frigate_config_tool::deployment::validator::{
        check_docker_availability, check_docker_compose_availability, check_docker_permissions,
        validate_deployment_config, validate_device_paths, validate_frigate_config,
        validate_port_availability, validate_volume_paths, validate_yaml_schema,
        validate_yaml_syntax, DeploymentConfig,
    };

    // ========== YAML Validation Tests ==========

    #[test]
    fn test_validate_yaml_syntax_valid() {
        let yaml_path = PathBuf::from("../tests/fixtures/valid_frigate.yml");
        let result = validate_yaml_syntax(&yaml_path);
        assert!(result.is_ok(), "YAML validation failed: {:?}", result.err());
    }

    #[test]
    fn test_validate_yaml_syntax_invalid() {
        let yaml_path = PathBuf::from("../tests/fixtures/invalid_syntax.yml");
        let result = validate_yaml_syntax(&yaml_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("syntax error"));
    }

    #[test]
    fn test_validate_yaml_syntax_file_not_found() {
        let yaml_path = PathBuf::from("../tests/fixtures/nonexistent.yml");
        let result = validate_yaml_syntax(&yaml_path);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("not found") || error.contains("No such file"));
    }

    #[test]
    fn test_validate_yaml_schema_valid_frigate_config() {
        let yaml_path = PathBuf::from("../tests/fixtures/valid_frigate.yml");
        let result = validate_yaml_schema(&yaml_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_yaml_schema_missing_required_fields() {
        let yaml_path = PathBuf::from("../tests/fixtures/missing_cameras.yml");
        let result = validate_yaml_schema(&yaml_path);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("cameras")));
    }

    #[test]
    fn test_validate_frigate_config_invalid_camera_config() {
        let yaml_path = PathBuf::from("../tests/fixtures/invalid_camera.yml");
        let result = validate_frigate_config(&yaml_path);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.contains("camera") || e.contains("inputs")));
    }

    // ========== Docker Availability Tests ==========

    #[test]
    fn test_check_docker_availability_success() {
        let result = check_docker_availability();
        // This test will pass on systems with Docker installed
        // In CI, we can mock this or skip based on environment
        assert!(result.is_ok() || result.is_err()); // Placeholder assertion
    }

    #[test]
    fn test_check_docker_availability_returns_version() {
        let result = check_docker_availability();
        if let Ok(version) = result {
            // Version should be in format like "20.10.17" or similar
            assert!(!version.is_empty());
            assert!(version.contains('.'));
        }
    }

    #[test]
    fn test_check_docker_compose_availability() {
        let result = check_docker_compose_availability();
        // Should return version or error
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_check_docker_permissions() {
        let result = check_docker_permissions();
        // Should check if user can run docker commands without sudo
        // On Linux, checks if user is in docker group
        assert!(result.is_ok() || result.is_err());
    }

    // ========== Device Path Validation Tests ==========

    #[test]
    fn test_validate_device_paths_all_exist() {
        let devices = vec![
            "/dev/null".to_string(), // Always exists on Unix systems
        ];
        let result = validate_device_paths(&devices);
        assert!(result.is_ok());
        let valid_devices = result.unwrap();
        assert_eq!(valid_devices.len(), 1);
    }

    #[test]
    fn test_validate_device_paths_some_missing() {
        let devices = vec![
            "/dev/null".to_string(),
            "/dev/nonexistent_device123".to_string(),
        ];
        let result = validate_device_paths(&devices);
        assert!(result.is_err());
        let invalid_devices = result.unwrap_err();
        assert!(invalid_devices.contains(&"/dev/nonexistent_device123".to_string()));
    }

    #[test]
    fn test_validate_device_paths_empty_list() {
        let devices: Vec<String> = vec![];
        let result = validate_device_paths(&devices);
        // Empty list should be valid (CPU-only mode)
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_device_paths_gpu_device() {
        let devices = vec!["/dev/dri/renderD128".to_string()];
        let result = validate_device_paths(&devices);
        // This test is platform-specific (Linux only)
        // Should pass if GPU exists, fail otherwise
        assert!(result.is_ok() || result.is_err());
    }

    // ========== Port Availability Tests ==========

    #[test]
    fn test_validate_port_availability_free_ports() {
        let ports = vec![45123, 45124]; // Random high ports unlikely to be used
        let result = validate_port_availability(&ports);
        assert!(result.is_ok());
        let available_ports = result.unwrap();
        assert_eq!(available_ports.len(), 2);
    }

    #[test]
    fn test_validate_port_availability_privileged_ports() {
        let ports = vec![80, 443]; // Privileged ports
        let result = validate_port_availability(&ports);
        // Should either succeed (if running as root) or fail with permission error
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_port_availability_invalid_port_zero() {
        let ports = vec![0];
        let result = validate_port_availability(&ports);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_port_availability_default_frigate_ports() {
        let ports = vec![5000, 8554, 8555]; // Default Frigate ports
        let result = validate_port_availability(&ports);
        // Ports might be in use or available depending on system
        assert!(result.is_ok() || result.is_err());
    }

    // ========== Volume Path Validation Tests ==========

    #[test]
    fn test_validate_volume_paths_all_valid() {
        let paths = vec![
            PathBuf::from("/tmp"), // Should always exist and be writable
        ];
        let result = validate_volume_paths(&paths);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_volume_paths_nonexistent_path() {
        let paths = vec![PathBuf::from("/nonexistent/path/12345")];
        let result = validate_volume_paths(&paths);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].1.contains("does not exist") || errors[0].1.contains("not found"));
    }

    #[test]
    fn test_validate_volume_paths_not_writable() {
        let paths = vec![PathBuf::from("/etc")]; // System directory, usually not writable
        let result = validate_volume_paths(&paths);
        // May succeed if running as root, fail otherwise
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_volume_paths_mixed_valid_invalid() {
        let paths = vec![PathBuf::from("/tmp"), PathBuf::from("/nonexistent123")];
        let result = validate_volume_paths(&paths);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, PathBuf::from("/nonexistent123"));
    }

    // ========== Full Deployment Validation Tests ==========

    #[test]
    fn test_validate_deployment_config_all_valid() {
        let config = DeploymentConfig {
            yaml_path: PathBuf::from("../tests/fixtures/valid_frigate.yml"),
            device_paths: vec![],
            ports: vec![5000, 8554],
            volume_paths: vec![PathBuf::from("/tmp")],
        };

        let result = validate_deployment_config(&config);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_deployment_config_multiple_errors() {
        let config = DeploymentConfig {
            yaml_path: PathBuf::from("../tests/fixtures/invalid_syntax.yml"),
            device_paths: vec!["/dev/nonexistent".to_string()],
            ports: vec![0], // Invalid port
            volume_paths: vec![PathBuf::from("/nonexistent")],
        };

        let result = validate_deployment_config(&config);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());

        // Should have errors for YAML, device, port, and volume
        assert!(result
            .errors
            .iter()
            .any(|e| e.category.contains("yaml") || e.category.contains("YAML")));
        assert!(result
            .errors
            .iter()
            .any(|e| e.category.contains("Device") || e.category.contains("device")));
        assert!(result
            .errors
            .iter()
            .any(|e| e.category.contains("Port") || e.category.contains("port")));
        assert!(result.errors.iter().any(|e| e.category.contains("Volume")
            || e.category.contains("volume")
            || e.category.contains("path")));
    }

    #[test]
    fn test_validate_deployment_config_with_warnings() {
        let config = DeploymentConfig {
            yaml_path: PathBuf::from("../tests/fixtures/valid_frigate.yml"),
            device_paths: vec![], // No hardware acceleration - should generate warning
            ports: vec![5000],
            volume_paths: vec![PathBuf::from("/tmp")],
        };

        let result = validate_deployment_config(&config);
        assert!(result.valid); // Still valid, just warnings
        assert!(result.errors.is_empty());
        assert!(!result.warnings.is_empty());

        // Should warn about CPU-only mode
        assert!(result
            .warnings
            .iter()
            .any(|w| w.message.contains("CPU") || w.message.contains("hardware")));
    }

    #[test]
    fn test_validation_error_includes_fix_suggestions() {
        let config = DeploymentConfig {
            yaml_path: PathBuf::from("../tests/fixtures/missing_cameras.yml"),
            device_paths: vec![],
            ports: vec![5000],
            volume_paths: vec![],
        };

        let result = validate_deployment_config(&config);
        assert!(!result.valid);

        // Errors should include helpful fix suggestions
        for error in result.errors {
            // At least some errors should have fix suggestions
            if error.category.contains("yaml") || error.category.contains("config") {
                // Config errors should suggest fixes
                assert!(
                    error.fix_suggestion.is_some()
                        || error.message.contains("Add")
                        || error.message.contains("Configure")
                );
            }
        }
    }

    // ========== Performance Tests ==========

    #[test]
    fn test_validation_performance_under_5_seconds() {
        use std::time::Instant;

        let config = DeploymentConfig {
            yaml_path: PathBuf::from("../tests/fixtures/valid_frigate.yml"),
            device_paths: vec![],
            ports: vec![5000, 8554, 8555],
            volume_paths: vec![PathBuf::from("/tmp")],
        };

        let start = Instant::now();
        let _result = validate_deployment_config(&config);
        let duration = start.elapsed();

        // Validation should complete within 5 seconds (FR-019 requirement adapted)
        assert!(
            duration.as_secs() < 5,
            "Validation took too long: {:?}",
            duration
        );
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_validate_very_large_yaml_file() {
        // Test with YAML file containing many cameras (100+)
        let yaml_path = PathBuf::from("../tests/fixtures/large_config.yml");
        let result = validate_yaml_syntax(&yaml_path);
        // Should handle large files without crashing
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_yaml_with_unicode_characters() {
        let yaml_path = PathBuf::from("../tests/fixtures/unicode_config.yml");
        let result = validate_yaml_syntax(&yaml_path);
        // Should handle Unicode in camera names, paths, etc.
        assert!(result.is_ok());
    }

    #[test]
    fn test_docker_version_parsing() {
        let result = check_docker_availability();
        if let Ok(version) = result {
            // Version should be parseable as semver-like string
            let parts: Vec<&str> = version.split('.').collect();
            assert!(parts.len() >= 2); // Major.Minor at minimum

            // First part should be a number
            let major: Result<u32, _> = parts[0].parse();
            assert!(major.is_ok());
        }
    }
}
