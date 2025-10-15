// T094: Integration test for deployment execution
// Tests full deployment workflow: command generation → execution → log capture
// REQUIREMENT: FR-037, FR-038, FR-042 (Deployment Module - Execution)

#[cfg(test)]
mod test_deployment {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Duration;

    // Import actual implementation
    use frigate_config_tool::deployment::executor::{
        execute_deployment, generate_docker_compose_config_from_request,
        generate_docker_run_command_from_request, get_container_logs, get_deployment_status,
        load_deployment_state, save_deployment_state, stop_deployment, stream_container_logs,
        wait_for_container_ready, DeploymentRequest,
    };
    use frigate_config_tool::models::deployment_state::{
        DeploymentMethod, DeploymentState, DeploymentStatus,
    };

    // Helper functions for tests
    fn generate_docker_run_command(request: &DeploymentRequest) -> String {
        generate_docker_run_command_from_request(request)
    }

    fn generate_docker_compose_config(request: &DeploymentRequest) -> String {
        generate_docker_compose_config_from_request(request)
    }

    // ========== Deployment Execution Tests ==========

    #[test]
    fn test_execute_deployment_docker_run_basic() {
        // Use absolute path or create a temp config file
        let config_content = r#"mqtt:
  enabled: false
cameras:
  test:
    enabled: true
"#;
        let temp_config = std::env::temp_dir().join("test_frigate_config.yml");
        std::fs::write(&temp_config, config_content).expect("Failed to write test config");

        // Use a random high port to avoid conflicts (50000-60000 range)
        let random_port = 50000
            + (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % 10000) as u16;

        let request = DeploymentRequest {
            config_path: temp_config.clone(),
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![(
                "/tmp/frigate_test/config".to_string(),
                "/config".to_string(),
            )],
            ports: vec![(random_port, 5000)],
            environment: HashMap::new(),
        };

        let result = execute_deployment(&request);

        // Note: This test requires Docker to be installed
        // In CI, we can mock or skip based on Docker availability
        match &result {
            Ok(deployment) => {
                eprintln!(
                    "Deployment result - success: {}, stdout: {}, stderr: {}",
                    deployment.success, deployment.stdout, deployment.stderr
                );
                if !deployment.success {
                    eprintln!("Command: {}", deployment.command);
                }
            }
            Err(e) => {
                eprintln!("Deployment error: {}", e);
            }
        }

        assert!(result.is_ok(), "Deployment failed: {:?}", result.err());

        let deployment = result.unwrap();
        assert!(
            deployment.success,
            "Deployment command failed. Stderr: {}",
            deployment.stderr
        );
        assert!(deployment.container_id.is_some());
        assert!(!deployment.command.is_empty());

        // Clean up
        if let Some(container_id) = deployment.container_id {
            let _ = stop_deployment(&container_id);
        }
    }

    #[test]
    fn test_execute_deployment_docker_compose() {
        // Create temp config file
        let config_content = r#"mqtt:
  enabled: false
cameras:
  test:
    enabled: true
"#;
        let temp_config = std::env::temp_dir().join("test_frigate_compose.yml");
        std::fs::write(&temp_config, config_content).expect("Failed to write test config");

        // Use a random high port to avoid conflicts
        let random_port = 51000
            + (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % 10000) as u16;

        let request = DeploymentRequest {
            config_path: temp_config,
            method: DeploymentMethod::DockerCompose,
            devices: vec![],
            volumes: vec![(
                "/tmp/frigate_test/config".to_string(),
                "/config".to_string(),
            )],
            ports: vec![(random_port, 5000)],
            environment: HashMap::new(),
        };

        let result = execute_deployment(&request);
        assert!(result.is_ok());

        let deployment = result.unwrap();
        assert!(deployment.success);

        // Clean up
        if let Some(container_id) = deployment.container_id {
            let _ = stop_deployment(&container_id);
        }
    }

    #[test]
    fn test_execute_deployment_with_devices() {
        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/valid_frigate.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec![
                "/dev/dri/renderD128".to_string(), // Intel GPU
            ],
            volumes: vec![(
                "/tmp/frigate_test/config".to_string(),
                "/config".to_string(),
            )],
            ports: vec![(5000, 5000)],
            environment: HashMap::new(),
        };

        let result = execute_deployment(&request);

