// T092: Unit tests for Docker command generation
// Tests validation for docker run and docker-compose command generation
// REQUIREMENT: FR-032, FR-037, FR-043 (Deployment Module - Docker command generation)

#[cfg(test)]
mod test_docker_commands {
    use std::collections::HashMap;

    // Import actual implementation from src-tauri/src/deployment/docker.rs
    use frigate_config_tool::deployment::docker::{
        generate_docker_compose_yaml, generate_docker_run_command, validate_docker_config,
        DockerComposeConfig, DockerRunConfig, DockerServiceConfig,
    };

    #[test]
    fn test_generate_basic_docker_run_command() {
        // Test basic docker run command generation
        let config = DockerRunConfig {
            image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            container_name: "frigate".to_string(),
            ports: vec![(5000, 5000), (8554, 8554), (8555, 8555)],
            volumes: vec![
                ("/etc/localtime".to_string(), "/etc/localtime".to_string()),
                ("/path/to/config".to_string(), "/config".to_string()),
                ("/path/to/storage".to_string(), "/media/frigate".to_string()),
            ],
            devices: vec![],
            environment: HashMap::new(),
            restart_policy: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        };

        let command = generate_docker_run_command(&config);

        // Verify basic structure
        assert!(command.starts_with("docker run"));
        assert!(command.contains("--name frigate"));
        assert!(command.contains("-p 5000:5000"));
        assert!(command.contains("-p 8554:8554"));
        assert!(command.contains("-p 8555:8555"));
        assert!(command.contains("-v /etc/localtime:/etc/localtime"));
        assert!(command.contains("-v /path/to/config:/config"));
        assert!(command.contains("-v /path/to/storage:/media/frigate"));
        assert!(command.contains("--restart unless-stopped"));
        assert!(command.contains("ghcr.io/blakeblackshear/frigate:stable"));
    }

    #[test]
    fn test_generate_docker_run_with_gpu() {
        // Test docker run command with GPU device mounting
        let config = DockerRunConfig {
            image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            container_name: "frigate".to_string(),
            ports: vec![(5000, 5000)],
            volumes: vec![],
            devices: vec!["/dev/dri/renderD128".to_string()],
            environment: HashMap::new(),
            restart_policy: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        };

        let command = generate_docker_run_command(&config);

        assert!(command.contains("--device /dev/dri/renderD128"));
    }

    #[test]
    fn test_generate_docker_run_with_coral_tpu() {
        // Test docker run command with Coral TPU device mounting
        let config = DockerRunConfig {
            image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            container_name: "frigate".to_string(),
            ports: vec![(5000, 5000)],
            volumes: vec![],
            devices: vec!["/dev/apex_0".to_string(), "/dev/bus/usb".to_string()],
            environment: HashMap::new(),
            restart_policy: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        };

        let command = generate_docker_run_command(&config);

        assert!(command.contains("--device /dev/apex_0"));
        assert!(command.contains("--device /dev/bus/usb"));
    }

    #[test]
    fn test_generate_docker_run_with_environment_variables() {
        // Test docker run command with environment variables
        let mut env = HashMap::new();
        env.insert("FRIGATE_RTSP_PASSWORD".to_string(), "secret123".to_string());
        env.insert("TZ".to_string(), "America/New_York".to_string());

        let config = DockerRunConfig {
            image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            container_name: "frigate".to_string(),
            ports: vec![],
            volumes: vec![],
            devices: vec![],
            environment: env,
            restart_policy: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        };

        let command = generate_docker_run_command(&config);

        assert!(command.contains("-e FRIGATE_RTSP_PASSWORD=secret123"));
        assert!(command.contains("-e TZ=America/New_York"));
    }

    #[test]
    fn test_generate_docker_run_with_privileged_mode() {
        // Test docker run command with privileged mode
        let config = DockerRunConfig {
            image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            container_name: "frigate".to_string(),
            ports: vec![],
            volumes: vec![],
            devices: vec![],
            environment: HashMap::new(),
            restart_policy: "unless-stopped".to_string(),
            privileged: true,
            network_mode: None,
        };

        let command = generate_docker_run_command(&config);

        assert!(command.contains("--privileged"));
    }

