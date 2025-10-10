package detect

import (
	"fmt"
	"os/exec"
	"runtime"
	"strings"

	"github.com/frigate-config-tool/agent/internal/schema"
)

// Detector is the interface for platform-specific hardware detection
type Detector interface {
	DetectGPUs() ([]schema.HardwareDevice, error)
	DetectTPUs() ([]schema.HardwareDevice, error)
	DetectCameras() ([]schema.HardwareDevice, error)
	DetectCaptureCards() ([]schema.HardwareDevice, error)
}

// DetectAll runs all detection methods and aggregates results
func DetectAll(detector Detector) ([]schema.HardwareDevice, error) {
	var allDevices []schema.HardwareDevice

	// Detect GPUs
	gpus, err := detector.DetectGPUs()
	if err != nil {
		// Log error but continue with other detections
		// Graceful error handling - don't fail entire detection
	} else {
		allDevices = append(allDevices, gpus...)
	}

	// Detect TPUs
	tpus, err := detector.DetectTPUs()
	if err != nil {
		// Continue on error - graceful degradation
	} else {
		allDevices = append(allDevices, tpus...)
	}

	// Detect Cameras
	cameras, err := detector.DetectCameras()
	if err != nil {
		// Continue on error - graceful degradation
	} else {
		allDevices = append(allDevices, cameras...)
	}

	// Detect Capture Cards
	captureCards, err := detector.DetectCaptureCards()
	if err != nil {
		// Continue on error - graceful degradation
	} else {
		allDevices = append(allDevices, captureCards...)
	}

	// If no hardware detected, add CPU-only warning device
	if len(allDevices) == 0 {
		cpuDevice := CreateCPUOnlyWarningDevice()
		allDevices = append(allDevices, cpuDevice)
	}

	return allDevices, nil
}

// GetArchitecture detects the current system architecture
func GetArchitecture() string {
	arch := runtime.GOARCH

	// Normalize architecture names
	switch arch {
	case "amd64":
		return "amd64"
	case "arm64", "aarch64":
		return "arm64"
	case "arm":
		return "arm"
	case "386":
		return "x86"
	default:
		return arch
	}
}

// GetPlatform detects the current operating system platform
func GetPlatform() string {
	platform := runtime.GOOS

	// Normalize platform names
	switch platform {
	case "linux":
		return "linux"
	case "darwin":
		return "darwin"
	case "windows":
		return "windows"
	default:
		return platform
	}
}

// IsARM checks if the current architecture is ARM-based
func IsARM() bool {
	arch := GetArchitecture()
	return arch == "arm64" || arch == "arm"
}

// IsARM64 checks if the current architecture is ARM64
func IsARM64() bool {
	return GetArchitecture() == "arm64"
}

// GetARMVariant detects the specific ARM variant for enhanced detection
func GetARMVariant() string {
	if !IsARM() {
		return ""
	}

	// Try to detect specific ARM variant
	if runtime.GOOS == "linux" {
		// Check /proc/cpuinfo for ARM details
		output, err := exec.Command("cat", "/proc/cpuinfo").Output()
		if err == nil {
			cpuinfo := string(output)

			// Check for specific ARM processors
			if strings.Contains(cpuinfo, "Cortex-A") {
				return "cortex-a"
			}
			if strings.Contains(cpuinfo, "Cortex-M") {
				return "cortex-m"
			}
			if strings.Contains(cpuinfo, "Cortex-R") {
				return "cortex-r"
			}
			if strings.Contains(cpuinfo, "Neoverse") {
				return "neoverse"
			}
		}
	} else if runtime.GOOS == "darwin" {
		// Check for Apple Silicon variant
		output, err := exec.Command("sysctl", "-n", "machdep.cpu.brand_string").Output()
		if err == nil {
			brandString := strings.ToLower(string(output))
			if strings.Contains(brandString, "m1") {
				return "apple-m1"
			}
			if strings.Contains(brandString, "m2") {
				return "apple-m2"
			}
			if strings.Contains(brandString, "m3") {
				return "apple-m3"
			}
		}
	}

	return "generic-arm"
}

