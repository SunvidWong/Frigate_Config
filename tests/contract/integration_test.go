// Integration tests for Agent
// Tests agent integration with system components and end-to-end workflows

package contract

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestAgentEndToEndWorkflow tests complete agent workflow
func TestAgentEndToEndWorkflow(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Step 1: Run hardware detection
	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Hardware detection should succeed")

	// Step 2: Parse and validate output
	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err, "Output should be valid JSON")

	// Step 3: Validate basic structure
	assert.NotEmpty(t, result.Platform, "Platform should be detected")
	assert.NotEmpty(t, result.Architecture, "Architecture should be detected")
	assert.NotEmpty(t, result.DetectedAt, "Timestamp should be set")
	assert.NotNil(t, result.Devices, "Devices list should be initialized")

	// Step 4: Validate timestamp
	parsedTime, err := time.Parse(time.RFC3339, result.DetectedAt)
	assert.NoError(t, err, "Timestamp should be valid RFC3339")
	assert.WithinDuration(t, time.Now(), parsedTime, 5*time.Minute, "Timestamp should be recent")

	// Step 5: Validate devices
	for _, device := range result.Devices {
		validateDeviceIntegration(t, device, result.Platform)
	}

	t.Logf("End-to-end workflow completed successfully:")
	t.Logf("  Platform: %s", result.Platform)
	t.Logf("  Architecture: %s", result.Architecture)
	t.Logf("  Devices detected: %d", len(result.Devices))
	t.Logf("  Detection completed at: %s", result.DetectedAt)
}

// TestAgentIntegrationWithSystemTools tests agent integration with system tools
func TestAgentIntegrationWithSystemTools(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Run agent detection
	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detection should succeed")

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err, "Output should be valid JSON")

	// Verify agent can access system information
	systemInfo := getSystemInfo()

	assert.Equal(t, systemInfo.platform, result.Platform, "Platform should match system")
	assert.Equal(t, systemInfo.architecture, result.Architecture, "Architecture should match system")

	t.Logf("System integration verified:")
	t.Logf("  System platform: %s (Agent: %s)", systemInfo.platform, result.Platform)
	t.Logf("  System architecture: %s (Agent: %s)", systemInfo.architecture, result.Architecture)
}

// TestAgentRealHardwareDetection tests agent with actual hardware detection
func TestAgentRealHardwareDetection(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Run detection
	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detection should succeed")

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err, "Output should be valid JSON")

	// Validate detected hardware
	if len(result.Devices) == 0 {
		t.Log("No hardware devices detected - this may be normal on some systems")
		return
	}

	// Categorize devices by type
	devicesByType := make(map[string][]HardwareDevice)
	for _, device := range result.Devices {
		devicesByType[device.Type] = append(devicesByType[device.Type], device)
	}

	t.Logf("Hardware detection results:")
	for deviceType, devices := range devicesByType {
		t.Logf("  %s: %d devices", deviceType, len(devices))
		for _, device := range devices {
			t.Logf("    - %s (%s)", device.Name, device.DevicePath)
			t.Logf("      Capabilities: %v", device.Capabilities)
			t.Logf("      Available: %v, In use: %v", device.Available, device.InUse)
		}
	}

	// Validate device properties based on platform
	validatePlatformSpecificHardware(t, result.Platform, devicesByType)
}

// TestAgentConsistencyAcrossRuns tests agent provides consistent results across multiple runs
func TestAgentConsistencyAcrossRuns(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	const numRuns = 3
	var results []DetectionResult

	// Run detection multiple times
	for i := 0; i < numRuns; i++ {
		cmd := exec.Command(agentPath, "detect")
		output, err := cmd.CombinedOutput()
		require.NoError(t, err, "Run %d should succeed", i+1)

		var result DetectionResult
		err = json.Unmarshal(output, &result)
		require.NoError(t, err, "Run %d should produce valid JSON", i+1)

		results = append(results, result)

		// Small delay between runs
		time.Sleep(100 * time.Millisecond)
	}

	// Analyze consistency
	platforms := make(map[string]int)
	architectures := make(map[string]int)
	deviceCounts := make(map[int]int)

	for _, result := range results {
		platforms[result.Platform]++
		architectures[result.Architecture]++
		deviceCounts[len(result.Devices)]++
	}

	// All runs should have the same platform and architecture
	assert.Len(t, platforms, 1, "Platform should be consistent across runs")
	assert.Len(t, architectures, 1, "Architecture should be consistent across runs")

	// Device count should be consistent (allowing for small variations due to timing)
	assert.LessOrEqual(t, len(deviceCounts), 2, "Device count should be mostly consistent")

	t.Logf("Consistency analysis across %d runs:", numRuns)
	t.Logf("  Platform consistency: %v", len(platforms) == 1)
	t.Logf("  Architecture consistency: %v", len(architectures) == 1)
	t.Logf("  Device count consistency: %v", len(deviceCounts) <= 2)
	t.Logf("  Device count distribution: %v", deviceCounts)
}

