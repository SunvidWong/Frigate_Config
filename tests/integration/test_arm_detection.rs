//! Integration tests for ARM architecture hardware detection
//!
//! These tests verify that hardware detection works correctly on ARM-based systems,
//! including both ARM64 (aarch64) and ARM32 architectures on Linux, macOS, and Windows.

use serde_json::Value;
use std::process::Command;

/// Test ARM architecture detection on Linux
#[test]
#[cfg(target_os = "linux")]
fn test_arm_detection_linux() {
    // Get the agent binary path
    let agent_path = get_agent_binary_path();

    // Execute the agent detect command
    let output = Command::new(&agent_path)
        .arg("detect")
        .output()
        .expect("Failed to execute agent");

    assert!(output.status.success(), "Agent command failed");

    // Parse JSON output
    let json_output: Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse JSON output");

    // Verify architecture is reported correctly
    if is_arm_architecture() {
        // On ARM systems, devices should report ARM architecture
        let devices = json_output.get("devices").expect("No devices field");
        assert!(devices.is_array(), "Devices should be an array");

        if let Some(device_array) = devices.as_array() {
            for device in device_array {
                let arch = device
                    .get("architecture")
                    .and_then(|a| a.as_str())
                    .expect("Device should have architecture field");

                assert!(
                    arch == "arm64" || arch == "aarch64" || arch == "arm",
                    "Architecture should be ARM variant, got: {}",
                    arch
                );

                // Verify platform is reported correctly
                let platform = device
                    .get("platform")
                    .and_then(|p| p.as_str())
                    .expect("Device should have platform field");

                assert_eq!(platform, "linux", "Platform should be linux");
            }
        }
    }
}

/// Test ARM architecture detection on macOS (Apple Silicon)
#[test]
#[cfg(target_os = "macos")]
fn test_arm_detection_macos() {
    // Get the agent binary path
    let agent_path = get_agent_binary_path();

    // Execute the agent detect command
    let output = Command::new(&agent_path)
        .arg("detect")
        .output()
        .expect("Failed to execute agent");

    assert!(output.status.success(), "Agent command failed");

    // Parse JSON output
    let json_output: Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse JSON output");

    // Verify architecture is reported correctly
    if is_arm_architecture() {
        // On Apple Silicon, devices should report arm64
        let devices = json_output.get("devices").expect("No devices field");
        assert!(devices.is_array(), "Devices should be an array");

        if let Some(device_array) = devices.as_array() {
            for device in device_array {
                let arch = device
                    .get("architecture")
                    .and_then(|a| a.as_str())
                    .expect("Device should have architecture field");

                assert_eq!(
                    arch, "arm64",
                    "Architecture should be arm64 on Apple Silicon"
                );

                // Verify platform is reported correctly
                let platform = device
                    .get("platform")
                    .and_then(|p| p.as_str())
                    .expect("Device should have platform field");

                assert_eq!(platform, "darwin", "Platform should be darwin");
            }

            // On Apple Silicon, we should detect Neural Engine
            let has_neural_engine = device_array.iter().any(|device| {
                device
                    .get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n.contains("Neural Engine"))
                    .unwrap_or(false)
            });

            if has_neural_engine {
                println!("✓ Apple Neural Engine detected on Apple Silicon");
            }
        }
    }
}

/// Test ARM architecture detection on Windows
#[test]
#[cfg(target_os = "windows")]
fn test_arm_detection_windows() {
    // Get the agent binary path
    let agent_path = get_agent_binary_path();

    // Execute the agent detect command
    let output = Command::new(&agent_path)
        .arg("detect")
        .output()
        .expect("Failed to execute agent");

    assert!(output.status.success(), "Agent command failed");

    // Parse JSON output
    let json_output: Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse JSON output");

    // Verify architecture is reported correctly
    if is_arm_architecture() {
        // On ARM Windows, devices should report arm64
        let devices = json_output.get("devices").expect("No devices field");
        assert!(devices.is_array(), "Devices should be an array");

        if let Some(device_array) = devices.as_array() {
            for device in device_array {
                let arch = device
                    .get("architecture")
                    .and_then(|a| a.as_str())
                    .expect("Device should have architecture field");

                assert_eq!(arch, "arm64", "Architecture should be arm64 on ARM Windows");

                // Verify platform is reported correctly
                let platform = device
                    .get("platform")
                    .and_then(|p| p.as_str())
                    .expect("Device should have platform field");

                assert_eq!(platform, "windows", "Platform should be windows");
            }
        }
    }
}