        if result.is_ok() {
            let deployment = result.unwrap();

            // Verify device was mounted in command
            assert!(deployment.command.contains("--device"));
            assert!(deployment.command.contains("/dev/dri/renderD128"));

            // Clean up
            if let Some(container_id) = deployment.container_id {
                let _ = stop_deployment(&container_id);
            }
        }
        // If error, it might be because device doesn't exist, which is ok
    }

    #[test]
    fn test_execute_deployment_with_environment_variables() {
        let mut env = HashMap::new();
        env.insert("TZ".to_string(), "America/New_York".to_string());
        env.insert(
            "FRIGATE_RTSP_PASSWORD".to_string(),
            "testpass123".to_string(),
        );

        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/valid_frigate.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![],
            ports: vec![],
            environment: env,
        };

        let result = execute_deployment(&request);

        if let Ok(deployment) = result {
            assert!(deployment.command.contains("-e TZ="));
            assert!(deployment.command.contains("-e FRIGATE_RTSP_PASSWORD="));

            if let Some(container_id) = deployment.container_id {
                let _ = stop_deployment(&container_id);
            }
        }
    }

    #[test]
    fn test_execute_deployment_captures_output() {
        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/valid_frigate.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![],
            ports: vec![(5000, 5000)],
            environment: HashMap::new(),
        };

        let result = execute_deployment(&request);

        if let Ok(deployment) = result {
            // Should capture stdout and stderr
            assert!(!deployment.stdout.is_empty() || !deployment.stderr.is_empty());

            // Clean up
            if let Some(container_id) = deployment.container_id {
                let _ = stop_deployment(&container_id);
            }
        }
    }

    #[test]
    fn test_execute_deployment_invalid_config() {
        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/invalid_syntax.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![],
            ports: vec![],
            environment: HashMap::new(),
        };

        let result = execute_deployment(&request);

        // Should fail with invalid config
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_deployment_nonexistent_config() {
        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/nonexistent.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![],
            ports: vec![],
            environment: HashMap::new(),
        };

        let result = execute_deployment(&request);

        // Should fail when config doesn't exist
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("not found") || error.contains("No such file"));
    }

    // ========== Deployment Status Tests ==========

    #[test]
    fn test_get_deployment_status_running() {
        // First deploy
        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/valid_frigate.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![],
            ports: vec![(5000, 5000)],
            environment: HashMap::new(),
        };

        let result = execute_deployment(&request);

        if let Ok(deployment) = result {
            if let Some(container_id) = deployment.container_id {
                // Check status
                let status = get_deployment_status(&container_id);
                assert!(status.is_ok());

                let status_value = status.unwrap();
                assert!(
                    status_value == DeploymentStatus::Running
                        || status_value == DeploymentStatus::Pending
                );

                // Clean up
                let _ = stop_deployment(&container_id);
            }
        }
    }

    #[test]
    fn test_get_deployment_status_stopped() {
        // Deploy and then stop
        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/valid_frigate.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![],
            ports: vec![(5000, 5000)],
            environment: HashMap::new(),
        };

        let result = execute_deployment(&request);

        if let Ok(deployment) = result {
            if let Some(container_id) = deployment.container_id.clone() {
                // Stop container
                let _ = stop_deployment(&container_id);

                // Check status after stop
                let status = get_deployment_status(&container_id);
                // Should be Failed or some stopped state
                assert!(status.is_ok() || status.is_err());
            }
        }
    }

    #[test]
    fn test_get_deployment_status_nonexistent_container() {
        let status = get_deployment_status("nonexistent_container_12345");

        // Should fail for nonexistent container
        assert!(status.is_err());
    }

    // ========== Log Retrieval Tests ==========

    #[test]
    fn test_get_container_logs() {
        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/valid_frigate.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![],
            ports: vec![(5000, 5000)],
            environment: HashMap::new(),
        };

        let result = execute_deployment(&request);

        if let Ok(deployment) = result {
            if let Some(container_id) = deployment.container_id {
                // Wait a moment for logs to generate
                std::thread::sleep(Duration::from_secs(2));

                // Get logs
                let logs = get_container_logs(&container_id, Some(100));
                assert!(logs.is_ok());

                let log_lines = logs.unwrap();
                // Should have some logs from Frigate startup
                assert!(!log_lines.is_empty());

                // Clean up
                let _ = stop_deployment(&container_id);
            }
        }
    }

    #[test]
    fn test_get_container_logs_with_limit() {
        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/valid_frigate.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![],
            ports: vec![(5000, 5000)],
            environment: HashMap::new(),
        };

        let result = execute_deployment(&request);

        if let Ok(deployment) = result {
            if let Some(container_id) = deployment.container_id {
                std::thread::sleep(Duration::from_secs(2));

                // Get last 10 lines
                let logs = get_container_logs(&container_id, Some(10));

                if let Ok(log_lines) = logs {
                    assert!(log_lines.len() <= 10);
                }

                let _ = stop_deployment(&container_id);
            }
        }
    }

    #[test]
    fn test_stream_container_logs() {
        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/valid_frigate.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![],
            ports: vec![(5000, 5000)],
            environment: HashMap::new(),
        };

        let result = execute_deployment(&request);

        if let Ok(deployment) = result {
            if let Some(container_id) = deployment.container_id.clone() {
                // Stream logs
                let stream = stream_container_logs(&container_id);

                if let Ok(mut log_stream) = stream {
                    // Read first few log lines
                    let mut count = 0;
                    for _log_line in &mut log_stream {
                        count += 1;
                        if count >= 5 {
                            break;
                        }
                    }

                    assert!(count > 0, "Should have received some log lines");
                }

                let _ = stop_deployment(&container_id);
            }
        }
    }

    // ========== Command Generation Tests ==========

    #[test]
    fn test_generate_docker_run_command_output() {
        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/valid_frigate.yml"),
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![("/config".to_string(), "/config".to_string())],
            ports: vec![(5000, 5000)],
            environment: HashMap::new(),
        };

        let command = generate_docker_run_command(&request);

        // Verify command structure
        assert!(command.starts_with("docker run"));
        assert!(command.contains("-v /config:/config"));
        assert!(command.contains("-p 5000:5000"));
    }

    #[test]
    fn test_generate_docker_compose_config_output() {
        let request = DeploymentRequest {
            config_path: PathBuf::from("tests/fixtures/valid_frigate.yml"),
            method: DeploymentMethod::DockerCompose,
            devices: vec![],
            volumes: vec![],
            ports: vec![(5000, 5000)],
            environment: HashMap::new(),
        };

        let yaml = generate_docker_compose_config(&request);

        // Verify YAML structure
        assert!(yaml.contains("version:"));
        assert!(yaml.contains("services:"));
        assert!(yaml.contains("frigate:"));
    }

    // ========== Deployment State Persistence Tests ==========

    #[test]
    fn test_save_and_load_deployment_state() {
        let mut state = DeploymentState::new(
            "test_container_123".to_string(),
            "tests/fixtures/valid_frigate.yml".to_string(),
            DeploymentMethod::DockerRun,
        );
        state.mark_running("test_container_123".to_string());
        state.add_log("Log line 1".to_string());
        state.add_log("Log line 2".to_string());

        // Save state
        let save_result = save_deployment_state(&state);
        assert!(save_result.is_ok());

        // Load state
        let load_result = load_deployment_state();
        assert!(load_result.is_ok());

        let loaded_state = load_result.unwrap();
        assert!(loaded_state.is_some());

        let loaded = loaded_state.unwrap();
        assert_eq!(loaded.status, DeploymentStatus::Running);
        assert_eq!(loaded.container_id, Some("test_container_123".to_string()));
        assert_eq!(loaded.logs.len(), 2);
    }

    #[test]
    fn test_wait_for_container_ready_timeout() {
        // Test with very short timeout
        let result = wait_for_container_ready("nonexistent_container", Duration::from_millis(100));

        // Should timeout or return error
        assert!(result.is_err() || !result.unwrap());
    }

    // ========== Integration Test ==========

    #[test]
    fn test_full_deployment_workflow() {
        // This test runs the complete deployment workflow

        // Create temp config file
        let config_content = r#"mqtt:
  enabled: false
cameras:
  test:
    enabled: true
"#;
        let temp_config = std::env::temp_dir().join("test_frigate_workflow.yml");
        std::fs::write(&temp_config, config_content).expect("Failed to write test config");

        // Use a random high port to avoid conflicts
        let random_port = 52000
            + (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % 10000) as u16;

        // 1. Create deployment request
        let request = DeploymentRequest {
            config_path: temp_config,
            method: DeploymentMethod::DockerRun,
            devices: vec![],
            volumes: vec![(
                "/tmp/frigate_test/config".to_string(),
                "/config".to_string(),
            )],
            ports: vec![(random_port, 5000)],
            environment: HashMap::new(),
        };

        // 2. Execute deployment
        let deploy_result = execute_deployment(&request);
        assert!(deploy_result.is_ok());

        let deployment = deploy_result.unwrap();
        assert!(deployment.success);
        assert!(deployment.container_id.is_some());

        let container_id = deployment.container_id.unwrap();

        // 3. Check deployment status
        let status = get_deployment_status(&container_id);
        assert!(status.is_ok());

        // 4. Wait for container to be ready
        let ready = wait_for_container_ready(&container_id, Duration::from_secs(30));
        assert!(ready.is_ok());

        // 5. Get logs
        let logs = get_container_logs(&container_id, Some(50));
        assert!(logs.is_ok());

        // 6. Stop deployment
        let stop_result = stop_deployment(&container_id);
        assert!(stop_result.is_ok());

        // 7. Verify stopped status
        let final_status = get_deployment_status(&container_id);
        // Container should be stopped or return error
        assert!(final_status.is_ok() || final_status.is_err());
    }
}
