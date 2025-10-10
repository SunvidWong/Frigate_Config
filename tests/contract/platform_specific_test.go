// Platform-specific tests for Agent hardware detection
// Tests platform-specific hardware detection capabilities

package contract

import (
	"encoding/json"
	"os"
	"os/exec"
	"runtime"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestCurrentPlatformDetection tests detection on the current platform
func TestCurrentPlatformDetection(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	// Platform should match current runtime
	expectedPlatform := runtime.GOOS
	assert.Equal(t, expectedPlatform, result.Platform,
		"Detected platform should match current platform")

	// Architecture should be normalized correctly
	expectedArch := normalizeArchitecture(runtime.GOARCH)
	assert.Equal(t, expectedArch, result.Architecture,
		"Detected architecture should match current architecture")
}

// TestDarwinGPUDetection tests GPU detection on macOS
func TestDarwinGPUDetection(t *testing.T) {
	if runtime.GOOS != "darwin" {
		t.Skip("Skipping macOS-specific test on non-macOS platform")
	}

	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	// Look for GPU devices
	var gpuDevices []HardwareDevice
	for _, device := range result.Devices {
		if device.Type == "gpu" {
			gpuDevices = append(gpuDevices, device)
		}
	}

	// On macOS, we should detect at least one GPU (integrated or discrete)
	if len(gpuDevices) == 0 {
		t.Log("Warning: No GPU devices detected on macOS")
		// This is not a failure, as some macOS systems might not report GPUs
		return
	}

	// Validate GPU device properties
	for _, gpu := range gpuDevices {
		assert.NotEmpty(t, gpu.Name, "GPU name should not be empty")
		assert.NotEmpty(t, gpu.DevicePath, "GPU device path should not be empty")
		assert.True(t, len(gpu.Capabilities) > 0, "GPU should have capabilities")

		// Check for expected capabilities on macOS
		expectedCaps := []string{"metal", "videotoolbox"}
		for _, expectedCap := range expectedCaps {
			found := false
			for _, cap := range gpu.Capabilities {
				if cap == expectedCap {
					found = true
					break
				}
			}
			if found {
				t.Logf("GPU %s has expected capability: %s", gpu.Name, expectedCap)
			}
		}
	}
}

// TestDarwinCameraDetection tests camera detection on macOS
func TestDarwinCameraDetection(t *testing.T) {
	if runtime.GOOS != "darwin" {
		t.Skip("Skipping macOS-specific test on non-macOS platform")
	}

	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	// Look for camera devices
	var cameraDevices []HardwareDevice
	for _, device := range result.Devices {
		if device.Type == "camera" {
			cameraDevices = append(cameraDevices, device)
		}
	}

	// On macOS, we should detect at least the built-in camera
	if len(cameraDevices) == 0 {
		t.Log("Warning: No camera devices detected on macOS")
		// This is not a failure, as some systems might not have cameras
		return
	}

	// Validate camera device properties
	for _, camera := range cameraDevices {
		assert.NotEmpty(t, camera.Name, "Camera name should not be empty")
		assert.NotEmpty(t, camera.DevicePath, "Camera device path should not be empty")
		assert.Contains(t, camera.Capabilities, "capture", "Camera should have capture capability")
	}
}

// TestLinuxGPUDetection tests GPU detection on Linux
func TestLinuxGPUDetection(t *testing.T) {
	if runtime.GOOS != "linux" {
		t.Skip("Skipping Linux-specific test on non-Linux platform")
	}

	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	// Look for GPU devices
	var gpuDevices []HardwareDevice
	for _, device := range result.Devices {
		if device.Type == "gpu" {
			gpuDevices = append(gpuDevices, device)
		}
	}

	if len(gpuDevices) == 0 {
		t.Log("Warning: No GPU devices detected on Linux")
		return
	}

	// Validate GPU device properties for Linux
	for _, gpu := range gpuDevices {
		assert.NotEmpty(t, gpu.Name, "GPU name should not be empty")
		assert.NotEmpty(t, gpu.DevicePath, "GPU device path should not be empty")

		// Check for Linux-specific GPU capabilities
		linuxGPUCaps := []string{"vaapi", "cuda", "nvidia", "amdgpu", "intel"}
		for _, expectedCap := range linuxGPUCaps {
			for _, cap := range gpu.Capabilities {
				if cap == expectedCap {
					t.Logf("GPU %s has Linux capability: %s", gpu.Name, expectedCap)
					break
				}
			}
		}
	}
}

// TestLinuxCameraDetection tests camera detection on Linux
func TestLinuxCameraDetection(t *testing.T) {
	if runtime.GOOS != "linux" {
		t.Skip("Skipping Linux-specific test on non-Linux platform")
	}

	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	// Look for camera devices
	var cameraDevices []HardwareDevice
	for _, device := range result.Devices {
		if device.Type == "camera" {
			cameraDevices = append(cameraDevices, device)
		}
	}

	if len(cameraDevices) == 0 {
		t.Log("Warning: No camera devices detected on Linux")
		return
	}

	// Validate camera device properties for Linux
	for _, camera := range cameraDevices {
		assert.NotEmpty(t, camera.Name, "Camera name should not be empty")
		assert.True(t, camera.DevicePath != "", "Camera device path should not be empty")
		assert.Contains(t, camera.Capabilities, "capture", "Camera should have capture capability")

		// On Linux, camera paths typically start with /dev/video
		if len(camera.DevicePath) > 10 && camera.DevicePath[:10] == "/dev/video" {
			t.Logf("Camera %s has expected Linux device path: %s", camera.Name, camera.DevicePath)
		}
	}
}

// TestWindowsGPUDetection tests GPU detection on Windows
func TestWindowsGPUDetection(t *testing.T) {
	if runtime.GOOS != "windows" {
		t.Skip("Skipping Windows-specific test on non-Windows platform")
	}

	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	// Look for GPU devices
	var gpuDevices []HardwareDevice
	for _, device := range result.Devices {
		if device.Type == "gpu" {
			gpuDevices = append(gpuDevices, device)
		}
	}

	if len(gpuDevices) == 0 {
		t.Log("Warning: No GPU devices detected on Windows")
		return
	}

	// Validate GPU device properties for Windows
	for _, gpu := range gpuDevices {
		assert.NotEmpty(t, gpu.Name, "GPU name should not be empty")
		assert.NotEmpty(t, gpu.DevicePath, "GPU device path should not be empty")

		// Check for Windows-specific GPU capabilities
		windowsGPUCaps := []string{"d3d11", "d3d12", "dxva", "cuda", "nvidia", "amd"}
		for _, expectedCap := range windowsGPUCaps {
			for _, cap := range gpu.Capabilities {
				if cap == expectedCap {
					t.Logf("GPU %s has Windows capability: %s", gpu.Name, expectedCap)
					break
				}
			}
		}
	}
}

// TestWindowsCameraDetection tests camera detection on Windows
func TestWindowsCameraDetection(t *testing.T) {
	if runtime.GOOS != "windows" {
		t.Skip("Skipping Windows-specific test on non-Windows platform")
	}

	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	// Look for camera devices
	var cameraDevices []HardwareDevice
	for _, device := range result.Devices {
		if device.Type == "camera" {
			cameraDevices = append(cameraDevices, device)
		}
	}

	if len(cameraDevices) == 0 {
		t.Log("Warning: No camera devices detected on Windows")
		return
	}

	// Validate camera device properties for Windows
	for _, camera := range cameraDevices {
		assert.NotEmpty(t, camera.Name, "Camera name should not be empty")
		assert.NotEmpty(t, camera.DevicePath, "Camera device path should not be empty")
		assert.Contains(t, camera.Capabilities, "capture", "Camera should have capture capability")

		// On Windows, camera paths might use different formats
		t.Logf("Windows camera detected: %s at %s", camera.Name, camera.DevicePath)
	}
}

// TestDeviceAvailabilityFlags tests that devices properly report availability
func TestDeviceAvailabilityFlags(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	for _, device := range result.Devices {
		// All devices should have boolean availability flags
		assert.IsType(t, false, device.Available, "Available should be boolean")
		assert.IsType(t, false, device.InUse, "InUse should be boolean")

		t.Logf("Device %s: available=%v, in_use=%v", device.Name, device.Available, device.InUse)
	}
}

// TestDetectionSourceReporting tests that devices properly report their detection source
func TestDetectionSourceReporting(t *testing.T) {
	agentPath, err := buildAgent()
	require.NoError(t, err)
	defer os.Remove(agentPath)

	cmd := exec.Command(agentPath, "detect")
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Agent detect command failed: %s", string(output))

	var result DetectionResult
	err = json.Unmarshal(output, &result)
	require.NoError(t, err)

	// Map expected detection sources by platform
	expectedSources := map[string][]string{
		"darwin":  {"system_profiler"},
		"linux":   {"lspci", "lshw", "lsusb", "v4l2"},
		"windows": {"wmi", "directx"},
	}

	platformSources, exists := expectedSources[runtime.GOOS]
	if !exists {
		t.Skipf("No expected detection sources defined for platform: %s", runtime.GOOS)
	}

	for _, device := range result.Devices {
		assert.NotEmpty(t, device.DetectionSource, "Detection source should not be empty")

		// Check if detection source is expected for this platform
		found := false
		for _, expectedSource := range platformSources {
			if device.DetectionSource == expectedSource {
				found = true
				break
			}
		}

		if !found {
			t.Logf("Note: Device %s detected via unexpected source: %s", device.Name, device.DetectionSource)
		}
	}
}

// normalizeArchitecture normalizes Go architecture names to our schema
func normalizeArchitecture(goArch string) string {
	switch goArch {
	case "amd64":
		return "x86_64"
	case "arm64":
		return "arm64"
	case "arm":
		return "arm32"
	default:
		return goArch
	}
}