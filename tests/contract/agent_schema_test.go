// Contract tests for Agent JSON schema
// Ensures agent output matches expected schema and contract

package contract

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestAgentSchemaContract tests that the agent outputs valid JSON
// according to the expected schema contract
func TestAgentSchemaContract(t *testing.T) {
	// Build the agent first
	agentPath, err := buildAgent()
	require.NoError(t, err, "Failed to build agent")
	defer os.Remove(agentPath)

	// Run agent detect command
	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	// Parse JSON output
	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err, "Failed to parse agent JSON output")

	// Validate schema contract
	validateDetectionResultContract(t, result)
}

// TestAgentErrorHandling tests agent error handling behavior
func TestAgentErrorHandling(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Test invalid command
	cmd := exec.Command(agentPath, "invalid")
	output, err := cmd.CombinedOutput()

	// Should fail with non-zero exit code
	assert.Error(t, err)

	// Should contain error message
	assert.Contains(t, string(output), "Unknown command")
}

// TestAgentDeviceSchema tests individual device schema compliance
func TestAgentDeviceSchema(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	// Validate each device
	for i, device := range result.Devices {
		t.Run(fmt.Sprintf("Device_%d", i), func(t *testing.T) {
			validateHardwareDeviceContract(t, device)
		})
	}
}

// TestAgentTimestampFormat validates timestamp format
func TestAgentTimestampFormat(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	// Parse timestamp to ensure RFC3339 format
	_, err = time.Parse(time.RFC3339, result.DetectedAt)
	assert.NoError(t, err, "Timestamp is not in RFC3339 format: %s", result.DetectedAt)
}

// TestAgentPlatformInfo validates platform and architecture info
func TestAgentPlatformInfo(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	// Validate platform
	validPlatforms := []string{"linux", "windows", "darwin"}
	assert.Contains(t, validPlatforms, result.Platform,
		"Platform should be one of %v, got: %s", validPlatforms, result.Platform)

	// Validate architecture
	validArchs := []string{"x86_64", "arm64", "arm32"}
	assert.Contains(t, validArchs, result.Architecture,
		"Architecture should be one of %v, got: %s", validArchs, result.Architecture)
}

// validateDetectionResultContract validates the top-level detection result contract
func validateDetectionResultContract(t *testing.T, result DetectionResult) {
	// Required fields
	assert.NotEmpty(t, result.Platform, "Platform should not be empty")
	assert.NotEmpty(t, result.Architecture, "Architecture should not be empty")
	assert.NotEmpty(t, result.DetectedAt, "DetectedAt should not be empty")

	// Devices should be a slice (can be empty)
	assert.NotNil(t, result.Devices, "Devices should be a slice, not nil")

	// Validate timestamp format
	_, err := time.Parse(time.RFC3339, result.DetectedAt)
	assert.NoError(t, err, "DetectedAt should be in RFC3339 format")
}

// validateHardwareDeviceContract validates individual device contract
func validateHardwareDeviceContract(t *testing.T, device HardwareDevice) {
	// Required fields
	assert.NotEmpty(t, device.ID, "Device ID should not be empty")
	assert.NotEmpty(t, device.Type, "Device type should not be empty")
	assert.NotEmpty(t, device.Name, "Device name should not be empty")
	assert.NotEmpty(t, device.DevicePath, "Device path should not be empty")
	assert.NotEmpty(t, device.Platform, "Device platform should not be empty")
	assert.NotEmpty(t, device.Architecture, "Device architecture should not be empty")
	assert.NotEmpty(t, device.DetectionSource, "Detection source should not be empty")

	// Validate device type
	validTypes := []string{"gpu", "tpu", "camera", "capture_card"}
	assert.Contains(t, validTypes, device.Type,
		"Device type should be one of %v, got: %s", validTypes, device.Type)

	// Validate platform
	validPlatforms := []string{"linux", "windows", "darwin"}
	assert.Contains(t, validPlatforms, device.Platform,
		"Device platform should be one of %v, got: %s", validPlatforms, device.Platform)

	// Validate architecture
	validArchs := []string{"x86_64", "arm64", "arm32"}
	assert.Contains(t, validArchs, device.Architecture,
		"Device architecture should be one of %v, got: %s", validArchs, device.Architecture)

	// Capabilities should be a slice (can be empty)
	assert.NotNil(t, device.Capabilities, "Capabilities should be a slice, not nil")

	// Boolean fields
	assert.IsType(t, false, device.Available, "Available should be boolean")
	assert.IsType(t, false, device.InUse, "InUse should be boolean")

	// Optional fields validation
	if device.Driver != nil {
		assert.NotEmpty(t, *device.Driver, "Driver should not be empty when present")
	}

	if device.VendorID != nil {
		assert.NotEmpty(t, *device.VendorID, "VendorID should not be empty when present")
	}

	if device.Error != nil {
		assert.NotEmpty(t, *device.Error, "Error should not be empty when present")
	}
}

// buildAgent builds the agent binary and returns the path
func buildAgent() (string, error) {
	// Get the agent directory
	agentDir := "../../agent"

	// Build the agent
	cmd := exec.Command("go", "build", "-o", "agent-test", "./cmd/agent")
	cmd.Dir = agentDir

	if err := cmd.Run(); err != nil {
		return "", fmt.Errorf("failed to build agent: %w", err)
	}

	// Return the full path to the built binary
	agentPath := filepath.Join(agentDir, "agent-test")
	return agentPath, nil
}

// Schema contracts (mirrored from agent/internal/schema)
type DetectionResult struct {
	Devices      []HardwareDevice `json:"devices"`
	Platform     string           `json:"platform"`
	Architecture string           `json:"architecture"`
	DetectedAt   string           `json:"detected_at"`
}

type HardwareDevice struct {
	ID              string   `json:"id"`
	Type            string   `json:"type"`
	Name            string   `json:"name"`
	DevicePath      string   `json:"device_path"`
	Capabilities    []string `json:"capabilities"`
	Driver          *string  `json:"driver,omitempty"`
	Platform        string   `json:"platform"`
	Architecture    string   `json:"architecture"`
	VendorID        *string  `json:"vendor_id,omitempty"`
	DetectionSource string   `json:"detection_source"`
	Available       bool     `json:"available"`
	InUse           bool     `json:"in_use"`
	Error           *string  `json:"error,omitempty"`
}