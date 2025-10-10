// Integration test for hardware detection command
// Tests the integration between Tauri backend and Go agent
// Per Constitution Principle VI: This test MUST be written first and MUST run before implementation

use std::process::Command;
use std::time::Duration;
use std::thread;

#[test]
fn test_hardware_detection_command_integration() {
    // This test verifies that the Tauri backend can execute the agent
    // and parse its output correctly

    // Run the agent directly first to ensure it's working
    let agent_result = run_agent_detect();
    assert!(agent_result.status.success(), "Agent should execute successfully");

    // Parse the JSON output
    let output = String::from_utf8_lossy(&agent_result.stdout);
    let json: serde_json::Value = serde_json::from_str(&output)
        .expect("Agent should output valid JSON");

    // Verify basic structure
    assert!(json.get("devices").is_some(), "Output should contain devices field");
    assert!(json.get("platform").is_some(), "Output should contain platform field");
    assert!(json.get("architecture").is_some(), "Output should contain architecture field");
    assert!(json.get("detected_at").is_some(), "Output should contain detected_at field");

    // Verify platform detection
    let platform = json["platform"].as_str().unwrap();
    assert!(["linux", "windows", "darwin"].contains(&platform), "Invalid platform: {}", platform);

    // Verify architecture detection
    let arch = json["architecture"].as_str().unwrap();
    assert!(["x86_64", "arm64", "arm32"].contains(&arch), "Invalid architecture: {}", arch);

    // Verify timestamp format
    let detected_at = json["detected_at"].as_str().unwrap();
    assert!(detected_at.contains('T'), "Timestamp should be ISO 8601 format: {}", detected_at);
    assert!(detected_at.contains('Z'), "Timestamp should be ISO 8601 format: {}", detected_at);

    // Verify devices array exists
    let devices = json["devices"].as_array().unwrap();
    // Empty array is acceptable if no hardware is detected
    println!("Detected {} devices", devices.len());
}

#[test]
fn test_hardware_detection_timeout() {
    // Test that agent completes within reasonable time (5 seconds as per spec)
    let start = std::time::Instant::now();

    let result = run_agent_detect();

    let duration = start.elapsed();
    assert!(result.status.success(), "Agent should complete successfully");
    assert!(duration < Duration::from_secs(5),
            "Agent should complete within 5 seconds, took {:?}", duration);

    println!("Agent completed in {:?}", duration);
}

#[test]
fn test_hardware_detection_error_handling() {
    // Test agent behavior with invalid command
    let result = Command::new("../src-tauri/bin/agent")
        .arg("invalid_command")
        .output()
        .expect("Should be able to execute agent");

    // Should fail with non-zero exit code
    assert!(!result.status.success(), "Agent should fail with invalid command");

    // Should print error message to stderr
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("Unknown command"), "Should print error message");
}

#[test]
fn test_hardware_detection_no_args() {
    // Test agent behavior with no arguments
    let result = Command::new("../src-tauri/bin/agent")
        .output()
        .expect("Should be able to execute agent");

    // Should fail with non-zero exit code
    assert!(!result.status.success(), "Agent should fail with no arguments");

    // Should print usage to stderr
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("Usage:"), "Should print usage information");
}

#[test]
fn test_hardware_detection_concurrent() {
    // Test that multiple agent runs don't interfere with each other
    let handles: Vec<_> = (0..3).map(|_| {
        thread::spawn(|| {
            let result = run_agent_detect();
            assert!(result.status.success(), "Agent should execute successfully");

            let output = String::from_utf8_lossy(&result.stdout);
            let json: serde_json::Value = serde_json::from_str(&output)
                .expect("Should parse JSON output");

            json
        })
    }).collect();

    // Wait for all threads to complete
    let results: Vec<serde_json::Value> = handles.into_iter()
        .map(|handle| handle.join().expect("Thread should complete"))
        .collect();

    // All results should have the same basic structure
    for (i, json) in results.iter().enumerate() {
        println!("Result {} devices: {}", i, json["devices"].as_array().unwrap().len());

        assert!(json.get("platform").is_some(), "Result {} should have platform", i);
        assert!(json.get("architecture").is_some(), "Result {} should have architecture", i);
        assert!(json.get("detected_at").is_some(), "Result {} should have detected_at", i);
    }
}