/// Test ARM-specific hardware capabilities
#[test]
fn test_arm_specific_capabilities() {
    // Get the agent binary path
    let agent_path = get_agent_binary_path();

    // Execute the agent detect command
    let output = Command::new(&agent_path)
        .arg("detect")
        .output()
        .expect("Failed to execute agent");

    assert!(output.status.success(), "Agent command failed");

    // Parse JSON output
    let json_output: Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse JSON output");

    if is_arm_architecture() {
        let devices = json_output.get("devices").expect("No devices field");

        if let Some(device_array) = devices.as_array() {
            // Check for ARM-specific capabilities
            for device in device_array {
                let device_type = device.get("type").and_then(|t| t.as_str()).unwrap_or("");

                let empty_vec = vec![];
                let capabilities = device
                    .get("capabilities")
                    .and_then(|c| c.as_array())
                    .unwrap_or(&empty_vec);

                match device_type {
                    "gpu" => {
                        // ARM GPUs might have different capabilities
                        // e.g., Mali, Adreno, Apple GPU
                        if cfg!(target_os = "macos") {
                            // Apple Silicon should have Metal and VideoToolbox
                            let caps_str: Vec<String> = capabilities
                                .iter()
                                .filter_map(|c| c.as_str().map(String::from))
                                .collect();

                            assert!(
                                caps_str.contains(&"metal".to_string()),
                                "Apple GPU should support Metal"
                            );
                        }
                    }
                    "tpu" => {
                        // Check for ARM-compatible TPUs
                        if cfg!(target_os = "macos") {
                            // Apple Neural Engine on M1/M2/M3
                            let name = device.get("name").and_then(|n| n.as_str()).unwrap_or("");

                            if name.contains("Neural Engine") {
                                let caps_str: Vec<String> = capabilities
                                    .iter()
                                    .filter_map(|c| c.as_str().map(String::from))
                                    .collect();

                                assert!(
                                    caps_str.contains(&"neural_engine".to_string())
                                        || caps_str.contains(&"coreml".to_string()),
                                    "Neural Engine should have appropriate capabilities"
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    } else {
        println!("Skipping ARM-specific capability tests on non-ARM architecture");
    }
}

/// Test ARM hardware enumeration consistency
#[test]
fn test_arm_hardware_enumeration() {
    // Get the agent binary path
    let agent_path = get_agent_binary_path();

    // Execute detection multiple times
    let mut outputs: Vec<Value> = Vec::new();

    for _ in 0..3 {
        let output = Command::new(&agent_path)
            .arg("detect")
            .output()
            .expect("Failed to execute agent");

        assert!(output.status.success(), "Agent command failed");

        let json_output: Value =
            serde_json::from_slice(&output.stdout).expect("Failed to parse JSON output");

        outputs.push(json_output);
    }

    // Verify consistency across runs
    if let Some(first_devices) = outputs[0].get("devices").and_then(|d| d.as_array()) {
        for output in &outputs[1..] {
            if let Some(devices) = output.get("devices").and_then(|d| d.as_array()) {
                assert_eq!(
                    first_devices.len(),
                    devices.len(),
                    "Device count should be consistent across detections"
                );

                // Verify each device is reported consistently
                for (i, device) in devices.iter().enumerate() {
                    let first_device = &first_devices[i];

                    // Check architecture consistency
                    assert_eq!(
                        first_device.get("architecture"),
                        device.get("architecture"),
                        "Architecture should be consistent for device {}",
                        i
                    );

                    // Check platform consistency
                    assert_eq!(
                        first_device.get("platform"),
                        device.get("platform"),
                        "Platform should be consistent for device {}",
                        i
                    );

                    // Check type consistency
                    assert_eq!(
                        first_device.get("type"),
                        device.get("type"),
                        "Type should be consistent for device {}",
                        i
                    );
                }
            }
        }
    }
}

/// Test cross-compile detection (agent reports correct architecture)
#[test]
fn test_agent_architecture_reporting() {
    // Get the agent binary path
    let agent_path = get_agent_binary_path();

    // Execute the agent detect command
    let output = Command::new(&agent_path)
        .arg("detect")
        .output()
        .expect("Failed to execute agent");

    assert!(output.status.success(), "Agent command failed");

    // Parse JSON output
    let json_output: Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse JSON output");

    let devices = json_output.get("devices").expect("No devices field");

    if let Some(device_array) = devices.as_array() {
        for device in device_array {
            let arch = device
                .get("architecture")
                .and_then(|a| a.as_str())
                .expect("Device should have architecture field");

            // Verify the reported architecture matches the current system
            let current_arch = std::env::consts::ARCH;

            // Normalize architecture names for comparison
            let normalized_reported = normalize_arch(arch);
            let normalized_current = normalize_arch(current_arch);

            assert_eq!(
                normalized_reported, normalized_current,
                "Reported architecture {} should match current system architecture {}",
                arch, current_arch
            );
        }
    }
}

// Helper functions

/// Check if the current system is ARM-based
fn is_arm_architecture() -> bool {
    let arch = std::env::consts::ARCH;
    arch.starts_with("arm") || arch.starts_with("aarch")
}

/// Get the agent binary path
fn get_agent_binary_path() -> String {
    // Debug: Print current working directory
    if let Ok(cwd) = std::env::current_dir() {
        eprintln!("Test running from: {:?}", cwd);
    }

    // Try to find the agent binary in standard locations
    let possible_paths = vec![
        "../agent/target/debug/agent",
        "../agent/target/release/agent",
        "./agent/target/debug/agent",
        "./agent/target/release/agent",
        "agent/agent",
        "./agent/agent",
        "../agent/agent",
        "../../agent/agent",
    ];

    for path in possible_paths {
        let path_buf = std::path::Path::new(path);
        eprintln!("Checking path: {:?} - exists: {}", path, path_buf.exists());
        if path_buf.exists() {
            eprintln!("Found agent at: {}", path);
            return path.to_string();
        }
    }

    // Fallback to "agent" in PATH
    eprintln!("Agent not found in any path, falling back to 'agent' in PATH");
    "agent".to_string()
}

/// Normalize architecture name for comparison
fn normalize_arch(arch: &str) -> String {
    match arch {
        "aarch64" | "arm64" => "arm64".to_string(),
        "arm" | "armv7" | "armv7l" => "arm".to_string(),
        other => other.to_string(),
    }
}
