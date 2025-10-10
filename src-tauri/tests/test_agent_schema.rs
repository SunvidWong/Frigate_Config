// Contract test for Agent JSON output schema
// Ensures the agent produces JSON that matches our expected schema
// Per Constitution Principle VI: This test MUST be written first and MUST fail before implementation

use serde_json::Value;

#[test]
fn test_agent_output_schema_structure() {
    // This test will FAIL until the agent is properly implemented

    // Expected JSON structure from agent
    let expected_fields = vec!["devices", "platform", "architecture", "detected_at"];

    // Simulate running the agent (will fail until implemented)
    let agent_output = run_agent_detect();

    // Parse JSON
    let json: Value = serde_json::from_str(&agent_output)
        .expect("Agent should produce valid JSON");

    // Verify top-level fields exist
    for field in expected_fields {
        assert!(
            json.get(field).is_some(),
            "Agent output must contain '{}' field",
            field
        );
    }

    // Verify platform is a valid string
    let platform = json["platform"].as_str()
        .expect("platform should be a string");
    assert!(
        ["linux", "windows", "darwin"].contains(&platform),
        "platform must be one of: linux, windows, darwin"
    );

    // Verify architecture is a valid string
    let arch = json["architecture"].as_str()
        .expect("architecture should be a string");
    assert!(
        ["x86_64", "arm64", "arm32"].contains(&arch),
        "architecture must be one of: x86_64, arm64, arm32"
    );

    // Verify devices is an array
    let devices = json["devices"].as_array()
        .expect("devices should be an array");

    // If devices exist, verify their structure
    if !devices.is_empty() {
        let device = &devices[0];
        let required_device_fields = vec![
            "id", "type", "name", "device_path", "capabilities",
            "platform", "architecture", "detection_source",
            "available", "in_use"
        ];

        for field in required_device_fields {
            assert!(
                device.get(field).is_some(),
                "Device must contain '{}' field",
                field
            );
        }

        // Verify device type is valid
        let device_type = device["type"].as_str()
            .expect("device type should be a string");
        assert!(
            ["gpu", "tpu", "camera", "capture_card"].contains(&device_type),
            "device type must be one of: gpu, tpu, camera, capture_card"
        );
    }

    // Verify detected_at is ISO 8601 timestamp
    let detected_at = json["detected_at"].as_str()
        .expect("detected_at should be a string");
    assert!(
        detected_at.contains('T') && detected_at.contains('Z'),
        "detected_at should be ISO 8601 format (contains T and Z)"
    );
}

#[test]
fn test_agent_device_types() {
    // Test that agent can detect different device types
    let agent_output = run_agent_detect();
    let json: Value = serde_json::from_str(&agent_output)
        .expect("Agent should produce valid JSON");

    let devices = json["devices"].as_array()
        .expect("devices should be an array");

    // At minimum, we expect the test to run without panicking
    // Actual device detection depends on the test environment
    for device in devices {
        let device_type = device["type"].as_str()
            .expect("Each device should have a type");

        // Verify type is one of the allowed values
        assert!(
            ["gpu", "tpu", "camera", "capture_card"].contains(&device_type),
            "Invalid device type: {}",
            device_type
        );

        // Verify device has required fields
        assert!(device["id"].is_string(), "Device must have string id");
        assert!(device["name"].is_string(), "Device must have string name");
        assert!(device["device_path"].is_string(), "Device must have string device_path");
        assert!(device["capabilities"].is_array(), "Device must have array capabilities");
        assert!(device["available"].is_boolean(), "Device must have boolean available");
        assert!(device["in_use"].is_boolean(), "Device must have boolean in_use");
    }
}

// Helper function to run the agent detect command
// This will initially fail until the agent binary is built and working
fn run_agent_detect() -> String {
    use std::process::Command;

    // Find the agent binary
    let agent_path = find_agent_binary();

    // Run agent detect
    let output = Command::new(agent_path)
        .arg("detect")
        .output()
        .expect("Failed to execute agent binary");

    // Check if command succeeded
    assert!(
        output.status.success(),
        "Agent command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Return stdout as string
    String::from_utf8(output.stdout)
        .expect("Agent output should be valid UTF-8")
}

// Helper to find agent binary
fn find_agent_binary() -> String {
    // Try different possible locations
    let possible_paths = vec![
        "../agent/agent",  // From src-tauri during tests
        "../../agent/agent",  // From target/debug/deps
        "agent/agent",
        "./agent",
        "../src-tauri/bin/agent",  // After build
    ];

    for path in possible_paths {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }

    panic!("Agent binary not found. Please build it first with: cd agent && go build -o agent cmd/agent/main.go");
}

#[cfg(test)]
mod schema_validation_tests {
    use super::*;

    #[test]
    fn test_agent_binary_exists() {
        // Verify that we can find the agent binary
        let path = find_agent_binary();
        assert!(std::path::Path::new(&path).exists(), "Agent binary should exist at: {}", path);
    }
}