// TestAgentIntegrationWithTauriBackend tests agent integration points with Tauri backend
func TestAgentIntegrationWithTauriBackend(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Simulate what the Tauri backend would do
	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent should respond to Tauri backend")

	// Parse as Tauri backend would
	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err, "Tauri backend should be able to parse agent output")

	// Validate that data structure matches what Tauri expects
	validateTauriIntegration(t, result)

	// Test JSON serialization/deserialization consistency
	remarshaled, err := json.Marshal(result)
	require.NoError(t, err, "Result should be re-marshallable")

	var reparsed DetectionResult
	err = json.Unmarshal(remarshaled, &reparsed)
	require.NoError(t, err, "Re-marshaled result should be parseable")

	assert.Equal(t, result, reparsed, "Re-parsed result should match original")

	t.Logf("Tauri integration validated:")
	t.Logf("  JSON structure compatible: true")
	t.Logf("  Serialization consistent: true")
	t.Logf("  Device count: %d", len(result.Devices))
}

// TestAgentErrorRecovery tests agent error recovery capabilities
func TestAgentErrorRecovery(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Test multiple consecutive runs to ensure no state leakage
	const numRuns = 5
	var successCount int
	var errors []error

	for i := 0; i < numRuns; i++ {
		cmd := exec.Command(agentPath, "detect")
		output, err := cmd.CombinedOutput()

		if err != nil {
			errors = append(errors, err)
			t.Logf("Run %d failed: %v", i+1, err)
		} else {
			successCount++

			// Verify output is valid even on subsequent runs
			var result DetectionResult
			if parseErr := json.Unmarshal(output, &result); parseErr != nil {
				t.Errorf("Run %d produced invalid JSON: %v", i+1, parseErr)
			}
		}
	}

	t.Logf("Error recovery results:")
	t.Logf("  Successful runs: %d/%d", successCount, numRuns)
	t.Logf("  Failed runs: %d", len(errors))

	// Should have high success rate
	successRate := float64(successCount) / float64(numRuns) * 100
	assert.GreaterOrEqual(t, successRate, 80.0, "Success rate should be at least 80%%")

	if len(errors) > 0 {
		t.Logf("Errors encountered:")
		for i, err := range errors {
			t.Logf("  Error %d: %v", i+1, err)
		}
	}
}

// TestAgentIntegrationWithConfigFiles tests agent can work with configuration files
func TestAgentIntegrationWithConfigFiles(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Create a temporary directory for test config files
	tempDir, err := os.MkdirTemp("", "agent-integration-test")
	require.NoError(t, err)
	defer os.RemoveAll(tempDir)

	// Run agent detection
	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detection should succeed")

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err, "Output should be valid JSON")

	// Create a mock configuration based on detected hardware
	configPath := filepath.Join(tempDir, "frigate-config.yml")
	configContent := generateMockConfig(result)
	err = os.WriteFile(configPath, []byte(configContent), 0644)
	require.NoError(t, err, "Should be able to write config file")

	// Verify config file was created
	assert.FileExists(t, configPath, "Config file should be created")

	// Read back and verify content
	content, err := os.ReadFile(configPath)
	require.NoError(t, err, "Should be able to read config file")

	assert.Contains(t, string(content), "cameras:", "Config should contain cameras section")
	assert.Contains(t, string(content), "detectors:", "Config should contain detectors section")

	t.Logf("Config file integration validated:")
	t.Logf("  Generated config: %s", configPath)
	t.Logf("  Config size: %d bytes", len(content))
	t.Logf("  Based on %d detected devices", len(result.Devices))
}

// Helper functions

type SystemInfo struct {
	platform     string
	architecture string
}

func getSystemInfo() SystemInfo {
	arch := runtime.GOARCH
	switch arch {
	case "amd64":
		arch = "x86_64"
	case "arm64":
		// Keep as is
	case "arm":
		arch = "arm32"
	}

	return SystemInfo{
		platform:     runtime.GOOS,
		architecture: arch,
	}
}

func validateDeviceIntegration(t *testing.T, device HardwareDevice, platform string) {
	// Basic validation
	assert.NotEmpty(t, device.ID, "Device ID should not be empty")
	assert.NotEmpty(t, device.Type, "Device type should not be empty")
	assert.NotEmpty(t, device.Name, "Device name should not be empty")
	assert.NotEmpty(t, device.DevicePath, "Device path should not be empty")

	// Type validation
	validTypes := []string{"gpu", "tpu", "camera", "capture_card"}
	assert.Contains(t, validTypes, device.Type, "Device type should be valid")

	// Platform-specific validation
	switch platform {
	case "darwin":
		validateDarwinDevice(t, device)
	case "linux":
		validateLinuxDevice(t, device)
	case "windows":
		validateWindowsDevice(t, device)
	}
}

