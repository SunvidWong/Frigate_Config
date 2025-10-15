// T103: Command execution logic
// Executes Docker commands and captures output
// REQUIREMENT: FR-038, FR-042 (Deployment Module - Command execution and log capture)

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::deployment::docker::{
    generate_docker_compose_yaml, generate_docker_run_command, DockerComposeConfig,
    DockerRunConfig, DockerServiceConfig,
};
use crate::models::deployment_state::{DeploymentMethod, DeploymentState, DeploymentStatus};

// Re-export for convenience

/// Deployment request containing all necessary configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRequest {
    pub config_path: PathBuf,
    pub method: DeploymentMethod,
    pub devices: Vec<String>,
    pub volumes: Vec<(String, String)>,
    pub ports: Vec<(u16, u16)>,
    pub environment: HashMap<String, String>,
}

/// Result of a deployment execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub success: bool,
    pub container_id: Option<String>,
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Execute a deployment request
pub fn execute_deployment(request: &DeploymentRequest) -> Result<DeploymentResult, String> {
    // Validate config file exists
    if !request.config_path.exists() {
        return Err(format!(
            "Configuration file not found: {}",
            request.config_path.display()
        ));
    }

    match request.method {
        DeploymentMethod::DockerRun => execute_docker_run(request),
        DeploymentMethod::DockerCompose => execute_docker_compose(request),
    }
}

/// Execute docker run deployment
fn execute_docker_run(request: &DeploymentRequest) -> Result<DeploymentResult, String> {
    // Generate unique container name with timestamp to avoid conflicts
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    let container_name = format!("frigate_test_{}", timestamp);

    // Generate docker run command
    let mut config = DockerRunConfig::new(
        "ghcr.io/blakeblackshear/frigate:stable".to_string(),
        container_name,
    );

    // Add ports
    for (host_port, container_port) in &request.ports {
        config = config.with_port(*host_port, *container_port);
    }

    // Add volumes
    for (host_path, container_path) in &request.volumes {
        config = config.with_volume(host_path.clone(), container_path.clone());
    }

    // Add devices
    for device in &request.devices {
        config = config.with_device(device.clone());
    }

    // Add environment variables
    for (key, value) in &request.environment {
        config = config.with_env(key.clone(), value.clone());
    }

    // Generate command string
    let command_str = generate_docker_run_command(&config);

    // Execute command
    execute_shell_command(&command_str)
}

/// Execute docker-compose deployment
fn execute_docker_compose(request: &DeploymentRequest) -> Result<DeploymentResult, String> {
    // Generate unique container name with timestamp to avoid conflicts
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    let container_name = format!("frigate_test_{}", timestamp);

    // Create docker-compose configuration
    let environment = request.environment.clone();

    let service = DockerServiceConfig {
        image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
        container_name,
        ports: request
            .ports
            .iter()
            .map(|(h, c)| format!("{}:{}", h, c))
            .collect(),
        volumes: request
            .volumes
            .iter()
            .map(|(h, c)| format!("{}:{}", h, c))
            .collect(),
        devices: request
            .devices
            .iter()
            .map(|d| format!("{}:{}", d, d))
            .collect(),
        environment,
        restart: "unless-stopped".to_string(),
        privileged: false,
        network_mode: None,
    };

    let mut services = HashMap::new();
    services.insert("frigate".to_string(), service);

    let compose_config = DockerComposeConfig {
        version: "3.9".to_string(),
        services,
    };

    // Generate docker-compose.yml content
    let yaml_content = generate_docker_compose_yaml(&compose_config);

    // Write to temporary file
    let temp_dir = std::env::temp_dir();
    let compose_file = temp_dir.join("docker-compose-frigate.yml");
    std::fs::write(&compose_file, &yaml_content)
        .map_err(|e| format!("Failed to write docker-compose.yml: {}", e))?;

    // Execute docker-compose up
    let command_str = format!("docker-compose -f {} up -d", compose_file.display());

    execute_shell_command(&command_str)
}