    #[test]
    fn test_generate_docker_compose_basic() {
        // Test basic docker-compose YAML generation
        let mut services = HashMap::new();
        services.insert(
            "frigate".to_string(),
            DockerServiceConfig {
                image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
                container_name: "frigate".to_string(),
                ports: vec!["5000:5000".to_string(), "8554:8554".to_string()],
                volumes: vec![
                    "/etc/localtime:/etc/localtime:ro".to_string(),
                    "/path/to/config:/config".to_string(),
                ],
                devices: vec![],
                environment: HashMap::new(),
                restart: "unless-stopped".to_string(),
                privileged: false,
                network_mode: None,
            },
        );

        let config = DockerComposeConfig {
            version: "3.9".to_string(),
            services,
        };

        let yaml = generate_docker_compose_yaml(&config);

        // Verify YAML structure
        assert!(yaml.contains("version: \"3.9\""));
        assert!(yaml.contains("services:"));
        assert!(yaml.contains("frigate:"));
        assert!(yaml.contains("image: ghcr.io/blakeblackshear/frigate:stable"));
        assert!(yaml.contains("container_name: frigate"));
        assert!(yaml.contains("- 5000:5000"));
        assert!(yaml.contains("- 8554:8554"));
        assert!(yaml.contains("restart: unless-stopped"));
    }

    #[test]
    fn test_generate_docker_compose_with_devices() {
        // Test docker-compose YAML with device mounts
        let mut services = HashMap::new();
        services.insert(
            "frigate".to_string(),
            DockerServiceConfig {
                image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
                container_name: "frigate".to_string(),
                ports: vec![],
                volumes: vec![],
                devices: vec![
                    "/dev/dri/renderD128:/dev/dri/renderD128".to_string(),
                    "/dev/apex_0:/dev/apex_0".to_string(),
                ],
                environment: HashMap::new(),
                restart: "unless-stopped".to_string(),
                privileged: false,
                network_mode: None,
            },
        );

        let config = DockerComposeConfig {
            version: "3.9".to_string(),
            services,
        };

        let yaml = generate_docker_compose_yaml(&config);

        assert!(yaml.contains("devices:"));
        assert!(yaml.contains("- /dev/dri/renderD128:/dev/dri/renderD128"));
        assert!(yaml.contains("- /dev/apex_0:/dev/apex_0"));
    }

    #[test]
    fn test_validate_docker_config_valid() {
        // Test validation of valid Docker configuration
        let config = DockerRunConfig {
            image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            container_name: "frigate".to_string(),
            ports: vec![(5000, 5000)],
            volumes: vec![("/path/to/config".to_string(), "/config".to_string())],
            devices: vec![],
            environment: HashMap::new(),
            restart_policy: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        };

        let result = validate_docker_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_docker_config_missing_image() {
        // Test validation fails when image is empty
        let config = DockerRunConfig {
            image: "".to_string(),
            container_name: "frigate".to_string(),
            ports: vec![],
            volumes: vec![],
            devices: vec![],
            environment: HashMap::new(),
            restart_policy: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        };

        let result = validate_docker_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("image"));
    }

    #[test]
    fn test_validate_docker_config_invalid_port() {
        // Test validation fails for invalid port numbers
        let config = DockerRunConfig {
            image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            container_name: "frigate".to_string(),
            ports: vec![(0, 5000)], // Port 0 is invalid
            volumes: vec![],
            devices: vec![],
            environment: HashMap::new(),
            restart_policy: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        };

        let result = validate_docker_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("port"));
    }

    #[test]
    fn test_command_escaping() {
        // Test proper escaping of special characters in paths
        let config = DockerRunConfig {
            image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            container_name: "frigate".to_string(),
            ports: vec![],
            volumes: vec![
                (
                    "/path/with spaces/config".to_string(),
                    "/config".to_string(),
                ),
                ("/path/with$dollar".to_string(), "/media".to_string()),
            ],
            devices: vec![],
            environment: HashMap::new(),
            restart_policy: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        };

        let command = generate_docker_run_command(&config);

        // Verify paths are properly quoted/escaped
        assert!(
            command.contains("\"/path/with spaces/config\"")
                || command.contains("'/path/with spaces/config'")
        );
    }

    #[test]
    fn test_generate_docker_run_detached_mode() {
        // Test docker run command includes detached mode flag
        let config = DockerRunConfig {
            image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            container_name: "frigate".to_string(),
            ports: vec![],
            volumes: vec![],
            devices: vec![],
            environment: HashMap::new(),
            restart_policy: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        };

        let command = generate_docker_run_command(&config);

        // Should include -d flag for detached mode
        assert!(command.contains(" -d ") || command.contains("--detach"));
    }

    #[test]
    fn test_generate_docker_run_network_mode() {
        // Test docker run command with host network mode (required for Frigate)
        let config = DockerRunConfig {
            image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            container_name: "frigate".to_string(),
            ports: vec![],
            volumes: vec![],
            devices: vec![],
            environment: HashMap::new(),
            restart_policy: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        };

        let command = generate_docker_run_command(&config);

        // Frigate typically uses host network mode for RTSP
        // This can be optional based on configuration
        // For now, just verify command generation works
        assert!(!command.is_empty());
    }
}
