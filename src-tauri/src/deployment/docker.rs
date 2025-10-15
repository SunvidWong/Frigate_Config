// T098, T099, T100: Docker command generation
// Generates docker run and docker-compose commands with devices and volumes
// REQUIREMENT: FR-032, FR-037, FR-043 (Deployment Module - Docker command generation)

use crate::models::volume_mapping::VolumeMapping;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerRunConfig {
    pub image: String,
    pub container_name: String,
    pub ports: Vec<(u16, u16)>,         // (host_port, container_port)
    pub volumes: Vec<(String, String)>, // (host_path, container_path)
    pub devices: Vec<String>,
    pub environment: HashMap<String, String>,
    pub restart_policy: String,
    pub privileged: bool,
    pub network_mode: Option<String>,
}

impl DockerRunConfig {
    pub fn new(image: String, container_name: String) -> Self {
        Self {
            image,
            container_name,
            ports: Vec::new(),
            volumes: Vec::new(),
            devices: Vec::new(),
            environment: HashMap::new(),
            restart_policy: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        }
    }

    pub fn with_port(mut self, host_port: u16, container_port: u16) -> Self {
        self.ports.push((host_port, container_port));
        self
    }

    pub fn with_volume(mut self, host_path: String, container_path: String) -> Self {
        self.volumes.push((host_path, container_path));
        self
    }