/// Execute a shell command and capture output
fn execute_shell_command(command: &str) -> Result<DeploymentResult, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code();
    let success = output.status.success();

    // Extract container ID from stdout (docker run returns container ID)
    let container_id = if success && !stdout.trim().is_empty() {
        Some(stdout.trim().lines().last().unwrap_or("").to_string())
    } else {
        None
    };

    Ok(DeploymentResult {
        success,
        container_id,
        command: command.to_string(),
        stdout,
        stderr,
        exit_code,
    })
}

/// Get deployment status for a container
pub fn get_deployment_status(container_id: &str) -> Result<DeploymentStatus, String> {
    let output = Command::new("docker")
        .args(["inspect", "--format", "{{.State.Status}}", container_id])
        .output()
        .map_err(|e| format!("Failed to get container status: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Container not found or error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    match status_str.as_str() {
        "running" => Ok(DeploymentStatus::Running),
        "created" => Ok(DeploymentStatus::Pending),
        "exited" | "dead" => Ok(DeploymentStatus::Failed),
        "paused" => Ok(DeploymentStatus::Pending),
        _ => Ok(DeploymentStatus::Failed),
    }
}

/// Stop a deployment
pub fn stop_deployment(container_id: &str) -> Result<(), String> {
    let output = Command::new("docker")
        .args(["stop", container_id])
        .output()
        .map_err(|e| format!("Failed to stop container: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to stop container: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Also remove the container
    let _ = Command::new("docker").args(["rm", container_id]).output();

    Ok(())
}

/// Get container logs
pub fn get_container_logs(container_id: &str, lines: Option<usize>) -> Result<Vec<String>, String> {
    let mut cmd = Command::new("docker");
    cmd.arg("logs");

    if let Some(n) = lines {
        cmd.arg("--tail").arg(n.to_string());
    }

    cmd.arg(container_id);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to get container logs: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to retrieve logs: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let logs = String::from_utf8_lossy(&output.stdout);
    let log_lines: Vec<String> = logs.lines().map(|s| s.to_string()).collect();

    Ok(log_lines)
}

/// Stream container logs (returns an iterator)
pub fn stream_container_logs(container_id: &str) -> Result<std::vec::IntoIter<String>, String> {
    // For now, we'll return all logs as an iterator
    // In a real implementation, this would use docker logs -f with streaming
    let logs = get_container_logs(container_id, None)?;
    Ok(logs.into_iter())
}

/// Wait for container to be ready
pub fn wait_for_container_ready(container_id: &str, timeout: Duration) -> Result<bool, String> {
    let start = SystemTime::now();

    loop {
        // Check if timeout exceeded
        if start.elapsed().unwrap_or(Duration::from_secs(0)) > timeout {
            return Ok(false);
        }

        // Check container status
        match get_deployment_status(container_id) {
            Ok(DeploymentStatus::Running) => return Ok(true),
            Ok(DeploymentStatus::Failed) => return Err("Container failed to start".to_string()),
            Ok(_) => {
                // Still pending, wait and retry
                std::thread::sleep(Duration::from_millis(500));
            }
            Err(e) => return Err(e),
        }
    }
}

/// Save deployment state to disk
pub fn save_deployment_state(state: &DeploymentState) -> Result<(), String> {
    let state_dir = std::env::temp_dir().join("frigate-config-tool");
    std::fs::create_dir_all(&state_dir)
        .map_err(|e| format!("Failed to create state directory: {}", e))?;

    let state_file = state_dir.join("deployment_state.json");

    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize state: {}", e))?;

    std::fs::write(&state_file, json).map_err(|e| format!("Failed to write state file: {}", e))?;

    Ok(())
}

/// Load deployment state from disk
pub fn load_deployment_state() -> Result<Option<DeploymentState>, String> {
    let state_file = std::env::temp_dir()
        .join("frigate-config-tool")
        .join("deployment_state.json");

    if !state_file.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(&state_file)
        .map_err(|e| format!("Failed to read state file: {}", e))?;

    let state: DeploymentState =
        serde_json::from_str(&json).map_err(|e| format!("Failed to deserialize state: {}", e))?;

    Ok(Some(state))
}

/// Generate docker run command from deployment request (helper for tests)
pub fn generate_docker_run_command_from_request(request: &DeploymentRequest) -> String {
    let mut config = DockerRunConfig::new(
        "ghcr.io/blakeblackshear/frigate:stable".to_string(),
        "frigate".to_string(),
    );

    for (host_port, container_port) in &request.ports {
        config = config.with_port(*host_port, *container_port);
    }

    for (host_path, container_path) in &request.volumes {
        config = config.with_volume(host_path.clone(), container_path.clone());
    }

    for device in &request.devices {
        config = config.with_device(device.clone());
    }

    for (key, value) in &request.environment {
        config = config.with_env(key.clone(), value.clone());
    }

    generate_docker_run_command(&config)
}

/// Generate docker-compose config from deployment request (helper for tests)
pub fn generate_docker_compose_config_from_request(request: &DeploymentRequest) -> String {
    let service = DockerServiceConfig {
        image: "ghcr.io/blakeblackshear/frigate:stable".to_string(),
        container_name: "frigate".to_string(),
        ports: request
            .ports
            .iter()
            .map(|(h, c)| format!("{}:{}", h, c))
            .collect(),
        volumes: request
            .volumes
            .iter()
            .map(|(h, c)| format!("{}:{}", h, c))
            .collect(),
        devices: request
            .devices
            .iter()
            .map(|d| format!("{}:{}", d, d))
            .collect(),
        environment: request.environment.clone(),
        restart: "unless-stopped".to_string(),
        privileged: false,
        network_mode: None,
    };

    let mut services = HashMap::new();
    services.insert("frigate".to_string(), service);

    let compose_config = DockerComposeConfig {
        version: "3.9".to_string(),
        services,
    };

    generate_docker_compose_yaml(&compose_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_command_from_request() {
        let mut env = HashMap::new();
        env.insert("TZ".to_string(), "UTC".to_string());

        let request = DeploymentRequest {
            config_path: PathBuf::from("/tmp/test.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec!["/dev/dri/renderD128".to_string()],
            volumes: vec![("/host/config".to_string(), "/config".to_string())],
            ports: vec![(5000, 5000)],
            environment: env,
        };

        let command = generate_docker_run_command_from_request(&request);

        assert!(command.contains("docker run"));
        assert!(command.contains("-p 5000:5000"));
        assert!(command.contains("-v /host/config:/config"));
        assert!(command.contains("--device /dev/dri/renderD128"));
        assert!(command.contains("-e TZ=UTC"));
    }

    #[test]
    fn test_generate_compose_from_request() {
        let request = DeploymentRequest {
            config_path: PathBuf::from("/tmp/test.yml"),
            method: DeploymentMethod::DockerCompose,
            devices: vec![],
            volumes: vec![("/host/config".to_string(), "/config".to_string())],
            ports: vec![(5000, 5000)],
            environment: HashMap::new(),
        };

        let yaml = generate_docker_compose_config_from_request(&request);

        assert!(yaml.contains("version:"));
        assert!(yaml.contains("services:"));
        assert!(yaml.contains("frigate:"));
        assert!(yaml.contains("5000:5000"));
        assert!(yaml.contains("/host/config:/config"));
    }

    #[test]
    fn test_save_and_load_state() {
        let state = DeploymentState::new(
            "test_container".to_string(),
            "/tmp/test.yml".to_string(),
            DeploymentMethod::DockerRun,
        );

        // Save
        let save_result = save_deployment_state(&state);
        assert!(save_result.is_ok());

        // Load
        let load_result = load_deployment_state();
        assert!(load_result.is_ok());

        let loaded = load_result.unwrap();
        assert!(loaded.is_some());

        let loaded_state = loaded.unwrap();
        assert_eq!(loaded_state.container_name, "test_container");
        assert_eq!(loaded_state.config_path, "/tmp/test.yml");
    }
}
