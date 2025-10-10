package main

import (
	"encoding/json"
	"fmt"
	"os"
	"runtime"
	"time"

	"github.com/frigate-config-tool/agent/internal/detect"
	"github.com/frigate-config-tool/agent/internal/schema"
)

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	command := os.Args[1]

	switch command {
	case "detect":
		handleDetect()
	case "detect-gpu-capabilities":
		handleDetectGPUCapabilities()
	case "detect-camera-capabilities":
		handleDetectCameraCapabilities()
	case "detect-tpu-capabilities":
		handleDetectTPUCapabilities()
	case "detect-capture-card-capabilities":
		handleDetectCaptureCardCapabilities()
	case "detect-availability":
		handleDetectAvailability()
	case "version":
		handleVersion()
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", command)
		printUsage()
		os.Exit(1)
	}
}

func printUsage() {
	fmt.Fprintln(os.Stderr, "Usage: agent <command>")
	fmt.Fprintln(os.Stderr, "Commands:")
	fmt.Fprintln(os.Stderr, "  detect                           Detect all hardware devices")
	fmt.Fprintln(os.Stderr, "  detect-gpu-capabilities          Detect GPU capabilities")
	fmt.Fprintln(os.Stderr, "  detect-camera-capabilities       Detect camera capabilities")
	fmt.Fprintln(os.Stderr, "  detect-tpu-capabilities          Detect TPU capabilities")
	fmt.Fprintln(os.Stderr, "  detect-capture-card-capabilities Detect capture card capabilities")
	fmt.Fprintln(os.Stderr, "  detect-availability              Detect hardware availability")
	fmt.Fprintln(os.Stderr, "  version                          Show version information")
}

func handleDetect() {
	// Get platform and architecture
	platform := runtime.GOOS
	arch := runtime.GOARCH

	// Normalize architecture names to match our schema
	switch arch {
	case "amd64":
		arch = "x86_64"
	case "arm64":
		// Keep as is
	case "arm":
		arch = "arm32"
	}

	// Create detection result
	result := schema.NewDetectionResult(platform, arch)

	// Perform platform-specific detection
	devices, err := detectHardware(platform, arch)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Warning: Detection error: %v\n", err)
		// Continue with empty devices list
	}

	// Add detected devices to result
	result.Devices = devices

	// Convert to JSON and output
	jsonOutput, err := result.ToJSON()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error generating JSON: %v\n", err)
		os.Exit(1)
	}

	fmt.Println(jsonOutput)
}

func handleDetectGPUCapabilities() {
	platform := runtime.GOOS
	arch := normalizeArch(runtime.GOARCH)

	// Create GPU capabilities detector
	gpuDetector := detect.NewGPUCapabilitiesDetector(nil)

	// Detect GPU capabilities
	capabilities, err := gpuDetector.DetectGPUCapabilities()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error detecting GPU capabilities: %v\n", err)
		os.Exit(1)
	}

	// Output as JSON
	output, err := json.MarshalIndent(map[string]interface{}{
		"timestamp":   time.Now().UTC().Format(time.RFC3339),
		"platform":    platform,
		"architecture": arch,
		"capabilities": capabilities,
	}, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error marshaling JSON: %v\n", err)
		os.Exit(1)
	}

	fmt.Println(string(output))
}

func handleDetectCameraCapabilities() {
	platform := runtime.GOOS
	arch := normalizeArch(runtime.GOARCH)

	cameraDetector := detect.NewCameraCapabilitiesDetector(nil)

	capabilities, err := cameraDetector.DetectCameraCapabilities()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error detecting camera capabilities: %v\n", err)
		os.Exit(1)
	}

	output, err := json.MarshalIndent(map[string]interface{}{
		"timestamp":   time.Now().UTC().Format(time.RFC3339),
		"platform":    platform,
		"architecture": arch,
		"capabilities": capabilities,
	}, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error marshaling JSON: %v\n", err)
		os.Exit(1)
	}

	fmt.Println(string(output))
}

func handleDetectTPUCapabilities() {
	platform := runtime.GOOS
	arch := normalizeArch(runtime.GOARCH)

	tpuDetector := detect.NewTPUCapabilitiesDetector(nil)

	capabilities, err := tpuDetector.DetectTPUCapabilities()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error detecting TPU capabilities: %v\n", err)
		os.Exit(1)
	}

	output, err := json.MarshalIndent(map[string]interface{}{
		"timestamp":   time.Now().UTC().Format(time.RFC3339),
		"platform":    platform,
		"architecture": arch,
		"capabilities": capabilities,
	}, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error marshaling JSON: %v\n", err)
		os.Exit(1)
	}

	fmt.Println(string(output))
}

func handleDetectCaptureCardCapabilities() {
	platform := runtime.GOOS
	arch := normalizeArch(runtime.GOARCH)

	cardDetector := detect.NewCaptureCardCapabilitiesDetector(nil)

	capabilities, err := cardDetector.DetectCaptureCardCapabilities()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error detecting capture card capabilities: %v\n", err)
		os.Exit(1)
	}

	output, err := json.MarshalIndent(map[string]interface{}{
		"timestamp":   time.Now().UTC().Format(time.RFC3339),
		"platform":    platform,
		"architecture": arch,
		"capabilities": capabilities,
	}, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error marshaling JSON: %v\n", err)
		os.Exit(1)
	}

	fmt.Println(string(output))
}

func handleDetectAvailability() {
	platform := runtime.GOOS
	arch := normalizeArch(runtime.GOARCH)

	availabilityDetector := detect.NewHardwareAvailabilityDetector(nil)

	availability, err := availabilityDetector.DetectHardwareAvailability()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error detecting hardware availability: %v\n", err)
		os.Exit(1)
	}

	output, err := json.MarshalIndent(map[string]interface{}{
		"timestamp":    time.Now().UTC().Format(time.RFC3339),
		"platform":     platform,
		"architecture": arch,
		"availability": availability,
	}, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error marshaling JSON: %v\n", err)
		os.Exit(1)
	}

	fmt.Println(string(output))
}

func handleVersion() {
	fmt.Printf("Frigate Config Agent v1.0.0\n")
	fmt.Printf("Platform: %s/%s\n", runtime.GOOS, runtime.GOARCH)
	fmt.Printf("Go Version: %s\n", runtime.Version())
}

func normalizeArch(arch string) string {
	switch arch {
	case "amd64":
		return "x86_64"
	case "arm64":
		return "arm64"
	case "arm":
		return "arm32"
	default:
		return arch
	}
}

// detectHardware is implemented in platform-specific files:
// - detect_linux.go (for Linux)
// - detect_darwin.go (for macOS)
// - detect_windows.go (for Windows)
func detectHardware(platform, arch string) ([]schema.HardwareDevice, error) {
	// Use platform-specific detection
	return platformDetectHardware(platform, arch)
}