#[test]
fn test_hardware_detection_json_schema() {
    // Verify JSON output matches expected schema more thoroughly
    let result = run_agent_detect();
    assert!(result.status.success(), "Agent should execute successfully");

    let output = String::from_utf8_lossy(&result.stdout);
    let json: serde_json::Value = serde_json::from_str(&output)
        .expect("Agent should output valid JSON");

    // Verify top-level fields are correct types
    assert!(json["devices"].is_array(), "devices should be an array");
    assert!(json["platform"].is_string(), "platform should be a string");
    assert!(json["architecture"].is_string(), "architecture should be a string");
    assert!(json["detected_at"].is_string(), "detected_at should be a string");

    // If devices exist, verify their structure
    if let Some(devices) = json["devices"].as_array() {
        for (i, device) in devices.iter().enumerate() {
            println!("Validating device {}: {}", i, serde_json::to_string_pretty(device).unwrap());

            // Required fields
            assert!(device.get("id").is_some(), "Device {} should have id", i);
            assert!(device.get("type").is_some(), "Device {} should have type", i);
            assert!(device.get("name").is_some(), "Device {} should have name", i);
            assert!(device.get("device_path").is_some(), "Device {} should have device_path", i);
            assert!(device.get("capabilities").is_some(), "Device {} should have capabilities", i);
            assert!(device.get("platform").is_some(), "Device {} should have platform", i);
            assert!(device.get("architecture").is_some(), "Device {} should have architecture", i);
            assert!(device.get("detection_source").is_some(), "Device {} should have detection_source", i);
            assert!(device.get("available").is_some(), "Device {} should have available", i);
            assert!(device.get("in_use").is_some(), "Device {} should have in_use", i);

            // Verify types
            assert!(device["id"].is_string(), "Device {} id should be string", i);
            assert!(device["type"].is_string(), "Device {} type should be string", i);
            assert!(device["name"].is_string(), "Device {} name should be string", i);
            assert!(device["device_path"].is_string(), "Device {} device_path should be string", i);
            assert!(device["capabilities"].is_array(), "Device {} capabilities should be array", i);
            assert!(device["platform"].is_string(), "Device {} platform should be string", i);
            assert!(device["architecture"].is_string(), "Device {} architecture should be string", i);
            assert!(device["detection_source"].is_string(), "Device {} detection_source should be string", i);
            assert!(device["available"].is_boolean(), "Device {} available should be boolean", i);
            assert!(device["in_use"].is_boolean(), "Device {} in_use should be boolean", i);

            // Verify device type is valid
            let device_type = device["type"].as_str().unwrap();
            assert!(["gpu", "tpu", "camera", "capture_card"].contains(&device_type),
                    "Device {} has invalid type: {}", i, device_type);

            // Verify capabilities are strings
            if let Some(capabilities) = device["capabilities"].as_array() {
                for (j, cap) in capabilities.iter().enumerate() {
                    assert!(cap.is_string(), "Device {} capability {} should be string", i, j);
                }
            }
        }
    }
}

// Helper function to run the agent detect command
fn run_agent_detect() -> std::process::Output {
    Command::new("../src-tauri/bin/agent")
        .arg("detect")
        .output()
        .expect("Should be able to execute agent")
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_agent_binary_exists() {
        // Verify agent binary exists and is executable
        let path = "../src-tauri/bin/agent";
        assert!(std::path::Path::new(path).exists(), "Agent binary should exist at: {}", path);

        // Test basic execution
        let result = Command::new(path)
            .arg("--help")
            .output();

        // Should fail (no --help support) but should execute
        match result {
            Ok(output) => {
                // Should fail but provide usage info
                assert!(!output.status.success(), "Agent should fail with --help");
            }
            Err(e) => {
                panic!("Should be able to execute agent: {}", e);
            }
        }
    }
}
