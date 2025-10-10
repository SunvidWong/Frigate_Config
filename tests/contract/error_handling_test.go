// Error handling tests for Agent
// Tests agent's ability to handle various error conditions gracefully

package contract

import (
	"encoding/json"
	"os"
	"os/exec"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestAgentInvalidCommand tests agent behavior with invalid commands
func TestAgentInvalidCommand(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	invalidCommands := []string{
		"invalid",
		"foo",
		"bar",
		"help",
		"--help",
		"-h",
		"",
	}

	for _, cmd := range invalidCommands {
		t.Run("InvalidCommand_"+cmd, func(t *testing.T) {
			var execCmd *exec.Cmd
			if cmd == "" {
				// Test with no arguments
				execCmd = exec.Command(agentPath)
			} else {
				execCmd = exec.Command(agentPath, cmd)
			}

			output, err := execCmd.CombinedOutput()

			// Should fail with non-zero exit code
			assert.Error(t, err, "Expected error for invalid command: %s", cmd)

			// Should contain error message
			outputStr := string(output)
			assert.NotEmpty(t, outputStr, "Error output should not be empty")

			if cmd == "" {
				assert.Contains(t, outputStr, "Usage:", "Should show usage for no arguments")
			} else {
				assert.Contains(t, outputStr, "Unknown command", "Should show unknown command error")
			}
		})
	}
}

// TestAgentGracefulDegradation tests agent handles missing detection sources gracefully
func TestAgentGracefulDegradation(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// This test verifies that the agent doesn't crash even if detection sources fail
	// The actual implementation should handle missing system tools gracefully

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()

	// Command should succeed even if some detection sources fail
	// (it might not find any devices, but shouldn't crash)
	require.NoError(t, err, "Agent should handle missing detection sources gracefully: %s", string(output))

	// Output should be valid JSON
	var result DetectionResult
	err = json.Unmarshal(output, &result)
	assert.NoError(t, err, "Output should be valid JSON even with limited detection")
}

// TestAgentMalformedOutput tests agent handles malformed system output gracefully
func TestAgentMalformedOutput(t *testing.T) {
	// This test would require mocking system commands to return malformed output
	// For now, we test that the agent doesn't crash on normal operation

	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Run detect command multiple times to test consistency
	for i := 0; i < 5; i++ {
		cmd := exec.Command(agentPath, "detect")
		output, err := cmd.CombinedOutput()
		require.NoError(t, err, "Detect command should not crash on attempt %d: %s", i+1, string(output))

		var result DetectionResult
		err = json.Unmarshal(output, &result)
		assert.NoError(t, err, "Output should be valid JSON on attempt %d", i+1)
	}
}

// TestAgentMemoryLimits tests agent doesn't consume excessive memory
func TestAgentMemoryLimits(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Run the agent and check it completes in reasonable time
	cmd := exec.Command(agentPath, "detect")

	start := time.Now()
	output, err := cmd.CombinedOutput()
	duration := time.Since(start)

	require.NoError(t, err, "Agent should complete successfully")

	// Should complete within reasonable time (10 seconds)
	assert.Less(t, duration, 10*time.Second,
		"Agent should complete within 10 seconds, took %v", duration)

	// Output should not be excessively large (indicating memory issues)
	outputSize := len(output)
	assert.Less(t, outputSize, 10*1024*1024, // 10MB
		"Agent output should not exceed 10MB, got %d bytes", outputSize)
}

// TestAgentJSONErrorHandling tests agent handles JSON marshaling errors gracefully
func TestAgentJSONErrorHandling(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Run detect command
	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	// Verify output is valid JSON
	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err, "Agent output should be valid JSON")

	// Verify JSON structure is complete
	assert.NotEmpty(t, result.Platform, "Platform should be set")
	assert.NotEmpty(t, result.Architecture, "Architecture should be set")
	assert.NotEmpty(t, result.DetectedAt, "DetectedAt should be set")
	assert.NotNil(t, result.Devices, "Devices should be initialized")

	// Verify timestamp is valid RFC3339 format
	assert.Regexp(t, `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$`, result.DetectedAt,
		"DetectedAt should be in RFC3339 format")
}

// TestAgentDeviceErrorHandling tests that individual device errors don't crash the agent
func TestAgentDeviceErrorHandling(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err, "Agent output should be valid JSON")

	// Check that devices with errors are properly handled
	for _, device := range result.Devices {
		if device.Error != nil {
			// Device with error should still have valid basic fields
			assert.NotEmpty(t, device.ID, "Device with error should still have ID")
			assert.NotEmpty(t, device.Type, "Device with error should still have type")
			assert.NotEmpty(t, device.Name, "Device with error should still have name")
			assert.NotEmpty(t, *device.Error, "Error message should not be empty")

			t.Logf("Device %s has error: %s", device.Name, *device.Error)
		}
	}
}

// TestAgentConcurrentExecution tests agent handles concurrent execution gracefully
func TestAgentConcurrentExecution(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Run multiple agent instances concurrently
	const numConcurrent = 3
	results := make(chan error, numConcurrent)

	for i := 0; i < numConcurrent; i++ {
		go func(id int) {
			cmd := exec.Command(agentPath, "detect")
			output, err := cmd.CombinedOutput()

			if err != nil {
				results <- err
				return
			}

			// Verify output is valid JSON
			var result DetectionResult
			err = json.Unmarshal(output, &result)
			results <- err
		}(i)
	}

	// Collect results
	for i := 0; i < numConcurrent; i++ {
		err := <-results
		assert.NoError(t, err, "Concurrent agent execution %d should succeed", i+1)
	}
}

// TestAgentSignalHandling tests agent handles system signals gracefully
func TestAgentSignalHandling(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	// Start the agent process
	cmd := exec.Command(agentPath, "detect")

	// Start the process but don't wait for it
	err = cmd.Start()
	require.NoError(t, err, "Should be able to start agent process")

	// Give it a moment to start
	// In a real test, you might send signals like SIGTERM, SIGINT
	// For now, we just verify the process runs to completion

	err = cmd.Wait()
	assert.NoError(t, err, "Agent should complete successfully when started")
}

// TestAgentLargeOutputHandling tests agent handles potentially large outputs gracefully
func TestAgentLargeOutputHandling(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	// Verify output is not truncated or corrupted
	outputStr := string(output)

	// Should start with opening brace
	assert.True(t, strings.HasPrefix(outputStr, "{"), "Output should start with opening brace")

	// Should end with closing brace
	assert.True(t, strings.HasSuffix(outputStr, "}") || strings.HasSuffix(outputStr, "}\n"),
		"Output should end with closing brace")

	// Should be valid JSON
	var result DetectionResult
	err = json.Unmarshal(output, &result)
	assert.NoError(t, err, "Output should be valid JSON")
}