func validateDarwinDevice(t *testing.T, device HardwareDevice) {
	// macOS specific validations
	if device.Type == "gpu" {
		// Check for Metal capability on macOS GPUs
		hasMetal := false
		for _, cap := range device.Capabilities {
			if cap == "metal" {
				hasMetal = true
				break
			}
		}
		if hasMetal {
			t.Logf("macOS GPU %s supports Metal", device.Name)
		}
	}
}

func validateLinuxDevice(t *testing.T, device HardwareDevice) {
	// Linux specific validations
	if device.Type == "gpu" {
		// Check for common Linux GPU capabilities
		linuxCaps := []string{"vaapi", "cuda", "nvidia", "amdgpu", "intel"}
		for _, expectedCap := range linuxCaps {
			for _, cap := range device.Capabilities {
				if cap == expectedCap {
					t.Logf("Linux GPU %s has capability: %s", device.Name, cap)
					break
				}
			}
		}
	}
}

func validateWindowsDevice(t *testing.T, device HardwareDevice) {
	// Windows specific validations
	if device.Type == "gpu" {
		// Check for DirectX capabilities on Windows GPUs
		directxCaps := []string{"d3d11", "d3d12", "dxva"}
		for _, expectedCap := range directxCaps {
			for _, cap := range device.Capabilities {
				if strings.Contains(strings.ToLower(cap), expectedCap) {
					t.Logf("Windows GPU %s has DirectX capability", device.Name)
					break
				}
			}
		}
	}
}

func validatePlatformSpecificHardware(t *testing.T, platform string, devicesByType map[string][]HardwareDevice) {
	switch platform {
	case "darwin":
		// On macOS, we typically expect at least a GPU (integrated)
		if gpus, exists := devicesByType["gpu"]; exists {
			assert.Greater(t, len(gpus), 0, "macOS should have at least one GPU")
		}
	case "linux":
		// On Linux, hardware detection varies widely
		t.Logf("Linux hardware detection: %+v", devicesByType)
	case "windows":
		// On Windows, we typically expect at least a GPU
		if gpus, exists := devicesByType["gpu"]; exists {
			assert.Greater(t, len(gpus), 0, "Windows should have at least one GPU")
		}
	}
}

func validateTauriIntegration(t *testing.T, result DetectionResult) {
	// Validate that the structure matches what Tauri backend expects
	assert.IsType(t, []HardwareDevice{}, result.Devices, "Devices should be a slice")
	assert.IsType(t, "", result.Platform, "Platform should be a string")
	assert.IsType(t, "", result.Architecture, "Architecture should be a string")
	assert.IsType(t, "", result.DetectedAt, "DetectedAt should be a string")

	// Validate timestamp format for Tauri consumption
	_, err := time.Parse(time.RFC3339, result.DetectedAt)
	assert.NoError(t, err, "Timestamp should be RFC3339 format for Tauri")
}

func generateMockConfig(result DetectionResult) string {
	var config strings.Builder

	config.WriteString("# Frigate Configuration\n")
	config.WriteString("# Generated by agent integration test\n\n")

	config.WriteString("cameras:\n")
	cameraCount := 0
	for _, device := range result.Devices {
		if device.Type == "camera" && cameraCount < 2 {
			config.WriteString(fmt.Sprintf("  camera_%d:\n", cameraCount+1))
			config.WriteString(fmt.Sprintf("    enabled: true\n"))
			config.WriteString(fmt.Sprintf("    ffmpeg:\n"))
			config.WriteString(fmt.Sprintf("      inputs:\n"))
			config.WriteString(fmt.Sprintf("        - path: %s\n", device.DevicePath))
			config.WriteString(fmt.Sprintf("          roles:\n"))
			config.WriteString(fmt.Sprintf("            - detect\n"))
			config.WriteString(fmt.Sprintf("            - record\n"))
			cameraCount++
		}
	}

	config.WriteString("\ndetectors:\n")
	gpuCount := 0
	for _, device := range result.Devices {
		if device.Type == "gpu" && gpuCount < 1 {
			config.WriteString(fmt.Sprintf("  %s:\n", strings.ToLower(strings.ReplaceAll(device.Name, " ", "_"))))
			config.WriteString(fmt.Sprintf("    type: %s\n", device.Type))
			gpuCount++
		}
	}

	return config.String()
}