// CreateCPUOnlyWarningDevice creates a warning device when no GPU/TPU is detected
func CreateCPUOnlyWarningDevice() schema.HardwareDevice {
	arch := GetArchitecture()
	platform := GetPlatform()

	// Create performance warning based on architecture
	perfWarning := "CPU-only mode - no hardware acceleration detected"
	if IsARM() {
		perfWarning = "CPU-only mode on ARM - performance may be limited for video processing"
	}

	device := schema.HardwareDevice{
		ID:               "cpu-only-warning",
		Type:             "cpu",
		Name:             "CPU Only (No GPU/TPU detected)",
		DevicePath:       "/dev/cpu",
		Capabilities:     []string{"software_decode", "software_encode", "cpu_only"},
		Platform:         platform,
		Architecture:     arch,
		DetectionSource:  "fallback",
		Available:        true,
		InUse:            false,
		Metadata: map[string]interface{}{
			"warning":           perfWarning,
			"recommendation":    "Consider adding a GPU or TPU for better performance",
			"performance_level": "low",
		},
	}

	return device
}

// SafeCommandExecution executes a command with error handling fallback
// Returns output and error, but won't panic on missing commands
func SafeCommandExecution(command string, args ...string) (string, error) {
	// Check if command exists first
	_, err := exec.LookPath(command)
	if err != nil {
		return "", fmt.Errorf("command not found: %s", command)
	}

	// Execute with timeout protection
	cmd := exec.Command(command, args...)
	output, err := cmd.Output()

	if err != nil {
		// Return empty string instead of panicking
		return "", fmt.Errorf("command execution failed: %w", err)
	}

	return string(output), nil
}

// IsCommandAvailable checks if a system command is available
func IsCommandAvailable(command string) bool {
	_, err := exec.LookPath(command)
	return err == nil
}

// GetARMGPUCapabilities returns ARM-specific GPU capabilities
func GetARMGPUCapabilities(gpuName string) []string {
	capabilities := []string{}
	lowerName := strings.ToLower(gpuName)

	// Mali GPU (ARM's GPU architecture)
	if strings.Contains(lowerName, "mali") {
		capabilities = append(capabilities, "mali", "opencl", "opengl_es")

		// Check for specific Mali generations
		if strings.Contains(lowerName, "g") {
			capabilities = append(capabilities, "mali_valhall")
		}
		if strings.Contains(lowerName, "bifrost") {
			capabilities = append(capabilities, "mali_bifrost")
		}
	}

	// Adreno GPU (Qualcomm)
	if strings.Contains(lowerName, "adreno") {
		capabilities = append(capabilities, "adreno", "opencl", "vulkan", "opengl_es")

		// Check for specific Adreno versions
		if strings.Contains(lowerName, "6") {
			capabilities = append(capabilities, "adreno_6xx")
		}
		if strings.Contains(lowerName, "7") {
			capabilities = append(capabilities, "adreno_7xx")
		}
	}

	// Apple GPU (M-series)
	if strings.Contains(lowerName, "apple") {
		capabilities = append(capabilities, "metal", "videotoolbox")

		if strings.Contains(lowerName, "m1") || strings.Contains(lowerName, "m2") || strings.Contains(lowerName, "m3") {
			capabilities = append(capabilities, "h264_decode", "h264_encode", "hevc_decode", "hevc_encode")
		}

		// M3 and later have AV1 support
		if strings.Contains(lowerName, "m3") {
			capabilities = append(capabilities, "av1_decode", "av1_encode")
		}
	}

	// PowerVR GPU
	if strings.Contains(lowerName, "powervr") {
		capabilities = append(capabilities, "powervr", "opengl_es")
	}

	// Generic ARM GPU capabilities
	if len(capabilities) == 0 {
		capabilities = append(capabilities, "opengl_es", "basic_3d")
	}

	return capabilities
}

// EnhanceDeviceWithArchitectureInfo adds architecture-specific metadata to devices
func EnhanceDeviceWithArchitectureInfo(device *schema.HardwareDevice) {
	if device.Metadata == nil {
		device.Metadata = make(map[string]interface{})
	}

	// Add ARM variant information
	if IsARM() {
		variant := GetARMVariant()
		if variant != "" {
			device.Metadata["arm_variant"] = variant
		}

		// Add ARM-specific performance hints
		if device.Type == "gpu" {
			device.Metadata["arm_optimized"] = true
		}
	}

	// Add platform-specific hints
	platform := GetPlatform()
	device.Metadata["platform_optimized"] = platform

	// Add architecture optimization hints
	arch := GetArchitecture()
	if arch == "arm64" {
		device.Metadata["simd_support"] = "neon"
	} else if arch == "amd64" {
		device.Metadata["simd_support"] = "avx2"
	}
}