    pub fn with_device(mut self, device_path: String) -> Self {
        self.devices.push(device_path);
        self
    }

    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }

    pub fn with_restart(mut self, policy: String) -> Self {
        self.restart_policy = policy;
        self
    }

    pub fn privileged(mut self) -> Self {
        self.privileged = true;
        self
    }

    pub fn with_network(mut self, mode: String) -> Self {
        self.network_mode = Some(mode);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerServiceConfig {
    pub image: String,
    pub container_name: String,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub devices: Vec<String>,
    pub environment: HashMap<String, String>,
    pub restart: String,
    pub privileged: bool,
    pub network_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerComposeConfig {
    pub version: String,
    pub services: HashMap<String, DockerServiceConfig>,
}

impl DockerComposeConfig {
    pub fn new() -> Self {
        Self {
            version: "3.9".to_string(),
            services: HashMap::new(),
        }
    }

    pub fn add_service(&mut self, name: String, service: DockerServiceConfig) {
        self.services.insert(name, service);
    }
}

impl Default for DockerComposeConfig {
    fn default() -> Self {
        Self::new()
    }
}

// T098: Generate docker run command
pub fn generate_docker_run_command(config: &DockerRunConfig) -> String {
    let mut cmd = vec!["docker run".to_string()];

    // Add detached mode
    cmd.push("-d".to_string());

    // Add container name
    cmd.push("--name".to_string());
    cmd.push(escape_arg(&config.container_name));

    // Add restart policy
    if !config.restart_policy.is_empty() {
        cmd.push("--restart".to_string());
        cmd.push(escape_arg(&config.restart_policy));
    }

    // Add privileged mode
    if config.privileged {
        cmd.push("--privileged".to_string());
    }

    // Add network mode
    if let Some(ref network) = config.network_mode {
        cmd.push("--network".to_string());
        cmd.push(escape_arg(network));
    }

    // T099: Add devices
    for device in &config.devices {
        cmd.push("--device".to_string());
        cmd.push(escape_arg(device));
    }

    // T100: Add volumes
    for (host, container) in &config.volumes {
        cmd.push("-v".to_string());
        let volume_arg = format!("{}:{}", escape_path(host), container);
        cmd.push(volume_arg);
    }

    // Add ports
    for (host_port, container_port) in &config.ports {
        cmd.push("-p".to_string());
        cmd.push(format!("{}:{}", host_port, container_port));
    }

    // Add environment variables
    for (key, value) in &config.environment {
        cmd.push("-e".to_string());
        cmd.push(format!("{}={}", escape_arg(key), escape_arg(value)));
    }

    // Add image
    cmd.push(config.image.clone());

    cmd.join(" ")
}

// T098: Generate docker-compose YAML
pub fn generate_docker_compose_yaml(config: &DockerComposeConfig) -> String {
    let mut yaml = String::new();

    yaml.push_str(&format!("version: \"{}\"\n", config.version));
    yaml.push_str("services:\n");

    for (service_name, service) in &config.services {
        yaml.push_str(&format!("  {}:\n", service_name));
        yaml.push_str(&format!("    image: {}\n", service.image));
        yaml.push_str(&format!("    container_name: {}\n", service.container_name));

        if !service.restart.is_empty() {
            yaml.push_str(&format!("    restart: {}\n", service.restart));
        }

        if service.privileged {
            yaml.push_str("    privileged: true\n");
        }

        if let Some(ref network) = service.network_mode {
            yaml.push_str(&format!("    network_mode: {}\n", network));
        }

        if !service.ports.is_empty() {
            yaml.push_str("    ports:\n");
            for port in &service.ports {
                yaml.push_str(&format!("      - {}\n", port));
            }
        }

        if !service.volumes.is_empty() {
            yaml.push_str("    volumes:\n");
            for volume in &service.volumes {
                yaml.push_str(&format!("      - {}\n", volume));
            }
        }

        if !service.devices.is_empty() {
            yaml.push_str("    devices:\n");
            for device in &service.devices {
                yaml.push_str(&format!("      - {}\n", device));
            }
        }

        if !service.environment.is_empty() {
            yaml.push_str("    environment:\n");
            for (key, value) in &service.environment {
                yaml.push_str(&format!("      - {}={}\n", key, value));
            }
        }
    }

    yaml
}

// T098: Validate Docker configuration
pub fn validate_docker_config(config: &DockerRunConfig) -> Result<(), String> {
    // Check image is not empty
    if config.image.is_empty() {
        return Err("Docker image name cannot be empty".to_string());
    }

    // Check container name is not empty
    if config.container_name.is_empty() {
        return Err("Container name cannot be empty".to_string());
    }

    // Validate ports
    for (host_port, container_port) in &config.ports {
        if *host_port == 0 {
            return Err("Host port cannot be 0".to_string());
        }
        if *container_port == 0 {
            return Err("Container port cannot be 0".to_string());
        }
    }

    // Validate restart policy
    let valid_policies = ["no", "always", "unless-stopped", "on-failure"];
    if !config.restart_policy.is_empty()
        && !valid_policies.contains(&config.restart_policy.as_str())
    {
        return Err(format!(
            "Invalid restart policy: {}. Must be one of: {}",
            config.restart_policy,
            valid_policies.join(", ")
        ));
    }

    Ok(())
}

// T099: Generate device mount arguments for GPU
pub fn generate_gpu_device_args(gpu_type: &str) -> Vec<String> {
    match gpu_type.to_lowercase().as_str() {
        "intel" => vec!["/dev/dri/renderD128".to_string()],
        "nvidia" => vec![], // NVIDIA uses runtime, not device mounts
        "amd" => vec![
            "/dev/dri/renderD128".to_string(),
            "/dev/dri/card0".to_string(),
        ],
        _ => vec![],
    }
}

// T099: Generate device mount arguments for Coral TPU
pub fn generate_coral_device_args() -> Vec<String> {
    vec!["/dev/apex_0".to_string(), "/dev/bus/usb".to_string()]
}

// T100: Convert VolumeMapping to docker volume argument
pub fn volume_mapping_to_docker_arg(mapping: &VolumeMapping) -> String {
    mapping.to_docker_arg()
}

// T178: Add volume mappings to Docker configuration
pub fn add_volume_mappings_to_config(
    config: DockerRunConfig,
    mappings: &[VolumeMapping],
) -> DockerRunConfig {
    let mut updated_config = config;

    for mapping in mappings {
        let host_path = mapping.host_path.display().to_string();
        let container_path = mapping.container_path.clone();
        updated_config = updated_config.with_volume(host_path, container_path);
    }

    updated_config
}

// T178: Convert volume mappings to Docker Compose volume list
pub fn volume_mappings_to_compose_volumes(mappings: &[VolumeMapping]) -> Vec<String> {
    mappings.iter().map(|m| m.to_docker_arg()).collect()
}

// Helper: Escape argument for shell
fn escape_arg(arg: &str) -> String {
    if arg.contains(' ')
        || arg.contains('$')
        || arg.contains('"')
        || arg.contains('\'')
        || arg.contains('\\')
    {
        format!("\"{}\"", arg.replace('"', "\\\""))
    } else {
        arg.to_string()
    }
}

// Helper: Escape path for shell (handles spaces)
fn escape_path(path: &str) -> String {
    if path.contains(' ') {
        format!("\"{}\"", path)
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_basic_docker_run_command() {
        let config = DockerRunConfig::new(
            "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            "frigate".to_string(),
        )
        .with_port(5000, 5000)
        .with_port(8554, 8554)
        .with_port(8555, 8555)
        .with_volume("/etc/localtime".to_string(), "/etc/localtime".to_string())
        .with_volume("/path/to/config".to_string(), "/config".to_string())
        .with_volume("/path/to/storage".to_string(), "/media/frigate".to_string())
        .with_restart("unless-stopped".to_string());

        let command = generate_docker_run_command(&config);

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
        let config = DockerRunConfig::new(
            "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            "frigate".to_string(),
        )
        .with_device("/dev/dri/renderD128".to_string());

        let command = generate_docker_run_command(&config);

        assert!(command.contains("--device /dev/dri/renderD128"));
    }

    #[test]
    fn test_generate_docker_run_with_coral_tpu() {
        let config = DockerRunConfig::new(
            "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            "frigate".to_string(),
        )
        .with_device("/dev/apex_0".to_string())
        .with_device("/dev/bus/usb".to_string());

        let command = generate_docker_run_command(&config);

        assert!(command.contains("--device /dev/apex_0"));
        assert!(command.contains("--device /dev/bus/usb"));
    }

    #[test]
    fn test_generate_docker_run_with_environment_variables() {
        let config = DockerRunConfig::new(
            "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            "frigate".to_string(),
        )
        .with_env("FRIGATE_RTSP_PASSWORD".to_string(), "secret123".to_string())
        .with_env("TZ".to_string(), "America/New_York".to_string());

        let command = generate_docker_run_command(&config);

        assert!(command.contains("-e FRIGATE_RTSP_PASSWORD=secret123"));
        assert!(command.contains("-e TZ=America/New_York"));
    }

    #[test]
    fn test_generate_docker_run_with_privileged_mode() {
        let config =
            DockerRunConfig::new("frigate:latest".to_string(), "frigate".to_string()).privileged();

        let command = generate_docker_run_command(&config);

        assert!(command.contains("--privileged"));
    }

    #[test]
    fn test_generate_docker_compose_basic() {
        let mut config = DockerComposeConfig::new();

        let mut env = HashMap::new();
        env.insert("TZ".to_string(), "UTC".to_string());

        let service = DockerServiceConfig {
            image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
            container_name: "frigate".to_string(),
            ports: vec!["5000:5000".to_string(), "8554:8554".to_string()],
            volumes: vec![
                "/etc/localtime:/etc/localtime:ro".to_string(),
                "/path/to/config:/config".to_string(),
            ],
            devices: vec![],
            environment: env,
            restart: "unless-stopped".to_string(),
            privileged: false,
            network_mode: None,
        };

        config.add_service("frigate".to_string(), service);

        let yaml = generate_docker_compose_yaml(&config);

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
        let mut config = DockerComposeConfig::new();

        let service = DockerServiceConfig {
            image: "frigate:latest".to_string(),
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
        };

        config.add_service("frigate".to_string(), service);

        let yaml = generate_docker_compose_yaml(&config);

        assert!(yaml.contains("devices:"));
        assert!(yaml.contains("- /dev/dri/renderD128:/dev/dri/renderD128"));
        assert!(yaml.contains("- /dev/apex_0:/dev/apex_0"));
    }

    #[test]
    fn test_validate_docker_config_valid() {
        let config = DockerRunConfig::new("frigate:latest".to_string(), "frigate".to_string())
            .with_port(5000, 5000);

        let result = validate_docker_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_docker_config_missing_image() {
        let config = DockerRunConfig::new("".to_string(), "frigate".to_string());

        let result = validate_docker_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("image"));
    }

    #[test]
    fn test_validate_docker_config_invalid_port() {
        let config = DockerRunConfig::new("frigate:latest".to_string(), "frigate".to_string())
            .with_port(0, 5000);

        let result = validate_docker_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("port"));
    }

    #[test]
    fn test_command_escaping() {
        let config = DockerRunConfig::new("frigate:latest".to_string(), "frigate".to_string())
            .with_volume(
                "/path/with spaces/config".to_string(),
                "/config".to_string(),
            )
            .with_volume("/path/with$dollar".to_string(), "/media".to_string());

        let command = generate_docker_run_command(&config);

        // Paths with spaces should be quoted
        assert!(
            command.contains("\"/path/with spaces/config\"")
                || command.contains("'/path/with spaces/config'")
        );
    }

    #[test]
    fn test_generate_docker_run_detached_mode() {
        let config = DockerRunConfig::new("frigate:latest".to_string(), "frigate".to_string());

        let command = generate_docker_run_command(&config);

        // Should include -d flag for detached mode
        assert!(command.contains(" -d ") || command.contains("docker run -d"));
    }

    #[test]
    fn test_generate_docker_run_network_mode() {
        let config = DockerRunConfig::new("frigate:latest".to_string(), "frigate".to_string())
            .with_network("host".to_string());

        let command = generate_docker_run_command(&config);

        assert!(command.contains("--network host"));
    }

    #[test]
    fn test_gpu_device_args() {
        let intel_devices = generate_gpu_device_args("intel");
        assert_eq!(intel_devices, vec!["/dev/dri/renderD128"]);

        let amd_devices = generate_gpu_device_args("amd");
        assert_eq!(amd_devices.len(), 2);
        assert!(amd_devices.contains(&"/dev/dri/renderD128".to_string()));
    }

    #[test]
    fn test_coral_device_args() {
        let devices = generate_coral_device_args();
        assert_eq!(devices.len(), 2);
        assert!(devices.contains(&"/dev/apex_0".to_string()));
        assert!(devices.contains(&"/dev/bus/usb".to_string()));
    }

    #[test]
    fn test_add_volume_mappings_to_config() {
        use crate::models::volume_mapping::{VolumeMapping, VolumeMappingType};
        use std::path::PathBuf;

        let config = DockerRunConfig::new("frigate:latest".to_string(), "frigate".to_string());

        let mappings = vec![
            VolumeMapping::new(
                PathBuf::from("/host/config"),
                "/config".to_string(),
                VolumeMappingType::Config,
            ),
            VolumeMapping::new(
                PathBuf::from("/host/recordings"),
                "/media/frigate/recordings".to_string(),
                VolumeMappingType::Recordings,
            ),
        ];

        let updated_config = add_volume_mappings_to_config(config, &mappings);

        assert_eq!(updated_config.volumes.len(), 2);
        assert_eq!(
            updated_config.volumes[0],
            ("/host/config".to_string(), "/config".to_string())
        );
        assert_eq!(
            updated_config.volumes[1],
            (
                "/host/recordings".to_string(),
                "/media/frigate/recordings".to_string()
            )
        );
    }

    #[test]
    fn test_volume_mappings_to_compose_volumes() {
        use crate::models::volume_mapping::{VolumeMapping, VolumeMappingType};
        use std::path::PathBuf;

        let mappings = vec![
            VolumeMapping::new(
                PathBuf::from("/host/config"),
                "/config".to_string(),
                VolumeMappingType::Config,
            ),
            VolumeMapping::new(
                PathBuf::from("/etc/localtime"),
                "/etc/localtime".to_string(),
                VolumeMappingType::Custom,
            )
            .read_only(),
        ];

        let volumes = volume_mappings_to_compose_volumes(&mappings);

        assert_eq!(volumes.len(), 2);
        assert_eq!(volumes[0], "/host/config:/config");
        assert_eq!(volumes[1], "/etc/localtime:/etc/localtime:ro");
    }
}
